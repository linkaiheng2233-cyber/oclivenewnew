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
        let mut md = String::from("# 自定义编排 (PIPELINE_CUSTOM)\n\n");
        md.push_str(&format!("模式: `{self:?}`\n\n## 步骤顺序\n\n"));
        for (i, s) in steps.iter().enumerate() {
            md.push_str(&format!("{}. `{}`\n", i + 1, s));
        }
        if self == Self::EmotionFirst {
            md.push_str("\n> 与 default 相比，本模式在文档与生成注释中强调 **情绪分析优先于事件检测** 的产品语义；完整宿主仍以 oclivenewnew `process_message` 为准。\n");
        }
        if self == Self::MemoryLast {
            md.push_str("\n> **memory_rank** 在文档中置于 **llm_generate** 之后，供实验性「先对话后检索」架构探索。\n");
        }
        md
    }
}
