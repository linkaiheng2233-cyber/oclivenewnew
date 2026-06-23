# 插件作者分流

> **读者**：开发目录插件或 Remote 后端的工程师。  
> **耗时**：约 1–2 天入门。  
> **下一篇**：[08 资料地图 §插件](../08_REFERENCE_MAP.md#4-插件)

---

## 建议顺序

1. [00 愿景](../00_VISION_AND_POSITIONING.md) — 六槽与 `builtin` / `remote` / `directory`
2. [02 三十分钟跑通](../02_THIRTY_MINUTE_START.md) — 跑通主仓
3. [PLUGIN_AUTHOR_LEARNING_PATH](../../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)
4. 示例 [`examples/directory-plugin-minimal/`](../../examples/directory-plugin-minimal/)

---

## 契约要点

| 主题 | 文档 |
|------|------|
| manifest | [PLUGIN_V1](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| 权限 | `process:spawn`、`network:*` 须用户授权 |
| 六槽接入 | 蓝图 `slot_registry` 或 legacy `plugin_backends` |
| 打包 | `pack_plugin` Tauri 命令 → `.oclive-plugin` |

---

## 调试

- 目录插件：`{app_data}/distros/chat-pro/plugins/`、`high_risk_grants.json`
- 日志 target：`oclive_plugin`（见 [05 调试](../05_DEBUGGING.md)）
- 编写器调试：姊妹仓 **oclive-pack-editor**

---

## 深度链接

- [DIRECTORY_PLUGINS](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [REMOTE_PLUGIN_PROTOCOL](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)
