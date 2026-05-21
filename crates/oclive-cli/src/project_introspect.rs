//! Analyze scaffold projects for replication (`init --from-existing`, `template create`, `kernel info`).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub project_name: String,
    pub project_type: String,
    pub preset: String,
    pub monolith_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monolith_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_source: Option<String>,
    pub kernel_dep_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<String>,
    pub has_roles: bool,
    pub with_example_plugin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_pack_hint: Option<String>,
}

pub fn analyze_project(root: &Path) -> Result<ProjectSnapshot> {
    let root = root
        .canonicalize()
        .with_context(|| format!("path {}", root.display()))?;
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        bail!("not a Cargo project root: {}", root.display());
    }
    let cargo = fs::read_to_string(&cargo_toml)?;
    let v: toml::Value = toml::from_str(&cargo)?;
    let pkg = v
        .get("package")
        .and_then(|p| p.as_table())
        .context("package")?;
    let project_name = pkg
        .get("name")
        .and_then(|n| n.as_str())
        .context("[package].name")?
        .to_string();
    let license = pkg
        .get("license")
        .and_then(|x| x.as_str())
        .map(str::to_string);
    let description = pkg
        .get("description")
        .and_then(|x| x.as_str())
        .map(str::to_string);
    let author = pkg
        .get("authors")
        .and_then(|a| a.as_array())
        .and_then(|arr| arr.first())
        .and_then(|x| x.as_str())
        .map(str::to_string);

    let project_type = if root.join("src/main.rs").is_file()
        || root.join("src/main_monolith.rs").is_file()
        || v.get("bin")
            .and_then(|b| b.as_array())
            .is_some_and(|a| !a.is_empty())
    {
        "kernel_server"
    } else {
        "library"
    };

    let monolith_path = root.join("monolith.toml");
    let monolith_enabled = monolith_path.is_file();
    let monolith_preset = if monolith_enabled {
        infer_monolith_preset(&monolith_path)?
    } else {
        None
    };

    let preset = infer_preset_from_settings(&root);
    let (kernel_dep_kind, kernel_source) = detect_kernel_runtime(&v, &root)?;

    let pipeline = detect_pipeline(&root);
    let (has_roles, role_pack_hint) = detect_roles(&root);
    let with_example_plugin = root
        .join("plugins/com.oclive.example.llamacpp_llm/manifest.json")
        .is_file();

    Ok(ProjectSnapshot {
        project_name,
        project_type: project_type.into(),
        preset,
        monolith_enabled,
        monolith_preset,
        license,
        author,
        description,
        kernel_source,
        kernel_dep_kind,
        pipeline,
        has_roles,
        with_example_plugin,
        role_pack_hint,
    })
}

fn infer_monolith_preset(path: &Path) -> Result<Option<String>> {
    let raw = fs::read_to_string(path)?;
    let file = crate::monolith_config::parse_monolith_toml(&raw)?;
    let w = &file.monolith.weld_modules;
    if w.is_empty() {
        return Ok(Some("latency".into()));
    }
    let set: std::collections::HashSet<_> = w.iter().map(|s| s.as_str()).collect();
    let has = |s: &str| set.contains(s);
    if has("memory") && has("prompt") && has("llm") && !has("emotion") && set.len() <= 4 {
        return Ok(Some("memory".into()));
    }
    if has("emotion") && has("memory") && has("llm") && set.len() <= 4 {
        return Ok(Some("embedded".into()));
    }
    if set.len() >= 6 {
        return Ok(Some("latency".into()));
    }
    Ok(Some("latency".into()))
}

fn infer_preset_from_settings(root: &Path) -> String {
    let Some(raw) = find_first_settings(root) else {
        return "minimal".into();
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return "minimal".into();
    };
    let backends = v.get("plugin_backends").and_then(|p| p.as_object());
    let Some(b) = backends else {
        return "minimal".into();
    };
    let llm = b.get("llm").and_then(|x| x.as_str()).unwrap_or("builtin");
    let ce = b
        .get("complex_emotion")
        .and_then(|x| x.as_str())
        .unwrap_or("none");
    match (llm, ce) {
        ("remote", "remote") => "full".into(),
        ("ollama", _) => "mixed".into(),
        _ => "minimal".into(),
    }
}

fn find_first_settings(root: &Path) -> Option<String> {
    let roles = root.join("roles");
    if !roles.is_dir() {
        return None;
    }
    for e in fs::read_dir(&roles).ok()?.flatten() {
        let s = e.path().join("settings.json");
        if s.is_file() {
            return fs::read_to_string(&s).ok();
        }
    }
    None
}

fn detect_kernel_runtime(v: &toml::Value, root: &Path) -> Result<(String, Option<String>)> {
    let Some(deps) = v.get("dependencies").and_then(|d| d.as_table()) else {
        return Ok(("none".into(), None));
    };
    let Some(rt) = deps.get("oclive_kernel_runtime") else {
        return Ok(("none".into(), None));
    };
    if let Some(path) = rt.get("path").and_then(|p| p.as_str()) {
        let abs = root.join(path);
        let canonical = abs.canonicalize().unwrap_or(abs);
        let repo_root = canonical
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or(canonical);
        return Ok(("path".into(), Some(repo_root.display().to_string())));
    }
    if let Some(ver) = rt.as_str() {
        return Ok(("version".into(), Some(ver.into())));
    }
    if let Some(ver) = rt.get("version").and_then(|v| v.as_str()) {
        return Ok(("version".into(), Some(ver.into())));
    }
    Ok(("unknown".into(), None))
}

