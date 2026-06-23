# AI 剧场发行版 — 信息架构（IA）

**状态**：模式 1 **已落地**（2026-06）。**最后更新**：2026-06-18
**配套**：[`DEVELOPMENT_ROADMAP.md`](DEVELOPMENT_ROADMAP.md)（思路/路线 SSOT · 模式 1 规格 · §5.5 冻结）· [`README.md`](README.md)（内核重开策略）· [`theater.oclive.toml`](theater.oclive.toml)（profile 模板）
**范围纪律**：仅模式 1。本 IA **不**设计模式 2/3 界面（路线图 §5/§6）。

---

## 0. 一句话

剧场不是聊天软件，是**一座舞台**：用户**看**两个反差灵魂当场对戏，用**戳一戳**淘气地改写剧情。IA 因此从 Chat Pro 的「**聊天列表中心**」改为「**舞台中心**」——但**复用同一套设计语言与 token 体系**，不另起炉灶。

---

## 1. 与 Chat Pro 的关系：复用什么、改写什么

| 维度 | Chat Pro（`ToolShell`） | AI 剧场（`TheaterShell`） | 策略 |
|------|--------------------------|----------------------------|------|
| **设计语言** | 扁平专业灰盘 + token（`--tool-*`，VS Code/Linear/Cursor 风） | **复用 token 体系**，叠加更暖/更俏皮的 accent | **复用** |
| **主题挂载** | `html[data-shell="tool"]` → `theme-tool.css` | `html[data-shell="theater"]` → `theme-theater.css`（以 tool 为 base 覆写 accent/圆角） | **复用机制** |
| **壳选择** | `resolveOcliveShell()` 读 `VITE_OCLIVE_SHELL` | 扩 `OcliveShellKind` 加 `'theater'`；profile `distro_id=theater` 驱动 | **扩展** |
| **状态注入** | `MAIN_SHELL_KEY` ← `useMainShell()` | **新增** `useTheaterShell()`（独立编排，复用底层 store/api） | **新增（瘦）** |
| **中心区** | 聊天气泡列表（我 vs TA） | **舞台画布**（两个角色对戏的剧本流） | **改写** |
| **输入区** | `ChatInput` 自由文本 | **戳一戳 Dock**（封闭芯片，模式 1 不开自由文本） | **改写** |
| **左图标栏 ActivityBar** | settings/plugins/models | **首屏隐藏**（剧场单一目的，零干扰） | **隐藏** |
| **顶栏** | RoleSelector + More | **极简**：剧目/卡司指示 + 「更多」（设置藏二级） | **瘦身** |
| **底层内核** | `process_message` / store / api | **同一套**（剧场是 profile + 前端差分，**不新增编排 stage**，§5.5） | **复用** |

> 复用原则：**底层（内核/store/api/设计 token）全复用；表现层（壳/中心区/交互区）按舞台范式改写。** 不 fork 编排，不重造编辑器（漏斗接 `oclive-pack-editor`）。

---

## 2. 顶层导航地图（模式 1）

剧场是**单屏主导 + 少量叠层**，不是多页应用。

```
TheaterShell（舞台壳 · data-shell="theater"）
│
├─ ① 开场即活（0:00 · 无独立界面，舞台直接播放预生成开场）
│
├─ ② 舞台主屏 [Stage]  ← 唯一常驻主界面
│   ├─ A. 舞台头（卡司/剧目指示 · 极简）
│   ├─ B. 舞台画布（剧本流：两角色对戏）
│   ├─ C. 戳一戳 Dock（封闭芯片 + 节拍/加载）
│   └─ D. 舞台脚注（模型/本地指示 · 极简状态）
│
├─ ③ 叠层（overlay · 按需，default 关）
│   ├─ 创作漏斗入口（戳「性格」峰值时浮出）  → 跳 oclive-pack-editor
│   ├─ 设置（藏「更多」二级；首屏不露）
│   └─ 重开/换剧目（轻量；可选）
│
└─ ④ 离场（创作漏斗 → 编辑器 / 关闭）
```

**刻意不在模式 1 出现**：自由文本输入、插件槽/市场、场景旅行栏、好感数值、debug 面板、模式 2/3 任何入口。（对照 ToolShell 中大量 `v-if="roleStore.interactionImmersive"` 的沉浸专属区——剧场首屏一律不挂。）

---

## 3. 舞台主屏 · 区域分解（Zone IA）

