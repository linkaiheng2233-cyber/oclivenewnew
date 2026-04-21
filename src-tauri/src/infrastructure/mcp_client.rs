//! MCP 客户端（最小闭环）：发现本地 server manifest，并支持工具调用。
//!
//! 当前支持：
//! - transport=`http`：POST JSON 到 `url`
//! - transport=`stdio`：启动命令，向 stdin 写入请求 JSON，读取 stdout JSON

use crate::error::{AppError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolManifest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub transport: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub tools: Vec<McpToolManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    pub server_id: String,
    pub tool_name: String,
    pub result: Value,
}

pub struct McpClient {
    root_dir: PathBuf,
    servers_cache: RwLock<Vec<McpServerManifest>>,
}

impl McpClient {
    #[must_use]
    pub fn new(app_data_dir: impl AsRef<Path>) -> Self {
        let root = app_data_dir.as_ref().join("mcp-servers");
        let _ = fs::create_dir_all(&root);
        Self {
            root_dir: root,
            servers_cache: RwLock::new(Vec::new()),
        }
    }

    fn read_manifests_from_disk(&self) -> Vec<McpServerManifest> {
        let mut out: Vec<McpServerManifest> = Vec::new();
        let Ok(rd) = fs::read_dir(&self.root_dir) else {
            return out;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("json"))
                != Some(true)
            {
                continue;
            }
            let Ok(raw) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(mut m) = serde_json::from_str::<McpServerManifest>(&raw) else {
                continue;
            };
            m.id = m.id.trim().to_string();
            if m.id.is_empty() {
                continue;
            }
            if m.transport.trim().is_empty() {
                m.transport = "http".to_string();
            }
            out.push(m);
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    pub fn list_servers(&self) -> Vec<McpServerManifest> {
        let next = self.read_manifests_from_disk();
        *self.servers_cache.write() = next.clone();
        next
    }

    fn find_server(&self, server_id: &str) -> Result<McpServerManifest> {
        let sid = server_id.trim();
        if sid.is_empty() {
            return Err(AppError::InvalidParameter("server_id required".to_string()));
        }
        let current = self.list_servers();
        current
            .into_iter()
            .find(|s| s.id == sid)
            .ok_or_else(|| AppError::InvalidParameter(format!("mcp server not found: {}", sid)))
    }

    pub fn call_tool(&self, server_id: &str, tool_name: &str, params: Value) -> Result<McpToolCallResult> {
        let server = self.find_server(server_id)?;
        let tool = tool_name.trim();
        if tool.is_empty() {
            return Err(AppError::InvalidParameter("tool_name required".to_string()));
        }
        let payload = json!({
            "tool": tool,
            "params": params
        });
        let result = match server.transport.trim().to_ascii_lowercase().as_str() {
            "stdio" => self.call_tool_stdio(&server, payload)?,
            _ => self.call_tool_http(&server, payload)?,
        };
        Ok(McpToolCallResult {
            server_id: server.id,
            tool_name: tool.to_string(),
            result,
        })
    }

    fn call_tool_http(&self, server: &McpServerManifest, payload: Value) -> Result<Value> {
        let Some(url) = server.url.as_ref() else {
            return Err(AppError::InvalidParameter(format!(
                "mcp server {} missing url",
                server.id
            )));
        };
        let cli = reqwest::blocking::Client::new();
        let resp = cli
            .post(url)
            .json(&payload)
            .send()
            .map_err(|e| AppError::Unknown(format!("mcp http call failed: {}", e)))?;
        let status = resp.status();
        let body: Value = resp
            .json()
            .map_err(|e| AppError::Unknown(format!("mcp http json decode failed: {}", e)))?;
        if !status.is_success() {
            return Err(AppError::Unknown(format!(
                "mcp http status={} body={}",
                status, body
            )));
        }
        Ok(body.get("result").cloned().unwrap_or(body))
    }

    fn call_tool_stdio(&self, server: &McpServerManifest, payload: Value) -> Result<Value> {
        let Some(cmd) = server.command.as_ref() else {
            return Err(AppError::InvalidParameter(format!(
                "mcp server {} missing command",
                server.id
            )));
        };
        let mut child = Command::new(cmd)
            .args(&server.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Unknown(format!("spawn mcp stdio failed: {}", e)))?;
        if let Some(stdin) = child.stdin.as_mut() {
            let body = serde_json::to_vec(&payload)?;
            stdin
                .write_all(&body)
                .map_err(|e| AppError::Unknown(format!("mcp stdin write failed: {}", e)))?;
        }
        let out = child
            .wait_with_output()
            .map_err(|e| AppError::Unknown(format!("mcp stdio wait failed: {}", e)))?;
        if !out.status.success() {
            return Err(AppError::Unknown(format!(
                "mcp stdio exit={} stderr={}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if text.is_empty() {
            return Ok(json!({}));
        }
        let v: Value = serde_json::from_str(&text)?;
        Ok(v.get("result").cloned().unwrap_or(v))
    }
}
