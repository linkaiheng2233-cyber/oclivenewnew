# 内核集成者路径：从脚手架到部署（单线）

本文给 **第三方 / 硬件 / 网关** 一条最短闭环，与 [PURE_KERNEL_BOUNDARY.md](PURE_KERNEL_BOUNDARY.md)、[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md) 一致。

本文接入的是当前 **完整参考运行时门面** `OcliveKernel`。文件名为兼容既有链接而保留；“平台”不是 OCLive 最小工具内核的定义，物理拆薄状态见 `K-CORE-BOUNDARY-01`。

[English](../../creator-docs-en/getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md)

---

## 1. 准备

1. 克隆 **[oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew)**（本仓库）。
2. 安装 **Rust**、**Node 22+**（跑 OOCP 黑盒时）。
3. 可选：与主仓并列克隆 **oclive doll core**（校企玩偶交付模板），见该目录 `README.md` 与下文互链。

---

## 2. 单线步骤

| 步骤 | 动作 | 产出 / 验收 |
|------|------|----------------|
| 1 | `cargo build -p oclive-cli` | CLI 可用 |
| 2 | `cargo run -p oclive-cli -- init --kernel-source <本仓库根> -o <项目> …` | 带 path 依赖的 **kernel_server** 或 **library** 工程 |
| 3 | 在生成工程根级 **`roles/<id>/`** 放入或编辑角色包（建议先用 `pack create` 或复制 [examples/robot-soul-minimal](../../examples/robot-soul-minimal/)） | 可 `pack validate`；设备交付建议 **`--profile robot-soul`**（见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)） |
| 4 | 目录插件 / 侧车（可选） | 见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)、[REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) |
| 5 | `cargo run -p oclive-cli -- pack validate <角色根> [--profile robot-soul]` | 当前参考宿主格式与 RobotSoulPack 规则 |
| 6 | 运行 | 跨进程：**`cargo run -p oclive_kernel_server -- --api`** 或 **`oclivenewnew-tauri --api`**；进程内：由生成的 `library` 调用 **`OcliveKernel`** |
| 7 | 部署 | 二进制 + 根级 `roles/` + `plugins/`（若用 directory）+ 环境变量：`OCLIVE_ROLES_DIR`、`OCLIVE_API_PORT`、`OCLIVE_HTTP_API_MOCK_LLM`（联调）等；主仓内置示例才位于 `distros/chat-pro/` |

这些步骤验证现有参考宿主。独立 [kernel minimal role contract](../../handoff/ROLE_PACK_BOUNDARY.md) 已确认内容边界，加载/CLI 接入尚待实现；`robot-soul` profile 通过不能代替该最小契约验收。

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

将角色蓝图的 `type: llm` 实例设为 `backend: remote`，并配置 `OCLIVE_REMOTE_LLM_URL`（见 [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)）。

---

## 5. 嵌入式 `library` 形态

- **`oclive-cli init --project-type library --kernel-source <oclivenewnew根>`** 生成可独立 `cargo check` 的 **`lib`**，直接重导出 `OcliveKernel` 以及 host/contracts/runtime/types；不依赖 Tauri，也不需要启动 HTTP。
- `OcliveKernel` 是可信 Rust 宿主的受支持集成门面。它与桌面和无头服务复用同一 `AppState`、`process_message`、Repository、PluginHost、Event Ring 与资源协调路径，不建立第二套编排。

### 最小进程内回合

```rust,no_run
use my_oclive_kernel::{types, KernelResult, OcliveKernel, OcliveKernelConfig};

async fn one_turn() -> KernelResult<()> {
    let config = OcliveKernelConfig::new("./data", "./roles");
    let kernel = OcliveKernel::start(config).await?;
    kernel.load_role("my-role").await?;

    let response = kernel
        .process_message(&types::SendMessageRequest {
            role_id: "my-role".into(),
            user_message: "你好".into(),
            ..Default::default()
        })
        .await?;

    println!("{}", response.reply);
    kernel.shutdown().await;
    Ok(())
}
```

### 稳定门面包含什么

| 类别 | 入口 |
|------|------|
| 生命周期 | `OcliveKernelConfig` → `OcliveKernel::start` / `builder` → 消费式 `shutdown(self)` |
| 角色 | `list_roles` · `load_role` · `role_info` |
| 回合 | `process_message` · `process_message_stream`；可信宿主另有 `*_with_origin` 的 `sensor` / `system` 边界 |
| Event Ring | 实现 `contracts::EventModuleRegistrar`；`propose_proactive_turn` → 一次性 permit → `process_proactive_turn`；`event_ring_diagnostics` |
| 宿主适配 | builder 可注入 `contracts::LlmClient` 与显式 `HostProfile`；错误保留 `KernelErrorBody` 稳定 code |

`shutdown(self)` 会停止受管目录插件/模型运行时并等待 SQLite pool 关闭。若要跨任务共享，可由宿主把句柄放入 `Arc`，但结束时仍应恢复唯一所有权并显式关闭。

**边界**：这是 Rust 源码级门面，不是 C ABI。内部 `AppState` 与 HTTP/Tauri 适配器不是集成合同。当前代码级闭环已验证；Linux/ARM 真机、资源预算和长时硬件 soak 仍属于 [TECHNICAL_DEBT_INVENTORY.md](../../handoff/TECHNICAL_DEBT_INVENTORY.md) 的 `V-EMBED-01`，不能因本接口存在而宣称硬件交付完成。

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
| [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) | `slot_registry` 与后端配置权威 |
| [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) | 磁盘角色包 + **RobotSoulPack** |
| [AGENTS.md](../../AGENTS.md) | 协作与测试分层 |

校企玩偶交付包（与本仓并列目录）：**oclive doll core** `README.md`（模板与打包脚本）。
