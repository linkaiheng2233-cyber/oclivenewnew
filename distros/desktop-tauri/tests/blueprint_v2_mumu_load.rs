//! 仓库黄金包 `roles/mumu`（v2 蓝图）可经 `RoleStorage` 加载。

mod common;

use oclive_kernel_host::infrastructure::storage::RoleStorage;

#[test]
fn load_migrated_mumu_blueprint_pack() {
    let roles_dir = common::roles_dir();
    let storage = RoleStorage::new(&roles_dir);
    let role = storage
        .load_role("mumu")
        .expect("load migrated mumu role pack");
    assert_eq!(role.id, "mumu");
    assert!(role
        .slot_registry
        .as_ref()
        .is_some_and(|r| r.contains_key("llm")));
    assert_eq!(role.ollama_model.as_deref(), Some("qwen2.5:7b"));
}
