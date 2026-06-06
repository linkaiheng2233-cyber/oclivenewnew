//! Directory plugin JSON-RPC: `reply_post_process.process`.

use crate::domain::builtin_reply_post_processor::BuiltinReplyPostProcessor;
use crate::domain::error_helpers::serde_to_ollama;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::RemotePluginAdapterBlocking;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use oclive_kernel_contracts::reply_post_processor::{
    PostProcessInput, PostProcessOutput, ReplyPostProcessor,
};
use oclive_kernel_types::models::RolePackBuiltinReplyPostProcessorConfig;
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

const METHOD_REPLY_POST_PROCESS: &str = "reply_post_process.process";

#[derive(Debug, Deserialize)]
struct RemotePostProcessResult {
    display_reply: String,
    #[serde(default)]
    diagnostic: Option<String>,
}

pub struct DirectoryReplyPostProcessor {
    adapter: RemotePluginAdapterBlocking,
    builtin: BuiltinReplyPostProcessor,
}

impl DirectoryReplyPostProcessor {
    /// # Errors
    ///
    /// Returns [`Err`] when the HTTP client cannot be built.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        builtin_cfg: RolePackBuiltinReplyPostProcessorConfig,
    ) -> std::result::Result<Self, reqwest::Error> {
        let http = RemoteHttpClientBlocking::new_standalone(cfg, high_risk_grants, None)?;
        Ok(Self {
            adapter: RemotePluginAdapterBlocking::from_http(http, remote_fallback_allowed),
            builtin: BuiltinReplyPostProcessor::new(builtin_cfg),
        })
    }
}

impl ReplyPostProcessor for DirectoryReplyPostProcessor {
    fn process_reply(&self, input: PostProcessInput<'_>) -> Result<PostProcessOutput> {
        let params = serde_json::json!({
            "raw_reply": input.raw_reply,
            "user_message": input.user_message,
            "role_id": input.role_id,
            "scene_id": input.scene_id,
            "locale": input.locale,
        });
        self.adapter.call_with_builtin_fallback(
            METHOD_REPLY_POST_PROCESS,
            params,
            |v| {
                let out: RemotePostProcessResult = serde_json::from_value(v)
                    .map_err(|e| serde_to_ollama("reply_post_process.process decode", e))?;
                Ok(PostProcessOutput {
                    display_reply: out.display_reply,
                    diagnostic: out.diagnostic,
                })
            },
            || self.builtin.process_reply(input),
        )
    }
}
