# 角色包编写器：校验路线

编写器（独立仓库，例如与本运行时**同级**的 `oclive-pack-editor`）与运行时 **不共享测试进程**；契约与表结构仍以本仓库为**规范原文**。

## 短期（当前）

- **权威**：运行时 **`load_role`** 与合并后的 **`validate_disk_manifest`**（`src-tauri/src/domain/role_manifest_validate.rs`）。  
- **编写器**：在导出前做与上述逻辑**方向一致**的轻量检查（id、name、`user_relations`、`default_relation`、`topic_weights` 场景键等），避免明显无效包；**不**保证与 Rust 逐条报错文案完全一致。  
- **验收**：导出包 → 设置 **`OCLIVE_ROLES_DIR`** 指向 roles 根 → 在 oclive 中加载并对话。

## 中期（可选）

- 将 **`role_manifest_validate`** 抽成 **独立 Rust crate**（单独仓库或本仓库子 crate），由：  
  - 运行时 **git 依赖**引用；  
  - 或提供 **CLI**（`oclive-validate-pack path/to/role`），编写器通过子进程/CI 调用。  
- **仍保持两仓独立**：编写器 UI 仓库不嵌套运行时源码；crate 以发布 crate / git tag 为边界。

## 与文档的关系

- 包字段与版本语义：`roles/README_MANIFEST.md`、`creator-docs/role-pack/PACK_VERSIONING.md`。  
- 编写器 README 链到上述路径，避免双份漂移。
