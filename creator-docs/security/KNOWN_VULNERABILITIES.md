# 已知漏洞跟踪（cargo-audit）

本文件记录 **`src-tauri/Cargo.lock`** 上 **`cargo audit`** 报告的 **漏洞级（vulnerability）** 命中，作为供应链风险管理与升级路线的单一事实来源。**不**包含 `cargo audit` 仅以 *warning* 报告的 *unmaintained* / *unsound* 条目（这些见 `cargo audit` 完整输出与 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md)）。

**全库文档索引**：[../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**轻量化与审计流程**：[../development/LIGHTWEIGHT_PROFILE.md](../development/LIGHTWEIGHT_PROFILE.md) §6.4

---

## 当前状态

| 项 | 值 |
|----|-----|
| **cargo-audit 版本** | **0.22.1**（建议固定该主版本以便报告可比） |
| **最近扫描日期** | **2026-05-13**（本地，`--no-fetch --stale` + 已缓存 `~/.cargo/advisory-db`） |
| **扫描路径** | `src-tauri/Cargo.lock` |
| **漏洞级命中数** | **5**（`cargo audit` 以 `error: N vulnerabilities found` 为准） |
| **警告级命中数** | **17**（未列入下表；含 gtk-rs *unmaintained*、*unsound* 等） |

> 若 CI 或本机无法拉取 advisory-db，可使用：`cargo audit --no-fetch --stale`（依赖本地已 fetch 的数据库）。

---

## 漏洞清单（漏洞级）

| RUSTSEC ID | Crate | 版本（锁文件） | 风险 / CVSS | 简要说明 | 解决方向 | 状态 |
|------------|-------|----------------|-------------|----------|----------|------|
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) | **rsa** | 0.9.10 | 中 / **5.9** | Marvin Attack：计时侧信道可能导致密钥恢复 | 经 **sqlx-mysql** 传递引入；升级 **sqlx ≥ 0.8** 并收紧 features，避免拉入 `rsa` | 跟踪中 |
| [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098) | **rustls-webpki** | 0.101.7 | 见 advisory | URI 名称的名称约束处理错误 | 随 **rustls** / **sqlx** 升级链至 advisory 指定区间 | 跟踪中 |
| [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099) | **rustls-webpki** | 0.101.7 | 见 advisory | 通配符证书名称约束错误 | 同上 | 跟踪中 |
| [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104) | **rustls-webpki** | 0.101.7 | 见 advisory | CRL 解析中可达 panic | 同上（需 **≥0.103.13** 等，以 advisory 为准） | 跟踪中 |
| [RUSTSEC-2024-0363](https://rustsec.org/advisories/RUSTSEC-2024-0363) | **sqlx** | 0.7.4 | 见 advisory | 二进制协议截断/溢出类误解释 | 升级至 **≥ 0.8.1**（advisory 要求） | 跟踪中 |

**依赖关系摘要**（2026-05-13 `cargo audit` 输出）：

- **rsa** ← `sqlx-mysql` ← `sqlx` / `sqlx-macros-core` ← `oclivenewnew-tauri`
- **rustls-webpki** ← `rustls` ← `sqlx-core` ← `sqlx` / 各 sqlx 子 crate
- **sqlx** 直接依赖应用 crate

---

## 解决路线图

### 短期（本轮）

- 已在本仓库建档：**本文件** + [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md) 交叉引用。
- **可选缓解**：在 `Cargo.toml` 对 `sqlx` 使用 `default-features = false` 并仅启用 **`runtime-*` + `sqlite` + `migrate` + `macros`** 等应用实际所需 features，减少 **MySQL / PostgreSQL** 相关传递依赖进入锁文件的概率（须全量 `cargo test` 验证）。

### 中期（下一功能周期）

- **优先升级 `sqlx` 至 0.8.1+**，对齐 RUSTSEC-2024-0363，并重新运行 `cargo audit` 验证 **rsa / rustls-webpki** 链是否消除或降级。
- 评估 **reqwest** / **原生 TLS** 与 **rustls** 栈的版本对齐，避免多版本 **webpki** 并存。

### 长期

- **CI**：已增加 **`cargo audit`** job（`continue-on-failure: true`）；目标在依赖清理后改为 **`--deny warnings`** 或至少 **`--deny unmaintained`** 分阶段收紧。
- 维护约定：每个功能周期结束时在 `src-tauri` 目录执行 `cargo audit`（或 CI artifact），**更新本表中的日期、版本与行数**。

---

## 维护约定

1. 更新锁文件或升级依赖后，运行：  
   `cd src-tauri && cargo audit`  
   若网络受限：`cargo audit --no-fetch --stale`
2. 将 **漏洞级** 变化同步到上表；将策略变化同步到 [LIGHTWEIGHT_PROFILE.md §6.4](../development/LIGHTWEIGHT_PROFILE.md)。
3. 不在对外文案中宣称「零漏洞」；使用 **「已知漏洞跟踪中」** 并链接本文件。
