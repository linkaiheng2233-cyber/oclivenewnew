//! 蓝图条件分支的受限谓词（预置枚举，非任意表达式）。

use super::turn_context::TurnContext;
use crate::domain::emotion_analyzer::EmotionResultExt;
use crate::models::Emotion;
use oclive_kernel_core::models::EmotionResult;
use serde::Deserialize;

/// JSON `branch.predicate` 与运行时求值共用模型。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PipelinePredicate {
    /// `ctx.flags.agent_handled == Some(true)`（通常在 `run_agent` 之后才有意义）。
    AgentHandled,
    /// 当前有效场景 ID 与给定值完全一致（`TurnContext.scene.effective_scene_id`）。
    SceneIdEquals {
        #[serde(rename = "sceneId")]
        scene_id: String,
    },
    /// 七维向量中 **数值最大** 的一维名称与给定标识一致（如 `sadness`、`joy`）。
    EmotionDominant {
        #[serde(rename = "emotion")]
        emotion: String,
    },
}

impl PipelinePredicate {
    /// 求值；缺少上下文时返回 `false`（不中断流水线，除非调用方另行策略）。
    pub fn eval(&self, ctx: &TurnContext) -> bool {
        match self {
            PipelinePredicate::AgentHandled => ctx.flags.agent_handled == Some(true),
            PipelinePredicate::SceneIdEquals { scene_id } => ctx
                .scene
                .effective_scene_id
                .as_deref()
                .is_some_and(|s| s == scene_id.as_str()),
            PipelinePredicate::EmotionDominant { emotion } => {
                let Some(er) = ctx.emotion.user_emotion.as_ref() else {
                    return false;
                };
                let want = emotion.trim();
                if want.is_empty() {
                    return false;
                }
                dominant_dimension_name(er).eq_ignore_ascii_case(want)
                    || emotion_dominant_matches_discrete_label(er, want)
            }
        }
    }
}

fn dominant_dimension_name(er: &EmotionResult) -> &'static str {
    let dims: [(&str, f64); 7] = [
        ("joy", er.joy),
        ("sadness", er.sadness),
        ("anger", er.anger),
        ("fear", er.fear),
        ("surprise", er.surprise),
        ("disgust", er.disgust),
        ("neutral", er.neutral),
    ];
    let (mut name, mut best) = dims[0];
    for (n, v) in dims.iter().copied().skip(1) {
        if v > best {
            name = n;
            best = v;
        }
    }
    name
}

/// 将离散 `Emotion` 标签映射为与 `EmotionDominant` 可比较的蛇形名（如 `sad`）。
pub fn emotion_label_snake(e: Emotion) -> &'static str {
    match e {
        Emotion::Happy => "happy",
        Emotion::Sad => "sad",
        Emotion::Angry => "angry",
        Emotion::Neutral => "neutral",
        Emotion::Excited => "excited",
        Emotion::Confused => "confused",
        Emotion::Shy => "shy",
    }
}

/// 若 `emotion` 与离散主导情绪标签一致（如 `sad`），则用 `to_emotion()` 比较。
pub fn emotion_dominant_matches_discrete_label(er: &EmotionResult, emotion: &str) -> bool {
    let want = emotion.trim();
    if want.is_empty() {
        return false;
    }
    let dom = er.to_emotion();
    emotion_label_snake(dom).eq_ignore_ascii_case(want)
        || format!("{}", dom).eq_ignore_ascii_case(want)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_handled_false_when_none() {
        let ctx = TurnContext::new();
        assert!(!PipelinePredicate::AgentHandled.eval(&ctx));
    }

    #[test]
    fn scene_id_equals() {
        let mut ctx = TurnContext::new();
        ctx.scene.effective_scene_id = Some("default".into());
        assert!(PipelinePredicate::SceneIdEquals {
            scene_id: "default".into()
        }
        .eval(&ctx));
        assert!(!PipelinePredicate::SceneIdEquals {
            scene_id: "other".into()
        }
        .eval(&ctx));
    }

    #[test]
    fn emotion_dominant_uses_max_dimension() {
        let mut ctx = TurnContext::new();
        ctx.emotion.user_emotion = Some(EmotionResult {
            joy: 0.1,
            sadness: 0.9,
            anger: 0.0,
            fear: 0.0,
            surprise: 0.0,
            disgust: 0.0,
            neutral: 0.0,
        });
        assert!(PipelinePredicate::EmotionDominant {
            emotion: "sadness".into()
        }
        .eval(&ctx));
        assert!(!PipelinePredicate::EmotionDominant {
            emotion: "joy".into()
        }
        .eval(&ctx));
    }
}
