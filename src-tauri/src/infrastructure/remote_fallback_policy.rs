//! 远端 HTTP 插件失败时是否静默降级内置（与 `app_settings.remote_fallback_to_builtin` 及 `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN` 对齐）。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const ENV_REMOTE_FALLBACK: &str = "OCLIVE_REMOTE_FALLBACK_TO_BUILTIN";

#[must_use]
pub fn new_remote_fallback_switch(initial: bool) -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(initial))
}

/// 自数据库读出的原始值 → 是否允许降级（缺省 / 无法解析 → `true`）。
#[must_use]
pub fn remote_fallback_from_db_value(raw: Option<String>) -> bool {
    !matches!(
        raw.as_deref().map(|s| s.trim().to_ascii_lowercase()),
        Some(s) if matches!(s.as_str(), "0" | "false" | "no" | "off")
    )
}

/// 环境变量覆盖（若设置）：`0`/`false`/`no`/`off` → 不允许降级；`1`/`true`/`yes`/`on` → 允许。
#[must_use]
pub fn remote_fallback_env_override() -> Option<bool> {
    std::env::var(ENV_REMOTE_FALLBACK).ok().and_then(|v| {
        let t = v.trim().to_ascii_lowercase();
        match t.as_str() {
            "0" | "false" | "no" | "off" => Some(false),
            "1" | "true" | "yes" | "on" => Some(true),
            _ => None,
        }
    })
}

#[inline]
pub fn remote_fallback_load(flag: &Arc<AtomicBool>) -> bool {
    flag.load(Ordering::Relaxed)
}