复用 ToolShell 的纵向骨架（头/主/脚），但每区语义改写为舞台。

### Zone A · 舞台头（Stage Header）
- **内容**：**可切换剧目**（早餐 / 超市 / 回家路上 / 洗澡睡觉；点击标题下拉）+ **卡司对**（两角色名/小头像；**可配置**，默认 mumu × 枫侵月，设置 → 卡司 Tab 换角后顶栏/立绘同步）。
- **对应 ToolShell**：`tool-top-bar`（RoleSelector 位）。剧场**不放 RoleSelector**（模式 1 固定双槽，非自由多角色），顶栏为**场景选择器** + **只读卡司指示** + 右侧「更多」入口（设置藏此）。
- **token**：`--tool-topbar-h` / `--tool-chrome-editor` / `--tool-divider`。

### Zone B · 舞台画布（Stage Canvas）—— 核心
- **范式**：**剧本流**，不是「我 vs TA」气泡。两角色的台词以**剧本/对白**呈现（角色名 + 台词 + 可选舞台提示 `(动作)`），观看者是**观众**而非对话方。
- **对应 ToolShell**：`tool-chat-scroll` + `ChatMessageList` 的位置，但**新组件 `TheaterStage` / `TheaterScriptLine`**（复用 `VirtualScrollContainer` 滚动与 `theme` token）。
- **关键状态**：
  - `playing`：开场/补丁后逐行推进（可带轻微逐字/淡入，呼应「活着」）。
  - `idle`：一个固定节拍演完，等待戳。
  - `patching`：见 Zone C 加载节拍。
- **视觉差异**：双角色用**两种 accent 区分嗓音**（左/右对齐或色条），强化「两个不同的人」。
- **token**：`--tool-space-4/6`、双 accent 为 theater 新增 `--theater-cast-a` / `--theater-cast-b`。

### Zone C · 戳一戳 Dock（Poke Dock）—— 交互核心
- **范式**：替代 `ChatInput`。**封闭芯片集**（模式 1 不开自由文本），一眼可见可点：
  - 🍵 喝苦中药 · ⏰ 改到快迟到 · 🥦 逼吃讨厌的菜 · 😼 换个称呼 · 🎭 微调性格
- **场景门控**：**仅早餐场景**显示 `PokeDock`；超市 / 回家路上 / 洗澡睡觉只播官方预生成对白（无戳点 forks）。
- **交互**：点芯片 → `patching` 节拍 → 舞台插入**局部 patch 小剧情**（默认 `mode=patch`，保留官方 skeleton 尾部）→ 回 `idle`；可选 **双候选背景板**（`TheaterVariantBackdrop`）拖拽切换另一种合理分支。
- **对应 ToolShell**：`tool-input-area`（`InteractionModeBar` + `ChatInput` 位）。剧场**新组件 `PokeDock` / `PokeChip` / `BeatLoader`**。
- **含金量排序**：高含金量芯片（苦中药/迟到/讨厌菜/性格）优先且显眼；中性变量不入首屏 Dock（路线图 §4.3）。
- **🎭 性格芯片**：是**创作漏斗暗门**（见 Zone 叠层）。
- **token**：`--tool-radius-lg`、`--tool-row-h`、theater accent。

### Zone D · 舞台脚注（Stage Footer）
- **内容**：极简——模型来源指示（`本地千问` / `云端` / `预生成`）、可选节拍计数。**无**好感分、无场景标签、无插件状态。
- **对应 ToolShell**：`ToolStatusBar` 瘦身版（新 `TheaterFooter` 或复用 `ToolStatusBar` 传精简 props）。
- **token**：`--tool-statusbar-h` / `--tool-chrome-status`。

---

## 4. 叠层 IA（Overlays · default 关）

| 叠层 | 触发 | 内容 | 落点 |
|------|------|------|------|
| **创作漏斗入口** | 戳 🎭「微调性格」并看到效果的**峰值瞬间** | 「✨ 喜欢捏它的性格？从头造一个属于你的灵魂 →」 | 跳 `oclive-pack-editor`（复用，不重造） |
| **设置** | Zone A「更多」二级 | 语言/外观/**卡司（双槽导入+应用）**/舞台/模型 key（BYOK 在此，非首屏） | `TheaterSettingsSheet`（通用 · 舞台 · **卡司** · 模型 Tab） |
| **重开/换剧目**（可选） | 「更多」或脚注 | 重置当前剧目到开场 | 轻量 action |

