use crate::api::error::CommandError;
use crate::error::AppError;
use oclive_kernel_host::state::{AppState, SharedAppState};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tauri::State;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePluginScaffoldRequest {
    pub plugin_id: String,
    pub plugin_name: String,
    /// node | python | rust
    pub language: String,
    /// skill | agent | module_ext
    pub plugin_type: String,
    #[serde(default)]
    pub base_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePluginScaffoldResponse {
    pub plugin_dir: String,
}

fn safe_file_stem(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
}

fn default_plugins_root(state: &AppState) -> PathBuf {
    state
        .storage
        .roles_dir()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("plugins")
}

fn write_template_files(
    plugin_dir: &Path,
    plugin_id: &str,
    plugin_name: &str,
    language: &str,
    plugin_type: &str,
) -> Result<(), AppError> {
    fs::create_dir_all(plugin_dir)?;
    let is_reply_pp = plugin_type.eq_ignore_ascii_case("reply_post_process");
    let manifest = if is_reply_pp {
        serde_json::json!({
            "schema_version": 1,
            "id": plugin_id,
            "name": plugin_name,
            "version": "0.1.0",
            "description": format!("{} reply post-process polish scaffold", plugin_name),
            "description_zh": "可选：润色 LLM 原始回复为展示文本（reply_post_process.process）。",
            "author": "creator",
            "provides": ["reply_post_process"],
            "category": "reply_post_process",
            "permissions": ["process:spawn"],
            "process": {
                "command": if language == "python" { "python" } else { "node" },
                "args": [if language == "python" { "rpc_server.py" } else { "rpc_server.mjs" }],
                "cwd": "."
            },
            "config": {
                "env": {
                    "OCLIVE_POLISH_OLLAMA_URL": "http://127.0.0.1:11434",
                    "OCLIVE_POLISH_MODEL": "qwen2.5:3b",
                    "OCLIVE_POLISH_MAX_EXCERPT": "800"
                }
            }
        })
    } else {
        serde_json::json!({
            "id": plugin_id,
            "name": plugin_name,
            "version": "0.1.0",
            "description": format!("{} scaffold ({})", plugin_name, plugin_type),
            "author": "creator",
            "runtime": language,
            "permissions": ["process:spawn"],
            "tools": [
              { "name": "example_tool", "description": "example tool placeholder" }
            ]
        })
    };
    fs::write(
        plugin_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).map_err(AppError::from)?,
    )?;
    let readme = format!(
        "# {}\n\n- id: `{}`\n- language: `{}`\n- type: `{}`\n",
        plugin_name, plugin_id, language, plugin_type
    );
    fs::write(plugin_dir.join("README.md"), readme)?;
    if is_reply_pp {
        const REPLY_PP_FILES: &[(&str, &str)] = &[
            ("rpc_server.mjs", include_str!("../../../examples/reply-post-process-polish/rpc_server.mjs")),
            ("preset_cache.mjs", include_str!("../../../examples/reply-post-process-polish/preset_cache.mjs")),
            ("preset_builder.mjs", include_str!("../../../examples/reply-post-process-polish/preset_builder.mjs")),
            ("polish_rules.mjs", include_str!("../../../examples/reply-post-process-polish/polish_rules.mjs")),
            ("ollama_client.mjs", include_str!("../../../examples/reply-post-process-polish/ollama_client.mjs")),
        ];
        for (name, content) in REPLY_PP_FILES {
            fs::write(plugin_dir.join(name), content)?;
        }
        let readme = format!(
            "# {}\n\nReply post-process polish scaffold (rule-gated LLM). See `examples/reply-post-process-polish/README.md` in oclivenewnew.\n\nModules: `preset_cache.mjs`, `preset_builder.mjs`, `polish_rules.mjs`, `ollama_client.mjs`, `rpc_server.mjs`.\n",
            plugin_name
        );
        fs::write(plugin_dir.join("README.md"), readme)?;
        return Ok(());
    }
    match language {
        "python" => {
            fs::write(
                plugin_dir.join("main.py"),
                "import json\nimport sys\n\n# TODO: implement MCP stdio server\nprint(json.dumps({\"result\": {\"ok\": True}}))\n",
            )?;
        }
        "rust" => {
            fs::create_dir_all(plugin_dir.join("src"))?;
            fs::write(
                plugin_dir.join("Cargo.toml"),
                "[package]\nname = \"plugin_scaffold\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\nserde_json = \"1\"\n",
            )?;
            fs::write(
                plugin_dir.join("src").join("main.rs"),
                "fn main() {\n    println!(\"{\\\"result\\\":{\\\"ok\\\":true}}\");\n}\n",
            )?;
        }
        _ => {
            fs::write(
                plugin_dir.join("package.json"),
                "{\n  \"name\": \"plugin-scaffold\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"scripts\": {\"start\":\"node index.js\"}\n}\n",
            )?;
            fs::write(
                plugin_dir.join("index.js"),
                "const fs = require('fs');\n\n// TODO: implement MCP server / plugin logic\nconsole.log(JSON.stringify({ result: { ok: true } }));\n",
            )?;
        }
    }
    Ok(())
}
/// # Errors
///
/// Returns [`Err`] with a human-readable message when the operation fails.
#[tauri::command]
pub fn create_plugin_scaffold(
    req: CreatePluginScaffoldRequest,
    state: State<'_, SharedAppState>,
    app: tauri::AppHandle,
) -> Result<CreatePluginScaffoldResponse, CommandError> {
    let plugin_id = safe_file_stem(req.plugin_id.trim());
    if plugin_id.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()).into());
    }
    let plugin_name = req.plugin_name.trim();
    if plugin_name.is_empty() {
        return Err(AppError::InvalidParameter("plugin_name required".into()).into());
    }
    let base = req
        .base_dir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_plugins_root(&state));
    if let Err(e) = fs::create_dir_all(&base) {
        return Err(AppError::IoError(e).into());
    }
    let plugin_dir = base.join(plugin_id.as_str());
    if plugin_dir.exists() {
        return Err(AppError::InvalidParameter(format!(
            "plugin dir already exists: {}",
            plugin_dir.display()
        ))
        .into());
    }
    if let Err(e) = write_template_files(
        &plugin_dir,
        plugin_id.as_str(),
        plugin_name,
        req.language.trim(),
        req.plugin_type.trim(),
    ) {
        return Err(e.into());
    }
    let _ = tauri::api::shell::open(&app.shell_scope(), &*plugin_dir.to_string_lossy(), None);
    Ok(CreatePluginScaffoldResponse {
        plugin_dir: plugin_dir.to_string_lossy().to_string(),
    })
}
