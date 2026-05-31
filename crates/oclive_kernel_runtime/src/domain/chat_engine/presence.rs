//! Whether the user's narrative scene matches the character's current scene (remote-presence check).

/// Remote-presence when `user_presence_scene` differs from `character_current_scene`; not remote when the character has no scene yet.
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
