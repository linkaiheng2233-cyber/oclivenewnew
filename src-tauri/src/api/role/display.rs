//! 身份选项的展示文案（与前端下拉 `UserRelationDto.name` 对齐）。

use crate::models::dto::UserRelationDto;
use crate::models::role::Role;

/// manifest 里 `display_name` 已映射到 `UserRelation.name`，若与 `id` 相同则视为未自定义，对常见英文键给中文 fallback。
pub(crate) fn user_relation_display_label(id: &str, name: &str) -> String {
    let t = name.trim();
    if !t.is_empty() && t != id {
        return name.to_string();
    }
    match id {
        "classmate" => "同学".to_string(),
        "friend" => "好友".to_string(),
        "family" => "家人".to_string(),
        "sibling" | "siblings" => "兄弟姐妹".to_string(),
        "parent" | "parents" => "父母".to_string(),
        "lover" => "恋人".to_string(),
        "rival" => "较劲".to_string(),
        "guardian" => "监护人".to_string(),
        "partner" => "伴侣".to_string(),
        "cousin" => "表亲".to_string(),
        "relative" => "亲戚".to_string(),
        "stranger" => "陌生人".to_string(),
        "teacher" => "老师".to_string(),
        "colleague" => "同事".to_string(),
        _ => id.to_string(),
    }
}

pub(crate) fn user_relations_to_dto(role: &Role) -> Vec<UserRelationDto> {
    role.user_relations
        .iter()
        .map(|r| UserRelationDto {
            id: r.id.clone(),
            name: user_relation_display_label(&r.id, &r.name),
            prompt_hint: r.prompt_hint.clone(),
            favor_multiplier: r.favor_multiplier,
            initial_favorability: r.initial_favorability_clamped(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::user_relation_display_label;

    #[test]
    fn prefers_custom_name_when_differs_from_id() {
        assert_eq!(user_relation_display_label("friend", "死党"), "死党");
    }

    #[test]
    fn fallback_when_name_equals_id() {
        assert_eq!(
            user_relation_display_label("classmate", "classmate"),
            "同学"
        );
        assert_eq!(
            user_relation_display_label("sibling", "sibling"),
            "兄弟姐妹"
        );
        assert_eq!(user_relation_display_label("parent", "parent"), "父母");
    }
}
