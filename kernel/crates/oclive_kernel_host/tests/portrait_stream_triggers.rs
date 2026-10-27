//! Deterministic real-Host turn tests. Scripted model output proves wiring,
//! not that a stochastic model will choose a particular expression on demand.
#![allow(clippy::expect_used)]

use async_trait::async_trait;
use oclive_kernel_contracts::{LlmClient, LlmTokenSink};
use oclive_kernel_host::{OcliveKernel, OcliveKernelConfig};
use oclive_kernel_types::models::dto::SendMessageRequest;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct ScriptedLlm(Mutex<String>);

#[async_trait]
impl LlmClient for ScriptedLlm {
    async fn generate(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
        Ok(self.0.lock().expect("scripted reply").clone())
    }

    async fn generate_tag(&self, _: &str, _: &str) -> oclive_kernel_types::Result<String> {
        Ok("{}".into())
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        sink: LlmTokenSink,
    ) -> oclive_kernel_types::Result<String> {
        let raw = self.generate(model, prompt).await?;
        // Worst-case delimiter fragmentation: one Unicode character per token.
        for character in raw.chars() {
            sink(&character.to_string());
        }
        Ok(raw)
    }
}

/// Opt-in local model probe; no paid service and no user's existing database.
/// Recorded choices are observations, not deterministic semantic assertions.
#[tokio::test]
#[ignore = "requires local Ollama qwen2.5:7b and an explicit output file"]
async fn local_ollama_expression_observations() {
    use oclive_kernel_contracts::{LlmGenerateOpts, LlmGenerateOutcome};
    use oclive_kernel_host::infrastructure::ollama_client::OllamaClient;
    struct RecordingModel {
        inner: OllamaClient,
        raw: Mutex<Vec<String>>,
        prompts: Mutex<Vec<String>>,
    }
    #[async_trait]
    impl LlmClient for RecordingModel {
        async fn generate(&self, model: &str, prompt: &str) -> oclive_kernel_types::Result<String> {
            LlmClient::generate(&self.inner, model, prompt).await
        }
        async fn generate_tag(
            &self,
            model: &str,
            prompt: &str,
        ) -> oclive_kernel_types::Result<String> {
            self.inner.generate_tag(model, prompt).await
        }
        async fn generate_stream_with_opts(
            &self,
            model: &str,
            prompt: &str,
            sink: LlmTokenSink,
            opts: Option<&LlmGenerateOpts>,
        ) -> oclive_kernel_types::Result<LlmGenerateOutcome> {
            self.prompts
                .lock()
                .expect("main prompt")
                .push(prompt.to_string());
            let reply = self
                .inner
                .generate_stream_with_opts(model, prompt, sink, opts)
                .await?;
            self.raw
                .lock()
                .expect("raw output")
                .push(reply.reply.clone());
            Ok(reply)
        }
    }
    let output =
        std::env::var("OCLIVE_EMOTION_LIVE_OUTPUT").expect("explicit observation file required");
    let data = tempfile::tempdir().expect("isolated data");
    let roles = oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
        "CARGO_MANIFEST_DIR"
    ))])
    .expect("official roles");
    let model = Arc::new(RecordingModel {
        inner: OllamaClient::new("http://127.0.0.1:11434"),
        raw: Mutex::new(Vec::new()),
        prompts: Mutex::new(Vec::new()),
    });
    let kernel = OcliveKernel::builder(OcliveKernelConfig::new(data.path(), roles))
        .with_llm_client(model.clone())
        .build()
        .await
        .expect("local probe Host");
    let mut results = Vec::new();
    let selected_scene = std::env::var("OCLIVE_EMOTION_LIVE_SCENE").ok();
    for (expected, message) in [
        (
            "neutral",
            "沐沐，今天先安静地坐一会儿，和我简单打个招呼吧。",
        ),
        (
            "happy",
            "沐沐，你准备的生日惊喜太棒了，我很喜欢！现在可以开心地笑一笑啦。",
        ),
        (
            "sad",
            "请演一下难过的沐沐：你最珍惜的小花枯萎了，低着头告诉我你的心情。",
        ),
        (
            "angry",
            "请演一下生气的沐沐：有人故意毁掉你珍惜的作品，请认真表达你的不满。",
        ),
        (
            "excited",
            "沐沐！你抽中了最想要的大奖，正抱着奖品高兴地蹦起来，兴奋得说话都停不下来。请用这个瞬间的心情回应我！",
        ),
        (
            "confused",
            "请演一下困惑的沐沐：一只猫同时站在互不连通的两个房间里，你完全没弄懂怎么回事。",
        ),
        (
            "shy",
            "沐沐，你真的很可爱，认真笑起来的样子特别好看。请演一下被夸得不好意思、害羞的样子。",
        ),
    ] {
        if selected_scene.as_deref().is_some_and(|selected| selected != expected) {
            continue;
        }
        let visible = Arc::new(Mutex::new(String::new()));
        let target = visible.clone();
        let response = kernel
            .process_message_stream(
                &SendMessageRequest {
                    role_id: "mumu".into(),
                    user_message: message.into(),
                    session_id: Some(format!("live-{expected}")),
                    ..Default::default()
                },
                Arc::new(move |token| target.lock().expect("visible stream").push_str(token)),
            )
            .await
            .expect("local turn");
        let wire = visible.lock().expect("visible stream").clone();
        assert!(!wire.contains("[EMO]"));
        assert!(!wire.contains("portrait_hint"));
        assert!(!response.reply.trim().is_empty());
        results.push(serde_json::json!({"expected_scene":expected,"message":message,"wire":wire,
            "reply":response.reply,"bot_emotion":response.bot_emotion,"portrait_emotion":response.portrait_emotion,
            "visual_state_id":response.visual_state_id,"fallback":response.reply_is_fallback,
            "raw_main_reply":model.raw.lock().expect("raw output").last().cloned(),
            "main_prompt":model.prompts.lock().expect("main prompt").last().cloned()}));
        std::fs::write(
            &output,
            serde_json::to_vec_pretty(&results).expect("observation JSON"),
        )
        .expect("save local observations");
        eprintln!(
            "scene={expected}; portrait={}; fallback={}",
            response.portrait_emotion, response.reply_is_fallback
        );
    }
}

