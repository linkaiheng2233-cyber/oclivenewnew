//! MCP 客户端（最小闭环）：发现本地 server manifest，并支持工具调用。
//!
//! 完整实现依赖 `feature = "kernel-agent"`；关闭时仅为占位类型，避免拉入进程 / HTTP 调用路径。

pub use oclive_kernel_core::mcp::{
    McpInvoke, McpServerManifest, McpToolCallResult, McpToolManifest,
};

use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

#[cfg(feature = "kernel-agent")]
pub struct McpClient {
    root_dir: std::path::PathBuf,
    servers_cache: parking_lot::RwLock<Vec<McpServerManifest>>,
}

#[cfg(not(feature = "kernel-agent"))]
pub struct McpClient;

#[cfg(not(feature = "kernel-agent"))]
impl McpClient {
    #[must_use]
    pub fn new(_app_data_dir: impl AsRef<Path>) -> Self {
        Self
    }

    #[must_use]
    pub async fn list_servers(&self) -> Vec<McpServerManifest> {
        Vec::new()
    }

    pub async fn list_tools(&self, _server_id: &str) -> Result<Vec<McpToolManifest>> {
        Ok(Vec::new())
    }

    pub async fn call_tool(
        &self,
        _server_id: &str,
        _tool_name: &str,
        _params: Value,
    ) -> Result<McpToolCallResult> {
        Err(AppError::InvalidParameter(
            "[MCP_BUILD] kernel-agent feature disabled; MCP unavailable".into(),
        ))
    }
}

#[cfg(feature = "kernel-agent")]
impl McpClient {
    #[must_use]
    pub fn new(app_data_dir: impl AsRef<Path>) -> Self {
        let root = app_data_dir.as_ref().join("mcp-servers");
        Self {
            root_dir: root,
            servers_cache: parking_lot::RwLock::new(Vec::new()),
        }
    }

