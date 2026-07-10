# 组员工作区与边界（解耦说明）

> **读者**：视觉线 / 语音线组员。  
> **核心结论**：OClive 已 **内核单写者 + HTTP 契约** 解耦；本 sprint **不需要** 读懂全仓，也 **不需要** 改 `process_message` 编排。

---

## 1. 三层模型（背这一张图即可）

```text
┌─────────────────────────────────────────────────────────────┐
│  层 3 · 圈外集成（各组员主战场）                               │
│  · 视觉：Chat Pro Vue 渲染 performance_directive              │
│  · 语音：examples/voice-loop-minimal（ASR/TTS/中控）          │
└───────────────────────────┬─────────────────────────────────┘
                            │ 只认 JSON 契约
┌───────────────────────────▼─────────────────────────────────┐
│  层 2 · HTTP 契约（黑盒使用，本 sprint 不改）                  │
│  GET /health · POST /chat · POST /chat/stream               │
│  字段：reply · bot_emotion · performance_directive · …       │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│  层 1 · 灵魂内核（组长/内核维护者；组员默认不碰）               │
│  kernel/crates/oclive_kernel_host · process_message · SQLite        │
└─────────────────────────────────────────────────────────────┘
```

**组员把层 1 当「本地跑起来的服务」即可**，像调用 REST API 一样用层 2。

---

## 2. 视觉线 · 只管「怎么画」

### 2.1 职责一句话

把内核已经算好的 **`performance_directive`** 在 Chat Pro 主界面 **画出来**（PNG → 将来 Live2D）。

### 2.2 允许改动的目录（工作区）

| 路径 | 用途 |
|------|------|
| `distros/shared/src/components/visual/` | Stage、Live2D adapter |
| `distros/shared/src/components/role/CharacterInfo.vue` | 立绘 PNG 组件（可改） |
| `distros/chat-pro/src/shells/fluent/FluentShell.vue` | Chat Pro 主壳布局 |
| `distros/chat-pro/src/shells/tool/ToolShell.vue` | 备用壳（若启用） |
| `distros/shared/src/adapters/visual/` | 渲染 adapter 注册 |
| `distros/shared/src/stores/roleStore.ts` | 仅表现相关字段写入（小改） |
| `distros/shared/src/stores/chatStoreSend.ts` | 仅确认 directive 写入（小改） |
| `distros/shared/src/composables/useMainShell.ts` | 仅 portrait 相关 computed（小改） |
| `distros/chat-pro/roles/demo-doll/` **或** 演示用角色包 | catalog 资源与 `config.json`（**数据**，非代码） |

### 2.3 只读、一般不改

| 路径 | 原因 |
|------|------|
| `distros/shared/src/api/chat.ts` | invoke 封装；懂 `SendMessageResponse` 形状即可 |
| `distros/desktop-tauri/src/api/chat_backend.rs` | 桌面已固定走 HTTP 内核 |
| `kernel/crates/oclive_kernel_host/` | 内核黑盒 |
| `kernel/crates/oclive_kernel_runtime/` | 引擎公式，与 UI 无关 |

### 2.4 禁止碰

| 路径 / 行为 | 原因 |
|-------------|------|
| `.../process_message.rs` | 编排 SSOT |
| `migrations/`、`app.db` 表结构 | 持久化契约 |
| 六槽、`slot_registry`、蓝图 | 非 UI 层 |
| 在 UI 里再调 LLM 选立绘 | 架构禁止 |
| ASR/TTS | 语音同事轨道 |
| `examples/voice-loop-minimal/` | 语音同事轨道 |

### 2.5 不必读的文档

以下 **可跳过**，除非组长点名：

- `human-docs/06_KERNEL_LEARNING_PATH.md`
- `handoff/BUS_FACTOR_NOTES.md`
- `creator-docs/plugin-and-architecture/PLUGIN_V1.md` 全文
- `dual_core` / Monolith / 蓝图 v3 任意文档

**只需读：**

- 本页 + [TRACK_VISUAL_UPGRADE.md](./TRACK_VISUAL_UPGRADE.md)
- [DEV_ENVIRONMENT.md §2–§3、§9 视觉](./DEV_ENVIRONMENT.md)
- `HARDWARE_INTEGRATION.md` §5.1–5.2（`performance_directive` 字段表）

### 2.6 日常命令（仅视觉）

```powershell
cd D:\oclivenewnew
npm run tauri:dev          # 开发
npm run test:unit          # PR 前
npm run build              # PR 前
```

**不需要**：`cargo test --workspace`、OOCP 全套件（除非改了 `src-tauri`）。

---

## 3. 语音线 · 只管「怎么听、怎么说」

### 3.1 职责一句话

**麦克风/文本 → HTTP → `reply` → 扬声器**；内核当黑盒 API。

### 3.2 允许改动的目录（工作区）

| 路径 | 用途 |
|------|------|
| `examples/voice-loop-minimal/` | **主战场**：loop、ASR/TTS Python 引擎、HTTP 烟测（`loop.py --mic`） |
| （可选）`examples/voice-loop-minimal/asr/` · `tts/` | sherpa-onnx 封装（SSOT）；插件 `rpc_server.mjs` spawn 同模块 |
| **产品插件** | [`distros/chat-pro/plugins/com.oclive.voice.asr/`](../../distros/chat-pro/plugins/com.oclive.voice.asr/) · 独立通道 UI（不进六槽） |

**建议**：语音代码 **尽量只写在 `examples/voice-loop-minimal/`**，与 Chat Pro 前端 **零耦合**，方便以后原样搬到开发板。

