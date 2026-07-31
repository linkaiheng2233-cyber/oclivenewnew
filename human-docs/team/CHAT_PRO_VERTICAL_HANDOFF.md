# Chat Pro 垂直功能 · 团队交接总览

> **版本**：2026-06-23  
> **基线发行版**：Chat Pro（`distro_id = desktop`）  
> **目标产品方向**：陪伴类硬件 / 低延迟对话 · 后续上开发板  

---

## 项目信息表（负责人填写后转发）

| 项 | 填写 |
|----|------|
| 仓库 URL / 分支 | |
| 本地路径示例 | `D:\oclivenewnew` |
| 演示角色包 | `distros/chat-pro/roles/mumu`（或 `distros/chat-pro/roles/demo-doll`） |
| 组长 / 联调联系人 | |
| 周会时间 | |
| Ollama 模型（若用） | 例：`hermes3:3b` |
| 内核 HTTP 端口 | 默认 `8420` |

---

## 1. 这是什么项目（5 分钟）

**OClive / Chat Pro** 是一个 **AI 角色对话运行时**（Tauri + Vue + Rust），不是整机玩偶方案。

| 我们提供 | 我们不提供 |
|----------|------------|
| 对话编排、记忆、情绪、角色包 | 麦克风驱动、ASR/TTS 引擎、舵机 BSP |
| 本地 HTTP API（`:8420`） | Live2D Cubism SDK（尚未 bundled） |
| 每轮 JSON：`reply`、表情、立绘指令 | 云端默认账号 |

**一轮对话数据流：**

```text
用户输入（文字或 ASR 文本）
    → Chat Pro UI 或 你们的语音脚本
    → Tauri invoke 或 直接 HTTP
    → oclive-kernel-server（127.0.0.1:8420）
    → 返回 JSON（字段名是 reply，不是 response）
    → UI 显示 / TTS 播放 / 立绘切换
```

**关键架构（2026-06 现状）：**

- Chat Pro **桌面壳**与 **内核进程**已分离：对话写库只在内核进程完成。  
- **组员默认只改圈外层**（Vue 渲染 或 `examples/voice-loop-minimal`），内核当 HTTP 黑盒。  
- 目录白名单 / 黑名单：[SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md)

---

## 2. 工作线分工（解耦后各干一块）

| 轨道 | 负责人 | 文档 | 代码工作区（仅这些） |
|------|--------|------|----------------------|
| **视觉升级** | 成员 A | [TRACK_VISUAL_UPGRADE.md](./TRACK_VISUAL_UPGRADE.md) | `distros/shared/src/components/visual/` · Shell · `distros/chat-pro/roles/demo-doll/` |
| **语音识别** | 成员 B | [TRACK_VOICE_RECOGNITION.md](./TRACK_VOICE_RECOGNITION.md) | **`examples/voice-loop-minimal/`** |
| **延迟 / Chat Pro stream UI** | **组长**（本 sprint 默认） | 见下方说明 | `distros/shared/src/api/chatStream.ts` · `chatStoreSend.ts`（待建） |

**两人都不需要：** 改 `kernel/crates/oclive_kernel_host`、读 `process_message` 全流程、碰六槽/迁移/SQL。

**并行关系：**

- 语音 **POST 文本、读 JSON**；视觉 **读 invoke 响应 / `performance_directive`** —— **无共享源码目录**。  
- 联调靠：**同一 `:8420`、同一角色包（建议 `demo-doll`）**；**不要求** UI 与 `loop.py` 共享 `session_id`（见 [TRACK_VOICE §B6](./TRACK_VOICE_RECOGNITION.md)）。

### 2.1 延迟与 stream（本 sprint 边界）

| 能力 | 状态 | 谁做 |
|------|------|------|
| 内核 `POST /chat/stream`（SSE） | **已有** | 黑盒使用 |
| `examples/voice-loop-minimal --stream` | Week 3 可选 | 成员 B |
| Chat Pro UI 打字机（接 stream） | **未接** | **组长**或 sprint 后单开 |
| 「整句 1 秒内」 | 全链路目标 | stream + 小模型 + 关 `OCLIVE_PORTRAIT_EMOTION_LLM=0`；**本 sprint 不承诺 UI 达标** |

成员 B 可在 loop 侧独立测 ttft；成员 A 验收切图 **不依赖** stream。

---

## 3. Day 0 共同验收（半天，两人都要过）

> **开发环境安装、Ollama、Python、环境变量、故障排查**：见 **[DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md)**（本节只做功能验收）。

### 3.1 环境（按角色裁剪）

完整安装见 [DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md)；**不必两人都做全套**。

| 角色 | Day 0 最少要做 | 可以跳过 |
|------|----------------|----------|
| **视觉** | `setup-dev.ps1` · `npm install` · `npm run tauri:dev` | Python · 语音 example |
| **语音** | Python venv · 内核 `:8420` · `python loop.py` | `npm install`（若用无头内核）· 全量 `tauri:dev` 首次编译 |

```powershell
# 视觉 — 日常
cd D:\oclivenewnew
.\scripts\setup-dev.ps1
npm install
npm run tauri:dev

# 语音 — 日常（内核另终端，见 DEV_ENVIRONMENT §5B）
cd D:\oclivenewnew\examples\voice-loop-minimal
.\.venv\Scripts\Activate.ps1
python loop.py
```

工作区边界：[SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md)

### 3.2 无头 API（理解内核边界）

另开终端（或只用 `tauri:dev` 已 spawn 的内核）：

```powershell
curl.exe -s http://127.0.0.1:8420/health
```

### 3.3 手测 POST /chat

