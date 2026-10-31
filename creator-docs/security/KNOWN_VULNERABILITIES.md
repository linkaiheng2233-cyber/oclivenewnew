# 已知漏洞跟踪（Cargo / npm）

本文件以工作区根 **`Cargo.lock`** 上 `cargo audit` 的**漏洞级（vulnerability）**命中作为供应链风险与升级路线的单一事实来源；警告级 *unmaintained* / *unsound* / *yanked* 不计入漏洞数，但在文末附表维护当前风险与上游阻塞，详情仍以本轮 `cargo audit` 输出和 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) 为准。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**轻量化与审计流程**：[../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## 当前状态

| 项 | 值 |
|----|-----|
| **cargo-audit 版本** | **0.22.2**（建议固定该主版本以便报告可比） |
| **最近扫描日期** | **2026-11-01**（本地 `cargo audit --json` 刷新 advisory-db，共 **1395** 条；扫描 **698** 个依赖条目；漏洞级 **0**、警告级 **10**；退出码 **0**；本轮锁文件仅同步桌面 rc.4 版本，第三方依赖锁定图不变） |
| **本轮 npm 扫描** | 生产依赖 **0**；完整依赖图 **3 low / 43 moderate / 0 high / 0 critical**。该新公告快照不改写旧扫描结果；现行 production/full `--audit-level=high` 标准保持，未扩 ignore 或升级无关依赖 |
| **扫描路径** | 工作区根目录 `Cargo.lock` |
| **漏洞级命中数** | **0**（`cargo audit --json` 退出码 **0**） |
| **警告级命中数** | **报出 10 / 实际命中 11**（`glib`、5 个 `unic-*`、`zbus_names`/`zvariant`/`zvariant_derive`、`unsafe-libyaml`，均为 *unmaintained*/*unsound*；`proc-macro-error` RUSTSEC-2024-0370 仍由 [`.cargo/audit.toml`](../../.cargo/audit.toml) allowlist 抑制，见下「警告级跟踪」） |

> 若 CI 或本机无法拉取 advisory-db，可使用：`cargo audit --no-fetch --stale`（依赖本地已 fetch 的数据库）。

2026-10-29 将 workspace Tokio 最低版本收紧到 **1.53.3**，锁文件从 **1.53.2 → 1.53.3**；锁差量仅 Tokio 的 version / checksum，既有 feature、依赖边及其余锁定包全部保留，修补 **RUSTSEC-2026-0386**。同一公告库快照下，主仓报出警告由 **11 → 10**；在仓库外对相同锁文件做无 ignore 的缓存复核为 **11** 条，唯一多出项仍为 `proc-macro-error`，主仓 allowlist 原字节未改。跟踪的 Rust 源码未发现直接 `AsyncFd` 调用，但这不是完整的传递依赖／Unix 可达性验收；锁定图消除此项公告也不等于所有运行配置的安全保证。新独立内核 revision 接入与安装包重打包仍单独进行。

2026-10-27 新前端内测包将桌面自身版本从 `0.6.0-rc.1` 改为 `0.6.0-rc.2`；此轮 Cargo.lock 仅变更该工作区包的版本行，未变更任何依赖、校验和、feature 或忽略项。候选源码 `b4341eb22e987ffef4a19ccad3784e2402cd2bd1` 已按发版门禁审计，上表采用其扫描原件；原有警告未因 UI 更新解决。后续纯文档整理复用此时点证据，不声称重新扫描。

2026-10-27 候选版将桌面应用自身版本更新为 `0.6.0-rc.1`，并把仅用于属性测试的 `proptest` 限定为 `~1.11.0`，与 Tauri 的 `dom_query` 共用 `bit-set/bit-vec 0.8`；未降低测试数量或放宽重复依赖门禁。锁文件保留既有包的依赖边，新随机数依赖只服务于该测试工具链；未增加忽略项。新发现的 Tokio 警告针对 Unix `AsyncFd`，不属于本次 Windows 安装包的可调用接口；跨平台发布前仍须独立处理，不能把本次 Windows 验证外推到 Unix。

2026-09-27 锁文件仅将 `rustls` 从 0.23.43 更新到 0.23.45（版本和校验和两行），修复 RUSTSEC-2026-0285；该轮没有改动其他 package 或依赖边。此前可选 `media-png` 依赖边的记录属于 2026-09-05 扫描。扫描结果是时点证据，不构成所有运行配置的安全保证。

2026-10-07 的独立维护片仅将已撤回的 `chacha20 0.10.1` 更新到 **0.10.2**，锁差量仅 version / checksum，依赖边及全部 manifest 保持；[上游变更记录](https://github.com/RustCrypto/stream-ciphers/blob/master/chacha20/CHANGELOG.md)说明修复 SSE2 后端误用 SSE4.1 指令。后置 audit native 0、漏洞级 0、警告 6，stderr 为空；前置 fetch/audit native 0 但部分 registry yanked 查询超时，原输出保留，不能凭其退出码声称所有 registry 查询完成。新版本未撤回另由 crates.io 元数据核对。此片与 selector parser 预备片合批，完整链及精确 SHA 正式 CI 待验，未放宽 ignore 或据此关闭 npm 父债。范围及原件见 [DCL-51](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-51--兼容-chacha20-补丁与供应链合批)。

2026-10-18 上游新披露使库存量从 **0** 变为 **10 条漏洞级**（`dimension5-acceptance` 在 `main` 上实时暴露；该红与本仓当时的文档提交无关，是不带代码改动的锁文件存量问题）。本批按"能升就升、不为已知可修漏洞开 ignore"处理，**未新增任何 ignore 条目**：

- `quinn-proto` **0.11.17 → 0.11.19**（RUSTSEC-2026-0338 / 0340 / 0343，两条 7.5 high），经 `quinn 0.11.11 → reqwest 0.12.28` 传递。
- `rustls` **0.23.45 → 0.23.46**（RUSTSEC-2026-0346 / 0347 / 0348 / 0349，最高 4.0 medium），经 `hyper-rustls → reqwest` 传递。
- `zip` **2.4.2 → 9.0.5**（RUSTSEC-2026-0367 / 0368 / 0369，其中 0369 为 9.1 critical），是 `oclive-cli` / `oclive_kernel_host` / `oclivenewnew-tauri` 的**直接**依赖。三条均针对 `ZipArchive::extract()` / `ZipStreamReader::extract()` 的符号链接与 SUID/SGID 处理；本仓从不调用 `extract()`（只在内存中经 `by_index` 读取并自行做路径校验），因此运行时暴露面本来就窄，但按"不留已知漏洞"原则仍升级而非记账。
- 特性选择：9.x 的 `deflate` 元特性会把 `flate2` 后端切换到 `zlib-rs`，而 `png`/`oclive-cli` 已请求 `rust_backend`，两者叠加成双后端；故 workspace 声明改用 `deflate-flate2`，三个成员 crate 去掉与 workspace 重复的 `features = ["deflate"]`。我们从不设置压缩等级，因此不依赖 `deflate` 中的 `zopfli` 部分。
- 代码适配：zip 9 把 `ZipFile::name()` 从返回 `&str` 改为 `Result<Cow<'_, str>, ZipError>`（同为路径安全加固的一部分）；`role_pack.rs` 3 处与 `lora_adapters.rs` 2 处改为把非法条目名映射为既有错误形态，未放宽任何路径校验。
- 依赖图差量：移除 `zopfli 0.8.3` 与 `derive_arbitrary 1.4.2`，新增 `typed-path 0.12.3`；`flate2 1.1.9` 仍为 `rust_backend`（未引入 `zlib-rs`），依赖条目由 698 降至 **697**。
- 验证：`cargo audit --no-fetch --stale` **0 漏洞 / 10 允许警告 / exit 0**；`cargo deny check licenses bans` → `bans ok, licenses ok`；`cargo check` 覆盖三个 zip 消费者全通过。
- 警告级由 6 条增至 10 条是 advisory-db 由 1290 条扩充到 1338 条后既有图的新增登记（`zbus_names` / `zvariant` / `zvariant_derive` / `unsafe-libyaml`），**不是本次升级引入**，且均不阻断门禁。

2026-10-20 上游再次披露两条漏洞（同一 CVE-2026-111943，emSign 根证书信任问题），`main` 的 `dimension5-acceptance` 与 `npm-audit` 同时变红；本批仍按“能升就升、不为已知可修漏洞开 ignore”处理，**未新增任何 ignore 条目**：

- `rustls-webpki` **0.103.14 → 0.103.16**（RUSTSEC-2026-0382，证书校验可能接受被吊销的中间证书），经 `rustls → reqwest` 传递；修复版本要求 `>= 0.103.16`。
- `webpki-roots` **1.0.9 → 1.0.10**（RUSTSEC-2026-0381，内置根证书含已吊销的 emSign 中间证书）；修复版本要求 `>= 1.0.10`。
- 锁差量除上述两个 crate 的 version / checksum 外，仅 `tempfile` 的 `getrandom` 选择由 `0.4.3` 指回 `0.3.4`（Cargo 解析器产物；`getrandom` 0.2/0.3/0.4 三个版本本就共存，`cargo metadata --locked` 通过）。
- npm 侧：`vite` 由 `^6.0.3` 解析到 **6.4.5**（修复 `<= 6.4.3` 的 high），`vitest` **4.1.11 → 4.1.12**（moderate）；`npm audit --audit-level=high` 退出码 **0**，剩余 4 low / 2 moderate 不阻断现有 `--audit-level=high` 门禁。
- 验证：本地 `cargo audit` → **0 漏洞 / 10 允许警告 / exit 0**（1348 条公告、696 个依赖条目）；`npm audit --package-lock-only` → high **0**。本轮未改 Rust 源码与 manifest 依赖声明。

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
| [RUSTSEC-2026-0338](https://rustsec.org/advisories/RUSTSEC-2026-0338) | quinn-proto 0.11.17 | **已修复** — 锁文件 **0.11.19** | 2026-10-18 升级；零长度 STREAM 帧可致接收方 panic |
| [RUSTSEC-2026-0340](https://rustsec.org/advisories/RUSTSEC-2026-0340) | quinn-proto 0.11.17 | **已修复** — 锁文件 **0.11.19** | 同上；ACK_FREQUENCY 传输时对端最小 ACK 延迟可致 panic（7.5 high） |
| [RUSTSEC-2026-0343](https://rustsec.org/advisories/RUSTSEC-2026-0343) | quinn-proto 0.11.17 | **已修复** — 锁文件 **0.11.19** | 同上；出向数据报缓冲记账可致发送方 panic（7.5 high） |
| [RUSTSEC-2026-0346](https://rustsec.org/advisories/RUSTSEC-2026-0346) | rustls 0.23.45 | **已修复** — 锁文件 **0.23.46** | 同上；客户端会话值的 `Debug` 实现泄露 TLS 密钥（4.0 medium） |
| [RUSTSEC-2026-0347](https://rustsec.org/advisories/RUSTSEC-2026-0347) | rustls 0.23.45 | **已修复** — 锁文件 **0.23.46** | 同上；ECH 被拒后仍可向外部服务器发送缓冲应用数据（2.3 low） |
| [RUSTSEC-2026-0348](https://rustsec.org/advisories/RUSTSEC-2026-0348) | rustls 0.23.45 | **已修复** — 锁文件 **0.23.46** | 同上；Brotli 证书解压可在认证前分配至多 1 GiB（3.7 low） |
| [RUSTSEC-2026-0349](https://rustsec.org/advisories/RUSTSEC-2026-0349) | rustls 0.23.45 | **已修复** — 锁文件 **0.23.46** | 同上；未认证 ServerHello 可把后续连接降级为经典密钥交换（3.7 low） |
| [RUSTSEC-2026-0367](https://rustsec.org/advisories/RUSTSEC-2026-0367) | zip 2.4.2 | **已修复** — 锁文件 **9.0.5** | 同上；解压时不安全保留 SUID/SGID 位（7.3 high） |
| [RUSTSEC-2026-0368](https://rustsec.org/advisories/RUSTSEC-2026-0368) | zip 2.4.2 | **已修复** — 锁文件 **9.0.5** | 同上；`ZipArchive::extract` 创建的符号链接目标可越出目的目录 |
| [RUSTSEC-2026-0369](https://rustsec.org/advisories/RUSTSEC-2026-0369) | zip 2.4.2 | **已修复** — 锁文件 **9.0.5** | 同上；`extract()` 经符号链接应用 Unix 权限且不限制目标（9.1 critical）；本仓不调用 `extract()` |
| [RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381) | webpki-roots 1.0.9 | **已修复** — 锁文件 **1.0.10** | 2026-10-22 升级；内置根证书含已吊销 emSign 中间证书 |
| [RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382) | rustls-webpki 0.103.14 | **已修复** — 锁文件 **0.103.16** | 2026-10-22 升级；证书校验可接受被吊销的中间证书 |

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
| gtk-rs GTK3 簇（RUSTSEC-2024-0411…0420，10 ID） | `gtk`/`gdk`/`atk`/`gdkx11`/… | **已从 allowlist 移除** | 2026-10-19 复核：10 条公告均带 `withdrawn = 2026-08-14`，`cargo audit` 不再报出；`audit.toml` 中对应 9 条 ignore 已于 2026-10-19 清理提交中移除（移除后 `cargo audit --no-fetch` 仍为 `0 vulnerabilities` 与 `warning: 10 allowed warnings found`）。Linux WebView（wry/webkit2gtk）仍拉 GTK3，此前"上游无维护版本"的保留理由已由撤回解除 |
| **RUSTSEC-2024-0370** | `proc-macro-error` 1.0.4 | **开放 · allowlist 抑制（有效）** | 经 `tauri 2.11.5 → muda → gtk → glib → glib-macros` 传递引入（与 GTK3 簇同一 Linux 链）；上游无维护版本，故显式 allowlist 以保持门禁可读。解除条件：Tauri/wry 迁到不再依赖 `glib-macros` 的 GTK 绑定 |
| **RUSTSEC-2024-0384** | `instant`（**已不在依赖图**） | **已从 allowlist 移除** | 2026-10-19 复核：`instant` 不在 `Cargo.lock`，该条目不抑制任何输出；已于 2026-10-19 清理提交中从 `audit.toml` 移除 |
| **RUSTSEC-2025-0075 / 0080 / 0081 / 0098 / 0100** | `unic-*` 0.9 | **开放** | 经 Tauri `urlpattern` 传递引入；等待上游移除未维护依赖 |
| **RUSTSEC-2026-0221** | `event-listener` 5.4.1 | **已修复 · K-SUPPLY-11** | 2026-08-01 锁文件升级至 **5.4.2**；SQLx 与 zbus/Tauri 两条路径均已解析到修复版，未加入 ignore |
| **RUSTSEC-2025-0057** | `fxhash` | **已清零** | 2026-07-14 K-PLATFORM-01a Full · Tauri 2 锁图无 `fxhash` |
| **RUSTSEC-2024-0429** | `glib` | **开放** | `VariantStrIter` 路径；宿主未使用（Linux wry） |
| **RUSTSEC-2026-0386** | `tokio` | **锁定图已修补 · 1.53.3** | 2026-10-29 升级并收紧最低版本；刷新审计不再命中，未添加 ignore。Unix `AsyncFd` 安全构造器的修补不替代平台与调用链验收 |
| yanked | `spin` 0.9.8 | **已修复** — 锁文件 **0.9.9** | 2026-08-21 兼容范围锁文件更新；仍经 `flume` → `sqlx-sqlite` 引入 |
| yanked | `chacha20` 0.10.1 | **本地补丁已修复，合批正式验收待证** | 2026-10-07 更新至未撤回的 **0.10.2**；仅锁 version / checksum，后置 audit 不再命中。2026-09-05 的原观察保留为历史；不改随机数 API、TLS 或 ignore 策略 |
| **RUSTSEC-2026-0097** | `rand` 0.7 | **已清零** | 2026-07-14 K-PLATFORM-01a Full · Tauri 2 后无 `rand` 0.7 |
| **RUSTSEC-2026-0190** | `anyhow` | **已修复** — 锁文件 **1.0.104** | 2026-08-21 锁文件复核 |
| **RUSTSEC-2024-0449** | `unsafe-libyaml` 0.2.11 | **开放** | 经 `serde_yaml_ng` 传递引入；上游无维护版本，替换需换 YAML 后端 |
| **RUSTSEC-2026-0370 / 0371 / 0372** | `zvariant` / `zbus_names` / `zvariant_derive` 5.15.0 / 4.3.4 | **开放** | 经 zbus（Linux 侧 Tauri/wry 依赖链）传递引入；等待上游迁移到受维护的 D-Bus 绑定 |

忽略列表与理由见 [`.cargo/audit.toml`](../../.cargo/audit.toml) 与 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)。

**allowlist 现状复核（2026-10-19）**：受检基线 `f8026855`（工作树仅含本批文档改动），`cargo-audit` **0.22.2**，advisory-db 位于 `E:\Env\state\cargo-home\advisory-db`（**962** 个 crate 目录，刷新后使用）。`cargo audit` 报 `0 vulnerabilities` 与 `warning: 10 allowed warnings found`，退出码 **0**。为区分「抑制有效」与「残留条目」，临时清空 `audit.toml` 全部 **11** 条 `ignore` 后以 `cargo audit --no-fetch` 重跑：报出数量变为 **11**，唯一新增项是 `proc-macro-error` 的 **RUSTSEC-2024-0370**；9 条 GTK3 簇公告与 `instant` 的 **0384** 均未出现，即上表所列失效原因可由实验直接复现。复核结束后 `audit.toml` 已按原字节恢复（SHA256 `534DCC2A5491296480D7F5B121A27B11A2D749337A07D03C02A71A7EAFACF64D`），本条记录复核时的状态；失效条目的移除见同日的清理提交。

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
| `zip`（`distros/desktop-tauri`） | `0.6`（RUSTSEC 跟踪中） | **`2.x`**（`role_pack` / `plugin_pack` API 已适配）；2026-10-18 已升至 **9.0.5**（见上文 2026-10-18 批） |

复核命令：`npm outdated`（根目录）、`cargo tree -p oclivenewnew-tauri -i sha2`、`cargo tree -p oclivenewnew-tauri -i serde_yaml`（应为空）。

---

## npm 供应链（滚动时点）

### 2026-10-29 草稿同步后的补丁收口（待目标 SHA 远端验收）

主仓候选 `2fb8b7bd631690c06b4de2c4967d453ae79e156b` 的[草稿 PR #225](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/225) 首轮 [CI 40837019439](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/40837019439) 在 `npm-audit` 的生产扫描失败。本地同锁复现 production **5 high**、full **4 low / 2 moderate / 6 high / 0 critical**；这些是受影响包的计数，不是独立漏洞条数。旧扫描结果保留为历史，不作为本次通过证据。

| 依赖 / 公告 | 本次限定修补 |
|---|---|
| Vue I18n [命名参数转义](https://github.com/advisories/GHSA-8rc2-428q-7ww3)、[srcdoc 转义绕过](https://github.com/advisories/GHSA-83rm-6c8r-g728) | 四份现有 manifest 最低范围统一到 `^11.4.14`；`vue-i18n` 与四个 `@intlify` 包由 **11.4.8 → 11.4.16**，仍在 11.x 补丁线。公告修复下限 11.4.14；不因锁命中宣称产品已出现可达 XSS |
| 开发链 Undici | jsdom / cheerio 的 **7.30.0 → 7.30.1**、webdriver 的 **6.29.0 → 6.29.1**，均满足既有传递范围；修补内容见上游 [7.30.1](https://github.com/nodejs/undici/releases/tag/v7.30.1) / [6.29.1](https://github.com/nodejs/undici/releases/tag/v6.29.1)，不新增 override |
| workspace Vitest 残留 | 三个 workspace 由 **4.1.11 → 4.1.12**，与根目录原版本一致；删除其旧副本及未再被引用的旧 `@vitest` 链，复用锁中已有 4.1.12，覆盖[旧 mocker 的 Windows 路径公告](https://github.com/advisories/GHSA-xp26-3454-mjgp) |

原 npm **10.9.8** 的定向锁更新保留了旧 workspace Vitest 节点，原件与日志保存在本轮树外证据目录；仅临时用官方 npm **11.21.0** 同步锁，再用原 npm 10 执行 `npm ci --no-audit --no-fund` 和 `npm ls --all`，均 exit **0**，不更换全局工具链。锁还将既有根 Vite 范围元数据对齐到 manifest 的 `^6.4.5`（解析版未变），并重算 TypeScript 的 `devOptional` 归类（既有 Vue / Pinia 可选 peer，版本与依赖边未变，`npm explain typescript` 已核对）。其余包版本、四条 override、Cargo、CI、审计阈值与产品源码不变。

补丁后本地 production **0**、full **4 low / 0 moderate / 0 high / 0 critical**，两条 `--audit-level=high` 扫描 exit **0**；残余仍是 ESLint Markdown→KaTeX low 链，不是全图零风险。远端结果必须绑定补丁后的完整 SHA；草稿 gate 不替代正式 `ci-gate`，受保护 main 尚未合并，安装包未重打。独立内核的源码预览版及其审计是另一仓库的证据，不代证本主仓。

### 2026-10-06 新公告修复（本地验证，待目标 SHA CI）

`8cd7d5d1` 的正式 CI [37424944008](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37424944008) 在生产扫描命中三项 high；本机完整扫描另有开发测试链 critical。该失败不归因资源策略源码，也不能把此前扫描的 0 当作当前无风险证据。

| 公告 / 依赖 | 当前修复 / 边界 |
|---|---|
| [GHSA-g2v6-rqmx-r4w6](https://github.com/advisories/GHSA-g2v6-rqmx-r4w6) · Vue SSR 属性名校验 | Vue / compiler / renderer 统一 **3.5.43**，仍在原 3.5 补丁线；公告修复下限 3.5.42。锁命中不等于已证明本产品存在可达 SSR 攻击路径 |
| [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q) · source-map-js | **1.2.2**，原传递范围允许的补丁；未添加 override |
| [GHSA-5gmw-xhrv-c9v3](https://github.com/advisories/GHSA-5gmw-xhrv-c9v3) / [GHSA-85c8-ppgw-ccpr](https://github.com/advisories/GHSA-85c8-ppgw-ccpr) · Tinypool | 原 Vitest 3.2.7 的 1.x 链无兼容修复；三个 workspace 与根统一固定 **Vitest 4.1.11**，该依赖图不再含 tinypool，也修复其 mocker 已知命中。不升级 Vitest 5，不放宽 peer 或审计 |

原 npm 10.9.8 锁生成遇 Arborist `edgesOut` null；只临时使用官方 npm 11.21.0 生成锁，随后原 npm 10 `npm ci`、`npm ls --all` 均 exit 0。未改变全局工具链或 CI 配置。当前本地生产 JSON 扫描 **0**；完整图 **4 low / 1 moderate / 0 high / 0 critical**，两条 `--audit-level=high` 门禁均 exit 0。残余分别为 ESLint Markdown→KaTeX 链与 postcss-selector-parser，不能称完整图零漏洞；扫描结果仅对应本轮锁与时点。验证、失败和正式出口见 [DCL-48](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261006-48--新披露-npm-风险与测试链修复)。

**精确正式出口补记**：上述 `3eb1ca7cafabedd739075c6a506fa9ecfcb95425` 已在 [37427668306](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37427668306) 实际 success、17/17 jobs success 含 ci-gate；完整本地链 native 0。前段 4 low / 1 moderate 是该冻结锁的扫描，不改写成后续补丁数字。

### 2026-10-06 selector parser 单叶补丁（本地预备，未 main 验收）

实际链为 `@antfu/eslint-config → eslint-plugin-vue → postcss-selector-parser 7.1.5`；[GHSA-rj75-hqrm-r3gf](https://github.com/advisories/GHSA-rj75-hqrm-r3gf) 指定修复版 **7.1.6**，[上游发行](https://github.com/postcss/postcss-selector-parser/releases/tag/7.1.6)修复平坦选择器的二次复杂度。独立工作树只更新锁中该节点的 version/resolved/integrity，原合法范围、package 声明、peer 与工具链保持，不添加 override、不跟随 low 项的跨主版/降级建议；不把锁命中当产品攻击路径实证。

本片原 npm 10 以 `npm ci --ignore-scripts --no-audit --no-fund` 消费成功，完整 `npm ls --all` exit 0。实际 production audit **0**，full **4 low / 0 moderate / 0 high / 0 critical**；仅消除该 moderate，ESLint Markdown→KaTeX low 链仍留。当前片为局部预备，后续合批完整门禁和目标 SHA 正式 CI 尚未执行，不拿前片的正式绿替代本片。详见 [DCL-50](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261006-50--selector-parser-单叶补丁预备片)。

### 2026-10-07 shell-quote 临时兼容例外

前批 `67baa952bb7e1ff0ba987609d6fe729ca327f24d` 已完整本地 native 0，但[正式 CI 37505631986](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37505631986) 的 full audit 命中新登记的 [GHSA-pqg4-j6r4-53mv](https://github.com/advisories/GHSA-pqg4-j6r4-53mv)，最终 cancelled，正式验收不成立。该时点 production 0、full **4 low / 2 critical**；shell-quote 与 concurrently 两个 critical 包计数来自同一公告，不是两条独立漏洞。这两个锁节点在此前已验基线中也相同，不能把新审计结果归因 selector parser 或 ChaCha20 补丁。

| 临时例外 | 范围、验证与撤销条件 |
|---|---|
| `overrides.concurrently.shell-quote = "1.11.0"` | 维护者明确批准；只覆盖 `concurrently 9.2.4 → shell-quote`，保留原四条 override。上游当前 9.2.4 / 10.0.5 仍精确依赖 1.9.0，故仅升级父包不能消除该命中；不升级 concurrently 主版本。锁仅该节点的 version/resolved/integrity 改动 |
| 本地兼容证据 | 原 npm 10 `npm ci --ignore-scripts --no-audit --no-fund` 与 `npm ls --all` exit 0；concurrently 真实良性双子进程成功 exit 0、子进程失败 7 时 CLI exit 1；生产参数展开和代表性 quote/parse 保持。四种行分隔符在 comment 后的内存字符串由旧版允许改为 TypeError 拒绝，未在 shell 执行攻击字符串 |
| 当前滚动扫描 | production **0**、full **4 low / 0 moderate / 0 high / 0 critical**，两条 `--audit-level=high` native 0；ESLint Markdown→KaTeX low 继续跟踪。这是当前锁与扫描时点，不是产品零风险承诺 |
| 责任与撤销 | 项目依赖维护者负责跟进：上游支持线采用安全 shell-quote 后，更新 concurrently 及其依赖链、删除本条 override，再核锁、npm ci/peer/audit 与构建/退出码回归。不能仅因上游有新发行就移除；不自动更新其它例外 |

该补丁已冻结于 `1e81033d6c8efcff8bb9f502c642560e57335cf1`：一次完整本地链 native 0（1491.7835099 秒），[正式 CI 37513747676](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37513747676) attempt 1、17/17 success，含 ci-gate，未 rerun。父债保持 Partial，等待上游撤销条件与 KaTeX low 的各自收敛；不借 audit 0 判 Done。原因、批准与原始回执坐标见 [DCL-53](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-53--shell-quote-临时兼容补丁与撤销条件)，本次随资源原因说明实质合流回填，见 [DCL-54](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-54--资源原因说明合流与已验基线确认)。

### 2026-10-09 上游采用安全链并撤销临时例外

维护者已明确临时补丁应在上游采用安全版本后退出。[concurrently 9.2.5 官方发行](https://github.com/open-cli-tools/concurrently/releases/tag/v9.2.5)将精确依赖从 shell-quote 1.9.0 更新到 **1.12.0**，仍满足本仓原声明 `^9.2.1`。官方 npm 元数据与生成锁一致；本片更新父包 **9.2.4→9.2.5**、叶子 **1.11.0→1.12.0**，撤销 `overrides.concurrently`，其它四条例外及所有声明/脚本保持。锁差量恰为这两个节点，旧 2026-10-07 批准、扫描与失败记录原文保留，不能理解为当前仍需该补丁。

原 npm 10 的 `npm ci --ignore-scripts --no-audit --no-fund` / `npm ls --all`、原代表性 quote/parse、生产参数展开、良性双子进程成功 exit 0 / 子进程 7→CLI 1、实际 production build 均 native 0。四种 comment 后行分隔符仍在内存中被 TypeError 拒绝，攻击字符串未交 shell。锁仍由临时官方 npm 11.21.0 定向生成，不更改全局工具链；安装的 deprecated 提示与可选依赖缺失照实保留，不当作 peer 失败或零 stderr。

本次前后 production audit 均 **0**，full 均 **4 low / 0 moderate / 0 high / 0 critical**，两条 high 门禁 native 0。ESLint Markdown→KaTeX low 和其它 override 各自保持，K-SUPPLY-12 父债 **Partial**；扫描只对应当前锁/时点。完整本地出口和新 SHA 正式 CI 待本次批次冻结后取得，不能以旧 SHA 绿灯替代；计划、局部证据与停止线见 [DCL-73](../../handoff/debt-marathon/DEBT_CHANGELOG.md#dcl-20261009-73--上游安全补丁与临时-override-撤销)。

### 历史已验证时点（2026-08-21）

CI **`npm-audit`** job 以硬门禁运行 `npm audit --omit=dev --audit-level=high`；远端 CI [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) 的生产依赖扫描为 **0 vulnerabilities**。本地复现：仓库根目录运行同一命令。

K-SUPPLY-12 冻结实现 `728219e7` 上，完整 `npm audit` 与生产扫描均为 **0 vulnerabilities**，`npm ls eslint eslint-plugin-unicorn` 退出 0；旧 Vue 2/PostCSS 编译链已经退出 lockfile。远端 CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) 的 npm audit 与 Linux/Windows 前端门禁全部成功，台账据此升为 Done；未来扫描结果仍以命令实测为准，不把本次 0 扩写成永久保证。

2026-08-21 在不改变 `package.json` 声明范围的前提下刷新 `package-lock.json`：`rollup-plugin-visualizer` 解析到 7.1.1、`vue-tsc` 解析到 3.3.10、`webdriverio` 解析到 9.31.2。更新后本地生产依赖审计与完整 `npm audit` 均为 **0 vulnerabilities**，`npm ls --all` 退出 0；跨主版本候选保留为独立迁移任务，不混入本轮锁文件更新。

---

[English](../../creator-docs-en/security/KNOWN_VULNERABILITIES.md)
