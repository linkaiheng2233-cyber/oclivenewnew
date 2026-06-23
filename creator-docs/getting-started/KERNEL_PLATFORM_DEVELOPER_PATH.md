# 平台开发者路径：从脚手架到部署（单线）

本文给 **第三方 / 硬件 / 网关** 一条最短闭环，与 [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)、[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 一致。

[English](../../creator-docs-en/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)

---

## 1. 准备

1. 克隆 **[oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew)**（本仓库）。
2. 安装 **Rust**、**Node 20+**（跑 OOCP 黑盒时）。
3. 可选：与主仓并列克隆 **oclive doll core**（校企玩偶交付模板），见该目录 `README.md` 与下文互链。

---

## 2. 单线步骤

| 步骤 | 动作 | 产出 / 验收 |
|------|------|----------------|
| 1 | `cargo build -p oclive-cli` | CLI 可用 |
| 2 | `cargo run -p oclive-cli -- init --kernel-source <本仓库根> -o <项目> …` | 带 path 依赖的 **kernel_server** 或 **library** 工程 |
| 3 | 放入或编辑 **`distros/chat-pro/roles/<id>/`**（建议先用 `pack create` 或复制 [examples/robot-soul-minimal](../../examples/robot-soul-minimal/)） | 可 `pack validate`；设备交付建议 **`--profile robot-soul`**（见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)） |
| 4 | 目录插件 / 侧车（可选） | 见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)、[REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| 5 | `cargo run -p oclive-cli -- pack validate <角色根> [--profile robot-soul]` | 契约与 RobotSoulPack 规则 |
| 6 | 无头运行 | **`cargo run -p oclive_kernel_server -- --api`** 或生成工程内 **`cargo run`**（与 `--api` 等价）；或 **`oclivenewnew-tauri --api`** |
| 7 | 部署 | 二进制 + `distros/chat-pro/roles/` + `distros/chat-pro/plugins/`（若用 directory）+ 环境变量：`OCLIVE_ROLES_DIR`、`OCLIVE_API_PORT`、`OCLIVE_HTTP_API_MOCK_LLM`（联调）等 |

---

## 3. 无头与默认端口

- **默认 HTTP 端口**：**8420**（`OCLIVE_API_PORT` 可覆盖）。
- **联调无 LLM**：`OCLIVE_HTTP_API_MOCK_LLM=1`（内存库 + mock LLM）。
- **黑盒**：`examples/oocp-test-suite/run.mjs`（先 `GET /health`）。

详见 [examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md)、[OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md)。

---

## 4. 默认 LLM 仿真（侧车）

不接真模型时，可用 **OpenAI 兼容 HTTP** 范例：

- **[examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)**

`settings.json` 中 `plugin_backends.llm = "remote"` 并配置 `OCLIVE_REMOTE_LLM_URL`（见 [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)）。

---

## 5. 嵌入式 `library` 形态

- **`oclive-cli init --project-type library --kernel-source <oclivenewnew根>`** 生成 **`lib`**，依赖 **`oclive_kernel_runtime`**（无 Tauri）。
- 在自有进程中使用 **`oclive_kernel_runtime::`** 的 DTO、纯 `domain` 逻辑与校验；**完整对话编排**（`process_message`、`AppState`）仍在 **`oclivenewnew-tauri`**，需 HTTP 或进程内集成时再接宿主 crate。

---

## 6. Monolith（仅 kernel_server 脚手架）

高耦合焊接见 [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) 与 **`oclive build` / `oclive bench`**；**`library` 项目不使用 Monolith**。

---

## 7. OTA / 远程日志

列为 **P2**，不阻塞 K1–K4；见 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) K5。

---

## 8. 相关链接

| 文档 | 用途 |
|------|------|
| [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md) | `init` / `build` / `bench` / `pack` / `dev` |
| [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) | `plugin_backends` 权威 |
| [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) | 磁盘角色包 + **RobotSoulPack** |
| [AGENTS.md](../../AGENTS.md) | 协作与测试分层 |

校企玩偶交付包（与本仓并列目录）：**oclive doll core** `README.md`（模板与打包脚本）。
