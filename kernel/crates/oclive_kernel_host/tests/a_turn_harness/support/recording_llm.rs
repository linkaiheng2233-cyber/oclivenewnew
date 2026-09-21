//! 录制型 `LlmClient` 替身（补充约束 §3）：允许的方法直接返回脚本结果，不隐式委派；
//! 未预期方法记违规并使场景 FAIL；脚本耗尽不得自动变成另一条成功回复。
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::artifacts;
use async_trait::async_trait;
use oclive_kernel_contracts::{LlmClient, LlmGenerateOpts, LlmGenerateOutcome, LlmTokenSink};
use oclive_kernel_types::{error::AppError, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum Step {
    Ok(String),
    Err(String),
}

#[derive(Debug, Default)]
struct State {
    seq: usize,
    script_index: usize,
    violations: Vec<String>,
    capability_queries: usize,
    calls: Vec<serde_json::Value>,
}

pub struct RecordingLlm {
    scenario: String,
    script: Vec<Step>,
    artifacts_dir: PathBuf,
    state: Mutex<State>,
}

impl RecordingLlm {
    #[must_use]
    pub fn new(scenario: &str, script: Vec<Step>, artifacts_dir: &Path) -> Self {
        Self {
            scenario: scenario.to_string(),
            script,
            artifacts_dir: artifacts_dir.to_path_buf(),
            state: Mutex::new(State::default()),
        }
    }

    fn record(&self, method: &str, model: &str, prompt: &str, extra: serde_json::Value) {
        let mut st = self.state.lock().unwrap();
        st.seq += 1;
        let seq = st.seq;
        let prompt_sha = artifacts::sha256_hex(prompt.as_bytes());
        let mut rec = serde_json::json!({
            "seq": seq,
            "scenario": self.scenario,
            "method": method,
            "model": model,
            "prompt_sha256": prompt_sha,
            "prompt_bytes": prompt.len(),
        });
        if let (serde_json::Value::Object(dst), serde_json::Value::Object(src)) = (&mut rec, extra)
        {
            for (k, v) in src {
                dst.insert(k, v);
            }
        }
        st.calls.push(rec.clone());
        let _ = artifacts::append_jsonl(&self.artifacts_dir.join("calls.jsonl"), &rec);
        // 完整合成 prompt 落盘（材料完全合成，不含用户数据）。
        let _ = artifacts::append_jsonl(
            &self.artifacts_dir.join("prompts.jsonl"),
            &serde_json::json!({
                "seq": seq,
                "method": method,
                "model": model,
                "prompt": prompt,
            }),
        );
    }

    fn violation(&self, what: impl Into<String>) {
        let msg = what.into();
        let mut st = self.state.lock().unwrap();
        st.violations.push(msg.clone());
        let _ = artifacts::append_jsonl(
            &self.artifacts_dir.join("violations.jsonl"),
            &serde_json::json!({ "violation": msg, "scenario": self.scenario }),
        );
    }

    fn take_step(&self) -> Option<Step> {
        let mut st = self.state.lock().unwrap();
        let idx = st.script_index;
        st.script_index += 1;
        self.script.get(idx).cloned()
    }

    #[must_use]
    pub fn violations(&self) -> Vec<String> {
        self.state.lock().unwrap().violations.clone()
    }

    #[must_use]
    pub fn calls(&self) -> Vec<serde_json::Value> {
        self.state.lock().unwrap().calls.clone()
    }

    #[must_use]
    pub fn capability_queries(&self) -> usize {
        self.state.lock().unwrap().capability_queries
    }

    /// 方法级账（生成类方法计数；能力查询单列，不与生成长度混算）。
    #[must_use]
    pub fn method_counts(&self) -> serde_json::Value {
        let st = self.state.lock().unwrap();
        let mut counts = serde_json::Map::new();
        for c in &st.calls {
            let m = c
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let cur = counts
                .get(&m)
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0);
            counts.insert(m, serde_json::Value::from(cur + 1));
        }
        serde_json::json!({
            "generation_methods": serde_json::Value::Object(counts),
            "capability_queries_supports_prefix_cache": st.capability_queries,
            "violations": st.violations,
            "script_steps": self.script.len(),
        })
    }
}

