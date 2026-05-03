# 内核与 OOCP：完整发布路径（monorepo → crates.io → 镜像 → 桌面）

> **目标**：把 **`oclivenewnew`** 工作区内 **`oclive_validation` → `oclive_core` → `oclive_kernel_runtime` → `oclive_kernel_server` → `oclivenewnew-tauri`** 的依赖关系、版本策略、CI 门禁与对外交付物一次写清。  
> **契约**：OOCP 协议面以 **`creator-docs/oocp/OOCP_SPEC_COMPLETE_REFERENCE.md`** 与 **`crates/oclive_core/src/capabilities/mod.rs`** 为准；Tauri 命令以 **`KERNEL_ENTRY_CHECKLIST.md`** + **`invoke_lists/*.txt`** 为准。

---

## 1. 工件地图（谁发布、谁消费）

| 工件 | Crate / 产物 | 典型消费者 | 备注 |
|------|----------------|------------|------|
| 校验与共享类型 | `oclive_validation` | `oclive_kernel_runtime`、编写器 WASM 等 | 可先上 crates.io，API 面小 |
| 协议与 OOCP 核心类型 | `oclive_core` | `oclive_kernel_runtime`、未来轻量宿主 | 含 **`OOCP_*` 常量**、`dispatch_oocp_request` |
| 内核域 + SQLite + HTTP/WS | `oclive_kernel_runtime` | `oclivenewnew-tauri`、`oclive_kernel_server`、第三方嵌入 | **默认 `full` feature** 与官方桌面一致 |
| 无头进程 | `oclive_kernel_server` | Docker / systemd / 发行版「仅内核」 | 依赖 `kernel-http-api` 链 |
| 桌面 | `oclivenewnew-tauri` + 前端 `src/` | 终端用户 | **不**上 crates.io；走安装包与自动更新 |

---

## 2. 依赖 DAG（发布顺序约束）

```
oclive_validation
       ↓
   oclive_core   ← OOCP_VERSION / OOCP_METHODS / OOCP_EVENTS
       ↓
oclive_kernel_runtime   ← 编排、DB、migrations、http_api、OOCP WS
       ↓
  ┌────┴────┐
  ↓         ↓
kernel_server   oclivenewnew-tauri（path）
```

**规则**：对 crates.io 而言，**被依赖方必须先于依赖方**发布；或暂时继续 **path-only** 单体开发（当前默认）。

---

## 3. 版本策略（建议）

| 策略 | 做法 | 适用 |
|------|------|------|
| **齐版本（推荐初期）** | `validation` / `core` / `kernel_runtime` 同 **0.1.x** 抬升，CHANGELOG 合并叙述 | 小团队、协议与内核同频迭代 |
| **独立 SemVer** | 各 crate 独立 MAJOR；内核在 **MINOR** 引入 OOCP 新方法时，`oclive_core` 至少 **MINOR** 同步能力常量 | 协议稳定后、多宿主消费 `oclive_core` |
| **协议与 crate 解耦** | **OOCP 协议版本** = `capabilities.version`（`OOCP_VERSION`）；**crate 版本** = Cargo SemVer；二者不必数字相等，但 **CHANGELOG 必须写清对应关系** | 长期 |

---

## 4. 阶段 0：当前（path monorepo）

- **开发**：`cargo build` / `cargo test --workspace`；内核集成测试见 **`crates/oclive_kernel_runtime/tests/`**。  
- **CI**（根 `.github/workflows/ci.yml`）：`fmt`、`clippy -D warnings`、`test`、`cargo doc`（kernel + 断链门禁）、**`invoke_lists` ↔ CHECKLIST** 脚本。  
- **OOCP 试跑**：`KERNEL_SDK.md`、`OOCP_TRANSPORTS.md`、`DISTRIBUTION_DEV_GUIDE.md`。  
- **容器**：`Dockerfile.kernel-server` + `docker-compose.kernel-server.yml`（无头 `kernel_server`）。

---

## 5. 阶段 1：crates.io 元数据齐备（每个可发包）

对每个将发布的 `Cargo.toml` 至少具备：

- `license`（已与 AGPL 策略一致）  
- `description`、`readme`  
- **`repository`**、**`homepage`**（与 Git 托管 URL 一致；勿写占位）  
- `keywords` / `categories`（可选但利于发现）

并在 **根 `CHANGELOG.md` 或各 crate `CHANGELOG.md`** 中记录对外可见行为（OOCP 方法增减须显式写出）。

---

## 6. 阶段 2：path → 版本双写（过渡）

