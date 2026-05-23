# 已知漏洞跟踪（cargo-audit）

本文件记录 **`Cargo.lock`**（工作区根目录；`src-tauri` 与主应用共享）上 **`cargo audit`** 报告的 **漏洞级（vulnerability）** 命中，作为供应链风险管理与升级路线的单一事实来源。**不**包含 `cargo audit` 仅以 *warning* 报告的 *unmaintained* / *unsound* 条目（这些见 `cargo audit` 完整输出与 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)）。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**轻量化与审计流程**：[../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## 当前状态

| 项 | 值 |
|----|-----|
| **cargo-audit 版本** | **0.22.1**（建议固定该主版本以便报告可比） |
| **最近扫描日期** | **2026-05-20**（本地，`--no-fetch --stale` + 已缓存 `~/.cargo/advisory-db`）；**npm / 版本号复核 2026-05-20** |
| **扫描路径** | 工作区根目录 `Cargo.lock` |
| **漏洞级命中数** | **0**（`cargo audit` 退出码 **0**） |
| **警告级命中数** | **3**（`cargo audit` + [`.cargo/audit.toml`](../.cargo/audit.toml) 已记录并忽略 **11** 条 gtk-rs GTK3 / 工具链 *unmaintained*；见下表） |

> 若 CI 或本机无法拉取 advisory-db，可使用：`cargo audit --no-fetch --stale`（依赖本地已 fetch 的数据库）。

---

## 漏洞清单（漏洞级）

**已清零**（2026-05-20）。此前跟踪的 5 条均已通过 **sqlx 0.8.6**、**仅启用 `sqlite` + `runtime-tokio-rustls`**（不使用 umbrella `migrate`，改运行时 `sql_migrate`）、以及 **rustls 0.23 / webpki 0.103** 链解决：

| RUSTSEC ID | 原 Crate | 状态 |
|------------|----------|------|
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) | rsa（经 sqlx-mysql） | **已清零** — 解析图不再包含 `sqlx-mysql` / `rsa` |
| [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) | rustls-webpki 0.101 | **已清零** |
| [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) | rustls-webpki 0.101 | **已清零** |
| [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104) | rustls-webpki 0.101 | **已清零** |
| [RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363) | sqlx 0.7.4 | **已清零** — 已升级至 **0.8.6** |

---

## 解决路线图

### 已完成（2026-05-20）

- **sqlx ≥ 0.8.6**，`default-features = false`，features：`runtime-tokio-rustls`、`sqlite`（无 umbrella `migrate`）。
- 迁移：`src-tauri/src/infrastructure/sql_migrate.rs` 运行时应用 `migrations/*.sql`，与既有 `_sqlx_migrations` 表兼容。
- **CI**：`cargo audit` job 仍以 `continue-on-error: true` 提供可见性；漏洞清零后可分阶段改为失败即红。

### 维护约定

1. 更新锁文件或升级依赖后，在仓库根目录运行：  
   `cargo audit`  
   若网络受限：`cargo audit --no-fetch --stale`
2. 将 **漏洞级** 变化同步到上表；将策略变化同步到 [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md)。
3. 不在对外文案中宣称「零漏洞」；使用 **「漏洞级已清零（警告级仍跟踪）」** 并链接本文件。

---

## 警告级跟踪（2026-05-20 批次三）

| RUSTSEC / 类别 | Crate | 状态 | 原因 |
|----------------|-------|------|------|
| **RUSTSEC-2026-0002** | `lru` | **已修复** | `oclive-cli` 升级 **ratatui 0.30** → `lru` ≥ 0.16 |
| **RUSTSEC-2025-0134** | `rustls-pemfile` | **已修复** | `reqwest` **0.12** 链不再依赖该 crate |
| gtk-rs GTK3 簇（11 ID） | `gtk`/`gdk`/… | **已记录 + audit.toml ignore** | **Tauri 1.x / wry** Linux WebView；需 Tauri 2 方可移除 |
| **RUSTSEC-2025-0057** | `fxhash` | **开放** | 经 Tauri HTML 解析传递；无直接 API |
| **RUSTSEC-2024-0429** | `glib` | **开放** | `VariantStrIter` 路径；宿主未使用 |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **开放** | 经 `phf`/Tauri 宏；需上游 Tauri 2 |

忽略列表与理由见 [`.cargo/audit.toml`](../.cargo/audit.toml) 与 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)。

---

## npm / 版本号复核（2026-05-20）

| 项 | 原状 | 修正 |
|----|------|------|
| `eslint` | `^10.4.0`（npm 未发布） | **`^9.39.0`**（与 `@antfu/eslint-config@^9` 对齐） |
| `vue-virtual-scroller` | 首屏全局 `app.use`，但 UI 已改用 `VirtualScrollContainer` | **移除依赖**；首屏不再同步加载 |
| `sha2`（`src-tauri`） | `0.11.0`（crates.io 无稳定 0.11 线） | **`0.10`**（`sha2 0.10.9`） |
| `@antfu/eslint-config` | `^9.0.0` | 保持不变（与 ESLint 9 兼容） |

复核命令：`npm outdated`（根目录）、`cargo tree -p oclivenewnew-tauri -i sha2`。

---

[English](../../creator-docs-en/security/KNOWN_VULNERABILITIES.md)
