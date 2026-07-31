# Headless kernel minimal loop (K1)

Bilingual quick start for integrating **without the Vue desktop** — same domain orchestration as the main app via **`--api` HTTP**. See [PURE_KERNEL_BOUNDARY.md](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) · [KERNEL_IMPLEMENTATION_PLAN.md](../../creator-docs/getting-started/KERNEL_IMPLEMENTATION_PLAN.md) phase **K1**.

> **Shapes today**: **`--api`** on `oclivenewnew-tauri` (this doc), standalone **`oclive-kernel-server`**, or **`library` + `oclive_kernel_runtime`** — see [PURE_KERNEL_BOUNDARY.md](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) §5 and [KERNEL_PLATFORM_DEVELOPER_PATH.md](../../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md). `oclive-cli init --kernel-source <oclivenewnew>` links the real workspace; `init` without that flag stays a minimal serde stub.

---

## 中文

### 前置

- 在 **oclivenewnew 仓库根** 执行。
- 已安装 Rust、Node（跑 OOCP 时）。
- 角色包：默认 `roles/`（或 `OCLIVE_ROLES_DIR`）。

### 1. 构建并启动 HTTP API

```bash
cargo build -p oclivenewnew-tauri
```

**Windows（PowerShell）**

```powershell
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
$env:OCLIVE_API_TOKEN = "replace-with-a-long-random-token"
$env:RUST_LOG = "info"
.\target\debug\oclivenewnew-tauri.exe --api
```

**Linux / macOS**

```bash
export OCLIVE_HTTP_API_MOCK_LLM=1
export OCLIVE_API_TOKEN="replace-with-a-long-random-token"
export RUST_LOG=info
./target/debug/oclivenewnew-tauri --api
```

默认 **`http://127.0.0.1:8420`**（`OCLIVE_API_PORT` 或 `--port` 可覆盖）：

```bash
curl -s http://127.0.0.1:8420/health
```

`/health` 始终可公开探活；无头宿主默认必须设置 `OCLIVE_API_TOKEN`，其余路由调用时带 `x-oclive-api-token` 请求头。只有隔离的本地开发环境才能显式设置 `OCLIVE_API_ALLOW_UNAUTHENTICATED=1` 跳过认证。OOCP 与重启烟测脚本会读取同名变量并自动附加请求头。

```powershell
$env:OCLIVE_API_TOKEN = "replace-with-a-long-random-token"
curl -H "x-oclive-api-token: $env:OCLIVE_API_TOKEN" http://127.0.0.1:8420/role_info
```

### 2. OOCP 黑盒套件（推荐）

```bash
cd examples/oocp-test-suite
npm install
node run.mjs
```

见 [OOCP_TEST_SUITE.md](../../creator-docs/testing/OOCP_TEST_SUITE.md)。

### 3. 与 `oclive-cli`

- **无 `--kernel-source`**：`init` 生成 **serde 占位**工程，不能替代本节 `--api` 联调。
- **有 `--kernel-source <oclivenewnew 根>`**：生成带 **path 依赖** 的工程，指向本仓库 runtime；详见 [OCLIVE_CLI_GUIDE.md](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md)。

### 机器人 / 嵌入式形态（摘要）

| 方式 | 适用 |
|------|------|
| **HTTP `--api`** | 快速联调、CI、编写器试聊 |
| **Sidecar `remote` LLM** | 网关上跑模型 |
| **目录式插件** | 麦克风、扬声器、电机等外设 |
| **进程内 `library`** | 嵌入 `oclive_kernel_runtime`（见 K4 / [KERNEL_PLATFORM_DEVELOPER_PATH.md](../../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)） |

角色包与槽位：[ROLE_PACK_SPEC.md](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md)。

---

## English

### Prerequisites

- Run from **oclivenewnew repo root**.
- Rust installed; Node for OOCP.
- Role packs: default `roles/` (or set `OCLIVE_ROLES_DIR`).

### 1. Build and start HTTP API

```bash
cargo build -p oclivenewnew-tauri
```

**Windows (PowerShell)**

```powershell
$env:OCLIVE_HTTP_API_MOCK_LLM = "1"
$env:OCLIVE_API_TOKEN = "replace-with-a-long-random-token"
$env:RUST_LOG = "info"
.\target\debug\oclivenewnew-tauri.exe --api
```

**Linux / macOS**

```bash
export OCLIVE_HTTP_API_MOCK_LLM=1
export OCLIVE_API_TOKEN="replace-with-a-long-random-token"
export RUST_LOG=info
./target/debug/oclivenewnew-tauri --api
```

Default **`http://127.0.0.1:8420`** (`OCLIVE_API_PORT` or `--port` to override):

```bash
curl -s http://127.0.0.1:8420/health
```

`/health` is always public for readiness probes. Headless hosts must set `OCLIVE_API_TOKEN` by default and send it as `x-oclive-api-token` to every other route. Only isolated local development may explicitly bypass authentication with `OCLIVE_API_ALLOW_UNAUTHENTICATED=1`. The OOCP and restart smoke scripts read the same variable and attach the header automatically.

```bash
export OCLIVE_API_TOKEN="replace-with-a-long-random-token"
curl -H "x-oclive-api-token: $OCLIVE_API_TOKEN" http://127.0.0.1:8420/role_info
```

### 2. OOCP black-box suite (recommended)

```bash
cd examples/oocp-test-suite
npm install
node run.mjs
```

See [OOCP_TEST_SUITE.md](../../creator-docs/testing/OOCP_TEST_SUITE.md).

### 3. vs `oclive-cli`

- **Without `--kernel-source`**: `init` emits a **serde stub**; it does **not** replace this `--api` loop for bring-up.
- **With `--kernel-source <oclivenewnew root>`**: generated `Cargo.toml` uses **path** deps into this repo — see [OCLIVE_CLI_GUIDE.md](../../creator-docs/cli/OCLIVE_CLI_GUIDE.md).

---

## Robot / embedded integration

| Approach | When |
|----------|------|
| **HTTP `--api`** | Fast bring-up, CI, editor try-chat |
| **Sidecar `remote` LLM** | Model on gateway |
| **Directory plugins** | Mic, speaker, motors |
| **Library embed** | In-process `oclive_kernel_runtime` (see K4 / [KERNEL_PLATFORM_DEVELOPER_PATH.md](../../creator-docs/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)) |

Role pack & slots: [ROLE_PACK_SPEC.md](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md).
