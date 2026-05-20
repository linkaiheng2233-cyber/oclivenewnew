//! 自定义编排 `--pipeline` 文档生成。

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum PipelineArg {
    #[default]
    Default,
    EmotionFirst,
    MemoryLast,
}

impl PipelineArg {
    pub fn steps(self) -> &'static [&'static str] {
        match self {
            Self::Default => &[
                "load_recent_context",
                "user_emotion_analyze",
                "event_estimate",
                "memory_rank",
                "build_prompt",
                "llm_generate",
                "postprocess",
            ],
            Self::EmotionFirst => &[
                "load_recent_context",
                "user_emotion_analyze",
                "memory_rank",
                "event_estimate",
                "build_prompt",
                "llm_generate",
                "postprocess",
            ],
            Self::MemoryLast => &[
                "load_recent_context",
                "user_emotion_analyze",
                "event_estimate",
                "build_prompt",
                "llm_generate",
                "memory_rank",
                "postprocess",
            ],
        }
    }

    pub fn doc_markdown(self) -> String {
        let steps = self.steps();
        let mut md = String::from("# Custom pipeline (PIPELINE_CUSTOM)\n\n");
        md.push_str(&format!("Mode: `{self:?}`\n\n## Step order\n\n"));
        for (i, s) in steps.iter().enumerate() {
            md.push_str(&format!("{}. `{}`\n", i + 1, s));
        }
        if self == Self::EmotionFirst {
            md.push_str("\n> Compared to default, this mode documents **emotion analysis before event detection**; full host behavior is still oclivenewnew `process_message`.\n");
        }
        if self == Self::MemoryLast {
            md.push_str("\n> **memory_rank** is documented after **llm_generate** for experimental dialogue-first retrieval.\n");
        }
        md
    }
}