```powershell
$body = @{
  role_path = "D:/oclivenewnew/distros/chat-pro/roles/mumu"
  message   = "hello"
  scene_id  = "default"
} | ConvertTo-Json -Compress

Invoke-RestMethod -Uri "http://127.0.0.1:8420/chat" -Method POST -Body $body -ContentType "application/json; charset=utf-8"
```

**检查响应**：必须包含 `reply` 字段（字符串）。

### 3.4 必读约束（PR 前）

只需遵守 [SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md) 黑名单 + 下表：

| 规则 | 说明 |
|------|------|
| 回复字段 | **`reply`**，禁止自造 `response` |
| 工作区 | 视觉不动 `examples/voice-*`；语音不动 `distros/` 前端 |
| 内核 | 本 sprint **不改** `process_message.rs` / `kernel/crates/oclive_kernel_host` |
| ASR/TTS | **不进**六槽；只在 `examples/voice-loop-minimal` |
| Live2D | 勿把 Cubism 打进默认 CI |

invoke camelCase 等细节：视觉同学读 [paths/frontend.md](../paths/frontend.md) §关键路径即可，**不必读完整 [04 工程约束](../04_ENGINEERING_RULES.md)**，除非改 Tauri。

### 3.5 Day 0 勾选

**环境**（详见 [DEV_ENVIRONMENT.md §9](./DEV_ENVIRONMENT.md)）：

- [ ] `setup-dev.ps1` 或等价检查通过  
- [ ] 视觉：`npm run tauri:dev` 能对话；语音：`python loop.py` 能打印 `reply`  

**理解**：

- [ ] 能解释「UI 进程 vs 内核进程」  
- [ ] 能用 curl/Invoke-RestMethod 调通 `/chat`  
- [ ] 已打开各自轨道文档  

---

## 4. 联调节点（建议排期）

| 时间 | 内容 | 参与 |
|------|------|------|
| **W1 周五** | B 演示 `voice-loop-minimal` 键盘版；A 演示主界面 catalog 切图 | A + B + 组长 |
| **W2 周三** | 对齐 `session_id`、角色路径、`OCLIVE_ROLES_DIR` | A + B |
| **W2 周五** | 语音 loop + 视觉：发一句后 **同时** 听到 reply + 看到立绘变化 | A + B |
| **W3** | Live2D 许可 memo 评审；是否解冻 Cubism | A + 组长 |

**联调检查清单：**

1. 内核 `GET /health` → `"ok": true`  
2. **语音线**：`loop.py` 固定 `session_id` 连聊 3 轮能引用上下文（**Chat Pro UI 不参与此项**）  
3. **视觉线**：`demo-doll` 下 `performance_directive.path` 或 `fallback_image` 非空时主界面图片变化  
4. **可选**：B 打日志 directive，A 截图 UI，对比同角色同一句  

---

## 5. PR 与测试（按工作区）

| 谁 | 改了什么 | PR 前命令 |
|----|----------|-----------|
| **视觉** | 仅 `distros/` 前端 | `npm run test:unit` · `npm run build` |
| **视觉** | 含 `distros/desktop-tauri/` | 上项 + `cargo test -p oclivenewnew-tauri --lib` |
| **语音** | 仅 `examples/voice-loop-minimal/` | README 验收步骤 + `python loop.py` 手测 |
| **任何人** | 含 `kernel/crates/*` | **本 sprint 需组长批准** + `npm run check:release` |

越界文件见 [SCOPE_AND_BOUNDARIES.md §5](./SCOPE_AND_BOUNDARIES.md)。

---

## 6. 资料索引

| 主题 | 路径 |
|------|------|
| **工作区与边界（优先）** | [SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md) |
| **开发环境配置** | [DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md) |
| 硬件接入 SSOT | [HARDWARE_INTEGRATION.md](../../HARDWARE_INTEGRATION.md) |
| 立绘与视觉表现边界 | [立绘 RFC](../../creator-docs/rfc/RFC_PORTRAIT_FACILITY.md) · [视觉表现 RFC](../../creator-docs/rfc/RFC_VISUAL_PRESENTATION_FACILITY.md) |
| Live2D 冻结决策（历史） | [archive/LIVE2D_CUBISM_DEFER.md](../../handoff/archive/LIVE2D_CUBISM_DEFER.md) |
| 无头联调 | [examples/headless-kernel-minimal/README.md](../../examples/headless-kernel-minimal/README.md) |
| 语音 loop 起点 | [examples/voice-loop-minimal/README.md](../../examples/voice-loop-minimal/README.md) |
| 架构简图 | [01 简架构](../01_ARCHITECTURE_SIMPLE.md) |

---

## 7. 常见问题

**Q：我们还要懂整个 OClive 吗？**  
A：**不需要。** 按 [SCOPE_AND_BOUNDARIES.md](./SCOPE_AND_BOUNDARIES.md) 只改白名单目录；内核是 `:8420` 上的 API。

**Q：我们要改内核吗？**  
A：本 sprint **默认不改** 层 1（`kernel/crates/oclive_kernel_host`）。视觉 = 层 3 UI；语音 = 层 3 `examples/`。

**Q：Chat Pro 和玩偶固件是什么关系？**  
A：同一套 HTTP API。玩偶上可 **没有 Vue**，只跑 `oclive-kernel-server` + 你们中控；Chat Pro 是联调与演示壳。

**Q：1 秒内回复谁负责？**  
A：全链路问题。**本 sprint**：B 可在 HTTP 侧实验 stream/ttft；视觉用 `OCLIVE_PORTRAIT_EMOTION_LLM=0` 减 LLM 轮次；**Chat Pro UI 接 stream 由组长负责**，未接入前不要要求 UI 打字机达标。

**Q：不懂 Rust 能干活吗？**  
A：可以。视觉线以 **Vue/TS** 为主；语音线以 **Python/示例 HTTP** 为主。仅当要加 Tauri 命令时才碰 Rust。
