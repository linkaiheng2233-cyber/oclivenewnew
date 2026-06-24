# 官方发行版 handoff 索引

> **仓库叙事**：`oclivenewnew` = **内核平台**（`kernel/`）+ **官方发行版集合**（`distros/`）。对外品牌仍为 OCLive / oclivenewnew。

| 发行版 | 目录 | 说明 |
|--------|------|------|
| **共享桌面层** | [`shared/`](shared/) | `@oclive/desktop-shared` — API、stores、通用 chat 组件 |
| **Chat Pro** | [`chat-pro/`](../distros/chat-pro/) | 桌面主产品（ToolShell / FluentShell） |
| **AI Theater** | [`theater/`](../distros/theater/) | 剧场第三发行版前端 |
| **Tauri 宿主** | [`desktop-tauri/`](../distros/desktop-tauri/) | 共享桌面壳（原 `src-tauri`） |

**架构 RFC**：[ARCHITECTURE_DECOUPLING_RFC.md](./ARCHITECTURE_DECOUPLING_RFC.md)

**迁移路径清单**：[STALE_PATHS_MIGRATION_CHECKLIST.md](./STALE_PATHS_MIGRATION_CHECKLIST.md)

**Chat Pro 发版验收**：[MUMU_UI_ACCEPTANCE_CHECKLIST.md](./MUMU_UI_ACCEPTANCE_CHECKLIST.md)

**校企合作全量镜像仓**：[SCHOOL_ENTERPRISE_FORK.md](./SCHOOL_ENTERPRISE_FORK.md)（推荐仓名 **`oclive-school`** · 初始化 `node scripts/init-school-fork.mjs`）
