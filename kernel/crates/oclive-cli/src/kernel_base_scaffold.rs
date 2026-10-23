//! Contract-only development project; no reference Host assembly or default providers.

use crate::init::{InitArgs, InitTemplateArg, ProjectConfig, ProjectTypeArg};
use anyhow::{bail, Context, Result};
use std::{fs, path::Path};

pub const FILES: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        include_str!("templates/kernel_base/Cargo.toml.hbs"),
    ),
    (
        "src/lib.rs",
        include_str!("templates/kernel_base/lib.rs.hbs"),
    ),
    (
        "tests/base_contracts.rs",
        include_str!("templates/kernel_base/base_contracts.rs.hbs"),
    ),
    (
        "README.md",
        include_str!("templates/kernel_base/README.md.hbs"),
    ),
    (".gitignore", "/target/\n"),
];

pub fn selected_files(
    cfg: &ProjectConfig,
) -> impl Iterator<Item = &'static (&'static str, &'static str)> + '_ {
    FILES.iter().filter(|(path, _)| {
        if *path == "tests/base_contracts.rs" {
            cfg.base_memory_example && !cfg.base_consumer_example
        } else {
            true
        }
    })
}

pub fn validate_options(args: &InitArgs) -> Result<()> {
    let host_options = args.preset.is_some()
        || args.backend_memory.is_some()
        || args.backend_emotion.is_some()
        || args.backend_event.is_some()
        || args.backend_prompt.is_some()
        || args.backend_llm.is_some()
        || args.backend_agent.is_some()
        || args.backend_complex_emotion.is_some()
        || args.monolith
        || args.monolith_preset.is_some()
        || args.monolith_bench_preset.is_some()
        || !args.weld_modules.is_empty()
        || args.dual_core
        || args.with_role_pack.is_some()
        || args.with_example_plugin
        || args.chat_storage_location.is_some()
        || args.pipeline != crate::pipeline::PipelineArg::Default
        || matches!(args.project_type, Some(ProjectTypeArg::KernelServer));
    if host_options
        || args.quick
        || args.smart
        || args.template_url.is_some()
        || args.from_existing.is_some()
        || args.share
    {
        bail!("kernel-base is a contract-only library; runtime/Host options and alternate init flows are not supported");
    }
    Ok(())
}

pub fn validate_source(root: &Path) -> Result<()> {
    for name in ["oclive_kernel_contracts", "oclive_kernel_types"] {
        if !root
            .join("kernel/crates")
            .join(name)
            .join("Cargo.toml")
            .is_file()
        {
            bail!("kernel-base --kernel-source needs kernel/crates/{name}/Cargo.toml");
        }
    }
    Ok(())
}

fn validate_consumer_source(root: &Path) -> Result<()> {
    let example = root.join("kernel/crates/oclive_kernel_runtime/examples/external_memory_base");
    for file in [example.join("main.rs"), example.join("memory.rs")] {
        if !file.is_file() {
            bail!(
                "--base-example consumer needs the canonical Memory example at {}",
                file.display()
            );
        }
    }
    for file in [root.join("LICENSE"), root.join("Cargo.lock")] {
        if !file.is_file() {
            bail!(
                "--base-example consumer needs {} in the source checkout",
                file.display()
            );
        }
    }
    Ok(())
}

pub fn build_config(args: &InitArgs) -> Result<ProjectConfig> {
    validate_options(args)?;
    let source = args.kernel_source.as_ref()
        .context("kernel-base requires --kernel-source <OCLive source root>; it has no stub or Host fallback")?
        .canonicalize().context("kernel-base source path")?;
    validate_source(&source)?;
    let mut cfg = crate::template_catalog::project_config_from_template(
        &args.project_name,
        InitTemplateArg::KernelBase,
    );
    cfg.kernel_source = Some(source);
    let example = args
        .base_example
        .unwrap_or(crate::init::BaseExampleArg::Memory);
    cfg.base_consumer_example = example == crate::init::BaseExampleArg::Consumer;
    cfg.base_memory_example = example == crate::init::BaseExampleArg::Memory;
    if cfg.base_consumer_example {
        validate_consumer_source(
            cfg.kernel_source
                .as_deref()
                .expect("kernel source set above"),
        )?;
    }
    crate::init::apply_cargo_metadata_cli(&mut cfg, args);
    crate::init::ensure_cargo_license_default(&mut cfg);
    Ok(cfg)
}

pub fn write_project(cfg: &ProjectConfig, out: &Path, context: &serde_json::Value) -> Result<()> {
    validate_source(
        cfg.kernel_source
            .as_deref()
            .context("kernel-base source missing")?,
    )?;
    if out.exists() && fs::read_dir(out)?.next().transpose()?.is_some() {
        bail!("Output directory not empty: {}", out.display());
    }
    let renderer = handlebars::Handlebars::new();
    // Render before creating output; a template rendering failure writes no project files.
    let rendered = selected_files(cfg)
        .map(|(path, template)| {
            renderer
                .render_template(template, context)
                .with_context(|| format!("render kernel-base {path}"))
                .map(|contents| (*path, contents))
        })
        .collect::<Result<Vec<_>>>()?;
    for (path, contents) in rendered {
        let destination = out.join(path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination, contents)?;
    }
    if cfg.base_consumer_example {
        let root = cfg
            .kernel_source
            .as_deref()
            .context("kernel-base source missing")?;
        let example =
            root.join("kernel/crates/oclive_kernel_runtime/examples/external_memory_base");
        let copies = [
            (example.join("main.rs"), out.join("src/main.rs")),
            (example.join("memory.rs"), out.join("src/memory.rs")),
            (root.join("LICENSE"), out.join("LICENSE")),
            (root.join("Cargo.lock"), out.join("Cargo.lock")),
        ];
        for (source, destination) in copies {
            if destination.exists() {
                bail!("refusing to overwrite {}", destination.display());
            }
            fs::copy(&source, &destination).with_context(|| {
                format!("copy {} to {}", source.display(), destination.display())
            })?;
        }
    }
    Ok(())
}
