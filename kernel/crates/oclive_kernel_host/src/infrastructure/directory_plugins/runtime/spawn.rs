use super::super::manifest::OclivePluginManifest;
use super::transport::parse_ready_line;
use super::{DebugLogRing, DirectoryPluginRuntime};
use crate::infrastructure::background_process::configure_background_process;
use parking_lot::Mutex;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn non_empty_config_json(config_json: Option<&str>) -> Option<&str> {
    config_json
        .map(str::trim)
        .filter(|config| !config.is_empty())
}

impl DirectoryPluginRuntime {
    fn persisted_plugin_config_json(&self, plugin_id: &str) -> Option<String> {
        let path = self
            .app_data_dir()
            .join("plugin-data")
            .join(plugin_id)
            .join("config.json");
        if !path.is_file() {
            return None;
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(error) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    error_code = "PLUGIN_CONFIG_READ_FAILED",
                    plugin_id,
                    config_path = %path.display(),
                    %error,
                    "directory plugin persisted config could not be read"
                );
                return None;
            }
        };
        match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(serde_json::Value::Object(_)) => Some(raw),
            Ok(_) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    error_code = "PLUGIN_CONFIG_INVALID",
                    plugin_id,
                    config_path = %path.display(),
                    "directory plugin persisted config must be a JSON object"
                );
                None
            }
            Err(error) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    error_code = "PLUGIN_CONFIG_INVALID",
                    plugin_id,
                    config_path = %path.display(),
                    %error,
                    "directory plugin persisted config is invalid JSON"
                );
                None
            }
        }
    }

    pub(crate) fn ensure_rpc_url_impl(
        &self,
        plugin_id: &str,
        ignore_disabled: bool,
        config_json: Option<&str>,
    ) -> Result<String, String> {
        let id = plugin_id.trim();
        if id.is_empty() {
            return Err("plugin_id required".to_string());
        }
        if !ignore_disabled && self.effective_slots().is_plugin_disabled(id) {
            return Err(format!("plugin disabled: {}", id));
        }
        if self.rpc_urls.lock().contains_key(id) {
            self.invalidate_rpc_if_child_dead(id);
        }
        if let Some(u) = self.rpc_urls.lock().get(id) {
            if self.validate_rpc_endpoint(id, u) {
                return Ok(u.clone());
            }
            self.clear_plugin_process(id);
        }
        let lock = {
            let mut map = self.startup_locks.lock();
            map.entry(id.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        let _startup = lock.lock();
        if self.rpc_urls.lock().contains_key(id) {
            self.invalidate_rpc_if_child_dead(id);
        }
        if let Some(u) = self.rpc_urls.lock().get(id) {
            if self.validate_rpc_endpoint(id, u) {
                return Ok(u.clone());
            }
            self.clear_plugin_process(id);
        }
        let root = self
            .plugin_roots
            .read()
            .get(id)
            .map(|entry| entry.root.clone())
            .ok_or_else(|| format!("unknown directory plugin_id={}", id))?;
        let manifest = self.load_manifest_cached(id, &root)?;
        if manifest.process.is_some() {
            if !oclive_validation::manifest_declares_process_spawn(&manifest.permissions, true) {
                tracing::warn!(
                    target: "oclive_plugin",
                    "directory plugin id={} spawn blocked: manifest missing process:spawn permission",
                    id
                );
                return Err(format!(
                    "directory plugin spawn not permitted: plugin_id={} missing process:spawn in manifest permissions",
                    id
                ));
            }
            if !self.high_risk_grants.is_process_spawn_granted(id) {
                tracing::warn!(
                    target: "oclive_plugin",
                    "directory plugin id={} spawn blocked: grant process:spawn missing",
                    id
                );
                return Err(format!(
                    "directory plugin spawn not granted: plugin_id={}",
                    id
                ));
            }
        }
        let explicit_config = non_empty_config_json(config_json);
        let persisted_config = explicit_config
            .is_none()
            .then(|| self.persisted_plugin_config_json(id))
            .flatten();
        let effective_config = explicit_config.or(persisted_config.as_deref());
        let (url, child, started_ms) =
            self.spawn_child_handshake(id, root, (*manifest).clone(), effective_config)?;
        self.children.lock().insert(id.to_string(), child);
        self.rpc_urls.lock().insert(id.to_string(), url.clone());
        self.process_started_ms
            .lock()
            .insert(id.to_string(), started_ms);
        tracing::info!(
            target: "oclive_plugin",
            "directory plugin id={} rpc_url={}",
            id,
            url
        );
        Ok(url)
    }

    fn spawn_child_handshake(
        &self,
        plugin_id: &str,
        root: PathBuf,
        manifest: OclivePluginManifest,
        config_json: Option<&str>,
    ) -> Result<(String, std::process::Child, u64), String> {
        let proc = manifest
            .process
            .as_ref()
            .ok_or_else(|| format!("plugin {} has no process section", plugin_id))?;
        let prefix = if manifest.ready_prefix.trim().is_empty() {
            "OCLIVE_READY"
        } else {
            manifest.ready_prefix.trim()
        };
        let log_ring = Arc::new(Mutex::new(DebugLogRing::default()));
        self.debug_log_rings
            .lock()
            .insert(plugin_id.to_string(), log_ring.clone());

        let mut cmd = Command::new(&proc.command);
        for a in &proc.args {
            cmd.arg(a);
        }
        let cwd = proc
            .cwd
            .as_ref()
            .map(|c| root.join(c))
            .unwrap_or_else(|| root.clone());
        cmd.current_dir(&cwd);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        configure_background_process(&mut cmd);
        if let Some(cfg) = config_json {
            let t = cfg.trim();
            if !t.is_empty() {
                cmd.env("OCLIVE_PLUGIN_CONFIG", t);
            }
        }
        if self.roles_dir.is_dir() {
            cmd.env("OCLIVE_ROLES_DIR", &self.roles_dir);
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.debug_log_rings.lock().remove(plugin_id);
                return Err(format!("spawn plugin {}: {}", plugin_id, e));
            }
        };
        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                let _ = child.kill();
                self.debug_log_rings.lock().remove(plugin_id);
                return Err(format!("plugin {}: no stdout", plugin_id));
            }
        };
        let stderr = match child.stderr.take() {
            Some(s) => s,
            None => {
                let _ = child.kill();
                self.debug_log_rings.lock().remove(plugin_id);
                return Err(format!("plugin {}: no stderr", plugin_id));
            }
        };

        let handshake = Arc::new(Mutex::new(Vec::<String>::new()));
        let handshake_out = handshake.clone();
        let ring_out = log_ring.clone();
        thread::spawn(move || {
            let r = BufReader::new(stdout);
            for result in r.lines() {
                match result {
                    Ok(line) => {
                        handshake_out.lock().push(line.clone());
                        ring_out.lock().push_line(format!("[stdout] {}", line));
                    }
                    Err(_) => break,
                }
            }
        });
        let ring_err = log_ring.clone();
        thread::spawn(move || {
            let r = BufReader::new(stderr);
            for result in r.lines() {
                match result {
                    Ok(line) => ring_err.lock().push_line(format!("[stderr] {}", line)),
                    Err(_) => break,
                }
            }
        });

        let deadline = Instant::now() + Duration::from_secs(30);
        let url = 'wait: loop {
            if Instant::now() > deadline {
                let _ = child.kill();
                self.debug_log_rings.lock().remove(plugin_id);
                return Err(format!(
                    "plugin {}: timeout waiting for {} URL",
                    plugin_id, prefix
                ));
            }
            for line in handshake.lock().iter() {
                if let Some(u) = parse_ready_line(line, prefix, plugin_id, &self.high_risk_grants) {
                    break 'wait u;
                }
            }
            thread::sleep(Duration::from_millis(50));
        };
        let started_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Ok((url, child, started_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::{non_empty_config_json, DirectoryPluginRuntime};
    use crate::infrastructure::high_risk_grants::HighRiskGrantStore;

    #[test]
    fn empty_debug_config_does_not_mask_persisted_config() {
        assert_eq!(non_empty_config_json(None), None);
        assert_eq!(non_empty_config_json(Some("  \r\n")), None);
        assert_eq!(
            non_empty_config_json(Some(" {\"enabled\":true} ")),
            Some("{\"enabled\":true}")
        );
    }

    #[test]
    fn persisted_plugin_config_requires_a_json_object() {
        let temp = tempfile::tempdir().expect("tempdir");
        let app_data = temp.path().join("app-data");
        let roles = temp.path().join("role-fixtures");
        let grants = HighRiskGrantStore::load(app_data.clone(), false);
        let runtime = DirectoryPluginRuntime::bootstrap_deferred_scan(&roles, &app_data, grants);
        let config_dir = app_data.join("plugin-data/com.example.test");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        let config_path = config_dir.join("config.json");

        std::fs::write(&config_path, "[]").expect("array config");
        assert_eq!(
            runtime.persisted_plugin_config_json("com.example.test"),
            None
        );

        std::fs::write(&config_path, "{\"enabled\":true}").expect("object config");
        assert_eq!(
            runtime.persisted_plugin_config_json("com.example.test"),
            Some("{\"enabled\":true}".to_string())
        );
    }
}