    async fn read_manifests_from_disk(root: &Path) -> Vec<McpServerManifest> {
        let mut out: Vec<McpServerManifest> = Vec::new();
        let Ok(mut rd) = tokio::fs::read_dir(root).await else {
            return out;
        };
        while let Ok(Some(entry)) = rd.next_entry().await {
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
            let Ok(raw) = tokio::fs::read_to_string(&path).await else {
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

    pub async fn list_servers(&self) -> Vec<McpServerManifest> {
        let _ = tokio::fs::create_dir_all(&self.root_dir).await;
        let next = Self::read_manifests_from_disk(self.root_dir.as_path()).await;
        *self.servers_cache.write() = next.clone();
        next
    }

    async fn find_server(&self, server_id: &str) -> Result<McpServerManifest> {
        let sid = server_id.trim();
        if sid.is_empty() {
            return Err(AppError::InvalidParameter("server_id required".to_string()));
        }
        let current = self.list_servers().await;
        current
            .into_iter()
            .find(|s| s.id == sid)
            .ok_or_else(|| AppError::InvalidParameter(format!("mcp server not found: {}", sid)))
    }

    fn timeout_for(&self, server: &McpServerManifest) -> std::time::Duration {
        std::time::Duration::from_millis(server.timeout_ms.unwrap_or(12_000).max(500))
    }

    pub async fn list_tools(&self, server_id: &str) -> Result<Vec<McpToolManifest>> {
        use serde_json::json;
        let server = self.find_server(server_id).await?;
        let payload = json!({
            "method": "list_tools",
            "params": {}
        });
        let dynamic = match server.transport.trim().to_ascii_lowercase().as_str() {
            "stdio" => self.call_raw_stdio(&server, payload).await,
            _ => self.call_raw_http(&server, payload).await,
        };
        match dynamic {
            Ok(v) => {
                if let Some(arr) = v
                    .get("tools")
                    .and_then(|x| x.as_array())
                    .or_else(|| v.as_array())
                {
                    let mut out = Vec::new();
                    for item in arr {
                        if let Ok(m) = serde_json::from_value::<McpToolManifest>(item.clone()) {
                            out.push(m);
                        } else if let Some(name) = item.as_str() {
                            out.push(McpToolManifest {
                                name: name.to_string(),
                                description: None,
                            });
                        }
                    }
                    if !out.is_empty() {
                        return Ok(out);
                    }
                }
                Ok(server.tools)
            }
            Err(_) => Ok(server.tools),
        }
    }

    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<McpToolCallResult> {
        use serde_json::json;
        let server = self.find_server(server_id).await?;
        let tool = tool_name.trim();
        if tool.is_empty() {
            return Err(AppError::InvalidParameter("tool_name required".to_string()));
        }
        let payload = json!({
            "tool": tool,
            "params": params
        });
        let result = match server.transport.trim().to_ascii_lowercase().as_str() {
            "stdio" => self.call_tool_stdio(&server, payload).await?,
            _ => self.call_tool_http(&server, payload).await?,
        };
        Ok(McpToolCallResult {
            server_id: server.id,
            tool_name: tool.to_string(),
            result,
        })
    }

    async fn call_tool_http(&self, server: &McpServerManifest, payload: Value) -> Result<Value> {
        let body = self.call_raw_http(server, payload).await?;
        Ok(body.get("result").cloned().unwrap_or(body))
    }

    async fn call_raw_http(&self, server: &McpServerManifest, payload: Value) -> Result<Value> {
        let Some(url) = server.url.as_ref() else {
            return Err(AppError::InvalidParameter(format!(
                "mcp server {} missing url",
                server.id
            )));
        };
        let cli = reqwest::Client::builder()
            .timeout(self.timeout_for(server))
            .build()
            .map_err(|e| {
                AppError::InvalidParameter(format!(
                    "[MCP_HTTP] client build server={}: {}",
                    server.id, e
                ))
            })?;
        let server_id = server.id.clone();
        let url = url.clone();
        let resp = cli
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                AppError::InvalidParameter(format!("[MCP_HTTP] call failed ({}): {}", server_id, e))
            })?;
        let status = resp.status();
        let body: Value = resp.json().await.map_err(|e| {
            AppError::InvalidParameter(format!("[MCP_HTTP] json decode ({}): {}", server_id, e))
        })?;
        if !status.is_success() {
            return Err(AppError::InvalidParameter(format!(
                "[MCP_HTTP] non-success server={} status={} body={}",
                server_id, status, body
            )));
        }
        Ok(body)
    }

    async fn call_tool_stdio(&self, server: &McpServerManifest, payload: Value) -> Result<Value> {
        let v = self.call_raw_stdio(server, payload).await?;
        Ok(v.get("result").cloned().unwrap_or(v))
    }

    async fn call_raw_stdio(&self, server: &McpServerManifest, payload: Value) -> Result<Value> {
        use serde_json::json;
        use std::process::Stdio;
        use tokio::io::AsyncWriteExt;
        use tokio::process::Command;
        let Some(cmd) = server.command.as_ref() else {
            return Err(AppError::InvalidParameter(format!(
                "mcp server {} missing command",
                server.id
            )));
        };
        let timeout = self.timeout_for(server);
        let mut child = Command::new(cmd)
            .args(&server.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                AppError::InvalidParameter(format!(
                    "[MCP_STDIO] spawn failed ({}): {}",
                    server.id, e
                ))
            })?;
        let body = serde_json::to_vec(&payload)?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&body).await.map_err(AppError::IoError)?;
            let _ = stdin.shutdown().await;
        }
        let output = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| {
                AppError::InvalidParameter(format!(
                    "[MCP_STDIO] timeout server={} timeout_ms={}",
                    server.id,
                    timeout.as_millis()
                ))
            })?
            .map_err(AppError::IoError)?;
        if !output.status.success() {
            return Err(AppError::InvalidParameter(format!(
                "[MCP_STDIO] process error server={} exit={} stderr={}",
                server.id,
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return Ok(json!({}));
        }
        let v: Value = serde_json::from_str(&text).map_err(AppError::from)?;
        Ok(v)
    }
}

#[async_trait]
impl McpInvoke for McpClient {
    async fn list_servers(&self) -> Vec<McpServerManifest> {
        McpClient::list_servers(self).await
    }

    async fn list_tools(&self, server_id: &str) -> Result<Vec<McpToolManifest>> {
        McpClient::list_tools(self, server_id).await
    }

    async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        params: Value,
    ) -> Result<McpToolCallResult> {
        McpClient::call_tool(self, server_id, tool_name, params).await
    }
}
