//! Module 9: merge [`crate::models::PromptStyleOverride`] into a [`crate::models::Role`] view for prompt building.
//!
//! When override fields are unset or blank, behavior matches packs without overrides.
#![allow(dead_code)] // pub(crate)：主路径经 `role_manager` 等子模块；收窄可见性后易触发未接线 dead_code

use std::borrow::Cow;

use crate::models::{PromptStyleOverride, Role};

fn norm_non_empty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Returns a borrowed `Role` when there is no effective style override; otherwise an owned clone with patched fields.
pub fn role_view_with_prompt_style<'a>(
    role: &'a Role,
    style: Option<&PromptStyleOverride>,
) -> Cow<'a, Role> {
    let Some(s) = style else {
        return Cow::Borrowed(role);
    };

    let reply_quality_anchor = norm_non_empty(&s.reply_quality_anchor);
    let core_personality = norm_non_empty(&s.core_personality);
    let description = norm_non_empty(&s.description);

    if reply_quality_anchor.is_none() && core_personality.is_none() && description.is_none() {
        return Cow::Borrowed(role);
    }

    let mut out = role.clone();
    if let Some(v) = reply_quality_anchor {
        out.reply_quality_anchor = Some(v.to_string());
    }
    if let Some(v) = core_personality {
        out.core_personality = v.to_string();
    }
    if let Some(v) = description {
        out.description = v.to_string();
    }
    Cow::Owned(out)
}
