//! `save_role_slot_registry` 写盘与 `role_cache` 失效。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::state::AppState;
use oclive_kernel_types::models::dto::SaveRoleSlotRegistryRequest;
use oclive_validation::load_blueprint_v2_for_role_dir;
use oclivenewnew_tauri::api::role::{get_role_info_impl, save_role_slot_registry_impl};
use std::path::PathBuf;
use std::sync::Arc;

fn roles_src_mumu() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../roles/mumu")
}

#[tokio::test]
async fn save_role_slot_registry_writes_and_reloads() {
    let src = roles_src_mumu();
    let tmp = tempfile::tempdir().unwrap();
    let role_dir = tmp.path().join("mumu");
    copy_dir_all(&src, &role_dir);

    let llm = Arc::new(oclive_kernel_host::infrastructure::MockLlmClient {
        reply: "ok".to_string(),
    });
    let state = AppState::new_in_memory_with_llm(llm, tmp.path().to_path_buf())
        .await
        .expect("state");

    let loaded = load_blueprint_v2_for_role_dir(&role_dir, "999.0.0").unwrap();
    let mut reg = loaded.slot_registry;
    let entry = reg.get_mut("llm").expect("mumu has llm");
    let prev_backend = entry.backend.clone();
    entry.backend = if prev_backend == "ollama" {
        "remote".into()
    } else {
        "ollama".into()
    };

    let info = save_role_slot_registry_impl(
        &state,
        &SaveRoleSlotRegistryRequest {
            role_id: "mumu".into(),
            slot_registry: reg.clone(),
        },
    )
    .await
    .expect("save");

    assert_eq!(
        info.slot_registry_pack.as_ref().and_then(|m| m.get("llm")),
        reg.get("llm")
    );

    let disk = load_blueprint_v2_for_role_dir(&role_dir, "999.0.0").unwrap();
    assert_eq!(
        disk.slot_registry.get("llm").unwrap().backend,
        reg["llm"].backend
    );

    reg.get_mut("llm").unwrap().backend = prev_backend;
    save_role_slot_registry_impl(
        &state,
        &SaveRoleSlotRegistryRequest {
            role_id: "mumu".into(),
            slot_registry: reg,
        },
    )
    .await
    .expect("restore");

    let info2 = get_role_info_impl(&state, "mumu", None)
        .await
        .expect("get_role_info");
    assert!(info2.slot_registry_pack.is_some());
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), to).unwrap();
        }
    }
}
