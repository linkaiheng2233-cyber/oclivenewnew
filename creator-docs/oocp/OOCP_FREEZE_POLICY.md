# OOCP 冻结策略（v0.x → v1.0）

> 生效范围：OOCP 协议自身（方法名 / 参数 key / 事件名 / 错误码 / capabilities schema）  
> 关联文档：[`OOCP_SPEC_v0_1.md`](./OOCP_SPEC_v0_1.md)、[`OOCP_TRANSPORTS.md`](./OOCP_TRANSPORTS.md)、[`OOCP_SPEC_COMPLETE_REFERENCE.md`](./OOCP_SPEC_COMPLETE_REFERENCE.md)（实现级编排）

---

## 1) 为什么要冻结

oclive 希望成为“内核 + 发行版（distribution）”式的平台。平台化的前提是：

- 发行版实现者可以只基于 OOCP 文档开发，而不需要读 Rust/Tauri 源码；
- 内核升级不会频繁打破发行版兼容性；
- 生态可以围绕稳定契约扩展，而不是围绕某个 UI/框架耦合实现扩展。

---

## 2) 冻结对象（协议宪法）

### 2.1 永久冻结（任何版本都不改名）

- `SendMessageResponse` 中的字段 **`reply`**（不是 `response`）

### 2.2 v1.0 冻结（进入兼容承诺）

当 OOCP 协议发布 v1.0 后，以下对象进入冻结与兼容承诺：

- **方法名**：如 `session.create` / `chat.send_message` 等
- **方法参数 key**：如 `session_ns` / `role_id` / `user_message`（含 camelCase/underscore 的 JSON key 约定）
- **事件名**：如 `chat.monologue` / `trace.append`（若存在）
- **错误码**：如 `INVALID_PARAMS` / `UNSUPPORTED_METHOD` 等
- **capabilities 首帧 schema**：`{ type, version, methods, events, limits, auth_required }`

---

## 3) 版本与兼容规则（语义版本）

OOCP 的协议版本由 `capabilities.version` 给出，遵循语义版本（SemVer）。

### 3.1 MINOR（向后兼容）

允许：

- 新增 **可选** 字段（客户端忽略未知字段仍可工作）
- 新增方法 / 事件（旧客户端不使用即可）
- 扩展错误 `data` 的结构（保持 `code`/`message` 不变）

### 3.2 MAJOR（不兼容）

不允许在 v1.0 之后随意做；必须走 MAJOR：

- 删除或重命名方法
- 删除或重命名参数 key
- 改变返回结构的语义（例如把字符串改成对象，或改变关键字段含义）
- 删除或重命名错误码

---

## 4) Deprecation（弃用）流程

当需要从 v1.x 迁移到 v2.0：

1. 在 v1.x 先标注 deprecated（文档 + capabilities 可选标记）
2. 至少保留 **2 个 MINOR** 的迁移窗口
3. 提供迁移指南（旧方法 → 新方法映射）

---

## 5) 实施原则（对维护者）

- **契约优先**：先改 spec/冻结策略，再改实现
- **capabilities 作为唯一入口**：发行版只依赖 capabilities 探测，不依赖隐式假设
- **最小稳定集合**：尽量保持“核心闭环方法集”稳定，小功能通过新增方法/事件扩展

