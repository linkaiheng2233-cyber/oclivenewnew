# 轨道 A · 视觉升级（立绘 / Live2D）

> **读者**：负责 Chat Pro 主界面视觉表现的工程师。  
> **前置**：[DEV_ENVIRONMENT.md](./DEV_ENVIRONMENT.md)（§3 Windows 全套 · §9 视觉线验收）→ [CHAT_PRO_VERTICAL_HANDOFF.md](./CHAT_PRO_VERTICAL_HANDOFF.md) → [paths/frontend.md](../paths/frontend.md)  
> **预计周期**：2–3 周（Live2D 实装取决于 SDK 决策，见 Week 3）  
> **路径占位符**：`<REPO_ROOT>` = 本机 clone 路径（例：`D:\oclivenewnew`）。

---

## 0. 开发环境（视觉线）

| 必装 | 验证 |
|------|------|
| Node ≥ 20 · Rust · MSVC · WebView2 | `.\scripts\setup-dev.ps1` |
| Ollama + `hermes3:3b` 或 `qwen2.5:3b` | `ollama list` |
| 磁盘 ≥ 30GB 空闲 | [DEV_ENVIRONMENT.md §1](./DEV_ENVIRONMENT.md) |

**日常：**

```powershell
cd <REPO_ROOT>
npm run tauri:dev
```

**PR 前：**

```powershell
npm run test:unit
npm run build
# 若改 distros/desktop-tauri/：
cargo test -p oclivenewnew-tauri --lib
```

**工作区白名单 / 禁区** → [SCOPE_AND_BOUNDARIES.md §2](./SCOPE_AND_BOUNDARIES.md)

---

## 1. 你的目标（可验收）

| 阶段 | 完成定义（Done） |
|------|------------------|
| **W1** | 使用 **带 catalog 的角色包**（`demo-doll`），连聊后左侧立绘 PNG **随回复切换** |
| **W2** | `ChatProStage.vue` 接入 `FluentShell` / `ToolShell`；store 与 directive 字段对齐 |
| **W3** | Live2D 决策 memo 1 页；`kind=live2d` 时 PNG fallback **不 crash** |

**不在本轨道：** ASR/TTS（`examples/voice-loop-minimal/`）、`kernel/crates/` 内核、开发板 BSP。

**不在本 sprint（除非组长另排）：** Chat Pro **`/chat/stream` 打字机 UI**（延迟线 · 见 [CHAT_PRO §2](./CHAT_PRO_VERTICAL_HANDOFF.md)）。当前 UI 走整段 `send_message`，**无浏览器 Network 里的 `/chat` 请求**。

---

## 2. 背景：数据从哪来

Chat Pro 路径：

```text
用户发送 → Tauri invoke('send_message') → 内核 HTTP :8420 → 响应进 chatStoreSend → roleStore
```

**UI 挂载链（改立绘必知）：**

```text
FluentShell.vue / ToolShell.vue
  → RoleDetailView.vue
    → CharacterInfo.vue（PNG）
  （W2 目标：中间插入 ChatProStage.vue）
```

props 来源：`useMainShell.ts` 的 `portraitAssetRelPath`（来自 `roleStore.roleInfo.portraitAssetPath`）。

每轮结束后，关注字段（已在 `chatStoreSend.ts` 写入 store）：

| 字段 | 用途 |
|------|------|
| `bot_emotion` / `portrait_emotion` | legacy 七图情绪 tag |
| `visual_state_id` | catalog 条目 id |
| `performance_directive` | **渲染 SSOT** |

```json
{
  "kind": "image",
  "path": "assets/images/happy.webp",
  "fallback_image": "assets/images/neutral.webp",
  "live2d_model": null,
  "motion": null
}
```

**原则：** 选图在内核 **post_llm** 已完成；UI **禁止再调 LLM 选图**。

---

## 3. 必读文件（约 1 小时）

| # | 文件 | 看什么 |
|---|------|--------|
| 1 | [SCOPE_AND_BOUNDARIES.md §2](./SCOPE_AND_BOUNDARIES.md) | 白名单 |
| 2 | `distros/shared/src/stores/chatStoreSend.ts` | directive → `roleStore` |
| 3 | `distros/shared/src/composables/useMainShell.ts` | `portraitAssetRelPath` |
| 4 | `distros/chat-pro/src/views/RoleDetailView.vue` | 立绘容器 |
| 5 | `distros/shared/src/components/role/CharacterInfo.vue` | PNG 加载 |
| 6 | `handoff/LIVE2D_CUBISM_DEFER.md` | Live2D 为何 defer |