**漏斗纪律**（路线图 §4.6）：邀请**嵌在高潮**（戳性格成功的那一秒），**不**埋在角落菜单；先「改」（remix）后「造」（create）。

---

## 5. 状态机（首屏体验流 · 与「陌生人前 3 分钟」对齐）

```
[启动] → 预生成开场加载（极短/无感）
   ↓
[STAGE.playing] 开场对戏自动播放（0:00 即活，零配置）
   ↓ 播完
[STAGE.idle] Dock 高亮可戳         ← 「这里能戳」一眼可见
   ↓ 用户点芯片
[STAGE.patching] BeatLoader 节拍（本地千问只改一小段）
   ↓ 补丁回锚到下一固定节拍
[STAGE.playing] 插入/改写段播放    ← 小输入·大后果（第二声「卧槽」）
   ↓ 反复戳（含 🎭 性格）
[FUNNEL] 戳性格峰值 → 创作漏斗浮出  ← 「我也想造一个」
   ↓
[离场] → oclive-pack-editor
```

**失败/降级**（鲁棒性，复用现有 degraded 习惯）：
- 本地模型不可用 → 退到**预生成备选补丁**（仍出字，不空转）；脚注提示来源。
- 补丁接不上下一节拍 → 重生成（路线图 §4.4），对用户无感。

---

## 6. 组件清单（新建 vs 复用）

| 层 | 组件 | 新建/复用 | 备注 |
|----|------|-----------|------|
| 壳 | `TheaterShell.vue` | 新建 | 仿 `ToolShell` 骨架；`inject(MAIN_SHELL_KEY)` 或新 key |
| 编排 | `useTheaterShell.ts` | 新建（瘦） | 复用 store/api；**不**接沉浸/插件/场景旅行 |
| 中心 | `TheaterStage.vue` / `TheaterScriptLine.vue` | 新建 | 剧本流；滚动复用 `VirtualScrollContainer` |
| 交互 | `PokeDock.vue` / `PokeChip.vue` / `BeatLoader.vue` | 新建 | 替代 `ChatInput` |
| 头/脚 | `TheaterHeader.vue` / `TheaterFooter.vue` | 新建（薄） | 或复用 `ToolStatusBar` 精简 |
| 漏斗 | 复用 `IdentitySurpriseSheet` 模式 → `CreationFunnelSheet.vue` | 新建（仿现有 sheet） | 参考 onboarding sheet 范式 |
| 设置 | `SettingsView`（精简 Tab） | 复用 | 藏「更多」 |
| 主题 | `theme-theater.css` | 新建 | 以 `theme-tool.css` 为 base 覆写 accent/双卡司色 |
| 选择 | `useOcliveShell.ts` 扩 `'theater'` | 改 | `App.vue` 加分支 |

**新建组件数 ≈ 8–10 个薄组件**，无新增内核编排（§5.5）。

---

## 7. 数据与内核契约（复用，零新 stage）

- **对戏生成**：模式 1 = **预生成骨架（强模型·离线一次性）+ 本地千问局部补丁**。前端通过**现有 `send_message` / `process_message` 主链**驱动单角色回合；双角色对戏由**前端编排两次单角色调用 + 骨架拼接**实现，**不**在内核加「双角色 stage」（守 §5.5 冻结）。
- **角色包**：两个反差角色为标准 v2 角色包（落 `roles/`，开工清单），经 `theater.oclive.toml` profile 加载。
- **预生成骨架**：随发行版打包的静态资产（如 `resources/theater/*.json`），由作者用强模型一次性生成（路线图 §2 成本模型）。
- **零 key 首屏**：预生成 + 本地模型扛住；BYOK 藏设置（路线图 §4.5）。

---

## 8. 响应式与可访问性（继承 Chat Pro 习惯）

- **断点**：复用 `wideSplitLayout`（>720）思路；窄屏 Dock 横向滚动芯片。
- **a11y**：舞台 `aria-live="polite"` 播报新台词；芯片为真 `<button>` + `aria-label`；节拍 loader `role="status"`。
- **焦点**：复用 `useReturnFocusOnClose` 于漏斗/设置叠层。

---

## 9. 开发顺序建议（IA → 实现，落路线图 §7 开工清单）

