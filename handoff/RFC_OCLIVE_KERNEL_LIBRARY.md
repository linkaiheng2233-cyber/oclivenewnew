# RFC · OCLive 宿主无关 Library API（V1 草案）

**状态**：Wave 5 · T0 契约（2026-06-25）  
**驱动**：第二宿主（游戏 / 嵌入式 / 无头服务）立项前须冻结 crate 边界  
**配套**：[`ARCHITECTURE_LAYERING.md`](ARCHITECTURE_LAYERING.md) · [`ROLE_PACK_BOUNDARY.md`](ROLE_PACK_BOUNDARY.md)

---

## 1. 目标

提供 **不依赖 Tauri / 特定发行版 UI** 的 Rust API，使 `process_message` 编排可在：

- 无头 `oclive-kernel-server`
- 游戏/嵌入式宿主
- VS Code attach（已有 HTTP 薄客户端，非完整 library）

上 **以同一套 trait 与 DTO** 运行。

## 2. 非目标（本 RFC 不做）

- 大规模移动 `process_message` 出 `oclive_kernel_host`（与 Wave 4 结构债冲突）
- 为抽离而拆六槽叙事
- 默认开启 `dual_core` / `expert_routing`

## 3. Crate 边界（提议）

| Crate | 纳入 library 面 | 保留 host-only |
|-------|-----------------|----------------|
| `oclive_kernel_types` | 全部 DTO / 错误 | — |
| `oclive_kernel_contracts` | 全部 trait | — |
| `oclive_kernel_runtime` | `PromptBuilder`、policy、analyzer | Tauri 专用 |
| `oclive_kernel_host` | `process_message`、HTTP router（feature `api`） | `AppState` 桌面 wiring |
| **新 `oclive_kernel_library`（可选）** | 稳定 re-export 桶 + `KernelSession` 门面 | 实现细节 |

**原则**：library 门面 **re-export**，不复制编排逻辑。

## 4. 门面形状（草图）

```rust
// 未来 oclive_kernel_library
pub struct KernelSession { /* AppState 或薄包装 */ }

impl KernelSession {
    pub async fn send_message(&self, req: SendMessageRequest) -> Result<SendMessageResponse>;
    pub async fn generate_theater_scene(&self, req: TheaterSceneRequest) -> Result<TheaterSceneResponse>;
}
```

Theater / 独立通道 API **与** chat 主链 **并列导出**，不并入六槽。

## 5. 解冻条件

- [ ] 第二宿主 PoC 仓库立项
- [ ] Wave 4 D-PORT-02 窄 trait 至少 memory/emotion 两组落地
- [ ] `invoke_hotpath_matrix` + OOCP 全绿

## 6. 验收

- 文档评审 + `oclive_validation` 键表无漂移
- 示例 crate `examples/kernel-library-hello/`（仅 `cargo check`，可后续 PR）
