# 子计划：OOCP WebSocket（壳层）与 `axum` 去依赖评估

> **定位**：中长期项；与 `LIGHTWEIGHT_PROFILE.md` §5.1 中「壳层仍直连 `axum`」一行对应。  
> **主计划**：`handoff/LIGHTWEIGHT_FOLLOWUP_PLAN.md` 阶段 2（依赖去重）**不**与本子计划绑在同一 PR。

## 现状

- `src-tauri/src/domain/adapters/oocp_ws.rs` 使用 **`axum`** WebSocket 类型与中间件，与桌面进程内 OOCP 客户端路径耦合。
- Runtime（`kernel-http-api`）侧已在 `crates/oclive_kernel_runtime/src/http_api.rs` 等模块承载 Axum HTTP + OOCP WS 的**无头宿主**形态。

## 目标（评估用，非承诺排期）

1. **行为对齐**：列出 OOCP WS 在「桌面适配器」与「runtime `http_api`」两侧的帧序列、错误语义、超时与重连策略差异；收敛到单一事实或明确「桌面专有」分支。
2. **边界**：若 runtime 可暴露「仅 WS 升级 + 路由注册」的薄 API，评估桌面是否改为调用该 API，从而去掉 `oclivenewnew-tauri` 对 **`axum` 的直接依赖**（仍可能经 `oclive_kernel_runtime` 传递依赖链接 Axum，但壳层 `Cargo.toml` 不再声明）。
3. **风险**：Tauri 生命周期、`AppHandle` 注入与 Axum `Extension` 的耦合；需回归 OOCP 握手与多窗口场景。

## 验收草案

- `cargo check -p oclivenewnew-tauri`（默认特性）无壳层 `axum` 直连条目（以 `Cargo.toml` 为准）。
- 集成或 E2E：OOCP WS 连接与至少一条 chat 方法往返仍通过。

## 文档维护

- 决策或搁置结论更新 `creator-docs/kernel/LIGHTWEIGHT_PROFILE.md` §5.1 表格中 **`axum`** 行与 `handoff/LIGHTWEIGHT_FOLLOWUP_PLAN.md` 的阶段 2 脚注。
