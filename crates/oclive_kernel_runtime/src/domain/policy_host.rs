//! 宿主 `policy.toml` 热重载提示文案（与桌面 `reload_policy_plugins` 一致）。

use crate::error::Result;
use crate::state::KernelAppState;

pub fn reload_policy_plugins_message(state: &KernelAppState) -> Result<String> {
    let count = state.reload_policy_plugins()?;
    Ok(format!("policy plugins reloaded: {} scene bindings", count))
}
