//! Module 9: PromptStyle override layer.
//!
//! Design goal: when override is unset, behavior must be **identical** to existing role packs.

use crate::models::{PromptStyleOverride, Role};
use std::borrow::Cow;

fn norm_non_empty(opt: &Option<String>) -> Option<&str> {
    opt.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Apply prompt style overrides to a `Role` view.
///
/// Returns a borrowed `Role` when no effective override is present.
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
