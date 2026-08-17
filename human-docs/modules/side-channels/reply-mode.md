# 独立通道开工包 · 回复模式

> **读者**：改 `reply_mode` 分段、延迟与前端多气泡渲染的工程师。  
> **读完能做什么**：在 post 独立通道内改回复呈现，不把分段逻辑写进六槽。  
> **耗时**：约 **35 min**  
> **SSOT 范围**：人类 checklist；RFC 见 [RFC_REPLY_MODE](../../../creator-docs/rfc/RFC_REPLY_MODE.md)  
> **最后更新**：2026-08-16

---

## 1. 你插在哪

- **MODULE_MAP**：[§11 `reply_mode`](../../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)
- **配置**：角色包 `config.json` → `reply_mode`
- **锚点**：`turn_pipeline/post.rs` · `reply_post_process` 之后
- **默认**：`single`，与现状完全一致

---

## 2. 边界

| 能改 | 禁止 |
|------|------|
| 分段策略、分隔符协议、前端气泡渲染 | 写入六槽 `plugin_backends` |
| `config.json` 的 `mode` / `segments` / `separator` / `delays_ms` / `streaming` | 在 Vue 层硬编码 role id |
| 一条消息 + `reply_segments` 元数据的落库形状 | 新增聊天表或把分段塞进记忆正文 |

## 3. 阅读清单

1. [RFC_REPLY_MODE](../../../creator-docs/rfc/RFC_REPLY_MODE.md)
2. [MODULE_MAP §11](../../../handoff/MODULE_MAP_AND_HANDOFF.md#11-独立通道能力增强注册表--非六槽)
3. `kernel/crates/oclive_kernel_host/src/domain/reply_mode.rs`
4. `kernel/crates/oclive_kernel_types/src/models/reply_mode_config.rs`
5. `distros/shared/src/utils/replySegments.ts`

## 4. 开发流程

- [ ] 改纯函数先补 `reply_mode.rs` / `replySegments.ts` 单测
- [ ] post 切分保持 `reply` 兼容字段，`reply_presentation` 可选
- [ ] 历史加载用 `metadata.reply_segments` 还原，不重复切 IDB 缓存
- [ ] `npm run test:unit` · `cargo test -p oclive_kernel_host reply_mode`

## 5. 验收

- [ ] 未配置 `reply_mode` 的角色包行为与现状一致
- [ ] 分隔符缺失、超段、空段、CRLF、自定义分隔符均有测试
- [ ] 撤回 / 删除 / 重新生成按一轮整体处理
- [ ] 前端不硬编码角色 id，只读 `pack_reply_mode`
