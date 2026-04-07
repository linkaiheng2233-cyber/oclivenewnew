//! 用户叙事场景与角色所在场景是否一致（异地判定）。

/// `user_presence_scene` 与 `character_current_scene` 不同时为异地；角色尚未有场景时不算异地。
#[must_use]
pub fn user_is_remote_from_character(
    user_presence_scene: &str,
    character_current_scene: Option<&str>,
) -> bool {
    match character_current_scene {
        Some(cs) => cs != user_presence_scene,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_when_scenes_differ() {
        assert!(user_is_remote_from_character("home", Some("school")));
    }

    #[test]
    fn co_present_when_same() {
        assert!(!user_is_remote_from_character("home", Some("home")));
    }

    #[test]
    fn no_character_scene_is_not_remote() {
        assert!(!user_is_remote_from_character("home", None));
    }
}
