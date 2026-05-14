# OOCP 协议测试套件（S0–S11）

**状态（`main`）**：已入库 **`examples/oocp-test-suite/`**（`run.mjs` + JSON schema）；CI 工作流 **`.github/workflows/ci.yml`** 中的 **`oocp-test-suite`** job 会构建 `oclivenewnew-tauri`、拉起 **`--api` HTTP 服务**、轮询 **`GET /health`**、执行 **`node run.mjs`**（失败则 job 失败）。

## 运行方式

- **本地**：见 [`examples/oocp-test-suite/README.md`](../../examples/oocp-test-suite/README.md)。
- **环境变量**：
  - `OCLIVE_API_BASE`：默认 `http://127.0.0.1:8420`
  - `OCLIVE_OOCP_ROLE_PATH`：角色包目录（默认 `<repo>/roles/mumu`）
  - **`OCLIVE_HTTP_API_MOCK_LLM=1`**（仅 `--api`）：使用内存库 + 固定回复的 Mock LLM，**CI 默认开启**，无需本机 Ollama。

## 场景表（HTTP 黑盒）

| ID | 断言要点 |
|----|-----------|
| S0 | `GET /health` → 200，body `ok` |
| S1 | `POST /chat` 空消息 → 400，`error.code=empty_message` |
| S2 | 非法 `role_path` → 400，`invalid_role_path` 或 `load_role_failed` |
| S3 | `role_path=""` → 400，带错误体 |
| S4 | 合法聊天 → 200，顶层 `reply` 非空 |
| S5 | 成功响应含 `personality_source`（`vector` \| `profile`） |
| S6 | 传入 `session_id` 时回显一致 |
| S7 | 传入 `scene_id` 时响应 `scene_id` 一致 |
| S8 | 中文 + emoji 用户句 → 200 |
| S9 | 长用户句（400 字）→ 200 |
| S10 | 同 `session_id` 连续两轮 → 均 200 |
| S11 | 成功体含 `api_version`、`schema`、`timestamp` |

## 协议符合性报告

`npm run test:json` 输出 JSON，字段集合见 `examples/oocp-test-suite/schemas/oclive.protocol_conformance_report.v1.schema.json`。

## 与完整 OOCP 的关系

当前主程序 **`--api`** 为 **HTTP**（`GET /health`、`POST /chat`），**无 WebSocket 方法链**。本套件校验的是 **HTTP 试聊契约** 与编排结果；若规范中的 WS 语义落地，应在本目录扩展脚本与 CI 步骤。

**文档口径**：与本仓库根 **`README.md`**、**`AGENTS.md`** 对 CI job 名 **`oocp-test-suite`**、场景数 **S0–S11**、目录 **`examples/oocp-test-suite/`** 的叙述一致。

## 测试体系统览

见同目录 [`OVERVIEW.md`](./OVERVIEW.md)。