**Week 2+：** `handoff/PORTRAIT_VISUAL_PRESENTATION_IMPLEMENTATION_PLAN.md`（Phase 1–4）

**Live2D 参考（只读抄结构）：**

- `distros/shared/src/components/visual/Live2DStageAdapter.vue`  
- `distros/theater/src/shells/theater/TheaterStagePanel.vue`

**不必读：** `process_message`、语音 example、PLUGIN_V1 全文。

---

## 4. Week 1 任务清单

### 任务 A1 · 验证现有管线（0.5 天）

**步骤：**

1. `npm run tauri:dev`，在 Chat Pro 与 **demo 角色**（A2 完成后）或临时 mumu 发几句。  
2. **不要**在浏览器 Network 里找 `/chat`——对话走 **Tauri invoke**，Network 看不到内核 HTTP。  
3. 用以下任一方式看表现字段：  
   - Vue DevTools → **Pinia** → `roleStore.roleInfo`（`portraitAssetPath`、`visualStateId`）  
   - 临时在 `chatStoreSend.ts` 的 `send_message` 成功后 `console.log(res.performance_directive)`  
4. 对照 [paths/frontend.md](../paths/frontend.md)：`invoke` 参数为 camelCase。

**预期结论（重要）：**

| 角色包 | 预期 |
|--------|------|
| **`distros/chat-pro/roles/mumu`（默认）** | `config.json` **无** `portrait_catalog` → `performance_directive` **常为 null** → 仅 legacy 七图，**不算管线坏了** |
| **`distros/chat-pro/roles/demo-doll`（A2）** | 启用 catalog 后 directive **应有** `path` / `fallback_image` |

**Done：** 短报告截图：Pinia 字段 + 是否 null + 结论「需 A2」。

### 任务 A2 · 最小 catalog 演示角色（1 天）

**推荐：** 新建 `distros/chat-pro/roles/demo-doll/`（勿改官方 mumu，除非组长批准）。

**最小 `config.json` 片段**（完整样例见 OOCP fixture）：

```json
{
  "portrait_catalog": { "enabled": true },
  "visual_presentation": {
    "enabled": true,
    "backend": "image"
  }
}
```

**`portrait_catalog.json`：** 复制改 [examples/oocp-test-suite/fixtures/portrait-catalog/](../../examples/oocp-test-suite/fixtures/portrait-catalog/)（含 `portrait_catalog.json` + `assets/images/*.webp` + `pipeline.ocblueprint` 等），或从 mumu 复制 2–3 张 PNG 做 2–3 个 `assets[].id`。

**校验：**

```powershell
cd <REPO_ROOT>
cargo run -p oclive-cli -- pack validate distros/chat-pro/roles/demo-doll --profile robot-soul
```

**Done：**

- [ ] Chat Pro **加载** `demo-doll` 角色  
- [ ] 连聊 3 句，`portraitAssetPath` 变化或 `CharacterInfo` 图变化  

### 任务 A3 · ChatProStage 抽象（1–2 天）

| 动作 | 路径 |
|------|------|
| **新建** | `distros/shared/src/components/visual/ChatProStage.vue` |
| **修改** | `distros/chat-pro/src/views/RoleDetailView.vue` 或 `FluentShell.vue` / `ToolShell.vue` |
| **只读对齐** | `distros/shared/src/components/visual/Live2DStageAdapter.vue` |
| **可选** | `distros/shared/src/adapters/visual/index.ts` |

**`ChatProStage.vue` 行为：**

1. props：`roleId`, `name`, `emotion`, `performanceDirective`（与 Theater adapter 一致）  
2. `kind !== 'live2d'` → `CharacterInfo` + `path` ?? `fallback_image`  
3. `kind === 'live2d'` → hint 文案 + PNG fallback（**不**引入 Cubism，见 defer 文档）  

**Done：**

- [ ] 主界面布局无回归  
- [ ] `npm run test:unit` 通过  

---

## 5. Week 2 任务清单

### 任务 A4 · 确认 store 全字段（0.5 天）

`chatStoreSend.ts` 应已写入（**先读再改**）：

- `portraitAssetPath` ← `performance_directive.path ?? fallback_image`  
- `visualStateId` ← `visual_state_id`  

