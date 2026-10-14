//! Optional author declarations; no source inference, binding or execution authority.

use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use oclive_validation::GROUP_SLOT_TYPES;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct WiringArgs {
    /// A package Cargo.toml with optional package.metadata.oclive.wiring
    #[arg(long, default_value = "Cargo.toml")]
    manifest_path: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declaration {
    schema_version: u32,
    call_scope: Option<Vec<String>>,
    #[serde(default)]
    slots: BTreeMap<String, SlotDeclaration>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SlotDeclaration {
    implementation: Option<String>,
    via: Option<String>,
    scope: Option<DeclarationScope>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclarationScope {
    Project,
    ExampleTest,
}

fn validate_label(value: &str, field: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_:-.".contains(&byte)),
        "{field} must be a nonempty identifier label of at most 128 ASCII letters, digits, _, :, - or .; contents omitted"
    );
    Ok(())
}

fn validate_declaration(declaration: &Declaration) -> Result<()> {
    ensure!(
        declaration.schema_version == 1,
        "unsupported wiring schema_version; expected 1"
    );
    for (slot, value) in &declaration.slots {
        ensure!(
            GROUP_SLOT_TYPES.contains(&slot.as_str()),
            "wiring contains an unsupported slot key; contents omitted"
        );
        for (field, label) in [
            ("implementation", &value.implementation),
            ("via", &value.via),
        ] {
            if let Some(label) = label {
                validate_label(label, field)?;
            }
        }
    }
    if let Some(scope) = &declaration.call_scope {
        let mut seen = BTreeSet::new();
        for slot in scope {
            ensure!(
                GROUP_SLOT_TYPES.contains(&slot.as_str()),
                "call_scope contains an unsupported slot key; contents omitted"
            );
            ensure!(seen.insert(slot), "call_scope contains a repeated slot key");
        }
    }
    Ok(())
}

fn base_name(slot: &str) -> Result<&'static str> {
    match slot {
        "memory" => Ok("MemoryBase"),
        "emotion" => Ok("EmotionBase"),
        "event" => Ok("EventBase"),
        "prompt" => Ok("PromptBase"),
        "llm" => Ok("LlmBase"),
        "agent" => Ok("AgentBase"),
        _ => bail!("slot has no Base display mapping"),
    }
}

fn render(args: &WiringArgs) -> Result<String> {
    let raw = std::fs::read_to_string(&args.manifest_path)
        .with_context(|| format!("cannot read wiring manifest {:?}", args.manifest_path))?;
    // TOML errors can echo an offending line, which may contain a credential.
    let cargo: toml::Value = toml::from_str(&raw)
        .map_err(|_| anyhow::anyhow!("invalid Cargo.toml TOML; contents omitted"))?;
    let project = cargo
        .get("package")
        .and_then(|value| value.get("name"))
        .and_then(toml::Value::as_str)
        .context("expected [package].name; select a package manifest, not only a workspace")?;
    validate_label(project, "[package].name")?;
    let wiring = cargo
        .get("package")
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get("oclive"))
        .and_then(|value| value.get("wiring"));
    let declaration = wiring
        .map(|value| {
            value.clone().try_into::<Declaration>().map_err(|_| {
                anyhow::anyhow!("invalid package.metadata.oclive.wiring fields; contents omitted")
            })
        })
        .transpose()?;
    if let Some(declaration) = &declaration {
        validate_declaration(declaration)?;
    }
    let mut output = String::new();
    writeln!(output, "Project: {project}")?;
    // Path's Debug formatting escapes controls instead of sending them to a terminal.
    writeln!(
        output,
        "Source: {:?} [package.metadata.oclive.wiring] (author declaration)",
        args.manifest_path
    )?;
    writeln!(
        output,
        "Wiring metadata: {}",
        if declaration.is_some() {
            "provided"
        } else {
            "not provided"
        }
    )?;
    writeln!(output, "Host verification: not connected")?;
    writeln!(
        output,
        "Binding / call / target functionality: not verified"
    )?;
    writeln!(output, "\nInterfaces (relationships, not execution order):")?;
    for slot in GROUP_SLOT_TYPES {
        let base = base_name(slot)?;
        let item = declaration
            .as_ref()
            .and_then(|value| value.slots.get(*slot));
        if let Some(implementation) = item.and_then(|value| value.implementation.as_deref()) {
            writeln!(output, "  {base} -- declared --> {implementation}")?;
        } else {
            writeln!(output, "  {base} -- implementation not provided")?;
        }
        writeln!(output, "    Contract: oclive_kernel_contracts::{base}")?;
        writeln!(
            output,
            "    Entry declaration: {}",
            item.and_then(|value| value.via.as_deref())
                .unwrap_or("not provided")
        )?;
        let scope = match item.and_then(|value| value.scope.as_ref()) {
            Some(DeclarationScope::Project) => "project (author declaration)",
            Some(DeclarationScope::ExampleTest) => "example-test (not a runtime binding)",
            None => "not provided",
        };
        writeln!(output, "    Scope declaration: {scope}")?;
    }
    match declaration
        .as_ref()
        .and_then(|value| value.call_scope.as_ref())
    {
        Some(scope) => writeln!(
            output,
            "\nCall scope declaration: {} (not a call log)",
            if scope.is_empty() {
                "none declared".to_string()
            } else {
                scope.join(", ")
            }
        )?,
        None => writeln!(output, "\nCall scope declaration: not provided")?,
    }
    Ok(output)
}

pub fn run(args: WiringArgs) -> Result<()> {
    // Validate the entire chosen source before printing; failures produce no partial view.
    print!("{}", render(&args)?);
    Ok(())
}
