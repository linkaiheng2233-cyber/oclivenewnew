//! Process-local LLM token cache + app_data file backup port.

use std::path::Path;

/// Cloud LLM API token resolution (DB row → process cache → file backup).
pub trait UserLlmSecretsPort: Send + Sync {
    fn set_cached_remote_llm_token(&self, token: Option<String>);
    fn cached_remote_llm_token(&self) -> Option<String>;
    /// Persist cloud LLM token to `{app_data}` backup file.
    ///
    /// # Errors
    ///
    /// Returns I/O or path errors as strings.
    fn write_token_file(&self, app_data: &Path, token: &str) -> Result<(), String>;
    fn read_token_file(&self, app_data: &Path) -> Option<String>;
}
