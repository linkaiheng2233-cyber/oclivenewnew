//! Read-only Scaffold Package discovery and contract diagnostics.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use oclive_scaffold::{
    build_scaffold_lock, load_scaffold_manifest, merge_scaffold_configs,
    project_scaffold_config_path, project_scaffold_lock_path, read_optional_scaffold_config,
    resolve_scaffold_catalog, scan_scaffold_catalog, user_scaffold_config_path,
    write_scaffold_lock_atomic, CatalogScan, ResolvedCatalog, ScaffoldConfig, ScaffoldSource,
};
use semver::Version;

#[derive(Parser, Debug)]
pub struct ScaffoldCli {
    #[command(subcommand)]
    pub command: ScaffoldCommands,
}

#[derive(Subcommand, Debug)]
pub enum ScaffoldCommands {
    /// List discovered and selected packages without executing them
    List(ScaffoldCatalogArgs),
    /// Inspect one selected package, its source, permissions, and declarations
    Inspect(ScaffoldInspectArgs),
    /// Strictly validate one local v1 manifest
    Validate(ScaffoldValidateArgs),
    /// Resolve sources deterministically; optionally persist the project lock
    Resolve(ScaffoldResolveArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ScaffoldCatalogArgs {
    /// Project root used for project-local discovery and configuration
    #[arg(short = 'o', long = "path", default_value = ".")]
    pub path: PathBuf,
    /// Override source priority for this invocation (comma-separated)
    #[arg(long, value_enum, value_delimiter = ',')]
    pub source_order: Vec<ScaffoldSourceArg>,
    /// Emit machine-readable JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct ScaffoldInspectArgs {
    /// Reverse-domain package ID
    pub id: String,
    #[command(flatten)]
    pub catalog: ScaffoldCatalogArgs,
}

#[derive(Args, Debug)]
pub struct ScaffoldValidateArgs {
    /// Path to `oclive.scaffold.json`
    pub manifest: PathBuf,
    /// Treat the local manifest as a project- or user-scoped package
    #[arg(long, value_enum, default_value_t = LocalValidationSource::Project)]
    pub source: LocalValidationSource,
    /// Emit machine-readable JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug)]
pub struct ScaffoldResolveArgs {
    #[command(flatten)]
    pub catalog: ScaffoldCatalogArgs,
    /// Atomically write `.oclive/scaffold.lock.json`
    #[arg(long)]
    pub write_lock: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum ScaffoldSourceArg {
    Project,
    User,
    Official,
}

impl From<ScaffoldSourceArg> for ScaffoldSource {
    fn from(value: ScaffoldSourceArg) -> Self {
        match value {
            ScaffoldSourceArg::Project => Self::Project,
            ScaffoldSourceArg::User => Self::User,
            ScaffoldSourceArg::Official => Self::Official,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum LocalValidationSource {
    Project,
    User,
}

impl From<LocalValidationSource> for ScaffoldSource {
    fn from(value: LocalValidationSource) -> Self {
        match value {
            LocalValidationSource::Project => Self::Project,
            LocalValidationSource::User => Self::User,
        }
    }
}

pub fn run(cli: ScaffoldCli) -> Result<()> {
    match cli.command {
        ScaffoldCommands::List(args) => run_list(args),
        ScaffoldCommands::Inspect(args) => run_inspect(args),
        ScaffoldCommands::Validate(args) => run_validate(args),
        ScaffoldCommands::Resolve(args) => run_resolve(args),
    }
}

fn run_list(args: ScaffoldCatalogArgs) -> Result<()> {
    let context = resolve_context(&args)?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "scan": context.scan,
                "resolved": context.resolved,
            }))?
        );
        return Ok(());
    }
    println!(
        "Scaffold packages: {} selected from {} candidates (declarations only)",
        context.resolved.packages.len(),
        context.scan.candidates.len()
    );
    for candidate in &context.scan.candidates {
        let selected = context.resolved.packages.iter().any(|package| {
            package.source == candidate.source && package.locator == candidate.locator
        });
        let status = if selected { "selected" } else { "not selected" };
        println!(
            "- {}@{} [{} / {:?} / {status}] {}",
            candidate.manifest.package.id,
            candidate.manifest.package.version,
            candidate.source.as_str(),
            candidate.trust,
            candidate.locator
        );
        print_warnings(&candidate.warnings);
    }
    if !context.resolved.shadowed.is_empty() {
        println!("Shadowed candidates:");
        for package in &context.resolved.shadowed {
            println!(
                "- {}@{} [{}] {} (selected: {})",
                package.id,
                package.version,
                package.source.as_str(),
                package.locator,
                package.selected_source.as_str()
            );
        }
    }
    Ok(())
}

fn run_inspect(args: ScaffoldInspectArgs) -> Result<()> {
    let context = resolve_context(&args.catalog)?;
    let package = context
        .resolved
        .packages
        .iter()
        .find(|package| package.manifest.package.id == args.id)
        .with_context(|| {
            let available = context
                .resolved
                .packages
                .iter()
                .map(|package| package.manifest.package.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "scaffold package `{}` is not selected; available: {available}",
                args.id
            )
        })?;
    if args.catalog.json {
        println!("{}", serde_json::to_string_pretty(package)?);
        return Ok(());
    }
    println!(
        "{}@{}\nSource: {} ({:?})\nLocator: {}\nMaintainer: {}\nNamespace: {}\nPermissions: {}\nGenerators: {}\nCommands: {}",
        package.manifest.package.id,
        package.manifest.package.version,
        package.source.as_str(),
        package.trust,
        package.locator,
        package.manifest.package.maintainer,
        package.manifest.command_namespace,
        display_permissions(&package.manifest.permissions),
        package.manifest.generators.len(),
        package.manifest.commands.len()
    );
    print_warnings(&package.warnings);
    Ok(())
}

fn run_validate(args: ScaffoldValidateArgs) -> Result<()> {
    let path = canonical_file(&args.manifest)?;
    let source = args.source.into();
    let candidate = load_scaffold_manifest(&path, source, &reader_version()?)
        .with_context(|| format!("validate scaffold manifest {}", path.display()))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&candidate)?);
    } else {
        println!(
            "Valid scaffold package {}@{} [{} / {:?}]",
            candidate.manifest.package.id,
            candidate.manifest.package.version,
            source.as_str(),
            candidate.trust
        );
        println!("Maintainer: {}", candidate.manifest.package.maintainer);
        println!(
            "Permissions: {}",
            display_permissions(&candidate.manifest.permissions)
        );
        print_warnings(&candidate.warnings);
    }
    Ok(())
}

