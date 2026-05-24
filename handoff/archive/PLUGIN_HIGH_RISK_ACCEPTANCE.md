# 目录插件 / MCP 高风险能力 — 验收表（演示向）

**用途**：满足 [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) **§A4.1** 的「可演示」底线：权限弹窗、拒绝后降级、用户可见说明。自动化测试可后续补。

**权威**：宿主行为以 [DIRECTORY_PLUGINS.md](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)、[BRIDGE_API_REFERENCE.md](../creator-docs/plugin-and-architecture/BRIDGE_API_REFERENCE.md) 及 `manifest.json` 权限枚举为准。

---

## 手工演示脚本（建议顺序）

1. 准备测试用目录插件（或最小 manifest），分别声明下列权限之一。  
2. **首次**触发该能力前：应出现**授权/确认**（或等价 UI）；**拒绝**后：功能**降级**且无静默失败（有 toast / 占位 / 日志 reason）。  
3. 在 **设置 → 插件与后端** 或插件管理中核对启用状态与错误提示。

---

## 对照表

| 能力 / 权限别名 | 触发方式（示例） | 期望：首次高风险前 | 拒绝后期望 |
|-----------------|------------------|---------------------|------------|
| **`process:spawn`**（stdio MCP 等） | 启用需拉起子进程的 MCP server 或等价 | 必须显式授权 | 不得启动子进程；调用方可见失败原因 |
| **`network:*` 出站** | Remote 插件 HTTP、HTTP MCP、`network:fetch` 等 | 必须显式授权 | 请求不发；Remote 可回退内置或可见错误 |
| **目录插件读取敏感路径** | manifest `permissions` 与宿主白名单交集 | 越权时拒绝并提示 | 不崩溃主路径 |
| **`directory_plugin_invoke` 高风险 API** | 插件调用桥接表中标注需授权的方法 | 与文档一致 | 拒绝后插件侧收到可解析错误 |

---

## 记录

| 日期 | 宿主版本 | 演示人 | 备注 |
|------|-----------|--------|------|
| 2026-05-15 | oclivenewnew（本分支） | — | 宿主已实现 `high_risk_grants.json` + `HIGH_RISK_CAPABILITY_NOT_GRANTED`；Agent 调试面板可 grant；Remote `network:*` 仍待迭代。 |

---

[产品发版勾选表](./PRODUCT_RELEASE_CHECKLIST.md) · [缺口主清单 §A4](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md#a4-插件与安全边界p0)
