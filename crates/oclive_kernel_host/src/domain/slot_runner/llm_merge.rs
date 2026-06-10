use crate::domain::ports::LlmClient;
use crate::error::{AppError, Result};
use std::sync::Arc;

fn no_reply_error() -> AppError {
    crate::domain::error_helpers::ollama_msg("llm", "no slot produced a reply")
}

pub(super) async fn fallback_first(
    instances: &[(String, Arc<dyn LlmClient>)],
    ollama_model: &str,
    prompt: &str,
) -> Result<String> {
    let mut last_err = None;
    for (key, llm) in instances {
        match llm.generate(ollama_model, prompt).await {
            Ok(reply) => {
                tracing::info!(
                    target: "oclive_plugin",
                    slot_key = %key,
                    reply_len = reply.len(),
                    "llm_generate slot (fallback first success)"
                );
                return Ok(reply);
            }
            Err(e) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    slot_key = %key,
                    err = %e,
                    "llm_generate slot failed (fallback)"
                );
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(no_reply_error))
}

pub(super) async fn fastest_wins(
    instances: &[(String, Arc<dyn LlmClient>)],
    ollama_model: &str,
    prompt: &str,
) -> Result<String> {
    if instances.len() == 1 {
        return instances[0].1.generate(ollama_model, prompt).await;
    }
    let mut set = tokio::task::JoinSet::new();
    for (key, llm) in instances {
        let key = key.clone();
        let llm = Arc::clone(llm);
        let model = ollama_model.to_string();
        let prompt = prompt.to_string();
        set.spawn(async move {
            let reply = llm.generate(&model, &prompt).await?;
            Ok::<_, AppError>((key, reply))
        });
    }
    let mut last_err: Option<AppError> = None;
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(Ok((key, reply))) => {
                set.abort_all();
                tracing::info!(
                    target: "oclive_plugin",
                    slot_key = %key,
                    reply_len = reply.len(),
                    "llm_generate slot (fastest-wins)"
                );
                return Ok(reply);
            }
            Ok(Err(e)) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    err = %e,
                    "llm_generate slot failed (fastest-wins)"
                );
                last_err = Some(e);
            }
            Err(join_err) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    err = %join_err,
                    "llm_generate join failed (fastest-wins)"
                );
            }
        }
    }
    Err(last_err.unwrap_or_else(no_reply_error))
}

pub(super) async fn serial_last_wins(
    instances: &[(String, Arc<dyn LlmClient>)],
    ollama_model: &str,
    prompt: &str,
) -> Result<String> {
    if instances.len() == 1 {
        return instances[0].1.generate(ollama_model, prompt).await;
    }
    let mut last = String::new();
    let mut any_ok = false;
    for (key, llm) in instances {
        match llm.generate(ollama_model, prompt).await {
            Ok(reply) => {
                tracing::info!(
                    target: "oclive_plugin",
                    slot_key = %key,
                    reply_len = reply.len(),
                    "llm_generate slot (serial; last-wins)"
                );
                last = reply;
                any_ok = true;
            }
            Err(e) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    slot_key = %key,
                    err = %e,
                    "llm_generate slot failed"
                );
            }
        }
    }
    if any_ok {
        Ok(last)
    } else {
        Err(no_reply_error())
    }
}

pub(super) async fn serial_last_wins_stream(
    instances: &[(String, Arc<dyn LlmClient>)],
    ollama_model: &str,
    prompt: &str,
    on_token: oclive_kernel_contracts::LlmTokenSink,
) -> Result<String> {
    if instances.is_empty() {
        return Err(no_reply_error());
    }
    if instances.len() == 1 {
        return instances[0]
            .1
            .generate_stream(ollama_model, prompt, on_token)
            .await;
    }
    let mut last = String::new();
    let mut any_ok = false;
    for (i, (key, llm)) in instances.iter().enumerate() {
        let is_final = i + 1 == instances.len();
        let result = if is_final {
            llm.generate_stream(ollama_model, prompt, Arc::clone(&on_token))
                .await
        } else {
            llm.generate(ollama_model, prompt).await
        };
        match result {
            Ok(reply) => {
                tracing::info!(
                    target: "oclive_plugin",
                    slot_key = %key,
                    reply_len = reply.len(),
                    "llm_generate stream slot (serial; last-wins)"
                );
                last = reply;
                any_ok = true;
            }
            Err(e) => {
                tracing::warn!(
                    target: "oclive_plugin",
                    slot_key = %key,
                    err = %e,
                    "llm_generate stream slot failed"
                );
            }
        }
    }
    if any_ok {
        Ok(last)
    } else {
        Err(no_reply_error())
    }
}
