# official-vue-test-runner

官方目录插件 **`com.oclive.official_vue_test_runner`**：在工作区根目录调用 **Vitest**，供 **oclive-pack-editor**「前端测试」面板或 `directory_plugin_jsonrpc_invoke` 使用。

## 用途

- **`health`**：检查工作区路径、`package.json`、是否声明 `vitest` 依赖
- **`list_test_files`**：递归列出 `src/`（或任意 `root`）下的 `*.test.ts` / `*.spec.ts` 等
- **`run_test`**：在工作区执行 `npx vitest run --reporter=json`，返回通过率、失败摘要与耗时

## 安装

插件已位于本仓库 **`plugins/official-vue-test-runner/`**。无需复制到 `{app_data}/plugins`，编写器会在**工作区根**下的 `plugins/` 按 manifest `id` 解析。

若需单独安装到其他工程：

```bash
cargo run -p oclive-cli -- plugin install --path plugins/official-vue-test-runner
```

## 编写器配置

1. 打开 **oclive-pack-editor** → 创作模式 → **前端测试**
2. **工作区根目录** 填 **oclivenewnew 仓库根**（须同时包含 `package.json` 与 `plugins/official-vue-test-runner/`）
3. 点击 **健康检查** → **列出测试文件** → **运行全部** 或选中单文件运行

编写器通过 Tauri `directory_plugin_jsonrpc_invoke` 拉起本插件子进程（需 **`process:spawn`** 授权）。

## 本地烟测

```bash
cargo run -p oclive-cli -- plugin test --plugin-path plugins/official-vue-test-runner
```

或手动 JSON-RPC（另终端）：

```bash
node plugins/official-vue-test-runner/rpc_server.mjs
# 读 stdout 的 OCLIVE_READY URL，POST health / list_test_files / run_test
```

`run_test` 示例 params：

```json
{ "cwd": "D:/oclivenewnew", "runAll": true, "timeoutMs": 600000 }
```

## 依赖

- 工作区须已 `npm ci` / `npm install`（含 `vitest`）
- 运行 `run_test` 时使用 **`npx vitest`**（需 Node 与网络或本地 cache）

## 相关文档

- [creator-docs/testing/OVERVIEW.md](../../creator-docs/testing/OVERVIEW.md)（T14）
- [PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)
