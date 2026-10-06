//! New-file-only resource policy drafts, validated by the original offline preview.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct ResourcePolicyArgs {
    /// Existing distro profile; the input is never modified.
    #[arg(long)]
    pub distro_profile: PathBuf,
    /// TOML containing only [resource_coordination]; supplied keys replace existing keys.
    #[arg(
        long,
        required_unless_present = "interactive",
        conflicts_with = "interactive"
    )]
    pub policy_file: Option<PathBuf>,
    /// Ask for finite resource constraints, preview them, then confirm a new draft.
    #[arg(long)]
    pub interactive: bool,
    /// Captured ResourceCoordinationDiagnostics used to validate finite adapter constraints.
    #[arg(long)]
    pub diagnostics_file: PathBuf,
    /// New draft file in an existing directory; overwriting is never allowed.
    #[arg(long)]
    pub output: PathBuf,
    /// Select a GPU from the captured snapshot only.
    #[arg(long)]
    pub gpu_device_index: Option<u32>,
}

#[cfg(feature = "diagnostics-host")]
fn write_draft(args: ResourcePolicyArgs) -> Result<()> {
    use anyhow::{bail, Context};
    use oclive_kernel_host::domain::host_profile::load_host_profile_file;
    use oclive_kernel_types::ResourceSchedulingIntentState;
    use std::{io::Write, path::Path};

    if args.output.try_exists().context("check draft output")? {
        bail!("draft output already exists: {}", args.output.display());
    }
    load_host_profile_file(&args.distro_profile)
        .map_err(|error| anyhow::anyhow!("load source distro profile: {error}"))?;
    let mut document: toml::Value = toml::from_str(
        &std::fs::read_to_string(&args.distro_profile).context("read source distro profile")?,
    )
    .context("read editable distro TOML values")?;
    let (resources, clear_primary) = if args.interactive {
        let (capture, _) =
            crate::doctor_resource_plan::preview(&crate::doctor_resource_plan::ResourcePlanArgs {
                diagnostics_file: args.diagnostics_file.clone(),
                distro_profile: Some(args.distro_profile.clone()),
                gpu_device_index: args.gpu_device_index,
                json: false,
            })?;
        crate::resource_policy_wizard::collect(&capture)?
    } else {
        let path = args
            .policy_file
            .as_ref()
            .context("resource policy file required")?;
        let patch: toml::Value =
            toml::from_str(&std::fs::read_to_string(path).context("read resource policy patch")?)
                .context("read resource policy TOML values")?;
        let patch = patch
            .as_table()
            .context("policy patch must be a TOML table")?;
        if patch.len() != 1 || !patch.contains_key("resource_coordination") {
            bail!("policy patch must contain only [resource_coordination]");
        }
        (
            patch["resource_coordination"]
                .as_table()
                .context("[resource_coordination] must be a table")?
                .clone(),
            false,
        )
    };
    let target = document
        .as_table_mut()
        .context("distro profile must be a TOML table")?
        .entry("resource_coordination")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .context("source [resource_coordination] must be a table")?;
    // This is structural editing, not a second typed policy parser. Arrays replace in full.
    if clear_primary {
        target.remove("primary_adapter_id");
    }
    for (key, value) in resources {
        target.insert(key, value);
    }
    let text = toml::to_string_pretty(&document).context("serialize distro draft")?;
    let parent = args
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut draft = tempfile::NamedTempFile::new_in(parent).context("prepare new distro draft")?;
    draft
        .write_all(text.as_bytes())
        .context("write temporary distro draft")?;
    draft
        .as_file()
        .sync_all()
        .context("flush temporary distro draft")?;
    let (capture, candidate) =
        crate::doctor_resource_plan::preview(&crate::doctor_resource_plan::ResourcePlanArgs {
            diagnostics_file: args.diagnostics_file,
            distro_profile: Some(draft.path().to_path_buf()),
            gpu_device_index: args.gpu_device_index,
            json: false,
        })?;
    if capture.scheduling.state == ResourceSchedulingIntentState::Blocked {
        bail!(
            "resource policy intent blocked: {}",
            capture
                .scheduling
                .reason_codes
                .iter()
                .map(|code| crate::doctor_resource_plan::format_resource_reason(code))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if args.interactive {
        crate::resource_policy_wizard::confirm(&capture, &candidate)?;
    }
    draft
        .persist_noclobber(&args.output)
        .context("publish new distro draft without overwrite")?;
    eprintln!("offline resource policy draft: no live controllers or execution authority; comments and formatting are not retained");
    for reason in capture
        .scheduling
        .reason_codes
        .iter()
        .chain(&candidate.reason_codes)
    {
        let reason = crate::doctor_resource_plan::format_resource_reason(reason);
        eprintln!("  reason: {reason}");
    }
    println!("Wrote new distro draft: {}", args.output.display());
    println!(
        "  effective strategy: {:?}; GPU reserve MiB: {}",
        capture.policy.scheduling.strategy, capture.policy.gpu_safety_reserve_mib
    );
    println!(
        "  captured candidate: {:?}; not live admission",
        candidate.state
    );
    Ok(())
}

pub fn run(args: ResourcePolicyArgs) -> Result<()> {
    #[cfg(feature = "diagnostics-host")]
    {
        write_draft(args)
    }
    #[cfg(not(feature = "diagnostics-host"))]
    {
        let _ = args;
        anyhow::bail!(
            "`config resource-policy` requires building oclive-cli with feature `diagnostics-host`"
        );
    }
}
