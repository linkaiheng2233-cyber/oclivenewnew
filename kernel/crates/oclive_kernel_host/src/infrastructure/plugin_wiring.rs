//! Constructs [`PluginHost`](crate::domain::plugin_host::PluginHost) with infrastructure dependencies.

use crate::domain::plugin_host::PluginHost;
use crate::domain::ports::LlmClient;
use crate::infrastructure::backend_registry::BackendRegistry;
use crate::infrastructure::directory_plugins::DirectoryPluginRuntime;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::llm::MockLlmClient;
use crate::infrastructure::remote_fallback_policy::new_remote_fallback_switch;
use oclive_kernel_contracts::PluginBackendRegistryPort;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Builds a [`PluginHost`] from runtime handles (production path).
#[must_use]
pub fn build_plugin_host(
    llm: Arc<dyn LlmClient>,
    directory_runtime: Option<Arc<DirectoryPluginRuntime>>,
    app_data_dir: PathBuf,
    high_risk_grants: Arc<HighRiskGrantStore>,
    remote_fallback_allowed: Arc<AtomicBool>,
) -> PluginHost {
    let registry = Arc::new(BackendRegistry::from_runtime(
        llm,
        directory_runtime,
        app_data_dir,
        high_risk_grants,
        remote_fallback_allowed,
    )) as Arc<dyn PluginBackendRegistryPort>;
    PluginHost::from_registry(registry)
}

/// Test / demo host with mock LLM and permissive grants (no directory runtime).
#[must_use]
pub fn test_plugin_host() -> PluginHost {
    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let tmp = std::env::temp_dir();
    let grants = HighRiskGrantStore::load(tmp.clone(), false);
    let remote_fb = new_remote_fallback_switch(true);
    build_plugin_host(llm, None, tmp, grants, remote_fb)
}
