use crate::state::AppState;
use axum::extract::State;
use oclive_kernel_runtime::RUNTIME_API_VERSION;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
struct HealthJson {
    ok: bool,
    runtime_api_version: &'static str,
    schema_migration_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kernel_manifest: Option<oclive_kernel_runtime::KernelBinaryManifest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distro_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    distro_profile_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_profile_summary: Option<oclive_kernel_types::ActiveProfileSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    startup_warnings: Vec<String>,
}

async fn health(State(state): State<Arc<AppState>>) -> axum::response::Response {
    use axum::http::header::CONTENT_TYPE;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let version = state
        .db_manager
        .max_applied_migration_version()
        .await
        .ok()
        .flatten();
    let distro = oclive_kernel_runtime::distro_health_snapshot();
    let host_profile =
        crate::domain::host_profile::load_host_profile_from_env();
    let active_profile_summary = host_profile.active_profile_summary();
    let startup_warnings = state.startup_health.read().aggregated_warnings();
    let json = HealthJson {
        ok: true,
        runtime_api_version: RUNTIME_API_VERSION,
        schema_migration_version: version,
        kernel_manifest: Some(oclive_kernel_runtime::KernelBinaryManifest::from_compile_time_env()),
        distro_id: distro.distro_id,
        distro_profile_hash: distro.distro_profile_hash,
        active_profile_summary,
        startup_warnings,
    };
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "application/json")],
        axum::Json(json),
    )
        .into_response()
}

async fn health_plain() -> &'static str {
    "ok"
}

pub(crate) async fn health_route(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::http::header::ACCEPT;
    use axum::response::IntoResponse;

    let wants_json = headers
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("application/json"))
        || std::env::var("OCLIVE_HEALTH_JSON")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if wants_json {
        health(State(state)).await
    } else {
        (axum::http::StatusCode::OK, health_plain().await).into_response()
    }
}
