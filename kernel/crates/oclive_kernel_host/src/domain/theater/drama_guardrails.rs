//! Shared drama guardrails for theater builtin prompt fallback (mirrors official plugin v0.2).

const FULL_CORE: &str = "【戏剧性纪律】
· 反差：两人反应须鲜明对撞，禁止「好的」「嗯」式平淡接话；性格标签要在用词与态度上可见。
· 小输入大后果：poke 事件须改变节奏/情绪/关系张力，不是中性信息补充。
· 情绪起伏：至少一次可见转折（慌→窘→害羞；争→让→哄 等）。
· 接锚：须自然导向 skeleton 下一固定节拍，不断档、不另起炉灶。
· 禁止：旁白解说、机械复述 dramaSeed 原文、照抄罐头 fork 台词、OOC 万能反应。";

const COMPACT_CORE: &str =
    "【戏剧纪律（精简）】反差对撞、禁止平淡接话；小事件须改节奏/张力；至少一次情绪转折；禁止旁白与照抄罐头。";

fn scene_tone_hint(theater_scene: &str) -> &'static str {
    match theater_scene {
        "supermarket" => "\n【场景语气参考（勿照抄剧情）】\n强调动线与货架：推车、比价、试吃、收银排队；物件要具体（价签、空柜、购物袋），别写成泛化闲聊。\n",
        "way_home" => "\n【场景语气参考（勿照抄剧情）】\n户外步行感：路灯、风、重物、导航；突发小事故须有一瞬停顿再反应，关心与吐槽要有性格区分。\n",
        "bedtime" => "\n【场景语气参考（勿照抄剧情）】\n夜间氛围：压低音量、困意、窗外声；语气可渐软或渐怂，动作宜小（关窗、递杯、拉被子）。\n",
        _ => "\n【场景语气参考（勿照抄剧情）】\n早晨节奏偏快：碗筷声、书包、出门倒计时；拌嘴可带困意与互相嫌弃，但底色是照应与默契。\n",
    }
}

fn resolve_theater_scene(theater_scene: Option<&str>) -> &str {
    theater_scene
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("breakfast")
}

/// Full guardrails block for patch / ripple modes.
#[must_use]
pub fn drama_guardrails_full(theater_scene: Option<&str>) -> String {
    let scene = resolve_theater_scene(theater_scene);
    format!("\n{FULL_CORE}{}", scene_tone_hint(scene))
}

/// Compact guardrails for cast_* modes.
#[must_use]
pub fn drama_guardrails_compact(theater_scene: Option<&str>) -> String {
    let scene = resolve_theater_scene(theater_scene);
    format!("\n{COMPACT_CORE}{}", scene_tone_hint(scene))
}

/// Key substrings for drift tests (plugin ↔ builtin).
pub const PATCH_TITLE: &str = "【剧场即兴 · 戏剧性补丁】";
pub const GUARDRAILS_HEADER: &str = "【戏剧性纪律】";
pub const COMPACT_HEADER: &str = "【戏剧纪律（精简）】";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardrails_include_scene_hint_for_supermarket() {
        let block = drama_guardrails_full(Some("supermarket"));
        assert!(block.contains(GUARDRAILS_HEADER));
        assert!(block.contains("货架"));
    }
}
