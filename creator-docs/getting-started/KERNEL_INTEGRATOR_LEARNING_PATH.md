# 内核集成方学习路径

面向 **无头 HTTP、嵌入式、硬件侧二次开发**：在自有设备上拉起与 **oclive 内核** 同契约的运行时。边界必读 [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)；脚手架见 **`oclive-cli`**（`cargo run -p oclive-cli -- …`）。

---

## 入门（约 30 分钟）

| 步骤 | 做什么 | 读什么 |
|------|--------|--------|
| 1 | 理解「内核居中、六槽环绕」 | [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md) |
| 2 | 理解「纯净内核」与 library / kernel_server 分工 | [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md) |
| 3 | 生成最小无头/库骨架 | `cargo run -p oclive-cli -- init`（交互或 `--non-interactive`；见 [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)） |

**验收**：能在本机 `cargo build` 生成的骨架工程，并能区分生成工程的根级 `roles/` / `plugins/` 与主仓 `distros/chat-pro/roles/` / `plugins/`。当前非双核 `init` 示例仍是 legacy `roles/default/settings.json`；新 Stable 角色包以 v4 `pipeline.ocblueprint` 为 SSOT。

---

## 进阶（约 1–2 小时）

| 主题 | 读什么 |
|------|--------|
| **`process_message` 编排** | 宿主参考实现 **`kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`** 与 **`turn_pipeline.rs`**（[handoff/BUS_FACTOR_NOTES.md](../../handoff/BUS_FACTOR_NOTES.md) 摘要） |
| **`PluginHost` 第 1–6 模块** | **`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`** · [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) · [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| **`slot_registry`、legacy `plugin_backends` 与兜底** | [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) · 远端失败回退相关设置见 [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md) / 设置页「兜底」 |

**验收**：能描述一条 `send_message` 从入口到 LLM 再持久化的大致阶段名（便于对齐日志）。

---

## 高级（约半天）

| 主题 | 读什么 |
|------|--------|
| **OOCP / HTTP 与内核对话** | [OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) · 示例 [`examples/oocp-test-suite/`](../../examples/oocp-test-suite/) · 无头最小示例 [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |
| **Monolith 使用场景** | 需要 **编译期七焊接键焊接**、减少动态解析时选用；[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) · `oclive-cli init --monolith` + `build` / `bench` |
| **`--kernel-source`** | [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)（将脚手架依赖切到 path 的 `oclive_kernel_runtime` / `oclive_kernel_server`） |
| **进程内稳定门面** | `oclive_kernel_host::OcliveKernel`；生成 library 后按 [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) §5 完成角色加载、回合与关闭 |
| **平台单线文档** | [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) |

**验收**：能在目标设备上跑 **`--api` 健康检查**，或在自有进程中 `OcliveKernel::start → load_role → process_message → shutdown`；并完成一轮最小对话（可注入 mock `LlmClient`，或为 HTTP 开 mock LLM 环境变量）。

---

## 与主应用仓库的关系

- **契约**（DTO、`KernelErrorBody`）以 **`oclive_kernel_runtime`** 与 [KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md) 为准。  
- **完整宿主编排**只在 `kernel/crates/oclive_kernel_host` 维护一份，桌面薄壳位于 `distros/desktop-tauri`。可信 Rust 宿主应通过 **`OcliveKernel`** 使用它；`oclive-cli ... --project-type library --kernel-source` 已链接完整门面，而仅手工依赖 `oclive_kernel_runtime` 仍只有纯契约/策略。
- 嵌入式可裁剪外围，但应保持 **错误 JSON 形状** 与 Event Ring 权威边界一致，便于共用 FAQ、编写器和诊断工具；不要直接依赖内部 `AppState` 拼第二条编排。