1. **主题与壳骨架**：`theme-theater.css` + `useOcliveShell` 加 `'theater'` + `TheaterShell` 空骨架（头/主/脚三区）。
2. **舞台画布静态版**：`TheaterStage` 渲染**写死的**早餐开场（先验证「开场即活」观感）。
3. **戳 Dock + 预生成补丁**：芯片 → 切预生成的几个分叉（**先不接本地模型**，用罐头验证「小输入大后果」）。
4. **本地千问局部补丁**：替换罐头为真本地生成 + BeatLoader + 重锚。
5. **创作漏斗**：🎭 峰值 → sheet → 跳编辑器。
6. **粗剪给 3 个陌生人看**（路线图 §6 纪律 3）——**先验证「卧槽」，再优化引擎**。

> 纪律回扣：步骤 2–3 是「**先用手做粗剪**」，步骤 4 才是「**造引擎**」。不要颠倒（路线图 §6）。

---

## 10. 一句话收束

**复用 Chat Pro 的设计语言与内核，把「聊天列表」翻译成「舞台」、把「输入框」翻译成「戳一戳」。** 一座早餐舞台、两个反差灵魂、五个戳点、一道创作暗门——这就是模式 1 的全部 IA，其余等那一声「卧槽」。

---

# 附录：实现级设计细节（给 auto 直接照做）

> 以下基于对 Chat Pro 真实源码的抠取（`theme-tool.css` / `ToolShell.vue` / `ToolActivityBar.vue` / `ToolStatusBar.vue` / `ChatInput.vue` / `ChatMessage.vue` / `IdentitySurpriseSheet.vue`）。命名、token、prop/emit 范式**与现有代码一致**，降低 auto 的决策与试错。

## A. 文件清单（精确路径）

```
src/
├─ shells/theater/
│  ├─ TheaterShell.vue            # 壳骨架（仿 shells/tool/ToolShell.vue）
│  ├─ TheaterHeader.vue           # Zone A（仿 tool-top-bar，只读卡司 + 更多）
│  ├─ TheaterStage.vue            # Zone B 舞台画布（剧本流容器）
│  ├─ TheaterScriptLine.vue       # 单条台词（仿 ChatMessage.vue 范式）
│  ├─ PokeDock.vue                # Zone C 戳一戳 Dock（替代 ChatInput）
│  ├─ PokeChip.vue                # 单芯片
│  ├─ BeatLoader.vue             # patching 节拍 loader（role="status"）
│  ├─ TheaterFooter.vue           # Zone D（仿 ToolStatusBar 精简）
│  └─ CreationFunnelSheet.vue     # 漏斗 sheet（仿 IdentitySurpriseSheet.vue）
├─ composables/
│  ├─ useTheaterShell.ts          # 编排（仿 useMainShell.ts，瘦版）
│  └─ useOcliveShell.ts           # 改：OcliveShellKind 加 'theater'
├─ styles/
│  └─ theme-theater.css           # 主题（以 theme-tool.css 为 base）
└─ App.vue                        # 改：加 TheaterShell 分支
```

资产 / profile（开工清单，路线图 §7）：
```
examples/distro-profiles/theater.oclive.toml        # 复制自 handoff/theater/theater.oclive.toml
src-tauri/resources/distro-profiles/theater.oclive.toml
src-tauri/resources/theater/scenes/*.skeleton.json   # 四场景预生成骨架（强模型离线产出）
public/theater/scenes/*.skeleton.json                # Vite dev 镜像
src/composables/theater/theaterSceneCatalog.ts       # 场景目录 SSOT（preset id / pokeEnabled / prompt hints）
roles/theater-breakfast-a/  roles/theater-breakfast-b/  # 两个反差角色 v2 包
```

## B. 壳选择与挂载（改 2 个文件）

**`useOcliveShell.ts`**：
```ts
export type OcliveShellKind = 'tool' | 'fluent' | 'theater'
// resolve: VITE_OCLIVE_SHELL === 'theater' → 'theater'（fluent 同理；default 'tool'）
```

**`App.vue`**：加 `const TheaterShell = defineAsyncComponent(() => import('./shells/theater/TheaterShell.vue'))`；模板加 `<TheaterShell v-else-if="shellKind === 'theater'" />`。`data-shell` 已自动写入（现有 `onMounted` 逻辑）。

> 复用现有 `MAIN_SHELL_KEY` provider：`App.vue` 已 `provide(MAIN_SHELL_KEY, useMainShell())`。**决策**：模式 1 剧场仍可 `inject(MAIN_SHELL_KEY)` 复用底层（store/api/onSend），但**只取所需子集**；舞台专属状态（剧本流/补丁/节拍）放 `useTheaterShell()` 独立组合，避免污染 `useMainShell`。

