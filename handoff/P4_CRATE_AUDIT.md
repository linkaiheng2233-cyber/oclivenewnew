# Phase 4 · 占位 crate / oclive-cli 体量审计（2026-06-05）

## oclive_kernel_server

| 项 | 值 |
|----|-----|
| 路径 | `crates/oclive_kernel_server/` (~83 行 `main.rs`) |
| 依赖 | **`oclive_kernel_host`**（复用 `run_api_server`；另 transitively `oclive_kernel_runtime`） |
| 引用 | workspace 成员；VS Code / launcher **attach 8420 或 spawn**；[`DISTRO_KERNEL_LIFECYCLE.md`](../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) |

**结论：非死代码。** 多发行版单写者 Phase 2 的 headless 内核二进制；保留并标注为「发行版产物」，勿删。

## oclive_runtimed（已删除 · 2026-06-10）

| 项 | 值 |
|----|-----|
| 路径 | ~~`crates/oclive_runtimed/`~~（已删，原 ~184 行） |
| 职责 | 按 `role_id` 串行 `/chat` 的 scheduler 代理 + upstream health |
| 处置 | **2026-06-10 轮次 8 删除**（D-ORPHAN-01）；恢复见 `git log --diff-filter=D -- crates/oclive_runtimed` |

**原结论（2026-06-05）为「保留观察」；后因始终无部署方且不在 workspace，按 D-ORPHAN-01 删除。**

## oclive-cli 体量

| 项 | 值 |
|----|-----|
| `src/` 行数 | ~15,067（2026-06-05 统计） |
| 主要体积 | `init` 代码生成模板、Monolith 桩、`build`/`bench` |

**结论：暂不拆 crate。** 拆分模板至子 crate（如 `oclive_cli_templates`）收益为减主 crate 噪音，成本为 workspace 版本同步与 `cargo install` 路径变更。

**建议（记录即可）：**

1. 将 **嵌入字符串模板**（>500 行的 `.rs.txt` / `include_str!` 块）抽到 `crates/oclive_cli_templates/`，CLI 仅 `include` + 参数化。
2. Monolith 七键焊接桩保持 **单一模板源**（已与 RFC 对齐），避免在 CLI 与 `vendor/oclive_monolith_builtin` 双写。
3. 下一里程碑若 CLI 超过 ~20k 行或模板变更频繁，再开拆 crate PR。
