# 开源供应链安全（Supply Chain）

> **定位**：OClive 在「无法要求每个用户从源码构建」的前提下，用**自动化护栏 + 发布可验证 + 扩展点透明**降低供应链风险。  
> **不宣称**：零漏洞、全依赖已人工审计、位级可重复构建（见局限 §4）。  
> **台账**：[`handoff/TECHNICAL_DEBT_INVENTORY.md`](../../handoff/TECHNICAL_DEBT_INVENTORY.md) §供应链

**相关**：[KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) · [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) · [LICENSE_POLICY.md](../LICENSE_POLICY.md) · [LIGHTWEIGHT_PROFILE.md §6](../development/LIGHTWEIGHT_PROFILE.md)

---

## 1. 信任模型（诚实边界）

| 层 | 做法 | 用户侧 |
|----|------|--------|
| **Rust 依赖** | `Cargo.lock` + `cargo audit` + `cargo deny` | 可复现 `cargo build`；CI 与 KNOWN_VULN 表公开 |
| **官方预编译内核** | Release workflow + `SHA256SUMS` · 本地 `bundle-kernel-for-tauri.mjs` 同格式 | 下载后核对哈希，防换包 |
| **第三方插件/角色包** | 源码落本地目录 + 高风险能力须授权 | **运行前自行审 `manifest` / 源码** |
| **进程边界** | 内核独立进程、HTTP 契约、目录插件权限门 | LLM/插件崩溃不默认等同硬件失控 |

行业默认靠「总有人会看源码」的社会契约；OClive **不解决 XZ 类攻击的根因**，用多层护栏**压低概率、提高可追溯性**。

---

## 2. 已落地（基线）

| 护栏 | 位置 |
|------|------|
| **漏洞扫描** | `cargo audit` 0.22.1 · dimension5 · `ci.yml` · `cargo-audit-lockfile.yml` |
| **许可证 / 重复依赖** | 根 `deny.toml` · `cargo deny check licenses bans` · dimension5 · `oclive lint --deny` |
| **锁文件 ratchet** | dimension5 禁止 `sqlx-mysql` / `rsa` 回潮 |
| **漏洞 SSOT** | [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) |
| **审查边界** | [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) |
| **插件权限** | `plugin_permissions` / `high_risk_grants` · A4 三面一致 |
| **迁移完整性** | SQL 迁移 checksum（`sql_migrate.rs`） |

| **插件安装审源码** | 市场 / git / zip 安装后 info toast + `installPath` · CLI 提示 · 严格模式 `OCLIVE_PLUGIN_SIGNATURE_STRICT` | 运行前审本地目录 |
| **Release 哈希** | [`scripts/generate-sha256sums.mjs`](../../scripts/generate-sha256sums.mjs) · [`.github/workflows/release-kernel-checksums.yml`](../../.github/workflows/release-kernel-checksums.yml) | CI artifact / 手动挂 Release |

---

## 3. 核对 Release 内核哈希

GitHub Actions → **Release kernel checksums**（或 push tag `oclivenewnew-v*`）→ 下载 artifact 内 `SHA256SUMS`。

**Windows（PowerShell）**：

```powershell
Get-FileHash .\oclive-kernel-server.exe -Algorithm SHA256
# 对比 SHA256SUMS 中对应行
```

**Linux / macOS**：

```bash
sha256sum oclive-kernel-server
```

本地开发打包：`npm run bundle-kernel:tauri` 会在 `distros/desktop-tauri/resources/SHA256SUMS` 生成同格式清单（已 gitignore）。

---

## 4. 进行中 / 待办（见技术债 ID）

| ID | 项 | 优先级 | 状态 |
|----|-----|--------|------|
| **K-SUPPLY-02** | GitHub Release 挂 `SHA256SUMS` asset | P1 | workflow 已入库；**首次 Release 挂 asset = 维护者** |
| **K-SUPPLY-03** | 插件安装审源码提示 | P2 | **Done** |
| **K-SUPPLY-04** | `npm-audit` 升格 | P2 | **Observe** — 连续 2 周期 high/critical 再硬红 |
| **K-SUPPLY-05** | `deny.toml` multiple-versions → deny | P2 | **OPEN** — 先 `cargo tree -d` 去重 |
| **K-SUPPLY-06** | 位级可重复构建 | — | Deferred |
| **K-SUPPLY-07** | SBOM | — | Deferred |

---

## 5. 维护节奏

1. **`Cargo.lock` 变更的 PR**：`dimension5 --ci` 绿 + 更新 [KNOWN_VULNERABILITIES.md](./KNOWN_VULNERABILITIES.md) 扫描日期。
2. **发版前**：`cargo audit` · `cargo deny check licenses bans` · `oclive lint --deny`（本地与 CI 一致）。
3. **功能周期**：复查 [SECURITY_AUDIT_SCOPE.md](./SECURITY_AUDIT_SCOPE.md) 局限是否需收窄。
4. **`npm-audit`**：CI job 当前 `continue-on-error: true`（可见性）；升格条件见 §4 K-SUPPLY-04。

---

## 6. 用户 / 校企组建议

- **开发者**：`npm ci` + `cargo build`；勿依赖全局污染的工具链。
- **终端用户**：优先官方 Release；核对 SHA256（清单发布后）；插件启用前看本地目录。
- **硬件对接**：MCU / 执行器与 LLM 进程分域；见发行版 profile 与 HTTP 契约文档。

---

## 修订记录

| 日期 | 说明 |
|------|------|
| 2026-06-24 | Wave 1–2：SHA256 workflow、插件 installPath + 审源码 toast |
| 2026-06-24 | 初版：信任模型、基线护栏、与技术债 K-SUPPLY-* 对齐 |
