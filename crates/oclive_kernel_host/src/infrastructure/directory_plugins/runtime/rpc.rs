use super::{DirectoryPluginRuntime, PluginProcessDebugInfo, DEBUG_LOG_RING_CAP};

impl DirectoryPluginRuntime {
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the plugin cannot be started or the RPC URL is unavailable.
    pub fn ensure_rpc_url(&self, plugin_id: &str) -> Result<String, String> {
        self.ensure_rpc_url_impl(plugin_id, false, None)
    }
    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Same as [`Self::ensure_rpc_url`] but does not reject disabled plugins; may inject `OCLIVE_DEBUG_PLUGIN_CONFIG`.
    pub fn ensure_rpc_url_for_debug(
        &self,
        plugin_id: &str,
        config_json: Option<&str>,
    ) -> Result<String, String> {
        self.ensure_rpc_url_impl(plugin_id, true, config_json)
    }

    /// # Errors
    ///
    /// Returns [`Err`] with a human-readable message when the operation fails.
    /// Developer debug: ensure child handshake and return process info (returns snapshot if already running).
    pub fn spawn_plugin_for_test(
        &self,
        plugin_id: &str,
        config_json: Option<&str>,
    ) -> Result<PluginProcessDebugInfo, String> {
        let id = plugin_id.trim();
        if id.is_empty() {
            return Err("plugin_id required".to_string());
        }
        if self.rpc_urls.lock().contains_key(id) && self.children.lock().contains_key(id) {
            let ch = self.children.lock();
            let child = ch
                .get(id)
                .ok_or_else(|| "internal: child map inconsistent".to_string())?;
            let pid = child.id();
            let url = self.rpc_urls.lock().get(id).cloned().unwrap_or_default();
            let started_ms = self.process_started_ms.lock().get(id).copied().unwrap_or(0);
            return Ok(PluginProcessDebugInfo {
                plugin_id: id.to_string(),
                pid,
                rpc_url: url,
                started_at_ms: started_ms,
                cpu_percent: None,
                memory_kb: None,
            });
        }
        let url = self.ensure_rpc_url_for_debug(id, config_json)?;
        let (pid, started_ms) = {
            let ch = self.children.lock();
            let child = ch
                .get(id)
                .ok_or_else(|| "process missing after spawn".to_string())?;
            let pid = child.id();
            let started_ms = self.process_started_ms.lock().get(id).copied().unwrap_or(0);
            (pid, started_ms)
        };
        Ok(PluginProcessDebugInfo {
            plugin_id: id.to_string(),
            pid,
            rpc_url: url,
            started_at_ms: started_ms,
            cpu_percent: None,
            memory_kb: None,
        })
    }

    pub fn get_plugin_log_tail(&self, plugin_id: &str, lines: usize) -> Vec<String> {
        let id = plugin_id.trim();
        let ring = self.debug_log_rings.lock().get(id).cloned();
        let Some(ring) = ring else {
            return vec![];
        };
        let out = ring.lock().tail(lines.clamp(1, DEBUG_LOG_RING_CAP));
        out
    }

    pub fn clear_plugin_log_buffer(&self, plugin_id: &str) {
        let id = plugin_id.trim();
        let ring = self.debug_log_rings.lock().get(id).cloned();
        if let Some(ring) = ring {
            ring.lock().clear();
        }
    }

    #[must_use]
    pub fn list_managed_plugin_processes(&self) -> Vec<PluginProcessDebugInfo> {
        let urls = self.rpc_urls.lock();
        let times = self.process_started_ms.lock();
        let children = self.children.lock();
        let mut out: Vec<PluginProcessDebugInfo> = children
            .iter()
            .map(|(pid_key, child)| {
                let pid = child.id();
                let rpc_url = urls.get(pid_key).cloned().unwrap_or_default();
                let started_at_ms = times.get(pid_key).copied().unwrap_or(0);
                PluginProcessDebugInfo {
                    plugin_id: pid_key.clone(),
                    pid,
                    rpc_url,
                    started_at_ms,
                    cpu_percent: None,
                    memory_kb: None,
                }
            })
            .collect();
        out.sort_by(|a, b| a.plugin_id.cmp(&b.plugin_id));
        out
    }
}
