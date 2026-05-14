# 轻量化与供应链基线（LIGHTWEIGHT_PROFILE）

本文档记录 **Release 配置、依赖瘦身、审计与二进制体积基线**，与 `Cargo.toml` / `src-tauri/Cargo.lock` 保持一致。目标读者：维护者与发版负责人。

**相关**：已知漏洞清单与升级路线见 **[security/KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md)**；审查范围边界见 **[security/SECURITY_AUDIT_SCOPE.md](../security/SECURITY_AUDIT_SCOPE.md)**（与本文 §6.4 互补）。

---

## §1 工作区 Release 配置（根 `Cargo.toml`）

| 键 | 当前值 | 说明 |
|----|--------|------|
| `profile.release.opt-level` | `"z"` | 体积优先 |
| `profile.release.lto` | `true` | 全 crate 链接时 LTO；等价于 **fat LTO**（Rust 1.46+ `true` 语义） |
| `profile.release.strip` | *（未设置）* | 可选增加 `strip = "debuginfo"` 或 `"symbols"` 进一步缩小发行产物（需发版实测崩溃符号需求） |
| `profile.release.codegen-units` | *（未设置）* | 可选 `codegen-units = 1` 换更小体积与更可复现 perf（编译更慢） |

**`target-dir`**：见仓库根 [`.cargo/config.toml`](../../.cargo/config.toml)，构建产物可外置到 `../oclive-dev-artifacts/oclivenewnew-cargo-target/`。

---

## §6 供应链与体积

### §6.1 `cargo audit` 工具链

- **固定版本**：**cargo-audit 0.22.1**（与 CI `cargo-audit` job 一致，便于报告对齐）。
- **本地执行**：`cd src-tauri && cargo audit`  
  离线：`cargo audit --no-fetch --stale`（需本机曾成功 fetch `advisory-db`）。

### §6.4 审计结果状态（当前）

**已知漏洞跟踪中**；**不宣称零漏洞**。漏洞级命中与路线图以 **[KNOWN_VULNERABILITIES.md](../security/KNOWN_VULNERABILITIES.md)** 为准（最近更新日期见该文件）。

摘要（**2026-05-12**，`cargo audit --no-fetch --stale`，`src-tauri/Cargo.lock`；与当次 CLI 输出一致）：

- **漏洞级（error）**：**5** 条（`rsa`、`rustls-webpki` ×3 条 advisory、`sqlx`）。
- **警告级（warning）**：**17** 条（含 gtk-rs *unmaintained*、`rustls-pemfile` *unmaintained*、`glib` *unsound* 等）；**不**写入 KNOWN 表，但在发版评审时应通读 `cargo audit` 全文。

CI：`.github/workflows/ci.yml` 中 **`cargo-audit`** job 使用 **`continue-on-error: true`**，用于**可见性**而不阻塞合并；待依赖升级后收紧为失败即红。

### §6.5 未使用 / 可选依赖（审查结论）

| 项 | 状态 |
|----|------|
| **`sqlx` 默认 features** | 当前 `src-tauri/Cargo.toml` 使用 **`sqlx = { version = "0.7", features = [...] }`** 显式列表；若锁文件仍含 **`sqlx-mysql` / `sqlx-postgres`**，多为 **macros / compile-time** 或历史解析路径引入——**中期**应结合 **sqlx 0.8+** 与 **仅 sqlite** 特征再压一刀。 |
| **仅 dev / 工具向依赖** | 以 `cargo machete` / `cargo udeps`（可选）周期性核对；移除前须 `cargo test` 全绿。 |

> 已移除依赖的**历史列表**不永久驻留本文；以 `git log -p -- src-tauri/Cargo.toml` 为准。

### §6.6 重复依赖审查（`cargo tree -d`）

**结论（摘要）**：锁文件中常见 **多版本** 来自 **Tauri / WebView / tower / bitflags / sha2** 等与 **sqlx / axum** 栈的叠加，属 **可接受技术债**；**优先**随 **sqlx 0.8+** 与 **Tauri 大版本** 升级收敛，而非手工 pin 单 crate。

示例（节选 `cargo tree -d`）：

- `bitflags` v1 vs v2（`tauri` / `tower-http`）
- `block-buffer` / `crypto-common` 多版本（`sha2` 0.10 vs 0.11 链）

全量输出随锁文件变化；发版前可抽样复查。

### §6.7 `cargo-bloat` 基线（Windows x86_64，Release）

**采样命令**（仓库根外置 `target-dir` 时路径以本机为准）：

```bash
cd src-tauri
cargo bloat --release -n 8
```

**最近采样**：**2026-05-12**，`oclivenewnew-tauri.exe`（`--release`，配置见 §1；`cargo bloat --release -n 8`，外置 `target-dir` 下路径以本机为准）。

| 指标 | 值 |
|------|-----|
| **`.text` 段（bloat 报告）** | **7.6 MiB**（报告中的「63.1% 100.0%」行） |
| **PE 文件大小** | **12.0 MiB**（`cargo-bloat` 末行「the file size is …」） |

**Top 符号（按 bloat 报告 `.text` 贡献，节选）**：

| 占比（文件） | 大小 | 说明 |
|--------------|------|------|
| 1.4% | 170.4 KiB | `oclivenewnew_tauri::run::closure$3` |
| 0.9% | 113.1 KiB | `RoleStorage::load_role_from_dir` |
| 0.7% | 88.8 KiB | `tauri::app::Builder::build` |
| 0.7% | 87.6 KiB | `tauri_runtime_wry::handle_user_message` |
| 0.5% | 60.5 KiB | `plugin_bridge::dispatch_bridge_command::async_fn$0` |
| 0.4% ×2 | 52.3 KiB | `chat_engine::co_present::process_co_present::async_fn$0` |
| 0.3% | 41.6 KiB | `tauri::asset_protocol::asset_protocol_handler` |

> 数值随 **Rust 版本、依赖升级、LTO/strip** 变化；发版前更新本表日期与一行命令输出。

---

## 修订记录

| 日期 | 说明 |
|------|------|
| 2026-05-12 | §6.4 / §6.7：`cargo audit` 与 `cargo bloat --release -n 8` 复测，更新摘要日期与 bloat 数值（`.text` 7.6 MiB、PE 12.0 MiB）。 |
| 2026-05-13 | 初版：与当前 `main` 锁文件、`cargo audit` / `cargo bloat` 采样对齐；链接 KNOWN_VULNERABILITIES。 |
