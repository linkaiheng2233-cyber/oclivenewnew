# OOCP 协议测试套件（HTTP 黑盒）

本目录对 **`oclivenewnew-tauri --api`** 暴露的 **HTTP 表面契约** 做黑盒校验（`GET /health`、`POST /chat`、`POST /chat/stream`）。默认执行 **S0、S0b、S1–S12、S15、S16，共 16 场景**；可选 **S13/S14** 双核场景（降级与成功路径）。场景真值见 [`../../creator-docs/testing/OOCP_TEST_SUITE.md`](../../creator-docs/testing/OOCP_TEST_SUITE.md)。

> **说明**：当前主程序 HTTP API **未实现 WebSocket**；若完整 OOCP 需 WS 方法链，应在内核增加路由后再扩展本套件。

## 前置条件

- 已编译：`cargo build -p oclivenewnew-tauri`（产物位于 Cargo 配置的 `target-dir`）。
- 角色目录：默认使用 **`distros/chat-pro/roles/mumu`**（**v2** `pipeline.ocblueprint` 黄金包；可通过 `OCLIVE_OOCP_ROLE_PATH` 覆盖）。
- **可选**：设置 **`OCLIVE_HTTP_API_MOCK_LLM=1`** 时，`--api` 使用内存库 + 固定 Mock LLM，**无需本机 Ollama**（CI 与 `examples/oocp-test-suite` 默认依赖此路径）。
- **必需**：启动 `--api` 时设置 **`OCLIVE_API_TOKEN`**；本套件会自动带上 `x-oclive-api-token`。根目录重启烟测未收到外部 token 时会为自己的子进程生成随机 token。

## 启动内核（本地）

在仓库根目录：

```bash
export OCLIVE_ROLES_DIR="$PWD/distros/chat-pro/roles"   # Windows PowerShell: $env:OCLIVE_ROLES_DIR = (Resolve-Path .\distros\chat-pro\roles)
export OCLIVE_API_TOKEN="replace-with-a-long-random-token"
cargo build -p oclivenewnew-tauri
TARGET=$(cargo metadata --format-version=1 --no-deps | jq -r .target_directory)
"$TARGET/debug/oclivenewnew-tauri" --api --port 8420
```

另开终端：

```bash
cd examples/oocp-test-suite
npm test
```

CI 在 **`oocp-test-suite`** job 中于 **`node run.mjs`** 之后执行仓库根 **`node scripts/e2e-core-api-restart.mjs`**（**两次**「起 `--api` → `/health` → `POST /chat` → 杀进程」），验证宿主进程重启后 HTTP 表面仍可用（**A1.1** 子集；默认 Mock LLM）。本地亦可于仓库根执行 **`npm run test:e2e:core-api-restart`**（需已 `cargo build`）。

可选环境变量：

| 变量 | 含义 |
|------|------|
| `OCLIVE_API_BASE` | 默认 `http://127.0.0.1:8420` |
| `OCLIVE_API_TOKEN` | 启动内核所需的 API token；测试脚本自动附加到 `/health` 以外路由 |
| `OCLIVE_OOCP_ROLE_PATH` | 角色包目录绝对路径；默认 `<repo>/distros/chat-pro/roles/mumu` |
| `OCLIVE_HTTP_API_MOCK_LLM` | 设为 `1` 或 `true` 时，`--api` 使用 Mock LLM（无 Ollama） |
| `OCLIVE_OOCP_INCLUDE_S13` | 设为 `1` 时追加 S13（实验核失败后静默降级） |
| `OCLIVE_OOCP_INCLUDE_S14` | 设为 `1` 时追加 S14（实验核成功路径） |
| `OCLIVE_OOCP_INCLUDE_DUAL_CORE` | 设为 `1` 时同时开启 S13 + S14（等价 `--include-dual-core`） |

## JSON 报告

```bash
npm run test:json > report.json
```

输出符合 `schemas/oclive.protocol_conformance_report.v1.schema.json`。

报告包含：

- `dual_core` 摘要块（是否启用双核场景、S13/S14 开关与执行列表）
- `ci_context` 元信息块（生成时间、API 基址、GitHub Actions run id / sha / ref）

便于直接读取 CI 产物并用于对外演示材料。