若仍不切换：查 `CharacterInfo.vue` 是否 watch `portraitAssetRelPath`；是否用了 **无 catalog** 的角色包。

**Done：** demo-doll 下 Pinia 与屏幕立绘一致。

### 任务 A5 · 发行版 visual 模式（0.5 天 · 文档为主）

复制 `examples/distro-profiles/desktop.oclive.toml` → `examples/distro-profiles/desktop-visual.oclive.toml`：

```toml
[visual_presentation]
mode = "image_only"
```

**本 sprint：** 产出文件 + **memo** 说明如何加载（`OCLIVE_DISTRO_PROFILE` / bundled profile）。**是否默认生效由组长确认**，不阻塞 W1/W2 切图验收。

### 任务 A6 · 与语音线联调（0.5 天）

见 [TRACK_VOICE §B6](./TRACK_VOICE_RECOGNITION.md)：**不要假设**与 `loop.py` 共享 `session_id`。

**推荐联调：**

1. 你：Chat Pro 加载 **demo-doll**，手动发句，截图立绘变化。  
2. B：同角色跑 `loop.py`，日志打印 `performance_directive`。  
3. 对比 path / emotion 是否合理。

**Done：** 双方截图或日志各 1 份。

---

## 6. Week 3 · Live2D 预研

### 任务 A7 · 决策 memo（1 页）

写入 `handoff/` 或 Wiki，回答：

| 问题 | |
|------|---|
| Web Cubism 4 vs WASM vs 原生 | |
| SDK 许可与包体（MB） | |
| 复用 `Live2DStageAdapter.vue` / Theater 的方式 | |
| 是否满足解冻 [LIVE2D_CUBISM_DEFER.md](../../handoff/LIVE2D_CUBISM_DEFER.md) 条件 | |

### 任务 A8 · Cubism spike（memo 批准后 · 可选）

分支引入 SDK，仅 `ChatProStage` 挂载；CI **不得**默认依赖。

---

## 7. 禁区

| 禁止 | 原因 |
|------|------|
| UI 再调 LLM 选立绘 | 第 4 设施边界 |
| 改 `process_message` / `kernel/crates/` | 内核 |
| 改 `examples/voice-loop-minimal/` | B 轨道 |
| 未批准 bundle Cubism | 许可 / CI |

---

## 8. 延迟模式（硬件联调时）

内核进程环境变量（组长或 DEV_ENV 文档启动时设置）：

```powershell
$env:OCLIVE_PORTRAIT_EMOTION_LLM = "0"
```

关闭立绘第二次 LLM，切图走规则，换延迟。与 **stream UI** 无关。

---

## 9. 故障排查

| 现象 | 先查 |
|------|------|
| 图永远不变 | 是否 **demo-doll + catalog enabled**；mumu 默认无 catalog |
| `performance_directive` 全 null | 上条；内核日志 `OCLIVE_PORTRAIT_EMOTION_LLM` |
| 图 404 / 空白 | `portraitAssetPath` 是否为角色包内相对路径 |
| Pinia 有 path 但 UI 不变 | `RoleDetailView` / `CharacterInfo` props 链 |
| invoke 报错 missing key | camelCase · [paths/frontend.md](../paths/frontend.md) |

---

## 10. 交付物与 PR

**交付物：**

- [ ] `distros/chat-pro/roles/demo-doll/`（或等价）  
- [ ] `ChatProStage.vue` + Shell 接入  
- [ ] Live2D memo（W3）  
- [ ] 联调截图  

**PR 描述必勾选：**

- [ ] 未改 `kernel/crates/` · `examples/voice-loop-minimal/`  
- [ ] 未在 UI 增加 LLM 选图  
- [ ] `npm run test:unit` + `npm run build` 通过  

审阅：[CHAT_PRO_VERTICAL_HANDOFF.md §5](./CHAT_PRO_VERTICAL_HANDOFF.md)

---

## 11. 找谁问什么

| 问题 | 找 |
|------|-----|
| `pack validate` 失败 | 组长 + CLI 输出 |
| directive 逻辑 / null | 是否 catalog；组长看内核日志 |
| Cubism 许可 | 组长 / 产品 |
| invoke / DTO | `distros/shared/src/api/chat.ts` · `NAMING_CONVENTIONS.md` |
| stream 打字机 | 组长（本轨道外） |
| 语音 loop | 成员 B · [TRACK_VOICE_RECOGNITION.md](./TRACK_VOICE_RECOGNITION.md) |
