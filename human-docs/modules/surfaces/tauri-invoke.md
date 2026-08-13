# 宿主面开工包 · Tauri Invoke

> **读者**：新增或修改 Tauri 命令、前后端 IPC 封装的工程师。  
> **读完能做什么**：按注册纪律接线 invoke，热路径对齐 INVOKE_HOTPATH_MATRIX。  
> **耗时**：约 **40 min**  
> **SSOT 范围**：人类 checklist；矩阵见 [INVOKE_HOTPATH_MATRIX](../../../handoff/INVOKE_HOTPATH_MATRIX.md)
> **最后更新**：2026-07-14
> **下一篇**：[07 §1](../../07_COMMON_TASKS.md#1-新增-tauri-命令) · [frontend-chat-pro](frontend-chat-pro.md)

---

## 1. 你插在哪

- **Rust 实现**：`distros/desktop-tauri/src/api/<topic>.rs`  
- **注册**：`distros/desktop-tauri/src/lib.rs` → `generate_handler!` **仅此**  
- **前端封装**：`distros/shared/src/api/*.ts`（**camelCase**）  
- **业务**：委托 `oclive_kernel_host::service::*_impl`

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 新命令 api 模块 + TS 封装 | 在 `lib.rs` 写业务逻辑 |
| DTO 字段（同步 `oclive_kernel_types`） | snake_case 暴露给前端（须 camelCase） |
| 热路径测试 `invoke_hotpath_matrix` | API 层堆 `process_message` 编排 |

---

## 3. 阅读清单

1. [07 §1 新增 Tauri 命令](../../07_COMMON_TASKS.md#1-新增-tauri-命令)  
2. [INVOKE_HOTPATH_MATRIX](../../../handoff/INVOKE_HOTPATH_MATRIX.md)
3. [04 工程约束 §3](../../04_ENGINEERING_RULES.md)  
4. [BUS_FACTOR](../../../handoff/BUS_FACTOR_NOTES.md)
5. [ERROR_CODES](../../../creator-docs/getting-started/ERROR_CODES.md)

---

## 4. 开发流程

- [ ] 在 `api/<topic>.rs` 实现 · `*_impl` 委托  
- [ ] `lib.rs` 注册一行  
- [ ] `distros/shared/src/api/` 加 camelCase 函数  
- [ ] 若改 DTO → `dto/` + 前端类型  
- [ ] `cargo test -p oclivenewnew-tauri` 相关用例

---

## 5. 验收

- [ ] 前端 `invoke('snake_case')` 与 Rust 命令名一致  
- [ ] 响应 JSON 用 **`reply`** 等契约字段  
- [ ] 热路径改动考虑矩阵回归

---

## 6. 联调依赖

| 相关模块 | 数据关系 |
|----------|----------|
| [frontend-chat-pro](frontend-chat-pro.md) | UI 消费 |
| 内核 service | 实际业务 |
| [distro-hostprofile](distro-hostprofile.md) | 发行版能力门控 |
