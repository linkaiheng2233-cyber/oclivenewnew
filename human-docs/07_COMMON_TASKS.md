# 07 · 常见任务食谱

> **读者**：已读完 L5、要动手改代码的内核/全栈贡献者。  
> **读完能做什么**：按场景找到首要文件与须同步的文档/测试。  
> **耗时**：按需查阅。  
> **下一篇**：[08 资料地图](08_REFERENCE_MAP.md)。

完整导航表亦见 [CONTRIBUTING.md §代码导航](../CONTRIBUTING.md#代码导航按问题域)。

---

## 1. 新增 Tauri 命令

| 步骤 | 位置 |
|------|------|
| 实现 | `distros/desktop-tauri/src/api/<topic>.rs` |
| 注册 | `distros/desktop-tauri/src/lib.rs` → `generate_handler!` |
| 前端 | `distros/shared/src/api/*.ts`（**camelCase** 键，如 `pluginId`） |
| 业务 | 委托 `oclive_kernel_host::service::*_impl`，**勿在 api 堆编排** |

**还须同步**：若改 DTO → `oclive_kernel_types` + [ERROR_CODES](../creator-docs/getting-started/ERROR_CODES.md)

---

## 2. 改 Prompt 段落

| 步骤 | 位置 |
|------|------|
| 段落公式 | `kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/sections.rs` |
| 组装顺序 | `prompt_builder/mod.rs` |
| 注入输入 | `turn_pipeline/pre.rs` → `PromptInput` 字段 |
| guardrails | **内核常量** `KERNEL_DIALOGUE_GUARDRAILS`，不可被角色包替换 |

**约束**：`build_prompt(&PromptInput)` 返回 `String`，非 `Result`。

---

## 3. 加 `config.json` 字段

| 步骤 | 位置 |
|------|------|
| 解析 | `RoleStorage::load_role` / 相关 loader |
| 校验 | `kernel/crates/oclive_validation` |
| 文档 | [ROLE_PACK_SPEC](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| 使用 | 对应 `*_engine` 或 `turn_pipeline`，**非** API 层 |

---

## 4. 写 domain 单测（推荐首 PR）

```rust
// 模式：distros/desktop-tauri/tests/ 或 kernel/crates/oclive_kernel_host 内 #[cfg(test)]
let state = AppState::new_in_memory_with_llm(/* … */).await;
// 调用 domain 函数，断言 Result
```

参考：`distros/desktop-tauri/tests/invoke_hotpath_matrix.rs`、`narrative_hint_prompt_roundtrip.rs`

**命令**：`cargo test -p oclivenewnew-tauri --test <name>` 或 `npm run check:release`

---

## 5. 改 `plugin_backends` / 槽位解析

| 步骤 | 位置 |
|------|------|
| 合并规则 | `slot_runner.rs`（读函数头注释） |
| 解析 | `slot_resolver.rs`、`plugin_host/resolver.rs` |
| 后端表 | `infrastructure/backend_registry.rs` |
| 校验 | `oclive_validation` + [PLUGIN_V1](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |

---

## 6. 新持久化字段

| 步骤 | 位置 |
|------|------|
| SQL | `kernel/crates/oclive_kernel_host/migrations/0NN_*.sql` |
| trait | `domain/repository.rs` |
| impl | `infrastructure/repositories.rs` |
| **禁止** | 虚构表名；以 `001_init.sql` 与后续迁移为准 |

---

## 7. 改 HTTP / OOCP 契约

| 步骤 | 位置 |
|------|------|
| 路由 | `oclive_kernel_host/src/http_api/` |
| DTO | `oclive_kernel_types/src/models/dto.rs` |
| 错误码 | [KERNEL_ERROR_CODE_CONVENTION](../creator-docs/getting-started/KERNEL_ERROR_CODE_CONVENTION.md) |
| 黑盒 | `examples/oocp-test-suite/` |

---

## 8. 调整共景阶段顺序（高危）

| 步骤 | 位置 |
|------|------|
| 主编排 | `turn_pipeline.rs` / `co_present.rs` |
| **慎重** | 属主编排；须 OOCP / 集成测 + [DESIGN_DECISIONS](../creator-docs/architecture/DESIGN_DECISIONS.md) |

---

## 验收

- [ ] 能根据任务类型在 30 秒内打开首要文件
- [ ] 知道 Tauri 新命令要同步 `distros/shared/src/api/` camelCase

---

## 深度链接

- [CONTRIBUTING §常见修改场景](../CONTRIBUTING.md#常见修改场景)
- [EXTENSION_POINTS](../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)
