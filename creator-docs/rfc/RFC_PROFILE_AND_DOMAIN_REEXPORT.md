# RFC：发行版 Profile 解析统一与 Host Domain 再导出收敛

| 元数据 | 值 |
|--------|-----|
| 状态 | **已落地（Phase 2，2026-06-08）** — 共享 TOML 解析 + host re-export deprecated + ratchet |
| ID | K-PROFILE-01 · D-OPUS-05 · D-OPUS-06 |
| 受众 | 内核 / 发行版集成 / CLI ensure 维护者 |
| 相关 | [DISTRO_CAPABILITY_PROFILE.md](../kernel/DISTRO_CAPABILITY_PROFILE.md) · [NAMING_CONVENTIONS.md](../NAMING_CONVENTIONS.md) §4.2 |

---

## 1. 问题陈述

### 1.1 双份 TOML 解析（K-PROFILE-01 / D-OPUS-06）

`distro.oclive.toml` 曾在两处各自 `toml::from_str` 同名段：

| 路径 | 视图 | 用途 |
|------|------|------|
| `oclive_kernel_runtime::kernel_distro_profile` | `DistroProfileRequirements` | attach/replace、`kernel_strategy` |
| `oclive_kernel_host::host_profile` | `HostProfile` | memory retrieval、post_process、state_expression、`active_profile_summary` |

字段漂移风险：`host_flags` / `slots` / `prompt` 等同名段在两处独立 struct 维护。

### 1.2 Host `pub use runtime::domain::*`（D-OPUS-05）

`oclive_kernel_host::domain/mod.rs` 整块 re-export ~15 个 runtime 子模块，使 `crate::domain::prompt_builder` 与 `oclive_kernel_runtime::domain::prompt_builder` 并存，canonical import 表模糊。

---

## 2. 决策

### 2.1 单一解析入口（K-PROFILE-01）

1. **SSOT 模块**：`oclive_kernel_runtime::distro_oclive_file`
2. **API**：
   - `parse_distro_oclive_toml(raw) -> DistroOcliveFile`
   - `parse_distro_oclive_file(path) -> DistroOcliveFile`
   - `DistroOcliveFile::into_requirements(path_hint) -> DistroProfileRequirements`
3. **运行时视图**：`host_profile::host_profile_from_distro_file(&DistroOcliveFile) -> HostProfile`（枚举解析仍留在 host，因依赖 `PluginBackends`）
4. **兼容**：`K-PROFILE-03` hash/summary 行为不变；`parse_distro_requirements_*` 保留为薄包装。

字段表与 [DISTRO_CAPABILITY_PROFILE.md](../kernel/DISTRO_CAPABILITY_PROFILE.md) §3 对齐。

### 2.2 Re-export 分阶段收敛（D-OPUS-05，策略 B）

| 阶段 | 动作 |
|------|------|
| **当前** | `domain/mod.rs` re-export 块标记 `#[deprecated(since = "0.3.1")]`；host crate `#![allow(deprecated)]` 直至 ratchet 归零 |
| **门禁** | `scripts/check-host-reexport-imports.mjs` + `handoff/HOST_REEXPORT_BASELINE.json`（仅允许 ratchet 下降） |
| **新代码** | 禁止新增 `use crate::domain::{memory_engine,…}`；应 `use oclive_kernel_runtime::domain::…` |
| **终态** | ratchet → 0 后删除 re-export 块 |

未选策略 A（全仓一次性改 import）：回归面过大。未选策略 C（维持 status quo）：与 Dimension 5 设计债目标冲突。

---

## 3. 验收

```bash
node scripts/e2e-kernel-profile.mjs
cargo test -p oclive_kernel_runtime kernel_distro distro_oclive
cargo test -p oclive_kernel_host host_profile
node scripts/check-host-reexport-imports.mjs
node scripts/dimension5-acceptance.mjs --ci
```

---

## 4. 修订记录

| 日期 | 说明 |
|------|------|
| 2026-06-08 | 初版 + 实现：共享 `DistroOcliveFile`、deprecated re-export、ratchet 基线 |
