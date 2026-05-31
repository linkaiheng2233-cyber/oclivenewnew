//! When `plugin_backends.memory = local`, resolve the target `provider_id` from the list of registered provider ids (pure logic, convenient for unit tests and `plugin_host` reuse).

/// Resolution result: a `None` `provider_id` means the registry has no memory-capable provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalMemoryPick {
    pub provider_id: Option<String>,
    /// `local_memory_provider_id` is non-empty but matches none of the registered ids.
    pub hint_missed: bool,
    /// Multiple memory providers with no valid preferred given (or a lexicographic fallback when multiple candidates remain after a preferred miss).
    pub ambiguous_lexicographic: bool,
}

fn pick_sorted_str_ids(ids: &[&str], preferred: Option<&str>) -> LocalMemoryPick {
    let pref = preferred.map(str::trim).filter(|s| !s.is_empty());

    if ids.is_empty() {
        return LocalMemoryPick {
            provider_id: None,
            hint_missed: pref.is_some(),
            ambiguous_lexicographic: false,
        };
    }

    if let Some(h) = pref {
        if let Some(found) = ids.iter().find(|id| **id == h) {
            return LocalMemoryPick {
                provider_id: Some((*found).to_string()),
                hint_missed: false,
                ambiguous_lexicographic: false,
            };
        }
        return LocalMemoryPick {
            provider_id: Some(ids[0].to_string()),
            hint_missed: true,
            ambiguous_lexicographic: ids.len() > 1,
        };
    }

    LocalMemoryPick {
        provider_id: Some(ids[0].to_string()),
        hint_missed: false,
        ambiguous_lexicographic: ids.len() > 1,
    }
}

/// Resolve from a borrowed slice of registered ids; sorts and dedups internally, **cloning only the finally selected** `provider_id`.
#[must_use]
pub fn pick_local_memory_provider_refs(
    mut ids: Vec<&str>,
    preferred: Option<&str>,
) -> LocalMemoryPick {
    ids.sort_unstable();
    ids.dedup();
    pick_sorted_str_ids(&ids, preferred)
}

/// `ids` may be in any order; sorted and deduped internally. If `preferred` is non-empty after trimming, an exact match is preferred.
pub fn pick_local_memory_provider(
    mut ids: Vec<String>,
    preferred: Option<&str>,
) -> LocalMemoryPick {
    ids.sort();
    ids.dedup();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    pick_sorted_str_ids(&refs, preferred)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_ids() {
        let p = pick_local_memory_provider(vec![], None);
        assert_eq!(
            p,
            LocalMemoryPick {
                provider_id: None,
                hint_missed: false,
                ambiguous_lexicographic: false,
            }
        );
        let p2 = pick_local_memory_provider(vec![], Some("x"));
        assert!(p2.hint_missed);
        assert!(p2.provider_id.is_none());
    }

    #[test]
    fn single_no_hint() {
        let p = pick_local_memory_provider(vec!["a".into()], None);
        assert_eq!(p.provider_id, Some("a".into()));
        assert!(!p.hint_missed);
        assert!(!p.ambiguous_lexicographic);
    }

    #[test]
    fn hint_hits() {
        let p = pick_local_memory_provider(vec!["z".into(), "m".into()], Some("m"));
        assert_eq!(p.provider_id, Some("m".into()));
        assert!(!p.hint_missed);
        assert!(!p.ambiguous_lexicographic);
    }

    #[test]
    fn hint_miss_falls_back_lex_first() {
        let p = pick_local_memory_provider(vec!["z".into(), "m".into()], Some("missing"));
        assert_eq!(p.provider_id, Some("m".into()));
        assert!(p.hint_missed);
        assert!(p.ambiguous_lexicographic);
    }

    #[test]
    fn multi_without_hint_lex_first() {
        let p = pick_local_memory_provider(vec!["z".into(), "m".into()], None);
        assert_eq!(p.provider_id, Some("m".into()));
        assert!(!p.hint_missed);
        assert!(p.ambiguous_lexicographic);
    }

    #[test]
    fn refs_matches_owned_api() {
        let owned = pick_local_memory_provider(vec!["z".into(), "m".into()], Some("m"));
        let refs = pick_local_memory_provider_refs(vec!["z", "m"], Some("m"));
        assert_eq!(owned, refs);
    }
}
