//! 异地「生活轨迹 / 心声」专用 prompt（与独白、同场景主对话语义不同）

use crate::models::role::Role;

/// 未配置占位时使用的整段默认（阻断错位共景 RP）
pub const DEFAULT_REMOTE_STUB_MESSAGE: &str =
    "（你们此刻不在同一场景；切换到角色所在场景后再继续聊天吧。）";

/// 配置了 `stub_ooc` 但未配置任何旁白句时的兜底旁白
pub const DEFAULT_STUB_NARRATIVE_TAIL: &str = "她这会儿忙着呢，心里却忍不住瞄了一眼手机。";

fn collect_stub_lines(role: &Role) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    if let Some(ref lt) = role.life_trajectory {
        for s in &lt.stub_messages {
            let t = s.trim();
            if !t.is_empty() {
                out.push(t);
            }
        }
    }
    if out.is_empty() {
        if let Some(rp) = role.remote_presence.as_ref() {
            for s in &rp.stub_messages {
                let t = s.trim();
                if !t.is_empty() {
                    out.push(t);
                }
            }
        }
    }
    out
}

fn stub_rotation_index(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as usize)
        .unwrap_or(0))
        % len
}

/// 轮换或单条选取占位文案（优先 manifest `life_trajectory.stub_messages`，其次 settings 遗留字段）。**未配置 `stub_ooc` 时**表示整段占位。
pub fn pick_stub_message(role: &Role) -> String {
    let msgs = collect_stub_lines(role);
    if msgs.is_empty() {
        return DEFAULT_REMOTE_STUB_MESSAGE.to_string();
    }
    let idx = stub_rotation_index(msgs.len());
    msgs[idx].to_string()
}

