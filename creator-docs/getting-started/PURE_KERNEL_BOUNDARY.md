# 纯净内核：边界、灵魂与嵌入式范围

本文定义 oclive **「纯净内核」** 在工程与产品叙事中的含义，并与桌面宿主、无头服务、嵌入式库、机器人「灵魂」交付对齐。模块分层见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)；总览图见 [KERNEL_AND_MODULES_ARCHITECTURE.md](KERNEL_AND_MODULES_ARCHITECTURE.md)；实施阶段见 [KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)。

[English](../../creator-docs-en/getting-started/PURE_KERNEL_BOUNDARY.md)

---

## 1. 纯净内核是什么

**纯净内核**指运行时中**与 UI 无关、与具体硬件 BSP 无关、与某一闭源模型品牌无关**的那一层，负责：

| 职责 | 实现锚点（主仓） |
|------|------------------|
| **回合编排** | `src-tauri/src/domain/chat_engine/` · `process_message` |
| **槽位解析** | `PluginHost::resolve_for_role` · `plugin_backends` |
| **契约与持久化形状** | `oclive_kernel_runtime`（DTO / 纯 domain）· `migrations/001_init.sql` · `oclive_validation` |
| **无头入口（过渡）** | `http_api` · **`oclive-kernel-server`** · **`oclivenewnew-tauri --api`** |

```text
用户/设备边界          →  Vue / 硬件驱动 / 侧车进程（不在「内核」内）
纯净内核               →  process_message + PluginHost + Repository 契约
槽位实现（可替换）     →  builtin / remote / directory / local / ollama …
灵魂数据（可定制）     →  角色包 manifest + settings + 知识/人格文件
```

**不是** Linux 内核，也**不是**整个 Tauri 桌面应用。

---

## 2. 纯净内核明确不包含什么

- **Vue 前端**、Tauri `invoke`、窗口与主题。
- **具体 LLM 厂商 SDK**（应落在 `llm` 槽：ollama / remote / directory）。
- **板级 BSP**（麦克风驱动、电机、RTOS）；通过 **目录插件 / 侧车 / MCP** 接入，内核只消费契约化结果。
- **创作者文档 UI**、插件市场站点、启动器安装体验。
- **Prompt 正文语言**（角色包与模型侧内容语言）；与**界面 i18n** 分离。

---

## 3. 「自定义灵魂」交付单元

对外可说：**灵魂 = 可版本化的数据 + 可配置的槽位策略**，由内核在运行时加载，而非写死在编排代码里。

| 组成部分 | 说明 |
|----------|------|
| **角色包** | `manifest.json` · `settings.json` · `core_personality.txt` · 场景/知识等（见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)） |
| **有效后端** | `plugin_backends` 包默认 + 会话覆盖 + 环境变量合并结果（见 [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md)） |
| **关系与记忆** | `role_runtime`、长期记忆等由内核经 Repository 读写；策略由 `memory` 等槽实现 |

**机器人场景**：设备上通常只换「灵魂包」与少量 `settings`，不换编排内核版本（在 `min_runtime_version` 兼容前提下）。

工作名 **RobotSoulPack**（最小灵魂包）已与 **`oclive pack validate --profile robot-soul`** 对齐；字段与示例见 [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)、[examples/robot-soul-minimal](../../examples/robot-soul-minimal/README.md)。

---

## 4. 情感陪伴在架构中的位置

陪伴能力由**后端模块 + 设施模块**协作完成，而非单一「情感模块」黑盒（分层见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](OCLIVE_ARCHITECTURE_OVERVIEW.md)）：

- **emotion 后端模块** + **复杂情感专家模型设施子模块**：用户句情绪与跨回合叙事 `narrative_hint`。
- **memory / event**：关系与事件对后续回合的影响。
- **prompt / llm**：语言表达与 persona 注入。
- **agent**（可选）：工具与外部世界（MCP、目录插件）。

内核保证 **调用顺序与 DTO**；陪伴「好不好」由槽实现与角色包内容决定。

---

## 5. 部署形态与「一块钢板」

| 形态 | 用途 | Monolith | 说明 |
|------|------|----------|------|
| **桌面宿主** | 玩家 / 创作者 | 可选（独立工程） | Tauri + Vue + 同一 domain |
| **无头 HTTP** | 网关、机器人中控、CI 联调 | **Monolith 仅** `oclive-cli` 生成的 **kernel_server** 工程可选 | 主仓 **`oclive-kernel-server`** 与 **`oclivenewnew-tauri --api`** 等价（`http_api`）；默认端口 **8420**（`OCLIVE_API_PORT`） |
| **嵌入式 `library`** | 进程内嵌、自有 `main` | **不适用** Monolith | 链接 **`crates/oclive_kernel_runtime`**；`oclive-cli init --project-type library --kernel-source`；编排仍在 **`oclivenewnew-tauri`**（见 [KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md) §5） |
| **HTTP `--api`** | 联调、CI、编写器试聊 | N/A | 当前主仓过渡方案，见 [headless-kernel-minimal](../../examples/headless-kernel-minimal/README.md) |

**可拆可焊**：开发期槽位可替换（松耦合）；量产可选 Monolith 将选定 builtin 焊进单一二进制（紧耦合）。二者与 `settings.json` **正交**。

---

## 6. 嵌入式诚实范围表

### 在范围内（当前架构目标）

- Linux 用户态、**数百 MB 级 RAM** 以上的设备或网关。
- **Rust 异步**、HTTP/JSON-RPC、子进程目录插件、SQLite 持久化。
- 与桌面**共用角色包**与 `plugin_backends` 形状。
- 侧车 LLM（`remote`）、本机 Ollama（`ollama`）、目录插件扩展硬件。

### 明确不在范围内（勿过度承诺）

- **硬实时**、**MCU / KB 级 RAM**、无 OS 裸机。
- 内核内建**音视频编解码栈**（应走插件或设备侧服务）。
- 多租户云端**隔离与计费**（未作为内核一等公民；B2 可单独立项）。

---

## 7. 相关链接

- 实施计划：[KERNEL_IMPLEMENTATION_PLAN.md](KERNEL_IMPLEMENTATION_PLAN.md)
- 平台开发者单线：[KERNEL_PLATFORM_DEVELOPER_PATH.md](KERNEL_PLATFORM_DEVELOPER_PATH.md)
- 差距清单：[PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md) §B
- 校企玩偶交付：与主仓并列的 **oclive doll core** 目录（settings 模板、硬件插件示例、打包说明）；契约以本仓为准。
- Monolith RFC：[RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md)