fn run_resolve(args: ScaffoldResolveArgs) -> Result<()> {
    let context = resolve_context(&args.catalog)?;
    if args.write_lock {
        let lock = build_scaffold_lock(&context.resolved);
        let path = project_scaffold_lock_path(&context.project_root);
        write_scaffold_lock_atomic(&path, &lock)
            .with_context(|| format!("write scaffold lock {}", path.display()))?;
        eprintln!("Scaffold lock: {}", path.display());
    }
    if args.catalog.json {
        println!("{}", serde_json::to_string_pretty(&context.resolved)?);
    } else {
        println!(
            "Resolved {} scaffold packages with source order: {}",
            context.resolved.packages.len(),
            context
                .resolved
                .source_order
                .iter()
                .map(|source| source.as_str())
                .collect::<Vec<_>>()
                .join(" > ")
        );
        print_warnings(&context.resolved.warnings);
    }
    Ok(())
}

struct ResolutionContext {
    project_root: PathBuf,
    scan: CatalogScan,
    resolved: ResolvedCatalog,
}

fn resolve_context(args: &ScaffoldCatalogArgs) -> Result<ResolutionContext> {
    let project_root = canonical_directory(&args.path)?;
    let oclive_home = crate::registry::oclive_home();
    let user_config_path = user_scaffold_config_path(&oclive_home);
    let project_config_path = project_scaffold_config_path(&project_root);
    let user_config = read_optional_scaffold_config(&user_config_path)
        .with_context(|| format!("read user scaffold config {}", user_config_path.display()))?;
    let project_config =
        read_optional_scaffold_config(&project_config_path).with_context(|| {
            format!(
                "read project scaffold config {}",
                project_config_path.display()
            )
        })?;
    let mut config = merge_scaffold_configs(user_config.as_ref(), project_config.as_ref())?;
    apply_source_order_override(&mut config, &args.source_order)?;
    let reader_version = reader_version()?;
    let scan = scan_scaffold_catalog(
        &project_root.join(".oclive").join("scaffolds"),
        &oclive_home.join("scaffolds"),
        &reader_version,
    );
    let resolved = resolve_scaffold_catalog(&scan, &config, &reader_version)?;
    Ok(ResolutionContext {
        project_root,
        scan,
        resolved,
    })
}

fn apply_source_order_override(
    config: &mut ScaffoldConfig,
    source_order: &[ScaffoldSourceArg],
) -> Result<()> {
    if source_order.is_empty() {
        return Ok(());
    }
    let order = source_order
        .iter()
        .copied()
        .map(ScaffoldSource::from)
        .collect::<Vec<_>>();
    if order.len() != 3 {
        bail!("--source-order must contain project,user,official exactly once");
    }
    config.source_order = Some(order);
    Ok(())
}

fn reader_version() -> Result<Version> {
    Version::parse(env!("CARGO_PKG_VERSION")).context("parse compiled oclive-cli version")
}

fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("resolve project root {}", path.display()))?;
    if !canonical.is_dir() {
        bail!("project root is not a directory: {}", canonical.display());
    }
    Ok(canonical)
}

fn canonical_file(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("resolve scaffold manifest {}", path.display()))?;
    if !canonical.is_file() {
        bail!("scaffold manifest is not a file: {}", canonical.display());
    }
    Ok(canonical)
}

fn display_permissions(permissions: &[String]) -> String {
    if permissions.is_empty() {
        "none".to_string()
    } else {
        permissions.join(", ")
    }
}

fn print_warnings(warnings: &[oclive_scaffold::ValidationIssue]) {
    for warning in warnings {
        println!("  WARNING [{}] {}", warning.code, warning.message);
    }
}
