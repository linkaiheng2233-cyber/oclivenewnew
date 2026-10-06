//! CLI line prompts produce original typed constraints; the Host validates their meaning.
use anyhow::{bail, Context, Result};
use oclive_kernel_types::{
    ResourceCandidatePlan, ResourceCoordinationDiagnostics, ResourceResidencyPreference,
    ResourceSchedulingCommand, ResourceSchedulingStrategy,
};
use std::io::{self, BufRead, Write};
use toml::{map::Map, Value};

fn ask(input: &mut impl BufRead, output: &mut impl Write, label: &str) -> Result<String> {
    write!(output, "{label}: ")?;
    output.flush()?;
    let mut line = String::new();
    if input
        .read_line(&mut line)
        .context("read resource wizard answer")?
        == 0
    {
        bail!("resource policy wizard cancelled: input ended");
    }
    let answer = line.trim().to_owned();
    if answer == "q" {
        bail!("resource policy wizard cancelled");
    }
    Ok(answer)
}

fn required(input: &mut impl BufRead, output: &mut impl Write, label: &str) -> Result<String> {
    let answer = ask(input, output, label)?;
    if answer.is_empty() {
        bail!("resource wizard value required: {label}");
    }
    Ok(answer)
}

fn ids(input: &mut impl BufRead, output: &mut impl Write, label: &str) -> Result<Vec<String>> {
    let answer = required(input, output, label)?;
    let ids: Vec<_> = answer.split(',').map(|id| id.trim().to_owned()).collect();
    if ids.iter().any(String::is_empty) {
        bail!("resource wizard list contains an empty ID");
    }
    Ok(ids)
}

fn commands(
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<Vec<ResourceSchedulingCommand>> {
    let mut commands = Vec::new();
    loop {
        let choice = ask(input, output, "Command: 0 finish, 1 require, 2 resident, 3 on-demand, 4 coexist, 5 exclusive, 6 yield-then-run, 7 fallback")?;
        let command = match choice.as_str() {
            "0" => break,
            "1" => ResourceSchedulingCommand::Require {
                adapter_id: required(input, output, "Adapter ID")?,
            },
            "2" | "3" => ResourceSchedulingCommand::Residency {
                adapter_id: required(input, output, "Adapter ID")?,
                mode: if choice == "2" {
                    ResourceResidencyPreference::Resident
                } else {
                    ResourceResidencyPreference::OnDemand
                },
            },
            "4" => ResourceSchedulingCommand::Coexist {
                adapter_ids: ids(input, output, "Adapter IDs (comma-separated)")?,
            },
            "5" => ResourceSchedulingCommand::Exclusive {
                adapter_ids: ids(input, output, "Adapter IDs (comma-separated)")?,
            },
            "6" => ResourceSchedulingCommand::YieldThenRun {
                yielding_adapter_id: required(input, output, "Yielding adapter ID")?,
                target_adapter_id: required(input, output, "Target adapter ID")?,
            },
            "7" => ResourceSchedulingCommand::Fallback {
                adapter_id: required(input, output, "Adapter ID")?,
                profile_ids: ids(
                    input,
                    output,
                    "Profile IDs in preference order (comma-separated)",
                )?,
            },
            _ => bail!("invalid resource wizard command choice"),
        };
        commands.push(command);
    }
    Ok(commands)
}

pub(crate) fn collect(
    capture: &ResourceCoordinationDiagnostics,
) -> Result<(Map<String, Value>, bool)> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    writeln!(output, "Offline resource policy wizard; q cancels, blank keeps the original key. No device probe or live controllers. Comments and formatting are not retained.")?;
    writeln!(output, "Current intent: {:?}", capture.policy.scheduling)?;
    for adapter in &capture.adapters {
        writeln!(
            output,
            "Adapter {} ({:?})",
            adapter.descriptor.adapter_id, adapter.descriptor.control_mode
        )?;
        for profile in &adapter.descriptor.profiles {
            writeln!(
                output,
                "  profile {}: {:?}, selectable={}",
                profile.profile_id, profile.execution_target, profile.coordinator_selectable
            )?;
        }
    }
    let mut patch = Map::new();
    let choice = ask(
        &mut input,
        &mut output,
        "Strategy: blank keep, 1 compatibility-first, 2 primary-first, 3 latency-first, 4 custom",
    )?;
    let strategy = match choice.as_str() {
        "" => None,
        "1" => Some(ResourceSchedulingStrategy::CompatibilityFirst),
        "2" => Some(ResourceSchedulingStrategy::PrimaryFirst),
        "3" => Some(ResourceSchedulingStrategy::LatencyFirst),
        "4" => Some(ResourceSchedulingStrategy::Custom),
        _ => bail!("invalid resource wizard strategy choice"),
    };
    if let Some(strategy) = strategy {
        patch.insert("strategy".into(), Value::try_from(strategy)?);
    }
    let primary = ask(
        &mut input,
        &mut output,
        "Primary adapter ID: blank keep, - clear",
    )?;
    let clear_primary = primary == "-";
    if !primary.is_empty() && !clear_primary {
        patch.insert("primary_adapter_id".into(), Value::String(primary));
    }
    match ask(
        &mut input,
        &mut output,
        "Commands: blank keep, - clear, + replace",
    )?
    .as_str()
    {
        "" => {}
        "-" => {
            patch.insert("commands".into(), Value::Array(Vec::new()));
        }
        "+" => {
            patch.insert(
                "commands".into(),
                Value::try_from(commands(&mut input, &mut output)?)?,
            );
        }
        _ => bail!("invalid resource wizard commands choice"),
    }
    for (key, label) in [
        ("gpu_safety_reserve_mib", "GPU reserve MiB (blank keep)"),
        (
            "system_memory_safety_reserve_mib",
            "RAM reserve MiB (blank keep)",
        ),
        (
            "cpu_safety_reserve_threads",
            "CPU reserve threads (blank keep)",
        ),
    ] {
        let answer = ask(&mut input, &mut output, label)?;
        if !answer.is_empty() {
            let number = answer
                .parse::<u64>()
                .context("resource wizard reserve must be a nonnegative integer")?;
            patch.insert(
                key.into(),
                Value::Integer(
                    i64::try_from(number)
                        .context("resource wizard reserve exceeds TOML integer range")?,
                ),
            );
        }
    }
    Ok((patch, clear_primary))
}

pub(crate) fn confirm(
    capture: &ResourceCoordinationDiagnostics,
    plan: &ResourceCandidatePlan,
) -> Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "Validated intent: {:?}; effective reserves GPU MiB={}, RAM MiB={}, CPU threads={}",
        capture.policy.scheduling,
        capture.policy.gpu_safety_reserve_mib,
        capture.policy.system_memory_safety_reserve_mib,
        capture.policy.cpu_safety_reserve_threads
    )?;
    writeln!(
        output,
        "Captured candidate: {:?}; not live admission",
        plan.state
    )?;
    for reason in capture
        .scheduling
        .reason_codes
        .iter()
        .chain(&plan.reason_codes)
    {
        let reason = crate::doctor_resource_plan::format_resource_reason(reason);
        writeln!(output, "  reason: {reason}")?;
    }
    if ask(
        &mut input,
        &mut output,
        "Write NEW draft? Type yes to confirm",
    )? != "yes"
    {
        bail!("resource policy wizard cancelled: draft not confirmed");
    }
    Ok(())
}