#[tokio::test]
async fn all_mumu_expressions_and_intensities_reach_the_response_without_stream_metadata() {
    let data = tempfile::tempdir().expect("isolated app data");
    let roles = oclive_kernel_runtime::chat_pro_roles_dir(&[std::path::PathBuf::from(env!(
        "CARGO_MANIFEST_DIR"
    ))])
    .expect("official roles, read only");
    let llm = Arc::new(ScriptedLlm::default());
    let kernel = OcliveKernel::builder(OcliveKernelConfig::new(data.path(), &roles))
        .with_llm_client(llm.clone())
        .build()
        .await
        .expect("isolated Host");
    for emotion in [
        "happy", "sad", "angry", "neutral", "excited", "confused", "shy",
    ] {
        for (level, intensity) in [("mild", 0.2), ("moderate", 0.5), ("severe", 0.9)] {
            *llm.0.lock().expect("scripted reply") = format!(
                "我明白了。[EMO]{{\"labels\":[\"joy\"],\"intensity\":{intensity},\"portrait_hint\":\"{emotion}\"}}[/EMO]");
            let wire = Arc::new(Mutex::new(String::new()));
            let target = wire.clone();
            let response = kernel
                .process_message_stream(
                    &SendMessageRequest {
                        role_id: "mumu".into(),
                        user_message: "请继续。".into(),
                        session_id: Some(format!("portrait-{emotion}-{level}")),
                        ..Default::default()
                    },
                    Arc::new(move |token| target.lock().expect("wire").push_str(token)),
                )
                .await
                .expect("streaming turn");
            assert_eq!(wire.lock().expect("wire").as_str(), "我明白了。");
            assert_eq!(response.reply, "我明白了。");
            assert_eq!(
                response.bot_emotion, "happy",
                "display hint must not overwrite emotion policy"
            );
            assert_eq!(response.portrait_emotion, emotion);
            let id = format!("{emotion}_{level}");
            assert_eq!(response.visual_state_id.as_deref(), Some(id.as_str()));
            let directive =
                serde_json::to_value(response.performance_directive.expect("image directive"))
                    .expect("serialize");
            let path = format!("assets/images/{id}.png");
            assert_eq!(directive["path"], path);
            let bytes = std::fs::read(roles.join("mumu").join(&path)).expect("catalog file exists");
            assert!(
                bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
                "valid PNG header: {path}"
            );
        }
    }
    for (label, expected) in [
        ("joy", "happy"),
        ("sadness", "sad"),
        ("anger", "angry"),
        ("fear", "confused"),
        ("surprise", "excited"),
        ("disgust", "confused"),
        ("neutral", "neutral"),
    ] {
        *llm.0.lock().expect("scripted reply") = format!("这是 {label} 对应的独立测试台词。[EMO]{{\"labels\":[\"{label}\"],\"intensity\":0.5,\"portrait_hint\":\"invalid\"}}[/EMO]");
        let response = kernel
            .process_message(&SendMessageRequest {
                role_id: "mumu".into(),
                user_message: "继续。".into(),
                session_id: Some(format!("legacy-{label}")),
                ..Default::default()
            })
            .await
            .expect("fallback turn");
        assert_eq!(response.bot_emotion, expected);
        assert_eq!(response.portrait_emotion, expected);
        assert_eq!(
            response.visual_state_id,
            Some(format!("{expected}_moderate"))
        );
    }
    *llm.0.lock().expect("scripted reply") = "有点不好意思呢。[EMO]{\"labels\":[\"shy\"],\"intensity\":0.5,\"portrait_hint\":\"shy\"}[/EMO]".into();
    let independent = kernel
        .process_message(&SendMessageRequest {
            role_id: "mumu".into(),
            user_message: "你很可爱。".into(),
            session_id: Some("independent-display".into()),
            ..Default::default()
        })
        .await
        .expect("independent hint turn");
    assert_eq!(independent.portrait_emotion, "shy");
    assert_eq!(independent.visual_state_id.as_deref(), Some("shy_moderate"));
    assert_ne!(independent.bot_emotion, "shy");
}
