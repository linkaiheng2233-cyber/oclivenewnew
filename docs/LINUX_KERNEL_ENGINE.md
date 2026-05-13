# Oclive Linux 内核引擎 — 迁移与机器人向适配说明

**运维与安装步骤以** **[LINUX_KERNEL_DEPLOY.md](./LINUX_KERNEL_DEPLOY.md)** **为准**；本文侧重目标、阶段路线与安全原则。

本文描述 **无桌面 UI** 的 Linux 部署路径：`oclive_kernel_server` + `oclive_kernel_runtime`（及依赖链），目标为 **x86_64（优先）**，并预留 **aarch64** 与契约一致性。

---

## 0. 目标与边界

| 纳入 | 不纳入 |
|------|--------|
| Rust workspace 中内核与 `oclive_kernel_server` | `src-tauri`、Vue 前端、Windows/macOS 桌面安装包 |
| `GET /health`、`POST /chat`、OOCP WebSocket `/oocp`（与桌面同源 `http_api`） | 将 ASR/TTS/CV 并入内核 crate |
| 角色包（`roles/README_MANIFEST.md` + validation） | 硬件驱动 |

**契约原则**：OOCP/HTTP JSON 与角色包在 Linux 上的行为应与 Windows 桌面内核一致（同版本、同 feature 组合下）。

---

## 1. 环境变量（生产必看）

| 变量 | 默认 | 说明 |
|------|------|------|
| `OOCP_API_PORT` | `48888` | 监听端口 |
| `OOCP_API_BIND` | `127.0.0.1` | 监听地址；容器或局域网暴露常用 `0.0.0.0`，**务必配合鉴权与网络隔离** |
| `OOCP_API_TOKEN` | （空） | 非空时：**REST**（`/chat`、`/role-feedback*`）与 **OOCP WS** 均需 `Authorization: Bearer <token>` |
| `OCLIVE_ROLES_DIR` | 启发式解析 | **生产必须显式设置** 为角色根目录（含子角色目录与 `manifest.json`） |
| `OCLIVE_DB_PATH` | 临时目录下 sqlite | 建议固定路径以便持久化 |
| `OCLIVE_APP_DATA_DIR` | 派生自 db 父目录 | 插件/MCP 等目录数据 |
| `OCLIVE_REQUIRE_EXPLICIT_PATHS` | 关 | 为真时三路径未齐则退出码 **2**（生产推荐；Docker 镜像默认开） |
| `RUST_LOG` | — | `error` / `warn` / `info` / `debug` / `trace`（`env_logger`） |

**XDG 建议（最佳实践，非代码强制）**：裸机可将数据放在  
`$XDG_STATE_HOME/oclive/`（如 `~/.local/state/oclive/`）或  
`$XDG_DATA_HOME/oclive/`（如 `~/.local/share/oclive/`），  
通过环境变量指向上述路径即可。

---

## 2. 编译与测试（阶段 1）

**主目标发行版**：Ubuntu 22.04 LTS x86_64（与 CI `ubuntu-latest` 对齐）。

```bash
cargo build -p oclive_kernel_server --release
cargo test -p oclive_kernel_runtime
cargo test --workspace   # 与 CI 一致
```

启动前设置 `OCLIVE_ROLES_DIR` 指向仓库 `roles/`，例如：

```bash
export OCLIVE_ROLES_DIR=/path/to/oclivenewnew/roles
export OCLIVE_DB_PATH=/tmp/oclive-smoke.db
export OCLIVE_APP_DATA_DIR=/tmp/oclive-app
./target/release/oclive_kernel_server
```

验证：

```bash
curl -s "http://127.0.0.1:${OOCP_API_PORT:-48888}/health"
```

---

## 3. Docker 与合成模板（阶段 2）

- **镜像构建**：仓库根目录  
  `docker build -f Dockerfile.kernel-server -t oclive-kernel-server .`
- **Compose**：`delivery/docker-compose.yml` + `delivery/config.example.env`
- 镜像内默认 **`OOCP_API_BIND=0.0.0.0`** 以便端口映射；**生产请设置 `OOCP_API_TOKEN`**。

`.dockerignore` 已减小构建上下文；**未**将 `target/` 打入上下文。

---

## 4. 安全、运维与 systemd（阶段 3）

- **鉴权**：`OOCP_API_TOKEN` 为最小共享密钥；生产应叠加 **TLS（反向代理）**、**防火墙**、**内网**。
- **日志**：`RUST_LOG=info`；systemd 下由 **journald** 收集：`journalctl -u oclive-kernel -f`
- **单元模板**：`delivery/systemd/oclive-kernel.service.example`  
  安装前需创建系统用户（示例）：  
  `sudo useradd -r -m -s /bin/false oclive`  
  并将二进制与 `roles` 放到模板中路径或自行修改 `ExecStart` / `Environment`。

---

## 5. 机器人「自定义灵魂」与多模态（阶段 4）

**硬性边界**：语音识别（ASR）、语音合成（TTS）、视觉（CV）等 **不得** 以子模块形式并入 `oclive_kernel_runtime`；应作为 **外挂进程**，通过 **HTTP / OOCP / JSON-RPC** 与内核交互。

**将感知注入对话的最小模式**：外挂模块将分析结果整理为 **自然语言或结构化摘要**，由集成方作为 **`POST /chat` 的 `message` 字段的一部分**（或前缀标签，如 `[视觉] …`）传入。内核侧无需新增字段即可完成闭环；若将来需要独立 DTO 字段，再在 `dto` 层做版本化扩展。

示例说明与脚本：**[`examples/linux_kernel_multimodal_context/`](../examples/linux_kernel_multimodal_context/)**。

**按需瘦身**（验证用）：

```bash
cargo check -p oclive_kernel_runtime --no-default-features
# 按需叠加 feature，参见 crates/oclive_kernel_runtime/Cargo.toml 与 creator-docs/kernel/LIGHTWEIGHT_PROFILE.md
```

---

## 6. ARM / aarch64（阶段 5，按需）

1. 安装目标：`rustup target add aarch64-unknown-linux-gnu`  
2. 需交叉链接器（如 `aarch64-linux-gnu-gcc`）或在与目标 ABI 一致的环境中 **原生编译**。  
3. Docker 多架构：使用 `docker buildx build --platform linux/arm64 ...`（需本机 buildx 与基础镜像支持）。

详细板级适配（树莓派等）由集成方内核团队与 ODM 镜像对齐；本文仅定义 **契约与构建维度**。

---

## 7. CI

仓库 CI 在 **Ubuntu** 上已执行完整 `cargo test --workspace`。另设有 **Linux 专用 release 构建** 步骤：`cargo build -p oclive_kernel_server --release`，保证无头交付物在 Linux 上可编过。

---

## 8. 相关文档

- OOCP：`creator-docs/oocp/OOCP_SPEC_v0_1.md`
- 角色包：`roles/README_MANIFEST.md`
- 内核 Server：`crates/oclive_kernel_server/README.md`
- 特性裁剪：`creator-docs/kernel/LIGHTWEIGHT_PROFILE.md`
- 朋友内测（含 Windows 分发）：`docs/FRIEND_BETA_TEST_GUIDE.md`（与 Linux 无头互补）

---

## English summary

The **Linux kernel engine** is `oclive_kernel_server`: same HTTP/OOCP contracts as the Windows desktop kernel. Use **`OOCP_API_BIND`** (default `127.0.0.1`) and **`OOCP_API_TOKEN`** when exposing the service. Docker and systemd templates live under **`delivery/`**. Multimodal sensing stays **out of process**; inject context via **`POST /chat`** `message` text or future DTO extensions.
