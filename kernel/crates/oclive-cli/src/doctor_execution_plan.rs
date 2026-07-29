//! `oclive doctor execution-plan` — read-only capability and plan diagnostics.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct ExecutionPlanArgs {
    /// Role pack id under the roles directory.
    pub role_id: String,

    /// Roles root (default: resolved Chat Pro roles directory).
    #[arg(short = 'o', long)]
    pub roles_dir: Option<PathBuf>,

    /// App-data root containing installed plugins, grants, and plugin state.
    #[arg(long)]
    pub app_data_dir: Option<PathBuf>,

    /// Distro capability profile to evaluate instead of the active environment.
    #[arg(long)]
    pub distro_profile: Option<PathBuf>,

    /// Machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[cfg(feature = "diagnostics-host")]
fn build_report(args: &ExecutionPlanArgs) -> Result<oclive_kernel_types::ExecutionPlanDiagnostics> {
    use oclive_kernel_host::domain::execution_plan::{
        compile_execution_plan, CompileExecutionPlanInput,
    };
    use oclive_kernel_host::domain::host_profile::{
        load_host_profile_file, load_host_profile_from_env,
    };
    use oclive_kernel_host::infrastructure::capability_registry::build_capability_registry;
    use oclive_kernel_host::infrastructure::directory_plugins::DirectoryPluginRuntime;
    use oclive_kernel_host::infrastructure::high_risk_grants::HighRiskGrantStore;
    use oclive_kernel_host::infrastructure::storage::RoleStorage;
    use oclive_kernel_runtime::domain::plugin_resolution::{
        pick_llm_backend_env_override, remote_llm_url_token_configured,
        resolve_session_plugin_backends, HostBackendCeiling, SessionPluginResolutionInput,
    };
    use oclive_kernel_runtime::{find_app_data_dir_for_host, resolve_project_roles_dir};
    use oclive_kernel_types::{ExecutionPlanDiagnostics, EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION};
    use std::collections::BTreeMap;

    let roles_dir = match &args.roles_dir {
        Some(path) => path.clone(),
        None => resolve_project_roles_dir(&std::env::current_dir()?),
    };
    let app_data_dir = args
        .app_data_dir
        .clone()
        .unwrap_or_else(find_app_data_dir_for_host);
    let profile = match &args.distro_profile {
        Some(path) => load_host_profile_file(path)
            .map_err(|error| anyhow::anyhow!("load distro profile {}: {error}", path.display()))?,
        None => load_host_profile_from_env(),
    };
    let role = RoleStorage::new(&roles_dir).load_role(args.role_id.trim())?;
    let grants = HighRiskGrantStore::load(app_data_dir.clone(), true);
    let runtime = DirectoryPluginRuntime::bootstrap_with_host_profile(
        &roles_dir,
        &app_data_dir,
        grants.clone(),
        profile.clone(),
        true,
    );
    let registry = build_capability_registry(
        runtime.as_ref(),
        grants.as_ref(),
        &profile,
        role.id.as_str(),
    );
    let core = resolve_session_plugin_backends(&SessionPluginResolutionInput {
        pack_plugin_backends: role.plugin_backends.as_ref().clone(),
        pack_slot_registry: role.slot_registry.clone(),
        session_slot_overrides: BTreeMap::new(),
        user_llm_provider: String::new(),
        llm_env_override: pick_llm_backend_env_override(),
        remote_llm_url_token_configured: remote_llm_url_token_configured(),
        host_ceiling: HostBackendCeiling {
            skip_agent: profile.skip_agent,
            backends_ceiling: profile.backends_ceiling.clone(),
        },
    });
    let plan = compile_execution_plan(&CompileExecutionPlanInput {
        role_id: role.id.as_str(),
        distro_id: profile.distro_id.as_str(),
        core_backends: &core.backends,
        extensions: &role.blueprint_extensions,
        registry: &registry,
    });
    Ok(ExecutionPlanDiagnostics {
        schema_version: EXECUTION_PLAN_DIAGNOSTIC_SCHEMA_VERSION,
        plan,
        capability_registry: registry,
    })
}

pub fn run(args: ExecutionPlanArgs) -> Result<()> {
    #[cfg(feature = "diagnostics-host")]
    {
        let report = build_report(&args)?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!(
                "oclive doctor execution-plan — {} ({})",
                report.plan.role_id, report.plan.distro_id
            );
            println!("  activatable: {}", report.plan.activatable);
            println!("  flow_template: {:?}", report.plan.flow_template);
            println!(
                "  resource_coordination: {:?}",
                report.plan.resource_coordination
            );
            for extension in &report.plan.extensions {
                println!(
                    "  extension {}: {:?} provider={} reasons={}",
                    extension.instance_id,
                    extension.status,
                    extension.selected_provider_id.as_deref().unwrap_or("-"),
                    if extension.reason_codes.is_empty() {
                        "-".to_string()
                    } else {
                        extension.reason_codes.join(",")
                    }
                );
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "diagnostics-host"))]
    {
        let _ = args;
        anyhow::bail!(
            "`doctor execution-plan` requires building oclive-cli with feature `diagnostics-host`"
        );
    }
}

#[cfg(all(test, feature = "diagnostics-host"))]
mod tests {
    use super::*;
    use oclive_kernel_runtime::chat_pro_roles_dir;
    use tempfile::tempdir;

    #[test]
    fn mumu_plan_is_activatable_and_resource_coordination_is_explicitly_deferred() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let roles = chat_pro_roles_dir(&[manifest_dir]).expect("roles");
        let app_data = tempdir().expect("app data");
        let report = build_report(&ExecutionPlanArgs {
            role_id: "mumu".into(),
            roles_dir: Some(roles),
            app_data_dir: Some(app_data.path().to_path_buf()),
            distro_profile: None,
            json: true,
        })
        .expect("plan");
        assert!(report.plan.activatable);
        assert_eq!(
            report.plan.resource_coordination,
            oclive_kernel_types::ResourceCoordinationDiagnosticState::NotEvaluated
        );
    }
}
