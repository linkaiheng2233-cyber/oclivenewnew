# 已知漏洞跟踪（Cargo / npm）

本文件以工作区根 **`Cargo.lock`** 上 `cargo audit` 的**漏洞级（vulnerability）**命中作为供应链风险与升级路线的单一事实来源；警告级 *unmaintained* / *unsound* / *yanked* 不计入漏洞数，但在文末附表维护当前风险与上游阻塞，详情仍以本轮 `cargo audit` 输出和 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) 为准。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**轻量化与审计流程**：[../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## 当前状态

| 项 | 值 |
|----|-----|
| **cargo-audit 版本** | **0.22.2**（建议固定该主版本以便报告可比） |
| **最近扫描日期** | **2026-09-27**（本地 `cargo audit --no-fetch --stale`；advisory-db 1269 条，扫描 698 个依赖条目） |
| **扫描路径** | 工作区根目录 `Cargo.lock` |
| **漏洞级命中数** | **0**（`cargo audit` 退出码 **0**） |
| **警告级命中数** | **7**（`glib` · 5 个 `unic-*` · `chacha20` 0.10.1 yanked；本轮不改变既有 ignore 策略） |

> 若 CI 或本机无法拉取 advisory-db，可使用：`cargo audit --no-fetch --stale`（依赖本地已 fetch 的数据库）。

本次锁文件仅将 `rustls` 从 0.23.43 更新到 0.23.45（版本和校验和两行），修复 RUSTSEC-2026-0285；没有改动其他 package 或依赖边。此前可选 `media-png` 依赖边的记录属于 2026-09-05 扫描。扫描结果是时点证据，不构成所有运行配置的安全保证。

---

## 漏洞清单（漏洞级）

| RUSTSEC ID | Crate | 状态 | 说明 |
|------------|-------|------|------|
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) | `rsa` 0.9.10（经 `sqlx-mysql`） | **已清零** | workspace 经 `kernel/crates/oclive_sqlx` 直引 `sqlx-sqlite`；锁文件无 MySQL 路径 |
| [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) | rustls-webpki 0.101 | **已清零** | |
| [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) | rustls-webpki 0.101 | **已清零** | |
| [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104) | rustls-webpki 0.101 | **已清零** | |
| [RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363) | sqlx 0.7.4 | **已清零** — 已升级至 **0.8.6** | |
| [RUSTSEC-2026-0185](https://rustsec.org/advisories/RUSTSEC-2026-0185) | quinn-proto &lt; 0.11.15 | **已修复** — 锁文件 **0.11.17** | 2026-08-21 兼容范围锁文件更新 |
| [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204) | crossbeam-epoch 0.9.18 | **已修复** — **0.9.20** | 2026-07-09 PR #101 CI 供应链 |
| [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285) | rustls 0.23.43 | **已修复** — 锁文件 **0.23.45** | 2026-09-27 兼容补丁更新；TLS 1.3 握手消息加密级别边界检查 |
| [RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194) | quick-xml 0.39.4 | **已修复** — **0.41.0**（经 plist 1.10） | 同上 |
| [RUSTSEC-2026-0195](https://rustsec.org/advisories/RUSTSEC-2026-0195) | quick-xml 0.39.4 | **已修复** — **0.41.0** | 同上 |

---

## 解决路线图

### 已完成（2026-05-20）

- **sqlx ≥ 0.8.6**，`default-features = false`，features：`runtime-tokio-rustls`、`sqlite`（无 umbrella `migrate`）。
- 迁移：`kernel/crates/oclive_kernel_host/src/infrastructure/sql_migrate.rs` 运行时应用 `migrations/*.sql`，与既有 `_sqlx_migrations` 表兼容。
- **CI**：主工作流由 `dimension5-acceptance` 唯一持有 `cargo audit`，失败即红；`Cargo.lock` PR 另走 `cargo-audit-lockfile.yml`。

### 维护约定

1. 更新锁文件或升级依赖后，在仓库根目录运行：  
   `cargo audit`  
   若网络受限：`cargo audit --no-fetch --stale`
2. 将 **漏洞级** 变化同步到上表；将策略变化同步到 [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md)。
3. 不在对外文案中宣称「零漏洞」；按上表实际命中数表述（例：**「漏洞级 N 条开放（警告级仍跟踪）」** 或清零时 **「漏洞级已清零（警告级仍跟踪）」**），并链接本文件。

---

## 警告级跟踪（滚动更新）

| RUSTSEC / 类别 | Crate | 状态 | 原因 |
|----------------|-------|------|------|
| **RUSTSEC-2026-0002** | `lru` | **已修复** | `oclive-cli` 升级 **ratatui 0.30** → `lru` ≥ 0.16 |
| **RUSTSEC-2025-0134** | `rustls-pemfile` | **已修复** | `reqwest` **0.12** 链不再依赖该 crate |
| gtk-rs GTK3 簇（10 ID） | `gtk`/`gdk`/… | **已记录；9 条 audit.toml ignore，`gdkx11` 继续显示为警告** | Linux WebView（wry/webkit2gtk）仍拉 GTK3；Tauri 2 后仍需上游切换方可移除 ignore |
| **RUSTSEC-2025-0075 / 0080 / 0081 / 0098 / 0100** | `unic-*` 0.9 | **开放** | 经 Tauri `urlpattern` 传递引入；等待上游移除未维护依赖 |
| **RUSTSEC-2026-0221** | `event-listener` 5.4.1 | **已修复 · K-SUPPLY-11** | 2026-08-01 锁文件升级至 **5.4.2**；SQLx 与 zbus/Tauri 两条路径均已解析到修复版，未加入 ignore |
| **RUSTSEC-2025-0057** | `fxhash` | **已清零** | 2026-07-14 K-PLATFORM-01a Full · Tauri 2 锁图无 `fxhash` |
| **RUSTSEC-2024-0429** | `glib` | **开放** | `VariantStrIter` 路径；宿主未使用（Linux wry） |
| yanked | `spin` 0.9.8 | **已修复** — 锁文件 **0.9.9** | 2026-08-21 兼容范围锁文件更新；仍经 `flume` → `sqlx-sqlite` 引入 |
| yanked | `chacha20` 0.10.1 | **已观察，未处理** | 2026-09-05 当前 audit 命中；该版本在本轮之前已锁定，不是新增 PNG 依赖。仅同步风险记录，不在角色媒体切片中升级无关依赖 |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **已清零** | 2026-07-14 K-PLATFORM-01a Full · Tauri 2 后无 `rand` 0.7 |
| **RUSTSEC-2026-0190** | `anyhow` | **已修复** — 锁文件 **1.0.104** | 2026-08-21 锁文件复核 |

忽略列表与理由见 [`.cargo/audit.toml`](../../.cargo/audit.toml) 与 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)。

---

## npm / 版本号复核（2026-08-21）

| 项 | 原状 | 修正 |
|----|------|------|
| `eslint` | 根依赖 9.39.5 不满足 Unicorn 68 的 ESLint ≥10.4 peer 契约 | **已修复并远端验证 · K-SUPPLY-12**：ESLint **10.8.0** + Antfu **9.2.0** + Unicorn **72.0.0**，`npm ls eslint eslint-plugin-unicorn` 退出 0 |
| `webdriverio` | 9.29.1 经 `edgedriver` 解析到命中审计的 `fast-xml-parser` 5.10.0 | 升级 **9.30.0** 并解析到修复版 `fast-xml-parser` **5.10.1** |
| 开发模式 SFC 编译 | `vue3-sfc-loader` 0.9.5 捆绑 Vue 2 compiler 与旧 PostCSS | 移除旧 loader；改用官方 `@vue/compiler-sfc` + 受限 DEV-only 转换，只允许导入 `vue`，发行 bundle 编译器标记为 0 |
| `vue-virtual-scroller` | 首屏全局 `app.use`，但 UI 已改用 `VirtualScrollContainer` | **移除依赖**；首屏不再同步加载 |
| `sha2`（`distros/desktop-tauri`） | `0.11.0`（crates.io 无稳定 0.11 线） | **`0.10`**（`sha2 0.10.9`） |
| `@antfu/eslint-config` | `^9.0.0` | 当前解析版 9.1.0 的传递依赖已越过 ESLint 9 peer 范围；与上一行一起处理 |
| `serde_yaml` | `0.9`（crate archived，维护停止） | **`serde_yaml_ng 0.10`**（workspace 全量替换 `use serde_yaml_ng::`） |
| `zip`（`distros/desktop-tauri`） | `0.6`（RUSTSEC 跟踪中） | **`2.x`**（`role_pack` / `plugin_pack` API 已适配） |

复核命令：`npm outdated`（根目录）、`cargo tree -p oclivenewnew-tauri -i sha2`、`cargo tree -p oclivenewnew-tauri -i serde_yaml`（应为空）。

---

## npm 供应链（滚动时点）

### 2026-10-06 新公告修复（本地验证，待目标 SHA CI）

`8cd7d5d1` 的正式 CI [37424944008](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37424944008) 在生产扫描命中三项 high；本机完整扫描另有开发测试链 critical。该失败不归因资源策略源码，也不能把此前扫描的 0 当作当前无风险证据。

| 公告 / 依赖 | 当前修复 / 边界 |
|---|---|
| [GHSA-g2v6-rqmx-r4w6](https://github.com/advisories/GHSA-g2v6-rqmx-r4w6) · Vue SSR 属性名校验 | Vue / compiler / renderer 统一 **3.5.43**，仍在原 3.5 补丁线；公告修复下限 3.5.42。锁命中不等于已证明本产品存在可达 SSR 攻击路径 |
| [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q) · source-map-js | **1.2.2**，原传递范围允许的补丁；未添加 override |
| [GHSA-5gmw-xhrv-c9v3](https://github.com/advisories/GHSA-5gmw-xhrv-c9v3) / [GHSA-85c8-ppgw-ccpr](https://github.com/advisories/GHSA-85c8-ppgw-ccpr) · Tinypool | 原 Vitest 3.2.7 的 1.x 链无兼容修复；三个 workspace 与根统一固定 **Vitest 4.1.11**，该依赖图不再含 tinypool，也修复其 mocker 已知命中。不升级 Vitest 5，不放宽 peer 或审计 |

原 npm 10.9.8 锁生成遇 Arborist `edgesOut` null；只临时使用官方 npm 11.21.0 生成锁，随后原 npm 10 `npm ci`、`npm ls --all` 均 exit 0。未改变全局工具链或 CI 配置。当前本地生产 JSON 扫描 **0**；完整图 **4 low / 1 moderate / 0 high / 0 critical**，两条 `--audit-level=high` 门禁均 exit 0。残余分别为 ESLint Markdown→KaTeX 链与 postcss-selector-parser，不能称完整图零漏洞；扫描结果仅对应本轮锁与时点。验证、失败和正式出口见 [DCL-48](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261006-48--新披露-npm-风险与测试链修复)。

### 历史已验证时点（2026-08-21）

CI **`npm-audit`** job 以硬门禁运行 `npm audit --omit=dev --audit-level=high`；远端 CI [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) 的生产依赖扫描为 **0 vulnerabilities**。本地复现：仓库根目录运行同一命令。

K-SUPPLY-12 冻结实现 `728219e7` 上，完整 `npm audit` 与生产扫描均为 **0 vulnerabilities**，`npm ls eslint eslint-plugin-unicorn` 退出 0；旧 Vue 2/PostCSS 编译链已经退出 lockfile。远端 CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) 的 npm audit 与 Linux/Windows 前端门禁全部成功，台账据此升为 Done；未来扫描结果仍以命令实测为准，不把本次 0 扩写成永久保证。

2026-08-21 在不改变 `package.json` 声明范围的前提下刷新 `package-lock.json`：`rollup-plugin-visualizer` 解析到 7.1.1、`vue-tsc` 解析到 3.3.10、`webdriverio` 解析到 9.31.2。更新后本地生产依赖审计与完整 `npm audit` 均为 **0 vulnerabilities**，`npm ls --all` 退出 0；跨主版本候选保留为独立迁移任务，不混入本轮锁文件更新。

---

[English](../../creator-docs-en/security/KNOWN_VULNERABILITIES.md)
