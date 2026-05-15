# OOCP 协议测试套件（S0–S11）

**状态（`main`）**：已入库 **`examples/oocp-test-suite/`**（`run.mjs` + JSON schema）；CI 工作流 **`.github/workflows/ci.yml`** 中的 **`oocp-test-suite`** job 会构建 `oclivenewnew-tauri`、拉起 **`--api` HTTP 服务**、轮询 **`GET /health`**、执行 **`node run.mjs`**，随后执行 **`scripts/e2e-core-api-restart.mjs`**（**进程重启后再对话** 烟测；失败则 job 失败）。**`frontend`** job 在 **Ubuntu** 上在 **`npm run build`** 后另跑 **Playwright + `vite preview` 首屏**（**A1.1b**；Windows `frontend` 不跑 Playwright）。

## A1.1 PoC：核心 HTTP 进程重启烟测

- **脚本**：根目录 **`scripts/e2e-core-api-restart.mjs`**（Node 20+ 内置 `fetch`，无额外 npm 依赖）。  
- **行为**：在同一端口上 **启动 `--api` → `/health` → `POST /chat` → 终止进程 → 再次启动 → 再 `/health` + `/chat`**；两轮均须成功。默认 **`OCLIVE_HTTP_API_MOCK_LLM=1`**，**无需 Ollama**。  
- **本地**：`cargo build -p oclivenewnew-tauri` 后，仓库根目录执行 **`npm run test:e2e:core-api-restart`**（或手动设置 `OCLIVE_ROLES_DIR` / `OCLIVE_E2E_PORT` / `OCLIVE_E2E_BINARY`）。  
- **说明**：覆盖 **「关开恢复」** 的 **HTTP 宿主进程** 维度；**`vite build` + `vite preview` + Playwright** 首屏烟测见下文 **A1.1b**，CI 在 **`frontend`** job。**安装包 / Tauri 原生窗 / WebDriver 全屋** 另立项，见 [PRODUCT_RELEASE_CHECKLIST.md](../../handoff/PRODUCT_RELEASE_CHECKLIST.md) **A1.1c**。

## A1.1b：Web 预览壳 Playwright 烟测

- **用例**：根目录 [`e2e/preview-shell.spec.ts`](../../e2e/preview-shell.spec.ts)（`#app` 挂载与页签标题）。  
- **本地**：`npm run build && npm run test:e2e:preview`（首次需 `npx playwright install chromium`；Linux 可用 `npx playwright install --with-deps chromium`）。  
- **CI**：**`frontend`** job 在 **Ubuntu** 上由 workflow **先后台拉起 `vite preview`**（默认端口 **4180**），设 **`PW_TEST_USE_EXTERNAL=1`** 后执行 **`npm run test:e2e:preview`**；环境变量 **`PLAYWRIGHT_DISABLE_HEADLESS_SHELL=1`** 以减少额外浏览器下载。

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
| S1 | `POST /chat` 空消息 → 400，`error.code=EMPTY_MESSAGE` |
| S2 | 非法 `role_path` → 400，`INVALID_ROLE_PATH` 或内核加载码（如 `ROLE_NOT_FOUND`） |
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

---

[English](../../creator-docs-en/testing/OOCP_TEST_SUITE.md)
