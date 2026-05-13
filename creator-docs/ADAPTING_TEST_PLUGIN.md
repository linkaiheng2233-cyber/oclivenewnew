# 从零适配「语言 / 框架测试」目录插件（创作者指南）

> **目标读者**：熟悉 **React / RN / Svelte** 等栈、希望仿照 `official-vue-test-runner` 做一个「在 oclive 里一键跑测试」的社区插件。  
> **前提**：已读 **[目录插件总览](plugin-and-architecture/DIRECTORY_PLUGINS.md)**、**[BRIDGE_API_REFERENCE](plugin-and-architecture/BRIDGE_API_REFERENCE.md)**（`rpc:invoke` / `process:spawn`）。  
> **输出契约**：测试结果 JSON 请对齐 **[testing/TEST_OUTPUT_SCHEMA.md](testing/TEST_OUTPUT_SCHEMA.md)**（`kind` 可取 `oclive.unit_test_run.v1`）。

## 1. 仓库内落点

在 **oclivenewnew** 仓库（或你自托管的 roles 同级 `plugins/` 扫描根）创建目录，例如：

```text
plugins/official-react-test-runner/
  manifest.json
  rpc_server.mjs      # 或 rpc_server.cjs / 二进制入口
  ui/index.html       # 可选整壳
  README.md
```

内核通过 `manifest.json` 的 **`id`** 注册插件；**文件夹名不必等于 id**（扫描读 `id` 字段）。

## 2. `manifest.json` 最小骨架

- **`type`**: `ocliveplugin`  
- **`id`**: 反向域名，如 `com.example.react_test_runner`  
- **`shell.entry`**: 指向 `ui/index.html`（若需要预览面板）。  
- **`shell.bridge.invoke`**: 至少包含 **`rpc:invoke`**；若侧车会 `spawn` 子进程，再加 **`process:spawn`**。  
- **`process`**: `command` + `args` 拉起 JSON-RPC HTTP 侧车（与官方示例一致：stdout 打印 `OCLIVE_READY http://…/rpc`）。  
- **`rpcMethods`**: 列出宿主可 `directory_plugin_invoke` 的方法名。

权限与审计规则见 **[AGENTS.md](../../AGENTS.md)** 内核约束章节。

## 3. 替换测试运行器（Vitest → Jest / Mocha / Playwright）

1. **侧车进程**：在 `rpc_server.mjs` 的 `run_test` 分支，把  
   `npx vitest run … --reporter=json --outputFile=…`  
   换成你的命令，例如：  
   - Jest：`npx jest --json --outputFile=…`  
   - Mocha：`npx mocha --reporter json > …`  
   - Playwright：`npx playwright test --reporter=json`（再归一化到本 schema 的 `summary` / `failures`）。
2. **解析器**：实现 `parseYourRunnerReport(json) →` 与 **TEST_OUTPUT_SCHEMA** 一致的 `summary` / `suites` / `failures`。Vitest 与 Jest 的 JSON 顶层字段不同，**不要假设** `numPassedTests` 存在；可做分支或统一先转成中间模型。
3. **超时**：保留 `timeoutMs` 参数并在 `spawn` 上 `kill`，避免 CI 挂死。

## 4. 权限声明怎么改

- 仅 HTTP JSON-RPC、无子进程：理论上只需 **`rpc:invoke`**（仍由宿主策略决定是否展示高风险提示）。  
- 任何 `child_process` / `exec` / `npx`：必须声明 **`process:spawn`**（及宿主实现要求的网络权限若侧车还会拉包）。  
- 不要把无关权限写进 manifest，以免用户审核困惑。

## 5. `test_utils/oocp_mock.ts` 与 Vitest 的关系

- 该文件只构造 **OOCP 线级 JSON 信封**（`request` / `response`），**不 import Vitest**；可在 **Jest / ts-node / 任意 TS 编译链** 中复用。  
- 若目标栈不是 TypeScript，可复制其字段约定到对应语言，或生成 OpenAPI 式 fixture。

## 6. 宿主验证清单

1. 开发者模式 + 授予 `rpc:invoke`（及 `process:spawn` 若需要）。  
2. 插件管理 → 调试 spawn / `directory_plugin_invoke` 对你的 `health` / `run_test` 打一圈。  
3. 编写器「前端测试」视图：将工作区根指向含 `plugins/<你的目录>` 与项目 `package.json` 的仓库根。

## 7. 一小时骨架 checklist

| 时间 | 步骤 |
|------|------|
| 0–10 min | 复制 `plugins/official-vue-test-runner/manifest.json`，改 `id` / `rpcMethods`。 |
| 10–25 min | 复制 `rpc_server.mjs`，改 `run_test` 命令行 + 报告解析，返回 `structured` 符合 schema。 |
| 25–35 min | `ui/index.html`：最小面板调用 `directory_plugin_invoke` 展示 `summary`。 |
| 35–50 min | 本地 `directory_plugin_invoke` + 一次真实项目 `run_test`。 |
| 50–60 min | README + 指向 **TEST_OUTPUT_SCHEMA** 与 **本指南** 的链接。 |

## 8. 延伸阅读

- **统一输出**：[testing/TEST_OUTPUT_SCHEMA.md](testing/TEST_OUTPUT_SCHEMA.md)  
- **OOCP 协议测试**（另一类「测试输出」）：[oocp/OOCP_TEST_SUITE.md](oocp/OOCP_TEST_SUITE.md)  
- **官方 Vue 插件源码**：[../plugins/official-vue-test-runner/README.md](../plugins/official-vue-test-runner/README.md)
