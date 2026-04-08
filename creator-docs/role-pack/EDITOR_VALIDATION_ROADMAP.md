# 角色包编写器：校验路线

编写器（独立仓库，例如与本运行时**同级**的 `oclive-pack-editor`）与运行时 **不共享测试进程**；契约与表结构仍以本仓库为**规范原文**。

## 短期（当前）

- **权威**：运行时 **`load_role`**：顶层 JSON 键白名单（`oclive_validation::json_keys`）、合并后的 **`validate_disk_manifest`**、**`validate_min_runtime_version`**（与 `CARGO_PKG_VERSION` 比较），见 `src-tauri/src/infrastructure/storage.rs`。  
- **编写器**：导出前运行 **`manifest.json` / `settings.json` 顶层键检查**（与 Rust 白名单一致，见 `oclive-pack-editor/src/lib/jsonKeys.ts`）；若已构建 wasm（`npm run wasm:build`），则 **`validateManifestWasm`** 与 **`validate_disk_manifest` + `validate_min_runtime_version`** 同源；否则回退 TypeScript 轻量检查 + **`validateMinRuntimeVersion`**（`HOST_RUNTIME_VERSION` 须与 oclivenewnew `Cargo.toml` 对齐）。  
- **验收**：导出包 → 设置 **`OCLIVE_ROLES_DIR`** 指向 roles 根 → 在 oclive 中加载并对话。

## 中期（可选）

- 将 **`role_manifest_validate`** 抽成 **独立 Rust crate**（单独仓库或本仓库子 crate），由：  
  - 运行时 **git 依赖**引用；  
  - 或提供 **CLI**（`oclive-validate-pack path/to/role`），编写器通过子进程/CI 调用。  
- **仍保持两仓独立**：编写器 UI 仓库不嵌套运行时源码；crate 以发布 crate / git tag 为边界。

## 与文档的关系

- 包字段与版本语义：`roles/README_MANIFEST.md`、`creator-docs/role-pack/PACK_VERSIONING.md`。  
- 编写器 README 链到上述路径，避免双份漂移。
