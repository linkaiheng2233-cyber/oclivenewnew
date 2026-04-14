//! `plugin_backends.memory = local` 时，从已注册 provider id 列表中解析目标 `provider_id`（纯逻辑，便于单测与 `plugin_host` 复用）。

/// 解析结果：`provider_id` 为 `None` 表示注册表无 memory 能力 provider。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalMemoryPick {
    pub provider_id: Option<String>,
    /// `local_memory_provider_id` 非空但未命中任一已注册 id。
    pub hint_missed: bool,
    /// 多个 memory provider 且未给出有效 preferred（或 preferred 未命中后仍存在多候选时的字典序回退）。
    pub ambiguous_lexicographic: bool,
}

/// `ids` 可为任意顺序；内部会排序去重。`preferred` 为 trim 后非空则优先精确匹配。
pub fn pick_local_memory_provider(mut ids: Vec<String>, preferred: Option<&str>) -> LocalMemoryPick {
    ids.sort();
    ids.dedup();
    let pref = preferred.map(str::trim).filter(|s| !s.is_empty());

    if ids.is_empty() {
        return LocalMemoryPick {
            provider_id: None,
            hint_missed: pref.is_some(),
            ambiguous_lexicographic: false,
        };
    }

    if let Some(h) = pref {
        if let Some(found) = ids.iter().find(|id| *id == h) {
            return LocalMemoryPick {
                provider_id: Some((*found).clone()),
                hint_missed: false,
                ambiguous_lexicographic: false,
            };
        }
        let picked = ids[0].clone();
        return LocalMemoryPick {
            provider_id: Some(picked),
            hint_missed: true,
            ambiguous_lexicographic: ids.len() > 1,
        };
    }

    let picked = ids[0].clone();
    LocalMemoryPick {
        provider_id: Some(picked),
        hint_missed: false,
        ambiguous_lexicographic: ids.len() > 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_ids() {
        let p = pick_local_memory_provider(vec![], None);
        assert_eq!(p, LocalMemoryPick {
            provider_id: None,
            hint_missed: false,
            ambiguous_lexicographic: false,
        });
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
}