fn detect_pipeline(root: &Path) -> Option<String> {
    let order_rs = root.join("src/oclive_pipeline_order.rs");
    if order_rs.is_file() {
        let raw = fs::read_to_string(&order_rs).ok()?;
        if raw.contains("emotion-first") || raw.contains("EmotionFirst") {
            return Some("emotion-first".into());
        }
        if raw.contains("memory-last") || raw.contains("MemoryLast") {
            return Some("memory-last".into());
        }
    }
    let doc = root.join("docs/PIPELINE_CUSTOM.md");
    if doc.is_file() {
        let raw = fs::read_to_string(&doc).ok()?;
        if raw.contains("emotion-first") {
            return Some("emotion-first".into());
        }
        if raw.contains("memory-last") {
            return Some("memory-last".into());
        }
    }
    None
}

fn detect_roles(root: &Path) -> (bool, Option<String>) {
    let roles = root.join("roles");
    if !roles.is_dir() {
        return (false, None);
    }
    let mut ids = Vec::new();
    for e in fs::read_dir(&roles).into_iter().flatten().flatten() {
        if e.path().join("manifest.json").is_file() {
            ids.push(e.file_name().to_string_lossy().into_owned());
        }
    }
    if ids.is_empty() {
        return (true, None);
    }
    let hint = if ids
        .iter()
        .any(|id| id.contains("robot-soul") || id == "robot-soul-minimal")
    {
        Some("robot-soul-minimal".into())
    } else if ids.iter().any(|id| id == "default") {
        Some("default".into())
    } else {
        None
    };
    (true, hint)
}

pub fn build_init_command_line(snap: &ProjectSnapshot, output: &Path) -> String {
    let out = output.display().to_string();
    let mut parts = vec![
        "oclive init".to_string(),
        "--non-interactive".to_string(),
        format!("-o {}", shell_quote(&out)),
        format!("--project-name {}", shell_quote(&snap.project_name)),
        format!("--project-type {}", snap.project_type),
        format!("--preset {}", snap.preset),
    ];
    if snap.monolith_enabled {
        parts.push("--monolith".into());
        if let Some(ref mp) = snap.monolith_preset {
            parts.push(format!("--monolith-preset {mp}"));
        }
    }
    if let Some(ref lic) = snap.license {
        parts.push(format!("--license {}", shell_quote(lic)));
    }
    if let Some(ref a) = snap.author {
        parts.push(format!("--author {}", shell_quote(a)));
    }
    if let Some(ref d) = snap.description {
        parts.push(format!("--description {}", shell_quote(d)));
    }
    if let Some(ref ks) = snap.kernel_source {
        parts.push(format!("--kernel-source {}", shell_quote(ks)));
    }
    if let Some(ref rp) = snap.role_pack_hint {
        parts.push(format!("--with-role-pack {rp}"));
    } else if !snap.has_roles {
        parts.push("--skip-role-pack".into());
    }
    if snap.with_example_plugin {
        parts.push("--with-example-plugin".into());
    }
    if let Some(ref p) = snap.pipeline {
        if p != "default" {
            parts.push(format!("--pipeline {p}"));
        }
    }
    parts.join(" ")
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .any(|c| c.is_whitespace() || c == '"' || c == '\'')
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ShareFile {
    pub schema_version: u32,
    pub generated_at: String,
    pub source_path: String,
    pub snapshot: ProjectSnapshot,
    pub reproduce_command: String,
}

pub fn write_share_file(root: &Path, snap: &ProjectSnapshot, cmd: &str) -> Result<PathBuf> {
    let out = root.join(".oclive-share.toml");
    let share = ShareFile {
        schema_version: 1,
        generated_at: chrono_lite_now(),
        source_path: root.display().to_string(),
        snapshot: snap.clone(),
        reproduce_command: cmd.to_string(),
    };
    let toml_body = share_to_toml(&share)?;
    fs::write(&out, toml_body).with_context(|| format!("write {}", out.display()))?;
    Ok(out)
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

fn share_to_toml(share: &ShareFile) -> Result<String> {
    Ok(format!(
        "# Generated by oclive init --share\nschema_version = {}\ngenerated_at = \"{}\"\nsource_path = \"{}\"\nreproduce_command = \"{}\"\n\n[snapshot]\n{}\n",
        share.schema_version,
        share.generated_at,
        share.source_path.replace('\\', "\\\\").replace('"', "\\\""),
        share.reproduce_command.replace('\\', "\\\\").replace('"', "\\\""),
        snapshot_toml_section(&share.snapshot)?
    ))
}

fn snapshot_toml_section(s: &ProjectSnapshot) -> Result<String> {
    let j = serde_json::to_value(s)?;
    let mut lines = Vec::new();
    if let Some(obj) = j.as_object() {
        for (k, v) in obj {
            let line = match v {
                Value::String(s) => format!("{k} = \"{}\"", s.replace('"', "\\\"")),
                Value::Bool(b) => format!("{k} = {b}"),
                Value::Null => continue,
                _ => format!("{k} = \"{v}\""),
            };
            lines.push(line);
        }
    }
    Ok(lines.join("\n"))
}

pub fn git_head_short(path: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["-C", path.to_str()?, "rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_preset_remote() {
        let tmp = tempfile::tempdir().unwrap();
        let roles = tmp.path().join("roles/demo");
        fs::create_dir_all(&roles).unwrap();
        fs::write(
            roles.join("settings.json"),
            r#"{"plugin_backends":{"llm":"remote","complex_emotion":"remote"}}"#,
        )
        .unwrap();
        assert_eq!(infer_preset_from_settings(tmp.path()), "full");
    }
}
