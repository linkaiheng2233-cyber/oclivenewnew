# 技术债六月轮次归档（2026-06）

**历史证据，非当前 truth**：本页仅保留轮次 16–19 的当时 Done 记录，不定义当前债务状态、完成条件或开工权限。当前状态见 [主台账](../TECHNICAL_DEBT_INVENTORY.md)。

**来源**：迁自主台账在 `53479d8cbba6bffe92213dc65fde61fefac24882` 的 §5。四张表的文字和顺序保留，九处相对锚点仅按新目录重定位；历史原文中的“本文件 §1.5”指迁移前的主台账。

### 轮次 16 Done（2026-06-18）

| ID | 项 | 说明 |
|----|-----|------|
| **T-LAYER-16** | Theater 测迁出 domain | `theater_director_resolver` → `distros/desktop-tauri/tests/theater_director_resolver.rs` |
| **T-DOC-TD-01** | `theater_director` 文档扫尾 | DISTRO / ARCHITECTURE / NAMING / ROADMAP §7 / IA 头注 / domain README |
| **T-MINIMAL-TD-01** | minimal 插件自包含 | `examples/directory-plugin-theater-director-minimal/prompts/` 本地 `buildTheaterPrompt` |
| **T-CI-DRIFT-01** | prompt drift 门禁 | `dimension5-acceptance.mjs` + `test:theater:smoke` 双挂 |

### 轮次 17 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| [D-DOCDRIFT-01](../TECHNICAL_DEBT_INVENTORY.md#debt-d-docdrift-01)（历史引用） | monorepo 后文档路径机械迁移 | 一次性迁移脚本已在完成后删除（历史见 `git log -- scripts/migrate-doc-paths.mjs`）；现由 `check-stale-paths.mjs` 持续门禁 |
| [D-SCRIPT-02](../TECHNICAL_DEBT_INVENTORY.md#debt-d-script-02)（历史引用） | `check-stale-paths.mjs` 扩范围 | dimension5 十一检 |
| [D-ORPHAN-04](../TECHNICAL_DEBT_INVENTORY.md#debt-d-orphan-04)（历史引用） | 删 `kernel/crates/models/` 空目录 | workspace 无引用 |

### 轮次 18 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| [O-1](../TECHNICAL_DEBT_INVENTORY.md#debt-o-1)（历史引用） | plugin-bridge 资产内核化 | `kernel/crates/oclive_kernel_host/assets/plugin-bridge.iife.js`；删 desktop-tauri 副本 |
| [O-2](../TECHNICAL_DEBT_INVENTORY.md#debt-o-2)（历史引用） | expert 孤儿前端清理 | 10 文件删；Tauri expert API / validation / dual_core 链保留 |
| [D-DOC-RELOC-01](../TECHNICAL_DEBT_INVENTORY.md#debt-d-doc-reloc-01)（历史引用） | 文档名实归位 | `VSCODE_DISTRIBUTION` → `handoff/vscode/`；`USER_GUIDE` → `handoff/studio/`；`MUMU_UI_ACCEPTANCE` → `handoff/distros/` |

### 轮次 19 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| [K-SUPPLY-01](../TECHNICAL_DEBT_INVENTORY.md#debt-k-supply-01)（历史引用） | `cargo deny` 硬门禁 | dimension5 检查项（licenses+bans）· `ci.yml` dimension5 job 安装 cargo-deny |
| [K-SUPPLY-02](../TECHNICAL_DEBT_INVENTORY.md#debt-k-supply-02)（历史引用） | Release SHA256 | `generate-sha256sums.mjs` · `release-kernel-checksums.yml` · bundle 钩子 |
| [K-SUPPLY-03](../TECHNICAL_DEBT_INVENTORY.md#debt-k-supply-03)（历史引用） | 插件审源码 toast | `installPath` DTO · 市场/git/zip · CLI · i18n |
| **K-SUPPLY-DOC-01** | 供应链策略 SSOT | `creator-docs/security/SUPPLY_CHAIN.md` + 本文件 §1.5 |
