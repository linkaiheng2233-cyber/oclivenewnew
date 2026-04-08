# 后日待办 · 工具链与 CI（性价比备忘）

> **性质**：节奏提醒，不是必须立刻偿还的「技术债」。有精力再动；不打算做也可从本文删掉对应条。

---

## 三条一句话规则（记着即可）

1. **契约**：**oclivenewnew** 发版若动到包契约或校验 crate，在 **oclive-pack-editor** 跑 `npm run contract:json-keys`，并对齐 **`HOST_RUNTIME_VERSION`**（见该仓库 [CONTRIBUTING.md](https://github.com/linkaiheng2233-cyber/oclive-pack-editor/blob/main/CONTRIBUTING.md)）。
2. **自动化**：只有「失败会**很晚**才暴露或**影响面大**」时，才值得加重 CI / E2E；能很快人肉发现的问题，不必急着自动化。
3. **矩阵**：主力在 **Windows** 时，**Linux + Windows** CI 已覆盖大头；**macOS CI** 等要正式支持 Mac 包或 Mac 用户明显变多再上。

---

## 以后再说的具体项（可选）

| 项 | 何时值得做 | 备注 |
|----|------------|------|
| 更重 E2E（多浏览器、Tauri 真窗口、全流程联调） | UI / 导出**高频大改**，或对外**质量承诺**提高 | 维护成本与 flaky 风险高 |
| **macOS** 专项 CI 或测试 | 正式发 Mac 安装包或 Mac 反馈变多 | 与分钟数、runner 稳定性权衡 |
| 契约 / 版本号自动化（例如 CI 强制比对 `HOST_RUNTIME_VERSION` 与宿主版本） | **多人协作**、发版**很频繁**、或曾出过**对齐事故** | 可做脚本或 job，非必须 |

---

**相关**：产品体验向 backlog 见 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](./BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)。
