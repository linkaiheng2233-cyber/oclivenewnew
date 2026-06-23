//! JSON-RPC: `reply_post_process.process` (role pack `reply_post_processor.backend = remote`).

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
use oclive_validation::NETWORK_GRANT_REMOTE_PLUGIN;
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

pub struct RemoteReplyPostProcessorHttp {
    adapter: RemotePluginAdapterBlocking,
    builtin: BuiltinReplyPostProcessor,
}

impl RemoteReplyPostProcessorHttp {
    /// # Errors
    ///
    /// Returns [`Err`] when the HTTP client cannot be built.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
        builtin_cfg: RolePackBuiltinReplyPostProcessorConfig,
    ) -> std::result::Result<Self, reqwest::Error> {
        let http = RemoteHttpClientBlocking::new_standalone(
            cfg,
            high_risk_grants,
            Some(NETWORK_GRANT_REMOTE_PLUGIN.to_string()),
        )?;
        Ok(Self {
            adapter: RemotePluginAdapterBlocking::from_http(http, remote_fallback_allowed),
            builtin: BuiltinReplyPostProcessor::new(builtin_cfg),
        })
    }
}

impl ReplyPostProcessor for RemoteReplyPostProcessorHttp {
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

#[cfg(test)]
mod integration_mock {
    use super::*;
    use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
    use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
    use oclive_kernel_contracts::reply_post_processor::{PostProcessInput, ReplyPostProcessor};
    use oclive_kernel_types::models::RolePackBuiltinReplyPostProcessorConfig;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn spawn_mock_reply_rpc() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let url = format!("http://{addr}/rpc");
        thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut buf = vec![0u8; 8192];
                let _ = stream.read(&mut buf);
                let body = r#"{"jsonrpc":"2.0","id":1,"result":{"display_reply":"[remote] polished","diagnostic":"mock"}}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nx-oclive-remote-protocol: oclive-remote-jsonrpc-v1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        thread::sleep(Duration::from_millis(30));
        url
    }

    #[test]
    fn remote_http_returns_display_reply() {
        let url = spawn_mock_reply_rpc();
        let cfg =
            RemotePluginHttpConfig::for_reply_post_processor_remote(&url, Some(3000)).expect("cfg");
        let http = RemoteReplyPostProcessorHttp::new(
            cfg,
            Arc::new(AtomicBool::new(true)),
            HighRiskGrantStore::load(std::env::temp_dir(), false),
            RolePackBuiltinReplyPostProcessorConfig::default(),
        )
        .expect("client");
        let out = http
            .process_reply(PostProcessInput {
                raw_reply: "raw",
                user_message: "hi",
                role_id: "r",
                scene_id: "s",
                srid: "r",
                locale: "zh",
            })
            .expect("process");
        assert_eq!(out.display_reply, "[remote] polished");
    }
}
