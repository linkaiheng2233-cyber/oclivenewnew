//! Directory plugin JSON-RPC: `theater.build_prompt`.

use crate::domain::builtin_theater_director::BuiltinTheaterDirector;
use crate::domain::error_helpers::serde_to_ollama;
use crate::error::Result;
use crate::infrastructure::high_risk_grants::HighRiskGrantStore;
use crate::infrastructure::remote_plugin::adapter::RemotePluginAdapterBlocking;
use crate::infrastructure::remote_plugin::config::RemotePluginHttpConfig;
use crate::infrastructure::remote_plugin::RemoteHttpClientBlocking;
use oclive_kernel_contracts::theater_director::{
    TheaterDirectorPromptProvider, TheaterPromptBuildInput, TheaterPromptBuildOutput,
    MAX_THEATER_PROMPT_LEN, THEATER_BUILD_PROMPT_METHOD,
};
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct RemoteTheaterPromptResult {
    prompt: String,
}

pub struct DirectoryTheaterDirector {
    adapter: RemotePluginAdapterBlocking,
    builtin: BuiltinTheaterDirector,
}

impl DirectoryTheaterDirector {
    /// # Errors
    ///
    /// Returns [`Err`] when the HTTP client cannot be built.
    pub fn new(
        cfg: RemotePluginHttpConfig,
        remote_fallback_allowed: Arc<AtomicBool>,
        high_risk_grants: Arc<HighRiskGrantStore>,
    ) -> std::result::Result<Self, reqwest::Error> {
        let http = RemoteHttpClientBlocking::new_standalone(cfg, high_risk_grants, None)?;
        Ok(Self {
            adapter: RemotePluginAdapterBlocking::from_http(http, remote_fallback_allowed),
            builtin: BuiltinTheaterDirector,
        })
    }
}

fn validate_prompt(prompt: &str) -> Result<String> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Err(crate::error::AppError::InvalidParameter(
            "theater.build_prompt returned empty prompt".to_string(),
        ));
    }
    if trimmed.len() > MAX_THEATER_PROMPT_LEN {
        return Err(crate::error::AppError::InvalidParameter(
            format!(
                "theater.build_prompt prompt too long ({} > {})",
                trimmed.len(),
                MAX_THEATER_PROMPT_LEN
            ),
        ));
    }
    Ok(trimmed.to_string())
}

impl TheaterDirectorPromptProvider for DirectoryTheaterDirector {
    fn build_prompt(&self, input: &TheaterPromptBuildInput) -> Result<TheaterPromptBuildOutput> {
        let params = serde_json::to_value(input).map_err(|e| {
            crate::error::AppError::Unknown(format!("theater.build_prompt encode: {e}"))
        })?;
        self.adapter.call_with_builtin_fallback(
            THEATER_BUILD_PROMPT_METHOD,
            params,
            |v| {
                let out: RemoteTheaterPromptResult = serde_json::from_value(v)
                    .map_err(|e| serde_to_ollama("theater.build_prompt decode", e))?;
                let prompt = validate_prompt(&out.prompt)?;
                Ok(TheaterPromptBuildOutput { prompt })
            },
            || self.builtin.build_prompt(input),
        )
    }
}
