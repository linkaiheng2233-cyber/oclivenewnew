# OOCP 协议测试套件（Node 参考实现）

对照文档：**[`creator-docs/oocp/OOCP_TEST_SUITE.md`](../../creator-docs/oocp/OOCP_TEST_SUITE.md)**。

## 覆盖场景（S0–S11）

| 编号 | 名称 |
|------|------|
| S0 | HTTP `GET /health` 明文 `ok` |
| S1 | WebSocket 首帧 `capabilities` |
| S2 | `role.list` |
| S3 | `role.get_info`（含 `scenes`） |
| S4 | `session.create` |
| S5 | `session.switch_scene` |
| S6 | `chat.send_message`（首轮） |
| S8 | `chat.send_message` 再连续 3 次（共 4 轮用户消息） |
| S9 | `session.get_state`（v0.1 无 `plugin.list_slots`，见规范表脚注） |
| S10 | 无效方法 → `UNSUPPORTED_METHOD` / 等价错误 |
| S11 | `role.get_info` 包元数据（`version` / `author` / `description`） |
| S7 | `session.destroy`（_wire OK_；销毁后吊销语义见 OOCP_TEST_SUITE） |

脚本为保持会话有效，**在关闭前完成多轮对话**；日志顺序可能与表编号不完全一致，以 `run.mjs` 与文档为准。

## 前置

1. **构建 OOCP 客户端 SDK**（`tools/oocp-client` 的 `dist/` 不在仓库中）：

```bash
cd ../../tools/oocp-client
npm install
npm run build
```

2. **安装本示例依赖**：

```bash
cd examples/oocp-test-suite
npm ci
```

3. **启动无头内核**（另开终端；端口默认 `48888`）：

```bash
# 仓库根
export OCLIVE_ROLES_DIR="$(pwd)/roles"
export OCLIVE_DB_PATH="/tmp/oclive-oocp-test.db"
export OCLIVE_APP_DATA_DIR="/tmp/oclive-oocp-test-appdata"
cargo run -p oclive_kernel_server
```

Windows PowerShell 可用：

```powershell
$env:OCLIVE_ROLES_DIR = "D:\oclivenewnew\roles"
$env:OCLIVE_DB_PATH = "$env:TEMP\oclive-oocp-test.db"
$env:OCLIVE_APP_DATA_DIR = "$env:TEMP\oclive-oocp-test-appdata"
cargo run -p oclive_kernel_server
```

## 运行

```bash
npm test
```

打印符合 **[`creator-docs/testing/TEST_OUTPUT_SCHEMA.md`](../../creator-docs/testing/TEST_OUTPUT_SCHEMA.md)** 的 JSON 摘要（协议套件报告）：

```bash
node run.mjs --json
```

环境变量（可选）：

| 变量 | 默认 | 说明 |
|------|------|------|
| `OOCP_HTTP_PORT` | `48888` | 与内核 `OOCP_API_PORT` 对齐；`OOCP_HTTP_BASE` 未设时用于拼 `http://127.0.0.1:<port>` |
| `OOCP_HTTP_BASE` | `http://127.0.0.1:<OOCP_HTTP_PORT>` | HTTP 根（用于 `GET /health`） |
| `OOCP_WS_URL` | 由 `OOCP_HTTP_BASE` 推导为 `ws://…/oocp` | OOCP WebSocket |
| `OOCP_API_TOKEN` | 空 | 与内核 `OOCP_API_TOKEN` 对齐 |
| `OOCP_TEST_ROLE_ID` | 自动选 `mumu` 或列表首项 | 固定被测角色 id |

## 添加新场景

1. 更新 **`creator-docs/oocp/OOCP_TEST_SUITE.md`** 场景表。  
2. 在 **`run.mjs`** 中增加断言函数并在 `main()` 中调用。  
3. 若 CI 需要覆盖，确认 **`.github/workflows/ci.yml`** 中 `oocp-test-suite` job 仍安装依赖并执行 `npm test`。
