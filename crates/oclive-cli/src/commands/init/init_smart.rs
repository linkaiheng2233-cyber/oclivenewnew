//! Smart environment recommendations during init.

use super::InitArgs;

/// Print env-based init hints. Returns `true` when init should stop early (smart + non-interactive, no preset).
pub(crate) fn apply_smart_hints(args: &InitArgs) -> anyhow::Result<bool> {
    if args.smart {
        let probe = crate::env_probe::EnvironmentProbe::collect();
        crate::env_probe::print_init_recommendations(&probe, args.project_name.trim());
        if args.non_interactive && args.preset.is_none() {
            return Ok(true);
        }
    }

    let show_auto_smart = !args.non_interactive
        && !args.quiet
        && !args.no_smart
        && !args.smart
        && !args.list_templates
        && !args.check
        && args.template_url.is_none()
        && !args.quick;
    if show_auto_smart {
        let probe = crate::env_probe::EnvironmentProbe::collect();
        crate::env_probe::print_init_recommendations(&probe, args.project_name.trim());
    }
    Ok(false)
}
