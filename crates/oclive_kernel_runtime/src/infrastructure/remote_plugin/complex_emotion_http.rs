//! JSON-RPC：`complex_emotion.resolve_turn`（Remote 专用端点，与通用 `OCLIVE_REMOTE_PLUGIN_URL` 分离）。

use crate::domain::complex_emotion::{
    BuiltinKeywordComplexEmotionProvider, ComplexEmotionInput, ComplexEmotionOutput,
};
use crate::error::Result;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::jsonrpc::{self, RemoteRpcChannel};

const METHOD_RESOLVE_TURN: &str = "complex_emotion.resolve_turn";

pub struct RemoteComplexEmotionHttp {
    client: reqwest::blocking::Client,
    cfg: RemotePluginHttpConfig,
    fallback: BuiltinKeywordComplexEmotionProvider,
}

impl RemoteComplexEmotionHttp {
    pub fn new(cfg: RemotePluginHttpConfig) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(cfg.timeout)
            .connect_timeout(cfg.connect_timeout())
            .build()
            .expect("reqwest blocking client for complex_emotion");
        Self {
            client,
            cfg,
            fallback: BuiltinKeywordComplexEmotionProvider,
        }
    }

    pub fn resolve_turn(&self, input: &ComplexEmotionInput) -> Result<ComplexEmotionOutput> {
        let params = serde_json::to_value(input).map_err(|e| {
            crate::error::AppError::OllamaError(format!("complex_emotion params json: {}", e))
        })?;
        match jsonrpc::call_blocking(
            RemoteRpcChannel::Plugin,
            &self.client,
            &self.cfg.endpoint,
            METHOD_RESOLVE_TURN,
            params,
            self.cfg.bearer_token.as_deref(),
        ) {
            Ok(v) => {
                let mut out: ComplexEmotionOutput = serde_json::from_value(v).map_err(|e| {
                    crate::error::AppError::OllamaError(format!(
                        "complex_emotion result decode: {}",
                        e
                    ))
                })?;
                out.degraded_to_builtin = false;
                Ok(out)
            }
            Err(e) => {
                log::warn!(
                    target: "oclive_plugin",
                    "complex_emotion.resolve_turn remote failed endpoint={} err={}; fallback=builtin",
                    self.cfg.endpoint,
                    e
                );
                let mut o = self.fallback.resolve_turn_inner(input);
                o.degraded_to_builtin = true;
                Ok(o)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_name_matches_doc() {
        assert_eq!(METHOD_RESOLVE_TURN, "complex_emotion.resolve_turn");
    }
}