/// 关闭异地心声且异地时的完整回复：`stub_ooc` + 中文逗号 + 旁白句（旁白来自 `stub_messages` 轮换）；未配置 `stub_ooc` 时回退为 [`pick_stub_message`]。
pub fn compose_remote_stub_reply(role: &Role) -> String {
    let Some(ref lt) = role.life_trajectory else {
        return pick_stub_message(role);
    };
    let ooc = lt
        .stub_ooc
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let Some(ooc) = ooc else {
        return pick_stub_message(role);
    };

    let mut tails: Vec<&str> = lt
        .stub_messages
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if tails.is_empty() {
        if let Some(rp) = role.remote_presence.as_ref() {
            tails = rp
                .stub_messages
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    let tail = if tails.is_empty() {
        DEFAULT_STUB_NARRATIVE_TAIL
    } else {
        tails[stub_rotation_index(tails.len())]
    };
    format!("{}，{}", ooc, tail)
}

/// 组装异地心声主 LLM 提示（中文）
///
/// `worldview_snippet` 为 [`crate::models::knowledge::KnowledgeIndex::format_for_prompt`] 的输出；空则跳过【世界观设定】段（与共景主对话语义对齐）。
#[allow(clippy::too_many_arguments)]
pub fn build_remote_life_prompt(
    role: &Role,
    away_material: &str,
    character_scene_label: &str,
    user_scene_label: &str,
    user_message: &str,
    favorability: f64,
    relation_state: &str,
    virtual_time_label: &str,
    life_schedule_line: &str,
    worldview_snippet: &str,
) -> String {
    let core = role.core_personality.trim();
    let core_hint = if core.len() > 2800 {
        format!("{}…（已截断）", core.chars().take(2800).collect::<String>())
    } else {
        core.to_string()
    };
    const MAX_MATERIAL_IN_PROMPT: usize = 4000;
    let material = away_material.trim();
    let material_block = if material.is_empty() {
        "（创作者未提供本场景的异地素材，请根据人设与场景合理想象角色在当地的日常与心情。）"
            .to_string()
    } else if material.chars().count() > MAX_MATERIAL_IN_PROMPT {
        format!(
            "{}…（已截断）",
            material
                .chars()
                .take(MAX_MATERIAL_IN_PROMPT)
                .collect::<String>()
        )
    } else {
        material.to_string()
    };

    let summary_block = role
        .life_trajectory
        .as_ref()
        .and_then(|lt| lt.effective_summary())
        .map(|s| format!("\n【生活轨迹总述（创作者 manifest）】\n{s}\n"))
        .unwrap_or_default();

    let life_schedule_block = if life_schedule_line.trim().is_empty() {
        String::new()
    } else {
        format!("\n{}\n", life_schedule_line.trim())
    };

    let worldview_block = if worldview_snippet.trim().is_empty() {
        String::new()
    } else {
        format!(
            "\n【世界观设定】（角色包知识；与上方人设冲突时以本段为权威事实。）\n{}\n",
            worldview_snippet.trim()
        )
    };

    format!(
        r#"你是角色「{name}」。

【重要】用户与你不处于同一场景：你当前在「{char_scene}」；用户是从对话上下文场景「{user_scene}」发来这句话。本回合不要写成「当面聊天」式的即时对话，请以**生活轨迹与内心独白**为主（可含简短动作/环境声），语气自然、符合人设。

【虚拟时间】{vtime}{life_sched}
【关系阶段】{rel} · 【好感度约】{fav:.1}

【人设摘要】
{core}
{summary}{worldview}
【创作者提供的「你当前场景」异地素材】
{material}

【用户刚发的话】
{um}

【生成要求】本回合为**模型生成**的异地生活与心声，不是输出预设固定句。须以【人设摘要】为根，**延伸**角色此刻的动作、环境与内心活动；**文风、结构、碎碎念节奏**（是否带 OOC 括注、先说明再旁白等）以【生活轨迹总述】中创作者的约定为准——约定是**规范写法与气质**，正文内容仍须**每轮现编**并与【用户刚发的话】呼应。【异地素材】与总述中的示例是**情境与细节参考**，可化用、改写以贴合当轮，**禁止**复述或照抄原文。综合虚拟时间、好感度、关系阶段择不同侧面展开，**禁止**与常见固定套路雷同。若总述未规定结构，可选用自然口吻。输出为纯文本一段，不要标题或列表符号。字数约 120～400 字，不要提及「系统」「prompt」或元指令。"#,
        name = role.name,
        char_scene = character_scene_label,
        user_scene = user_scene_label,
        vtime = virtual_time_label,
        life_sched = life_schedule_block,
        rel = relation_state,
        fav = favorability,
        core = core_hint,
        summary = summary_block,
        worldview = worldview_block,
        material = material_block,
        um = user_message,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::{LifeTrajectoryDisk, Role};

    fn minimal_role(lt: LifeTrajectoryDisk) -> Role {
        Role {
            life_trajectory: Some(lt),
            ..Default::default()
        }
    }

    #[test]
    fn compose_stub_joins_ooc_and_tail() {
        let lt = LifeTrajectoryDisk {
            stub_ooc: Some("（你们不同场景，同景后再细聊。）".into()),
            stub_messages: vec!["她在写作业呢".into()],
            ..Default::default()
        };
        let r = minimal_role(lt);
        assert_eq!(
            compose_remote_stub_reply(&r),
            "（你们不同场景，同景后再细聊。），她在写作业呢"
        );
    }

    #[test]
    fn build_remote_includes_worldview_when_non_empty() {
        let r = Role::default();
        let p = build_remote_life_prompt(
            &r,
            "",
            "咖啡馆",
            "家",
            "嗯",
            50.0,
            "Close",
            "2026-01-01",
            "",
            "雾城设定：永不天亮。",
        );
        assert!(p.contains("【世界观设定】"));
        assert!(p.contains("永不天亮"));
    }

    #[test]
    fn compose_stub_without_ooc_falls_back_to_full_pick() {
        let lt = LifeTrajectoryDisk {
            stub_messages: vec!["（整段旧版占位。）".into()],
            ..Default::default()
        };
        let r = minimal_role(lt);
        assert_eq!(compose_remote_stub_reply(&r), "（整段旧版占位。）");
    }
}
