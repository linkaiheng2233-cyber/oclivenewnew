# 前端贡献者分流

> **读者**：改 Vue / Pinia / Tauri `invoke` 的工程师。  
> **耗时**：约 1 天入门。  
> **下一篇**：[05 调试](../05_DEBUGGING.md) · [08 资料地图](../08_REFERENCE_MAP.md)

---

## 建议顺序

1. [02 三十分钟跑通](../02_THIRTY_MINUTE_START.md) — `npm run tauri:dev` + `npm run check`
2. [03 术语表](../03_GLOSSARY.md) — **`reply`** 非 `response`；invoke **camelCase**
3. [04 工程约束](../04_ENGINEERING_RULES.md) §3、§4、§7
4. 读本仓 `src/api/` 封装与 `src/stores/chatStore.ts`

---

## 关键路径

| 任务 | 路径 |
|------|------|
| 发消息 | `src/api/chat.ts` → `send_message` |
| 聊天状态 | `src/stores/chatStore.ts` |
| 插件管理 | `Ctrl+Shift+F` → `SimplePluginManagerPanel.vue` |
| 模型管理 | `Ctrl+Shift+M` → `ModelManagerPanel.vue` |
| Tauri 命令全集 | `src-tauri/src/lib.rs` `generate_handler!` |

---

## 测试

| 场景 | 命令 |
|------|------|
| 单元 | `npm run test:unit` |
| 构建 | `npm run build` |
| E2E（Linux CI 对齐） | `npm run test:e2e:preview` |

---

## 深度链接

- [NAMING §8 前端对照](../../creator-docs/NAMING_CONVENTIONS.md#8-前端--后端术语对照)
- [COMPATIBILITY](../../creator-docs/COMPATIBILITY.md)