fn opts_record(opts: Option<&LlmGenerateOpts>) -> serde_json::Value {
    match opts {
        None => serde_json::json!({ "some": false }),
        Some(o) => serde_json::json!({
            "some": true,
            "keep_alive": o.keep_alive,
            "want_metrics": o.want_metrics,
            "temperature": o.temperature,
            "top_p": o.top_p,
            "max_output_tokens": o.max_output_tokens,
            "preferred_context_tokens": o.preferred_context_tokens,
        }),
    }
}

#[async_trait]
impl LlmClient for RecordingLlm {
    async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        self.violation("unexpected call: generate（本片主生成只允许 generate_with_opts）");
        self.record(
            "generate",
            model,
            prompt,
            serde_json::json!({ "outcome": "violation" }),
        );
        Err(AppError::Unknown(
            "A-HARNESS-UNEXPECTED-METHOD-generate".into(),
        ))
    }

    async fn generate_tag(&self, model: &str, prompt: &str) -> Result<String> {
        self.violation("unexpected call: generate_tag（Event/Portrait 标签路径应已关闭）");
        self.record(
            "generate_tag",
            model,
            prompt,
            serde_json::json!({ "outcome": "violation" }),
        );
        Err(AppError::Unknown(
            "A-HARNESS-UNEXPECTED-METHOD-generate_tag".into(),
        ))
    }

    async fn generate_with_opts(
        &self,
        model: &str,
        prompt: &str,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let opts_rec = opts_record(opts);
        if opts.is_none() {
            self.violation("generate_with_opts 收到 opts=None（主生成路径应始终给 Some）");
        }
        let step = self.take_step();
        let extra = match &step {
            Some(Step::Ok(text)) => serde_json::json!({
                "opts": opts_rec,
                "outcome": "ok",
                "reply_sha256": artifacts::sha256_hex(text.as_bytes()),
                "reply_bytes": text.len(),
            }),
            Some(Step::Err(_)) => serde_json::json!({ "opts": opts_rec, "outcome": "err" }),
            None => serde_json::json!({ "opts": opts_rec, "outcome": "script_exhausted" }),
        };
        self.record("generate_with_opts", model, prompt, extra);
        match step {
            Some(Step::Ok(text)) => Ok(LlmGenerateOutcome {
                reply: text,
                prompt_eval_ms: None,
            }),
            Some(Step::Err(marker)) => Err(AppError::OllamaError(marker)),
            None => {
                self.violation(
                    "script_exhausted：出现脚本外的额外主生成调用（不得自动变成另一条成功回复）",
                );
                Err(AppError::Unknown("A-HARNESS-SCRIPT-EXHAUSTED".into()))
            }
        }
    }

    async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
    ) -> Result<String> {
        let _ = on_token;
        self.violation("unexpected call: generate_stream（本片非流式）");
        self.record(
            "generate_stream",
            model,
            prompt,
            serde_json::json!({ "outcome": "violation" }),
        );
        Err(AppError::Unknown(
            "A-HARNESS-UNEXPECTED-METHOD-generate_stream".into(),
        ))
    }

    async fn generate_stream_with_opts(
        &self,
        model: &str,
        prompt: &str,
        on_token: LlmTokenSink,
        opts: Option<&LlmGenerateOpts>,
    ) -> Result<LlmGenerateOutcome> {
        let _ = on_token;
        self.violation("unexpected call: generate_stream_with_opts（本片非流式）");
        self.record(
            "generate_stream_with_opts",
            model,
            prompt,
            serde_json::json!({ "opts": opts_record(opts), "outcome": "violation" }),
        );
        Err(AppError::Unknown(
            "A-HARNESS-UNEXPECTED-METHOD-generate_stream_with_opts".into(),
        ))
    }

    async fn startup_probe(&self) -> Result<()> {
        self.violation(
            "unexpected call: startup_probe（本片 OCLIVE_SKIP_LLM_STARTUP_PROBE=1，期望 0）",
        );
        self.record(
            "startup_probe",
            "",
            "",
            serde_json::json!({ "outcome": "violation" }),
        );
        Ok(())
    }

    fn supports_prefix_cache(&self) -> bool {
        self.state.lock().unwrap().capability_queries += 1;
        false
    }
}