## C. 主题 token（`theme-theater.css`）

以 `theme-tool.css` 为**结构母版**，`:root[data-shell="theater"]` 复制 tool 的 `--tool-*` 全量映射，仅**覆写**以下让舞台更暖/俏皮（保持扁平、token 化）：

```css
:root[data-shell="theater"] {
  /* 继承 tool 的间距/字号/圆角/语义别名映射（整段复制 theme-tool.css 的 :root[data-shell="tool"] 主体） */

  /* —— 剧场覆写 —— */
  --tool-accent: #e08a3c;                 /* 暖橙，区别 Chat Pro 冷蓝 */
  --tool-radius: 8px;                      /* 更圆润、更亲和 */
  --tool-radius-lg: 12px;

  /* 双卡司嗓音色（剧场新增） */
  --theater-cast-a: #d96b6b;              /* 角色 A 嗓音 */
  --theater-cast-b: #4f86c6;              /* 角色 B 嗓音 */
  --theater-cast-a-soft: color-mix(in srgb, var(--theater-cast-a) 12%, transparent);
  --theater-cast-b-soft: color-mix(in srgb, var(--theater-cast-b) 12%, transparent);

  /* 舞台底色（比 chat editor 略带氛围） */
  --theater-stage-bg: color-mix(in srgb, var(--tool-elevated) 96%, var(--tool-accent) 4%);
}
:root[data-shell="theater"][data-theme="dark"] {
  /* 复制 tool dark 主体；覆写 --tool-accent / --theater-cast-* 暗色版 */
}
```

> **纪律**：不要新发明一套 `--theater-*` 间距/字号体系——**复用 `--tool-space-*` / `--tool-fs-*`**，只加「accent + 双卡司色 + 舞台底色」三类语义增量。

## D. 组件契约（prop / emit / class · 照搬现有范式）

### `TheaterShell.vue`（仿 `ToolShell.vue` 骨架）
- 布局类沿用：`theater-layout > theater-frame > theater-body__main`（结构对照 `tool-layout`）。
- 纵向：`TheaterHeader` → `TheaterStage`(flex:1, 滚动) → `PokeDock`（flex-shrink:0, 仿 `tool-input-area` 顶边框）→ `TheaterFooter`。
- 叠层：`CreationFunnelSheet`（条件渲染）、`Toast`（复用现有 `components/Toast.vue`）。
- **首屏不挂**：ActivityBar、SidePanel、插件/场景/debug 任何块。

### `TheaterStage.vue` / `TheaterScriptLine.vue`
- `TheaterStage` props：`lines: ScriptLine[]`、`state: 'playing'|'idle'|'patching'`。
- 滚动复用 `components/chat/VirtualScrollContainer.vue`（或简单 `overflow:auto`，模式 1 行数少可不虚拟化）。
- `aria-live="polite"` 包裹新行播报。
- `TheaterScriptLine` props：`{ cast: 'a'|'b', name: string, text: string, stageHint?: string }`。
- class：`script-line script-line--a|--b`；用 `--theater-cast-a/b` 做色条/名字色（**不是**用户/助手左右气泡，而是**剧本对白**：角色名加粗 + 台词 + 可选 `<span class="stage-hint">(动作)</span>`）。
- 入场动画复用 `ChatMessage` 的 `bubbleIn` 思路 + `@media (prefers-reduced-motion: reduce)` 关。

### `PokeDock.vue` / `PokeChip.vue`
- `PokeDock` props：`{ chips: PokeChipDef[], disabled: boolean }`；emit：`poke: [chipId: string]`。
- `PokeChipDef`：`{ id: 'tea'|'late'|'veggie'|'nickname'|'personality', emoji: string, labelKey: string, weight: 'high'|'neutral' }`（模式 1 只放 high）。
- `PokeChip` 是真 `<button>`（仿 ChatInput `.send` 的 token 化样式：`--radius-btn`/`--accent`/`:focus-visible` ring），`:disabled` 在 `patching` 期。
- 🎭`personality` 芯片 emit 后由 `useTheaterShell` 在补丁成功峰值置 `funnelVisible=true`。
- 容器：`poke-dock`（仿 `tool-input-area` 顶边框 + padding `--tool-space-4`）；窄屏 `overflow-x:auto` 横滚芯片。

### `BeatLoader.vue`
- props：`{ visible: boolean }`；`role="status"` + `aria-label`（i18n）。
- 视觉：三点/呼吸动画（呼应「思考节拍」），`prefers-reduced-motion` 降级为静态文案。

