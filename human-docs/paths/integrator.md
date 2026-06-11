# 集成方分流

> **读者**：无头 HTTP、嵌入式硬件、二次开发内核的工程师。  
> **耗时**：约 1–2 天入门。  
> **下一篇**：[KERNEL_INTEGRATOR_LEARNING_PATH](../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)

---

## 建议顺序

1. [02 三十分钟跑通](../02_THIRTY_MINUTE_START.md)
2. [01 简架构](../01_ARCHITECTURE_SIMPLE.md) — `process_message` 主链
3. [KERNEL_INTEGRATOR_LEARNING_PATH](../../creator-docs/getting-started/KERNEL_INTEGRATOR_LEARNING_PATH.md)（SSOT）
4. `cargo run -p oclive-cli -- init` — 最小骨架

---

## 关键入口

| 能力 | 路径 / 命令 |
|------|-------------|
| 无头 HTTP | `oclivenewnew-tauri --api` 或 `oclive-kernel-server` |
| 健康检查 | `GET :8420/health` |
| OOCP 烟测 | `examples/oocp-test-suite/run.mjs` |
| 共享数据目录 | `OCLIVE_APP_DATA` · [OCLIVE_APP_DATA.md](../../creator-docs/kernel/OCLIVE_APP_DATA.md) |
| VS Code 同源策略 | `resolve_kernel_action` · [CROSS_HOST_MEMORY](../../creator-docs/role-pack/CROSS_HOST_MEMORY.md) |

---

## 验收

- [ ] 能描述 `send_message` 阶段名（对齐 [BUS_FACTOR §1](../../handoff/BUS_FACTOR_NOTES.md#1-内核编排process_message)）
- [ ] 能在本机跑通 `--api` + mock LLM 一轮对话

---

## 深度链接

- [PURE_KERNEL_BOUNDARY](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md)
- [DISTRO_KERNEL_LIFECYCLE](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md) — bundled-first spawn · 单核 attach/replace
- [KERNEL_SCHEDULER_RESCOPE](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) — 调度范围重划
