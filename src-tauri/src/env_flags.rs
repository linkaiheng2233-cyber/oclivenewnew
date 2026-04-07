//! 应用级布尔环境开关（与按数值读取的 `infrastructure/llm_params`、`ollama_timeouts` 等区分）。

/// `1` / `true` / `yes` / `on`（ASCII 不区分大小写）视为开启；未设置或其它值视为关闭。
#[must_use]
pub fn env_flag_enabled(key: &str) -> bool {
    match std::env::var(key) {
        Ok(v) => {
            let t = v.trim();
            t == "1"
                || t.eq_ignore_ascii_case("true")
                || t.eq_ignore_ascii_case("yes")
                || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// `list_roles` 是否包含 `manifest.dev_only == true` 的包（`OCLIVE_LIST_DEV_ROLES`）。
#[must_use]
pub fn list_dev_roles_enabled() -> bool {
    env_flag_enabled("OCLIVE_LIST_DEV_ROLES")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_flag_enabled_respects_truthy_tokens() {
        let key = "OCLIVE_ENV_FLAGS_TEST_XY9";
        std::env::remove_var(key);
        assert!(!env_flag_enabled(key));
        for v in ["1", "true", "TRUE", "yes", "on"] {
            std::env::set_var(key, v);
            assert!(env_flag_enabled(key), "v={v}");
        }
        std::env::set_var(key, "0");
        assert!(!env_flag_enabled(key));
        std::env::remove_var(key);
    }
}
