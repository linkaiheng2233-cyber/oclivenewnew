# OOCP 协议测试套件（Node 参考实现）

对照文档：**[`creator-docs/oocp/OOCP_TEST_SUITE.md`](../../creator-docs/oocp/OOCP_TEST_SUITE.md)**。

## 前置

1. **构建 OOCP 客户端 SDK**（`tools/oocp-client` 的 `dist/` 不在仓库中）：

```bash
cd ../../tools/oocp-client
npm ci
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

环境变量（可选）：

| 变量 | 默认 | 说明 |
|------|------|------|
| `OOCP_HTTP_BASE` | `http://127.0.0.1:48888` | HTTP 根（用于 `GET /health`） |
| `OOCP_WS_URL` | 由 `OOCP_HTTP_BASE` 推导为 `ws://…/oocp` | OOCP WebSocket |
| `OOCP_API_TOKEN` | 空 | 与内核 `OOCP_API_TOKEN` 对齐 |
| `OOCP_TEST_ROLE_ID` | 自动选 `mumu` 或列表首项 | 固定被测角色 id |

## 添加新场景

1. 更新 **`creator-docs/oocp/OOCP_TEST_SUITE.md`** 场景表。  
2. 在 **`run.mjs`** 中增加断言函数并在 `main()` 中调用。  
3. 若 CI 需要覆盖，确认 **`.github/workflows/ci.yml`** 中 `oocp-test-suite` job 仍安装依赖并执行 `npm test`。
