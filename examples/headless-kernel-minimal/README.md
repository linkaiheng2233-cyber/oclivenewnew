# Headless kernel minimal loop (K1)

Bilingual quick start for integrating **without the Vue desktop** — same domain orchestration as the main app via **`--api` HTTP**. See [PURE_KERNEL_BOUNDARY.md](../../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) · [KERNEL_IMPLEMENTATION_PLAN.md](../../creator-docs/getting-started/KERNEL_IMPLEMENTATION_PLAN.md) phase **K1**.

> **Transition**: until `oclive_kernel_runtime` is split (K2), use this binary for robot/hardware bring-up; future `oclive-cli init` projects will `path`-link the runtime lib.

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
$env:RUST_LOG = "info"
.\target\debug\oclivenewnew-tauri.exe --api
```

**Linux / macOS**

```bash
export OCLIVE_HTTP_API_MOCK_LLM=1
export RUST_LOG=info
./target/debug/oclivenewnew-tauri --api
```

默认 **`http://127.0.0.1:8420`**（`OCLIVE_API_PORT` 或 `--port` 可覆盖）：

```bash
curl -s http://127.0.0.1:8420/health
```

### 2. OOCP 黑盒套件（推荐）

```bash
cd examples/oocp-test-suite
npm install
node run.mjs
```

见 [OOCP_TEST_SUITE.md](../../creator-docs/testing/OOCP_TEST_SUITE.md)。

### 3. 与 `oclive-cli` 占位工程

`cargo run -p oclive-cli -- init …` 当前为 **serde 占位**，不能替代本节 `--api`。K2 完成后将链接真实 runtime。

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
$env:RUST_LOG = "info"
.\target\debug\oclivenewnew-tauri.exe --api
```

**Linux / macOS**

```bash
export OCLIVE_HTTP_API_MOCK_LLM=1
export RUST_LOG=info
./target/debug/oclivenewnew-tauri --api
```

Default **`http://127.0.0.1:8420`** (`OCLIVE_API_PORT` or `--port` to override):

```bash
curl -s http://127.0.0.1:8420/health
```

### 2. OOCP black-box suite (recommended)

```bash
cd examples/oocp-test-suite
npm install
node run.mjs
```

See [OOCP_TEST_SUITE.md](../../creator-docs/testing/OOCP_TEST_SUITE.md).

### 3. vs `oclive-cli` stub projects

`oclive-cli init` still emits a **serde stub**; it does **not** replace this `--api` binary until K2 runtime linking lands.

---

## Robot / embedded integration

| Approach | When |
|----------|------|
| **HTTP `--api`** | Fast bring-up, CI, editor try-chat |
| **Sidecar `remote` LLM** | Model on gateway |
| **Directory plugins** | Mic, speaker, motors |
| **Library embed** | After K4: in-process runtime API |

Role pack & slots: [ROLE_PACK_SPEC.md](../../creator-docs/role-pack/ROLE_PACK_SPEC.md) · [SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md).
