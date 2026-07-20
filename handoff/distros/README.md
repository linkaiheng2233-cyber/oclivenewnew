# 官方发行版 handoff 索引

> **仓库叙事**：`oclivenewnew` = **内核平台**（`kernel/`）+ **官方发行版集合**（`distros/`）。对外品牌仍为 OCLive / oclivenewnew。

| 发行版 | 目录 | 说明 |
|--------|------|------|
| **共享桌面层** | [`shared/`](shared/) | `@oclive/desktop-shared` — API、stores、通用 chat 组件 |
| **Chat Pro** | [`chat-pro/`](../../distros/chat-pro/) | 桌面主产品（ToolShell / FluentShell） |
| **AI Theater** | [`theater/`](../../distros/theater/) | 剧场第三发行版前端 |
| **Tauri 宿主** | [`desktop-tauri/`](../../distros/desktop-tauri/) | 共享桌面壳（原 `src-tauri`） |

**架构 RFC**：[ARCHITECTURE_DECOUPLING_RFC.md](./ARCHITECTURE_DECOUPLING_RFC.md)

**Tauri v1→v2 迁移清单**（K-PLATFORM-01a Partial）：[TAURI_V2_MIGRATION_INVENTORY.md](./TAURI_V2_MIGRATION_INVENTORY.md)

**迁移路径清单**：[STALE_PATHS_MIGRATION_CHECKLIST.md](./STALE_PATHS_MIGRATION_CHECKLIST.md)

**Chat Pro 发版验收**：[MUMU_UI_ACCEPTANCE_CHECKLIST.md](./MUMU_UI_ACCEPTANCE_CHECKLIST.md)

**校企合作全量镜像仓**：[SCHOOL_ENTERPRISE_FORK.md](./SCHOOL_ENTERPRISE_FORK.md)（推荐仓名 **`oclive-school`** · 初始化 `node scripts/init-school-fork.mjs`）

## Distribution gaps (K-DIST-01 Minimal)

> **范围**：缺口清单与人工前置项。**Minimal 文档 ≠ Full 分发收口**；不采购证书、不上商店、不改安装体验。
> **Full**（真签名 + updater + 平台包发布）= 另册 · `blocked:needs-signing-secrets`。
> 供应链基线链：[SUPPLY_CHAIN.md](../../creator-docs/security/SUPPLY_CHAIN.md) · 台账 [TECHNICAL_DEBT · K-DIST-01](../TECHNICAL_DEBT_INVENTORY.md)。

| 缺口 | 现状 | 备注 |
|------|------|------|
| **Code signing** | **缺** Authenticode / Apple 签名与 notarize | 有：plugin pack 侧车签名（`desktop-tauri` `plugin_pack`）· 内核 [SHA256SUMS](../../creator-docs/security/SUPPLY_CHAIN.md#3-核对-release-内核哈希) |
| **Updater** | **缺** `tauri-plugin-updater` | `tauri.conf.json` 无 updater 插件；`plugin_update` 仅目录插件，非应用自更新 |
| **Linux packages** | **缺** deb/AppImage 等发布 CI | [`bundle.targets`](../../distros/desktop-tauri/tauri.conf.json) = `"all"`（本地可打）· 无发行流水线 |
| **macOS dmg** | **缺** 公证 + dmg 发布 | 同上 · 依赖 Apple 证书 / notarize |

**人工前置（Full 阻塞）**

| 项 | 说明 |
|----|------|
| Windows Authenticode 证书 | 维护者采购 + CI 秘密；**禁止**马拉松夜间代购 |
| Apple Developer + notarize | 维护者账号 / 证书；另册 |
| Updater 签名密钥 | Tauri updater 公私钥与发布端点；密钥不进仓 |

计划书：[K-DIST-01](../debt-marathon/long-plans/K-DIST-01.md)。