Cargo 支持（示例，**非提交指令**，仅说明形态）：

```toml
oclive_core = { version = "0.1.5", path = "../oclive_core" }
```

- **含义**：本地仍 path；发布到 crates.io 时 Cargo 剥离 `path`，使用 **registry 版本**。  
- **前提**：`oclive_core` **0.1.5** 已在 crates.io 存在。  
- **校验**：`cargo publish -p oclive_kernel_runtime --dry-run` 在双写正确时应能通过依赖解析（仍需满足其余 publish 规则）。

---

## 7. 阶段 3：按序 `cargo publish`

建议顺序（与 §2 DAG 一致）：

1. **`oclive_validation`** — `cargo publish -p oclive_validation`  
2. **`oclive_core`** — 依赖项改为 `oclive_validation = { version = "…" }`（或双写）  
3. **`oclive_kernel_runtime`** — 依赖 `core` / `validation` 均带版本  

每一步本地执行：

```bash
cargo publish -p <crate> --dry-run
cargo publish -p <crate>
```

**打 Git tag**：建议 `validation-v0.1.x`、`core-v0.1.x`、`kernel-runtime-v0.1.x` 或在 monorepo 使用统一 **`v0.1.x`** 标签 + Release Notes 子章节说明各 crate。

---

## 8. 阶段 4：容器镜像与 OOCP 端口契约

- **构建**：`docker build -f Dockerfile.kernel-server -t <registry>/oclive-kernel-server:<tag> .`  
- **配置**：**`OOCP_API_PORT`**、**`OCLIVE_ROLES_DIR`**、**`OCLIVE_DB_PATH`**、**`OCLIVE_APP_DATA_DIR`**、**`OOCP_API_TOKEN`**（生产务必评估）。  
- **契约**：镜像所带 **`capabilities.version`** 须与镜像标签在 Release Notes 中 **可追溯**（见 OOCP 完整参考 §版本）。

---

## 9. 阶段 5：桌面（Tauri）与内核版本对齐

- **桌面安装包**：随仓库发版流程（签名、更新通道）；**不**走 crates.io。  
- **对齐**：桌面捆绑的 **`oclive_kernel_runtime` 提交哈希或 crate 版本** 写入发行说明；OOCP 客户端应读取 **`capabilities.version`** 做兼容判断。  
- **invoke**：继续由 **`invoke_lists/*.txt` + feature** 生成；与 **CHECKLIST** 的 CI 脚本防止漂移。

---

## 10. CI / 质量门禁清单（建议保持）

| 门禁 | 命令 / 位置 | 说明 |
|------|----------------|------|
| 格式化 | `cargo fmt --all --check` | 根 CI |
| 静态分析 | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 根 CI |
| 测试 | `cargo test --workspace` | 根 CI |
| 最小特性 | `cargo check -p oclive_kernel_runtime --no-default-features` | 嵌入式 SKU |
| rustdoc | `cargo doc -p oclive_kernel_runtime --all-features --no-deps` + `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links` | 根 CI |
| CHECKLIST ↔ invoke | `scripts/check_kernel_entry_vs_invoke_lists.sh` | 根 CI |
| 发布干跑 | `cargo publish -p … --dry-run` | **依赖链就绪后**加入 CI job 或发版前手动 |

---

## 11. 回滚与 yank

- **crates.io yank**：仅隐藏版本下载；**不**撤销已下载副本；严重安全问题配合 **新版本 + 安全公告**。  
- **协议**：若 OOCP **MINOR** 引入仅加方法，旧客户端应仍能连；**MAJOR** 前须走 **`OOCP_FREEZE_POLICY.md`** 的冻结与迁移叙述。

---

## 12. 相关文档

| 文档 | 用途 |
|------|------|
| [OOCP_SPEC_COMPLETE_REFERENCE.md](../oocp/OOCP_SPEC_COMPLETE_REFERENCE.md) | OOCP 完整参考（与实现对齐） |
| [OOCP_SPEC_v0_1.md](../oocp/OOCP_SPEC_v0_1.md) | v0.1 叙述与示例 |
| [OOCP_TRANSPORTS.md](../oocp/OOCP_TRANSPORTS.md) | WS/HTTP/鉴权/端口 |
| [OOCP_FREEZE_POLICY.md](../oocp/OOCP_FREEZE_POLICY.md) | v0.x → v1.0 冻结策略 |
| [KERNEL_SDK.md](../kernel/KERNEL_SDK.md) | 库模式、脚本、Docker、publish 限制 |
| [ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md](../../handoff/ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md) | P0–P2 路线图与验收 |