### `TheaterFooter.vue`（仿 `ToolStatusBar.vue` 精简）
- props：`{ source: 'pregen'|'local'|'cloud', beat?: number }`。
- class：`theater-footer`（仿 `tool-status-bar` token：`--tool-statusbar-h`/`--tool-chrome-status`/`--tool-fs-sm`）。
- **无**好感/场景/identity 段（对照 ToolStatusBar 的 `interactionImmersive` 段一律不要）。

### `CreationFunnelSheet.vue`（仿 `IdentitySurpriseSheet.vue`）
- props：`{ visible: boolean }`；emit：`create: []`、`dismiss: []`。
- 结构照搬 sheet：`role="dialog" aria-live="polite"` + 标题 + `UiButton`（复用 `components/ui/UiButton.vue`，`variant="secondary"/"ghost"`）。
- 文案 i18n `theater.funnel.title` / `theater.funnel.create` / `theater.funnel.keep`。
- `create` → 跳 `oclive-pack-editor`（复用现有打开编辑器路径；attach 模式经 host 事件）。

## E. 数据契约（前端编排，零内核新 stage）

```ts
interface ScriptLine { id: string; cast: 'a' | 'b'; name: string; text: string; stageHint?: string }
interface Skeleton {                 // 预生成骨架（静态资产）
  scene: string                      // 'breakfast'
  cast: { a: { roleId: string; name: string }, b: { roleId: string; name: string } }
  beats: ScriptLine[]                // 固定节拍序列（开场对戏）
  forks: Record<string, {            // 戳点 → 该点的预生成备选补丁（罐头，步骤 3 用）
    chipId: string; insertAfterBeatId: string; patchLines: ScriptLine[]
  }[]>
}
```

- **步骤 3（罐头）**：戳 → 从 `skeleton.forks[chipId]` 取一条 `patchLines` 插入 `insertAfterBeatId` 之后 → 续播固定节拍。
- **步骤 4（本地）**：把罐头替换为「**本地千问只改一小段**」——以紧贴上下文 + 严格格式调 `send_message`（单角色），输出受限补丁；失败/超时 → 回退罐头 fork（鲁棒）。
- **双角色对戏**：前端按 `beats` 顺序播；需新内容时**分别**以 cast a / cast b 角色身份调用单角色主链，**不**在内核加双角色编排（守 §5.5）。

## F. i18n 命名空间（新增，中英镜像）

```
theater.header.cast            "{a} ✕ {b}"
theater.header.scene.breakfast "早餐 · 上学前"
theater.poke.tea / late / veggie / nickname / personality   芯片文案
theater.beat.loading           节拍 loader aria
theater.footer.source.pregen / local / cloud
theater.funnel.title / create / keep
```
落 `src/i18n/locales/fragments/`（仿现有 fragment 结构）；`verify:shared-i18n` 与 parity 测需同步中英。

## G. 验收锚点（按 IA §9 步骤）

| 步骤 | 自测 |
|------|------|
| 1 壳骨架 | `VITE_OCLIVE_SHELL=theater npm run dev` 出三区空骨架，`html[data-shell="theater"]` 生效，暖橙 accent |
| 2 静态舞台 | 写死早餐开场逐行播放，双卡司色区分，`aria-live` 播报 |
| 3 罐头戳点 | 点芯片 → BeatLoader → 插入 fork 段 → 续播；「小输入大后果」成立 |
| 4 本地补丁 | 千问只改一小段；断网/超时回退罐头不空转 |
| 5 漏斗 | 🎭 峰值 → sheet → 跳编辑器 |
| 6 粗剪 | 给 3 个陌生人看（路线图 §6）；先验「卧槽」再优化 |

## H. 复用清单（别重造）

| 直接复用 | 来源 |
|----------|------|
| `Toast.vue` / `UiButton.vue` / `VirtualScrollContainer.vue` | `src/components/` |
| store / api / `send_message` 主链 | `src/stores/`、`src/api/` |
| 主题机制 `data-shell` + token 别名映射 | `theme-tool.css` 范式 |
| sheet / a11y / focus 范式 | `IdentitySurpriseSheet.vue`、`useReturnFocusOnClose` |
| 设置叠层 | `SettingsView`（藏「更多」，精简 Tab） |
| 编辑器 | 姊妹仓 `oclive-pack-editor`（漏斗目标） |