### 3.3 只读、一般不改

| 路径 | 原因 |
|------|------|
| `HARDWARE_INTEGRATION.md` §4–§5 | 契约 SSOT |
| `kernel/crates/oclive_kernel_host/src/http_api/` | 理解路由即可 |
| 根目录 `distros/chat-pro/roles/mumu` | 提供 `role_path`，勿改人格除非组长要求 |

### 3.4 禁止碰

| 路径 / 行为 | 原因 |
|-------------|------|
| 整个 `distros/` 前端（Vue） | 不是语音线职责 |
| `distros/desktop-tauri/` | 桌面壳 |
| `kernel/crates/` | 内核 |
| 把 ASR 写进 `process_message` 或六槽 | 架构边界 |
| 公网暴露 `:8420` | 仅 loopback |

### 3.5 不必读的文档

- `human-docs/paths/frontend.md`
- `handoff/PORTRAIT_*`、`LIVE2D_*`
- `npm run tauri:dev` 相关章节（除非联调看 Chat Pro）

**只需读：**

- 本页 + [TRACK_VOICE_RECOGNITION.md](./TRACK_VOICE_RECOGNITION.md)
- [DEV_ENVIRONMENT.md §2 语音列、§4–§5、§9 语音](./DEV_ENVIRONMENT.md)
- `HARDWARE_INTEGRATION.md` §4 主循环伪代码

### 3.6 日常命令（仅语音）

**终端 1 — 内核（三选一）：**

```powershell
# A. 同事已开 tauri:dev，你不管
# B. 无头内核（推荐）
cargo build -p oclive-kernel-server   # 仅首次或内核升级时
# … 见 DEV_ENVIRONMENT §5 模式 B
# C. 使用组长提供的 oclive-kernel-server.exe + 启动脚本
```

**终端 2 — 你的代码：**

```powershell
cd D:\oclivenewnew\examples\voice-loop-minimal
.\.venv\Scripts\Activate.ps1
python loop.py
```

**PR 前**：README 里验收步骤可复现；**不需要** `npm run check`（除非你改了仓库根别的文件）。

### 3.7 语音线轻量环境（可不做全仓前端构建）

| 需要 | 不需要 |
|------|--------|
| Git clone 仓库（为了 `examples/` + `distros/chat-pro/roles/`） | `npm install` 全量 |
| Python venv | 首次 `npm run tauri:dev` 2 小时编译（可用无头内核代替） |
| Ollama + 小模型 | Vue / Rust 日常开发 |
| 内核进程 `:8420` | 读 `process_message` 源码 |

---

## 4. 两人如何对接（唯一耦合点）

| 耦合点 | 约定 |
|--------|------|
| **HTTP** | 同一 `127.0.0.1:8420`、同一角色包（建议 `distros/chat-pro/roles/demo-doll`） |
| **session_id** | **仅语音 loop 内**固定 UUID 测记忆；Chat Pro UI **默认不同 session**，联调 **不要求** UI 与 loop 共享 session（见 [TRACK_VOICE §B6](./TRACK_VOICE_RECOGNITION.md)） |
| **表现字段** | B 在 loop 日志打印 `data.performance_directive`；A 在 Pinia/UI 显示——**互不改对方目录** |
| **联调** | W2 周五：B 触发 `/chat` + 日志；A 在 Chat Pro 手动发句或对照 directive 截图 |

**没有共享代码目录**；耦合只在 **运行时 JSON**。

---

## 5. PR 范围对照

| 谁 | 典型 PR 文件数 | 审阅重点 |
|----|----------------|----------|
| **视觉** | `distros/shared/src/components/visual/*`、`FluentShell.vue`、`distros/chat-pro/roles/demo-doll/*` | 是否只在消费 directive；是否误触内核 |
| **语音** | `examples/voice-loop-minimal/*` | 是否只 HTTP；README 可复现 |
| **越界 PR** | 含 `kernel/crates/oclive_kernel_host` | 组长必须 review，本 sprint 默认拒绝 |

---

## 6. 与「全项目入门」的关系

| 文档 | 视觉 | 语音 |
|------|:----:|:----:|
| [SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md)（本文） | ✅ | ✅ |
| [DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md) | §3 全套 | §2 语音列 + §4–§5 |
| [TRACK_*.md](./TRACK_VISUAL_UPGRADE.md) | ✅ | ✅ |
| [CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md) | §1–§4  skim | §1–§4  skim |
| [human-docs/README.md](../README.md) L0–L6 全套 | ❌ 延后 | ❌ 延后 |
| [06 内核学习路径](../06_KERNEL_LEARNING_PATH.md) | ❌ | ❌ |

**结论**：组员是 **垂直功能开发者**，不是 **内核维护者**；按上表读文档即可，不必完成 human-docs 全套 L6 验收。

---

## 7. 常见问题

**Q：我还用 clone 整个 oclivenewnew 吗？**  
A：要。语音至少要有 `examples/` 和 `distros/chat-pro/roles/`；视觉要有 `distros/` 前端。但 **不必编译/理解全 workspace**。

**Q：语音同事完全不用 Chat Pro？**  
A：**开发阶段可以**。联调日打开 Chat Pro 只看立绘即可，日常只在 `voice-loop-minimal` 写代码。

**Q：视觉同事要懂 Rust 吗？**  
A：**默认不要**。只改 Vue；除非要新增 Tauri invoke（本 sprint 未安排）。

**Q：内核 bug 谁修？**  
A：组长。组员只提 Issue：`/health` JSON + `/chat` 请求体 + 错误 `code`。
