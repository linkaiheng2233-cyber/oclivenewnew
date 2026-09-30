# OClive 多轮优化巡检手册（Recurring Optimization Playbook）

> **定位**：一份**可反复运行**的地基巡检流程。不是一次性审查报告，而是每隔一段时间 / 关键节点照着跑一遍的"体检套餐"。
> **SSOT 范围**：本文只定义巡检的触发、分档、取证和记录方式；AI 可改范围、开发阶段、数字核实、模型委派和技术债状态分别以各自 SSOT 为准。
>
> **核心信条**：保证地基稳固才能走得远。但**地基是为了承载"惊喜"（官方剧场 demo / 发行版），不是为了自身完美**——见文末「§9 元纪律」。
>
> **创建**：2026-06-09 · **最后更新**：2026-10-01（文档声明支撑调查入口） · **维护者**：项目维护者 · **状态**：活跃手册（§8 仅保留最近五轮，完整历史见 Git）

---

## ★ 仓库物理布局（kernel / distros · 2026-06 重组）

> **巡检前 10 秒**：先查逻辑权责，再查物理路径。`kernel/` 同时包含契约与参考 Host，不等于目录内所有内容都属于小 Kernel。

| 层 | 目录 | 改什么 |
|----|------|--------|
| **内核与参考运行时** | [`kernel/`](../kernel/)（`kernel/crates/`、`kernel/fuzz/`、`examples/oocp-test-suite/` 等） | `process_message`、六槽端口与实现、迁移、OOCP、HTTP API |
| **共享桌面 UI** | [`distros/shared/`](../distros/shared/) | API 封装、stores、通用 chat 组件 |
| **Chat Pro** | [`distros/chat-pro/`](../distros/chat-pro/) | ToolShell / FluentShell、roles、plugins |
| **AI Theater** | [`distros/theater/`](../distros/theater/) | TheaterShell、剧场 composables |
| **Tauri 宿主** | [`distros/desktop-tauri/`](../distros/desktop-tauri/) | invoke 薄壳、打包资源、bundled kernel |
| **契约文档** | 根 `creator-docs/`、`handoff/` | 工具内核与参考运行时 SSOT；发行版索引见 [`handoff/distros/README.md`](distros/README.md) |

**禁止**：在 `kernel/` PR 里改 Chat Pro 壳样式；在 `distros/chat-pro/` PR 里改 `process_message` 编排。RFC：[`handoff/distros/ARCHITECTURE_DECOUPLING_RFC.md`](distros/ARCHITECTURE_DECOUPLING_RFC.md)。供应链：[`creator-docs/security/SUPPLY_CHAIN.md`](../creator-docs/security/SUPPLY_CHAIN.md) · `node scripts/dimension5-acceptance.mjs --ci`（检查项总数以脚本输出为准）。

---

## ★ OClive 定位与愿景（新对话对齐前置区 · 先读这一节）

> **目的**：让任何新接入的 AI / 协作者在 60 秒内对齐“OClive 到底是什么、哪些是本体、哪些是装配、为什么需要工程纪律”。巡检取舍以定位 SSOT 和源码为准。

### 一句话定位

**OCLive 以六槽契约与必要合法性边界组织能力，具体运行由发行版 Host 决定。** 巡检先读 [MODULE_MAP 权责与源码对照](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)，不要把此处定位摘要当另一份架构契约。

三层边界：

- **最小工具内核**：六槽能力契约与必要公共合法性/因果/错误边界；无具体调度或领域提交权。
- **完整参考运行时**：当前 `OcliveKernel` 门面 + 具体槽实现、SQLite、Event Ring、设施和 HTTP 依赖。
- **发行版 Host 与工具**：Host 负责组合、调度和领域应用；角色包、编写器、CLI、市场和云服务按场景选择。

“Linux”“Cursor”“机甲”可以作为演示比喻，但不能替代工程定义，也不能据此把所有陪伴功能归入最小内核。

### 角色连续性与工具行动是两类能力

| | 传统 Agent / AI 聊天软件 | **OClive** |
|---|---|---|
| 关心 | 模型**做什么**（扩展能力、调工具、完成任务） | 角色状态怎样连续、怎样被多个消费者按同一契约使用 |
| 主要端口 | `agent` 与工具适配 | `memory` / `emotion` / `event` / `prompt` / `llm` 与状态契约 |
| 关系 | 可以独立使用 | 可以独立使用；两者也可在同一装配中协作 |

> 措辞纪律：不要宣称内核“比模型更懂人”或把显式状态写成唯一真相。更准确的说法是：OCLive 用契约组织模型、记忆、事件、工具和宿主之间的协作，让角色状态可替换、可观察、可回退。

### 显式与隐式都不是正统答案

默认参考实现偏向本地小模型，使用较多显式记忆、情绪、事件和 Prompt 辅助；强模型装配可以更薄，让模型从语境中完成更多语义判断。巡检采用：**事实显式化，判断候选化，表达模型化。** 当前 `EmotionResult` 尚未统一实现 source/confidence/TTL/scope，因此检查文档时必须区分目标原则与已交付 DTO。

### 与传统 AI 聊天软件的四条根本差异（= 愿景四主轴 V1–V4,见 §3）

1. **【V1 可嵌入性】一颗内核可进入不同宿主。** 不锁死在桌面/Tauri/聊天框；但“理论可嵌入”与“某硬件已产品验证”必须分开写。
2. **【V2 可替换性】六个稳定端口可换实现。** 精确 backend 支持按槽位查真实性矩阵，不能概括成每槽都有同样四态；普通扩展不增加第七槽，当前参考 Host 不复制第二条权威回合链；这不禁止第三方 Host 按契约采用其他调度。
3. **【V3 可携带性】角色资产可跨兼容宿主迁移。** 角色包与长期状态的可携带能力按分层契约验证；UI、语音、插件能力仍受宿主 profile 限制。
4. **【V4 可学习与可创造】降低理解和创作门槛。** 角色创作者不必先学六槽；模块作者能只实现一个端口；维护者仍须用测试和迁移守住边界。

### 商业与治理定位

- **维护职责**：守内核契约、参考实现与兼容证据；官方发行版用于验证真实场景，不定义所有下游。
- **下游自由**：可以基于内核做发行版、商用工具或研究装配；无需接受官方市场或默认认知模块。
- **许可证**:**Apache-2.0**(permissive + 专利授权),刻意允许闭源商业下游自由生长——服务"做地基"的战略。

### 护城河与威胁（战略清醒区）

- **代码和口号都不是护城河**：更可靠的长期价值来自稳定契约、真实兼容证据、可复现评测、文档质量，以及使用者能够拿走和改造工具的事实。生态是可能结果，不是预设核心。
- **主要风险**：强模型进步可能削弱厚辅助模块的收益，低成本竞品也能复刻表面结构；因此内核必须允许装配变薄，并用契约、评测和真实兼容性证明价值。
- **应对**：先守住可验证的工具价值与反锁定边界，再让社区和生态从真实使用中生长。

### 当前真正的瓶颈（决定巡检该克制的根本理由）

**历史巡检曾把技术面评为 A−，但评级必须以 §8 最新一轮证据为准，不能把旧结论当永久事实。** 当前产品侧的关键瓶颈仍包括“还没有足够多陌生人亲眼见过它发光”，工程巡检只负责守住承载这一体验的地基。

- 当前"惊喜"= 官方**剧场 demo**(两个反差角色吃早饭准备上学,用户戳一下微改剧情——喝苦中药/快迟到/换称呼/改性格——看角色做出符合人设的有趣反应;强模型一次性预生成骨架,本地小模型只改用户动的那一小段,"弹改动加载"遮延迟)。目标:让陌生人 60 秒内脱口而出"卧槽"。
- 因此 **§9 元纪律** 是硬约束:**凡不直接服务于"让它发光"的优化,默认 Deferred 只记录不动手。** 这份巡检手册存在的意义是**防地基回退,不是追内核完美**。

### 项目分量（一句话量级）

Rust workspace 成员、源码、迁移、测试和文档规模均以 `cargo metadata --no-deps --format-version 1` 与 `node scripts/project-scale.mjs` 的实时输出为准；门禁与质量结论必须引用本轮命令输出，不在本手册硬编码会漂移的数量或评级。规模本身不证明质量，评价只落在可复现证据上。

---

## 0. 如何使用本手册

1. 先记受检完整 HEAD、工作树状态、巡检档位与取证副作用；读 [`AGENTS.md`](../AGENTS.md)、[AI 改动边界](AI_CHANGE_BOUNDARIES.md)和[数字核实协议](AI_VERIFICATION_PROTOCOL.md)。若巡检转成实施任务，再按 [OCLive 开发流水线](../.cursor/skills/oclive-dev-pipeline/SKILL.md)独立定 S/M/L 和 applicable 门禁；本机通用流水线文件缺失时按该 Skill 的缺失口径处理，不臆造阶段。模型分工只在实际选择委派时按[自适应流水线](workflows/oclive-adaptive-pipeline/SKILL.md)执行，**只读巡检不自动启动 Agent**。
2. 不要每次都全跑。按 **§1 触发条件** 决定本轮跑快、半或全档；三个巡检档位**不是**开发任务的 S/M/L 尺寸。
3. 每轮巡检从 **§2 基线门禁** 开始。首次 FAIL 先停止后续维度，记录原始失败并判明环境前提、检查器或产品根因；若是已支持的环境配置缺失，可修正配置后重跑基线，**同时保留首次 FAIL**。确定性代码/契约失败不得靠改低阈值或重试洗绿。
4. 按所选档位走对应维度，完整顺序是：**基线 → 一架构 → 二性能 → 三设计 → 四技术债 → 六文档 → 七条理与边界**。
5. 每个维度用**两把尺子**：① 传统正确性（能跑/对不对）；② **愿景对齐**（V1–V4，见 §3）。
6. 全档收尾才按 **§7** 给综合评分；快档、半档报告实测项、未测项和范围，**不沿用旧 A−**。按 §8 记录本轮；新债须先满足[核实协议](AI_VERIFICATION_PROTOCOL.md) L3，再进入 `TECHNICAL_DEBT_INVENTORY.md`，不能把观察直接写成 P0/P1 或 OPEN。
7. **凡输出带数字的审查/汇报**（含 AI 生成的质量报告），遵守 [`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md)；第三方结论默认「待核实」直至本轮命令和原始证据复核。

---

## 1. 触发条件与档位

| 档位 | 何时跑 | 范围 |
|------|--------|------|
| **快档（Smoke,~15min）** | 每次合并涉及 `process_message` / `plugin_host` / `host_profile` / CI 配置 / 迁移 / **路径或目录结构** 的改动后 | §2 基线 + **维度七第 1–2 项（路径漂移 + AI 入口时效）** + 受影响维度的对应 checklist |
| **半档（~1h）** | 每完成一个发行版里程碑、或每 2–4 周 | §2 基线 + 维度一 + 维度二 |
| **全档（半天）** | 每个 minor 版本发布前、或重大架构变更后 | 全部六维 + §7 评分 + §8 记录 |

> 经验法则：**默认走快档**。全档稀缺、刻意,别让巡检变成日常逃避区（见 §9）。

本表只决定**质量调查范围**。修 bug、改文档或偿还技术债时，按[第一条工程流水线](../.cursor/skills/oclive-dev-pipeline/SKILL.md)选任务尺寸和变更面门禁；跨层能力改动还须按[AI 改动边界](./AI_CHANGE_BOUNDARIES.md) G17 核对生产者、契约、适配/权限、消费者、状态/回退与测试。“本轮没有跑全档”不能替代 applicable 测试，巡检全档也不能自动把技术债升为 Done。真实模型、语音、硬件或生产数据检查须先明确隔离环境、预算、身份和副作用，不因选择“全档”而默认启动。

---

## 2. 基线门禁（巡检每轮必跑；首次 FAIL 先停查根因）

先核对工具与环境：受检 SHA、dirty/untracked、`rg` 及实际被门禁调用的 Python 解释器。Windows 的 TTS ratchet 默认调用 `py -3`；若本机只有 Python 3.10+ 而没有 `py`，用脚本支持的 `OCLIVE_VOICE_PYTHON` 指向**实际解释器可执行文件**，记录该配置，不把缺启动器误报为 TTS 产品失败。PowerShell 下逐条跑（**不要用 `&&`**）：

```powershell
node scripts/dimension5-acceptance.mjs --ci   # 必须 PASS；项数以脚本结尾输出为准
cargo test -p oclive_kernel_host --lib         # 必须全绿（注意：--lib 不含 doctest）
node scripts/check-domain-layering.mjs         # ratchet 数值不得上涨
git status                                      # 确认工作树状态 / 与 origin 差距
```

本表是**巡检基线**，不是每个代码/文档提交都必须照跑的开发门禁；开发验收按 diff 选 applicable 项。离线、锁文件或本机内存约束需要附加 `--locked`、`--offline`、`-j 1` 时，报告须列实际命令和覆盖限制，不把调整后的本地结果冒充 CI 原命令。任一项失败先保全退出码与原文，按 §0 区分环境、检查器和产品；修正环境后须重新通过本轮基线才继续巡检。

> **⚠️ doctest 盲区**：上面的 `--lib` 与日常 `npm run check:rust`（`cargo test --workspace --lib`）**都不跑 doctest**，但 CI `rust` job 跑 `cargo test --workspace`（**含 doctest**）。**本轮若改了公开 DTO 字段 / trait 签名 / crate 名 / re-export，必须补 `cargo test --workspace --doc`**，否则会出现「本地全绿 / 远程 CI 硬门禁红」（见 [`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md) §2.1）。

**判定**：门禁（择要）含 layering ratchet / **cargo audit** / **cargo deny（licenses+bans）** / lockfile（禁 sqlx-mysql·rsa 回潮）/ ensure-plan 快照 / CHANGELOG 中英 parity / Markdown 本地链接 / stale 路径 ratchet（doc + code）/ host re-export ratchet / theater prompt drift / **verify:ui** / **vite build** / **tauri beforeBuildCommand 路径 ratchet**。任一 FAIL → **本轮停止所有优化,先恢复基线**。

> **dimension5 检数 SSOT**：以 `dimension5-acceptance.mjs --ci` 脚本结尾输出的 **`PASS (N checks)`** 为准；文档中的「N 检」须与此对齐，勿另造数字。

**ratchet 锚点**（只降不升）：`domain→infrastructure` 的 use-import / FQ 阈值以 [`LAYERING_BASELINE.json`](./LAYERING_BASELINE.json) 和 `node scripts/check-domain-layering.mjs` 输出为准；host/runtime re-export import 阈值以 [`HOST_REEXPORT_BASELINE.json`](./HOST_REEXPORT_BASELINE.json) 和 `node scripts/check-host-reexport-imports.mjs` 输出为准。不要把某一轮的数字抄成手册里的长期阈值。

---

## 3. 愿景对齐四主轴（贯穿每个维度的第二把尺子）

> 这是本手册区别于通用「五维审查」的关键。每条发现都要标注它影响哪条主轴。

| 代号 | 愿景主轴 | 每轮拷问 |
|------|----------|----------|
| **V1 可嵌入性** | 内核不被某个 UI / 传输 / 硬件钉死 | 这是代码解耦、编译证明，还是已经有真机与资源预算证据？ |
| **V2 可替换性** | 六个稳定端口可换实现，外围能力不污染核心分类 | 这个 backend 是真跑通还是枚举占位？新能力是否误造“第七槽”？ |
| **V3 可携带性** | 角色资产可跨兼容宿主迁移 | 哪一层可携带，哪一层受 HostProfile 限制？是已验证还是仅设计？ |
| **V4 可学习与可创造** | 不同读者只需进入自己所需的层 | 是否降低创作者/模块作者的实际门槛，同时保留维护者所需证据？ |

---

## 4. 维度执行清单（按顺序）

### 维度一 · 架构全景 + 模块边界 ★每轮重点

**正确性 checklist**
- [ ] 核心 crate 依赖图仍严格单向（`types→contracts→runtime→host→{server,tauri}`）；CI planner、scaffold、validation、fuzz 等旁路成员按各自边界另查
- [ ] `domain→infrastructure` 反向依赖无新增（对照 `LAYERING_BASELINE.json`）
- [ ] `process_message`（`kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`）在当前参考 Host 内仍是唯一主编排入口，业务逻辑未泄漏到 `distros/desktop-tauri/src/api/*`
- [ ] 冻结项（dual_core / blueprint v3 / expert_routing）仍 feature-gated 默认不编译

**愿景拷问**
- [ ] 【V1】完整 `OcliveKernel` 已可由 library 进程内嵌入；最小内核物理抽离（**K-CORE-BOUNDARY-01**）进度——本轮裁决：阻塞 / 非阻塞？（§3.1）
- [ ] 【V2】逐槽核对 [`SLOT_BACKEND_REALITY_MATRIX.md`](./SLOT_BACKEND_REALITY_MATRIX.md)：backend 集合按槽位分别定义，不再假设固定“6×4”矩阵；每项标 ✅真跑通 / ⚠️占位 / ❌未实现
- [ ] 【V3】角色包 + 记忆跨宿主携带契约（`CROSS_HOST_MEMORY.md`）是设计还是已验证？
- [ ] 【V4】创作者表面（`meta` 子集）与内核面（slot_registry/蓝图）是否**架构性**隔离,而非仅文档约定

**方法**：`cargo metadata` 看依赖；逐槽填真实性矩阵；确认 `oclive_kernel_server` 不依赖 tauri。
**产物**：架构全景图 + 边界违规清单 + **槽态真实性矩阵**（本维度最高价值）。

---

### 维度二 · 性能热点与瓶颈

**正确性 checklist**
- [ ] `oclive_turn` target stage tracing 采样,确认 K-PERF-01~12 未回退
- [ ] SQLite `EXPLAIN QUERY PLAN` 抽查：`long_term_memory` 检索、`personality_vector` 索引（migration 033）
- [ ] SessionCache（DashMap）cap+TTL 有效、无泄漏
- [ ] 冷启动延迟（spawn → `/health` 就绪）分布

**愿景拷问**
- [ ] 【V1·低算力】无独显笔记本 / ARM SBC 上的内存 + CPU 足迹？有无"最小内核"裁剪路径？
- [ ] 【剧场实时】本地小模型局部补丁 + 弹加载 端到端延迟预算,瓶颈段（推理 / prompt 构建 / DB 写）？
- [ ] 【V3】两发行版同时活跃时共享 `app.db` 的锁竞争

**方法**：`cargo build --timings`、关键路径 tracing span 实测 `elapsed_ms`、`cargo-bloat` / 二进制体积（对照 `LIGHTWEIGHT_PROFILE.md`）、低配实跑计时。
**产物**：按优先级发现清单,**每条带量化数据**；嵌入式 + 剧场实时两场景专项结论。

---

### 维度三 · 设计优雅度与重复抽象

**正确性 checklist**
- [ ] D-PORT-02 已 Done 的拆窄防回退：`PluginBackendRegistryPort` 仍是由 `SlotBackendFactoryPort`、`LocalPluginRegistryPort`、`AgentMcpRegistryPort` 组成的组合端口，内存槽经 `MemoryBackendPort` 独立；只有出现新调用面证据时再评估 `D-PORT-03` 的转发层。
- [ ] D-SLOT-01 防回退：`builtin_v2` 仍仅为读兼容 alias，不得重新长出独立 V2 / Placeholder 实现
- [ ] 错误模型一致性（`AppError / TurnError / ProcessMessageError`）
- [ ] 按受检 HEAD 统计单实现 contracts trait：逐项判断保留为 DI 端口或降级具体类型，不沿用历史估数。
- [ ] `resolve_*` 命名混淆度 + rustdoc 覆盖率
- [ ] **认知负担 / 冗余抽查**（人类开发者友好度）：多分支手写同一大 struct（如 builder 函数群、测试字面量）是否可 `#[derive(Default)]` + `..Default::default()` + 共享 base 收敛；复制粘贴块、未用 import、自己引入的死代码——**行为等价前提下顺手清**（见 [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G9），**勿为清而清**触发无关大重构（§9）

**愿景拷问**
- [ ] 【V2】组合端口与槽实现是否仍保持可替换性？若怀疑转发层回潮，先对照 D-PORT-02 的 Done 边界及 D-PORT-03 的触发条件。
- [ ] 【V4】第三方模块作者面对的接口面是否清爽（决定"别人写得好我直接抄"能否转起来）？

**方法**：按受检范围统计 contracts trait 实现数，列单实现 trait 表逐个标注处置；对照组合端口的实际调用面与暴露面。
**产物**：发现清单 + 单实现 trait 处置表；D-PORT-02 / D-SLOT-01 只做防回退核验，不因巡检自动重开 Done 或重裁优先级。

---

### 维度四 · 技术债务清单更新

若本轮改为 **Document → Code 支撑调查**，先按[核实协议 §2.9](./AI_VERIFICATION_PROTOCOL.md#doc-code-support-audit)冻结声明清单、分层预算和停止线；这是只读建图，不等于启动全档巡检或在调查中直接偿债。新缺口先分类，达到入账门槛后再确定 owner、优先级与独立工程切片。

**checklist**
- [ ] 按本轮受检范围，对照 [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) 核对已有债务的状态、证据和实际代码；未覆盖的条目明记“未复核”，不声称全账重审。
- [ ] 已 Deferred 项：有新的触发条件或愿景影响证据吗？没有则保持原状态。
- [ ] 用 `rg` 扫描受检范围的 TODO/FIXME，逐项区分可执行欠账、模板文字和历史备注；不预设只有某个已知文件。
- [ ] 新发现先按[数字核实协议](./AI_VERIFICATION_PROTOCOL.md)标 L0–L3，保留复现命令、受检 SHA、与现有条目的去重结果和 V1–V4 影响；只有达到 L3、确定归属与优先级后才建议入库，不自动新建 `K-*` / `V-*` 或改 OPEN。

**产物**：带证据等级的发现与去重清单。只有确认新债或状态实质变化时才改技术债 SSOT；技术债转 Done / main CI / 发版证据按工程流水线强制 L，并以目标 SHA 的远端 CI 结论核定，不由一次巡检或文档更新直接宣布。

---

### 维度六 · 文档漂移修复

**checklist**
- [ ] `OCLIVE_ARCHITECTURE_OVERVIEW.md` 模块描述 vs 代码
- [ ] `ARCHITECTURE_LAYERING.md` 依赖规则 + 反向依赖标注
- [ ] `ROLE_PACK_SPEC.md` 字段 vs schema；`DISTRO_CAPABILITY_PROFILE.md` vs `host_profile.rs`
- [ ] `NAMING_CONVENTIONS.md` canonical 路径 vs 实际 import
- [ ] `CHANGELOG.md` + `CHANGELOG.en.md` `[Unreleased]` **中英 parity**（门禁项）
- [x] 姊妹仓：`oclive-vscode/ROADMAP.md`、`VSCODE_DISTRIBUTION.md`（"能力优先"）、pack-editor README deprecated 状态
- [ ] 许可证迁移后无残留 AGPL（除历史性引用）

**愿景拷问**
- [ ] 【V4】是否需为普通创作者抽一条与内核文档**物理分离**的"会发光黄金路径"（解决"文档只有内核开发者一种声音"）？

**产物**：漂移清单（路径 + 不一致 + 修复建议）+ 直接修正 + 创作者黄金路径结构建议。

---

### 维度七 · 条理与边界（防 AI 脱轨 ★每轮快档也跑前两项）

> **存在理由**：六维盯「对不对 / 够不够愿景」,**不盯「事实来源是否分裂」**。kernel/distros 拆分后,旧路径、过期文档、模糊边界让 AI「每次对话都合理,合起来互相打架」。本维度是给狂奔的火车装的轨道与限速器。

> **巡检前自问一句**：*如果我是一个只读了 `AGENTS.md` + `.cursor/rules` 的新 AI,会不会被带到错误的目录 / 过期的结论 / 没有 SSOT 的自由区?*

**1. 路径 SSOT 漂移（每轮快档必跑 · 自动化优先）**
- [ ] `node scripts/check-stale-paths.mjs` 通过（应覆盖 bare `roles/`、`src-tauri`、根级 `crates/`、非 `distros/chat-pro/plugins` 的 `plugins/`）
- [ ] Rust 侧 `roles` 解析有单一 SSOT（测试经 `tests/common` 或 `chat_pro_roles_dir()`,**禁止**每文件手写 `../roles`）
- [ ] JS 侧路径经 `scripts/lib/chat-pro-roles-dir.mjs`,未新增散落 `join('roles')`
- [ ] CI / 脚本 / `examples` 中 `OCLIVE_ROLES_DIR`、`cd fuzz`(应 `kernel/fuzz`)、Playwright `testDir`、`check:license` 插件路径与新布局一致

**2. AI 入口文档时效（每轮快档必跑）**
- [ ] `.cursor/rules/oclivenewnew.mdc` 不指向**已归档**文档（如 `04_4.6`）或失效路径（如根 `handoff/WEEKLY_DEV_GUIDE.md`）
- [ ] `AGENTS.md` 与源码无硬冲突（迁移目录、HTTP API 存在性、`process_message` 所在 crate、re-export 现状）
- [ ] 同一事实只有一个数字 SSOT（如 invoke 热路径条数以 `INVOKE_HOTPATH_MATRIX.md` 为准,AGENTS/CHANGELOG 引用不另造数字）
- [ ] 已归档文档在 README / DOCUMENTATION_INDEX / `.cursor/rules` 处**显式标注归档**,不再被当当前 truth

**3. 边界明确性（防 AI 自作主张 · 半档/全档）**
- [ ] 每个「已交付 / 草案 / 冻结 / Deferred」状态在**三处一致**：源码现实、handoff 台账、AGENTS 入口（重点查 theater_director、reply_post_process chain、portrait/visual、expert_routing）
- [ ] 「草案 / 冻结」不等于「仓库无代码」——逐调用点区分 Stable 与 Experimental：Stable `co_present` 已装配 `extra_sections`，`dual_pipeline_steps` 的空切片不能概括全局接线状态。
- [ ] 角色包 vs 蓝图改动层次清晰（`meta` 今日字段 vs `runtime_config` v3 目标）——见 `ROLE_PACK_BOUNDARY.md`
- [ ] AI 硬约束清单存在且最新（建议 `AI_CHANGE_BOUNDARIES.md` 或 AGENTS「禁止区」：不在角色任务改 `slot_registry`、不把 RFC Draft 当未实现而删 wiring、不引归档当 truth、改锁文件必跑 `cargo audit` 并更新 `KNOWN_VULNERABILITIES.md`）

**4. 门禁语义纯度（全档）**
- [x] CI job 只分两类：**硬门禁（红=不能合）** 与 **nightly/可见性（不挡 main）**；Stage 1 `ci-impact-plan` 是唯一非阻塞影子报告，不冒充验证 job
- [x] loom / fuzz / e2e-tauri / cli-bench / visual-smoke 已迁出 `ci.yml` 进入独立 Nightly/手动工作流；Nightly 内不吞失败，`npm-audit` 对生产与完整开发图均为硬门禁
- [ ] 本地 `check:release` 与 CI 全集差距已知并记录(不假装等价)

**愿景拷问**
- [ ] 【V4】新接入的第三方创作者 / AI 是否能在不踩旧路径的前提下跑通?事实来源是否单一?
- [ ] 【元纪律】本维度发现是否在「防回退」边界内?纯洁癖式重排默认 Deferred(见 §9)

**方法**：先跑 `check-stale-paths` / `check-domain-layering` 等 ratchet（自动化优先于人工通读）；人工只复核「自动化扫不到的语义矛盾」（文档状态 vs 代码、边界归属）。新发现按核实协议分级、对照已有 `D-ORDER-*` / `D-DOC-*` 去重；达到 L3 且确需跟踪时，才在技术债 SSOT 登记。
**产物**：① 路径漂移清单（自动化输出）；② 文档矛盾清单（声称 A 文件 vs 事实 B 文件）；③ AI 自由度缺口（无 SSOT 的边界）+ 处置建议。

---

## 5. 常用命令速查（PowerShell,逐条跑勿用 `&&`）

```powershell
node scripts/dimension5-acceptance.mjs --ci      # 维度五门禁（项数以脚本输出为准）
cargo test -p oclive_kernel_host --lib           # 核心单测（不含 doctest）
cargo test --workspace --doc                     # doctest（公开 DTO/trait/crate 改名后必跑;check:rust 不跑）
node scripts/check-domain-layering.mjs           # 分层 ratchet
node scripts/check-stale-paths.mjs               # 维度七:过期路径漂移(roles/src-tauri/crates)
node scripts/check-changelog-parity.mjs          # CHANGELOG 中英 parity
node scripts/check-host-reexport-imports.mjs     # re-export ratchet
npm run check:license                            # 许可证文件存在性
npm run test:unit                                # 前端最小烟测
cargo build --timings                            # 编译瓶颈分析
npm run check:rust                               # fmt + clippy(-D warnings) + test
```

---

## 6. 关键坐标（巡检常去的文件）

| 用途 | 路径 |
|------|------|
| 参考 Host 主编排入口 | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` |
| Prompt 公式 | `kernel/crates/oclive_kernel_runtime/src/domain/prompt_builder/` |
| 分层基线 | `handoff/LAYERING_BASELINE.json` |
| 技术债总账 | `handoff/TECHNICAL_DEBT_INVENTORY.md` |
| 迁移 SSOT | `kernel/crates/oclive_kernel_host/migrations/*.sql` |
| crate 速查 | `kernel/crates/README.md` |
| 命名 SSOT | `creator-docs/NAMING_CONVENTIONS.md` |
| 角色包路径 SSOT(JS) | `scripts/lib/chat-pro-roles-dir.mjs` |
| 过期路径 ratchet | `scripts/check-stale-paths.mjs` |
| 角色 vs 蓝图边界 | `handoff/ROLE_PACK_BOUNDARY.md` |
| 关键路径交接 | `handoff/BUS_FACTOR_NOTES.md` |

---

## 7. 综合输出模板（全档必填；快/半档只填实测与边界）

```
## 巡检轮次 N（YYYY-MM-DD；档位：快/半/全；受检 HEAD：完整 SHA）

### 基线：PASS / FAIL（记录首次失败、环境修正与复验；未恢复则止于基线）

### 发现清单（按优先级）
| # | 证据等级与原始来源 | 现状/问题 | 建议 | 工作量 | 愿景影响(V1-4) | 处置 |

### 本轮修复（Done）
### 本轮延后（Deferred；已入库写编号，未入库写原因）
### 未测项、环境限制与副作用

### 六维健康度评分（仅全档；双栏：正确性 / 愿景最优性）
| 维度 | 正确性 | 愿景最优性 | 理由 |
| 基线 | | — | |
| 一架构 | | | |
| 二性能 | | | |
| 三设计 | | | |
| 四技术债 | | — | |
| 六文档 | | | |

### 下一轮建议
```

**评分基准**：A=优且无新债 / B=良有小债 / C=可用但有结构隐患 / D=有阻塞风险 / F=基线破。

快/半档保留本模板的基线、发现、处置与未测项，评分栏写“未评分”，不引用旧轮次分数当现状。表内 Done 仅指**本轮已完成的具体修复**，不是技术债台账的 Done 状态；每项数字与结论都应能追溯到受检 SHA、命令、退出码或原始文件。

---

## 8. 巡检日志（滚动窗口）

本节只保留最近五轮，避免活跃手册随历史无限增长。更早轮次见本文件 Git 历史；未完成事项只以 [`TECHNICAL_DEBT_INVENTORY.md`](./TECHNICAL_DEBT_INVENTORY.md) 为准。巡检当轮先在交付报告或 PR 评论记录受检 SHA、实测和边界；§8 的滚动行可随下一次**有实质内容**的文档提交入账，不为补分数、run ID 或 CI 结论单独推送触发新一轮 CI。快/半档可以“未评分”；未实际执行的巡检不新增轮次。

| 轮次 | 日期 | 档位 | 基线 | 综合评分 | 关键发现 / 新增债 | 备注 |
|------|------|------|------|----------|-------------------|------|
| 28 | 2026-08-01 | 里程碑破坏性验证 | PASS（工程）/ FAIL（语音尾延迟门禁） | B+ | 完成 K-SUPPLY-12、本地 Nightly 分流和 11 场景影子模拟；修复 Nightly Loom 占位测试并以 feature 运行真实有界模型；并发覆盖资源桥 10、协调器 19、成人节拍 12、LLM 环境 2、远端 LLM 1、Loom 2 条。真实内核 5 分钟 60/60 请求且进程回收；GPU 24 层在 1MiB 临界余量被拒，改用 22 层后资源闭环恢复；新增 **K-VOICE-09** | 22 层五分钟完成 **46** 对 LLM/TTS，峰值余量 **1370MiB**、稳态增长 **74MiB**、318 次 GPU 采样零失败；TTFC p50/p95/max **6271/7475/9514ms**，因 max > 8s 仍红且未放宽阈值。影子模拟 **8 targeted / 3 fail-safe**，只证明规则回归；完整远端 CI 尚未声明 |
| 29 | 2026-08-02 | 远端收口 + 文档对账 | PASS（remote） | A− | 冻结实现 `728219e7` 的主 CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) **16/16** success，完整 Nightly [`30714480898`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714480898) 的视觉、fuzz、Loom、CLI benchmark、原生窗口与汇总 **6/6** success；据此关闭 **K-SUPPLY-12** 与 **D-CI-EXECUTION-02** 的远端待验状态。修正供应链、CI 路线图、轻量化与内核集成中英文档的旧配置/旧路径；明确完整 `process_message` 在 `oclive_kernel_host`，纯 library 仍是 V-EMBED-01 Partial，未改模块边界或生产代码 | K-VOICE-09 的 20-token 有界长段策略把 TTFC p50/p95(max) 从 **4135/4473ms** 降至 **3643/3826ms**，改善 **11.9%/14.5%**；短句 10/10 保持旧策略。仍缺 30 分钟真实矩阵与人工听感，故保持 **In progress**；历史 **9514ms** 尾样本也未被删除。K-CI-IMPACT-01 仍为 Shadow + 2 个真实 Compare 样本，未借本轮放开选择性门禁 |
| 30 | 2026-09-27 | 半（限定） | PASS（显式设置 `OCLIVE_VOICE_PYTHON`；默认环境缺 `py`，首次 TTS ratchet FAIL） | 未评分（半档） | 受检 HEAD `62fe6575`：Dimension 5 28 项 PASS、Host lib 634/634、分层 use-import 3/FQ 1 均未上涨；包级依赖方向符合既有边界。TTFT/TTFC 专项 SSOT 最近实测仍为 2026-08-02，当前 HEAD 未重新测这两项；修正 `PERF_PHASES.md` 的现行桌面流式传输口径 | `c41aa921` 限定接入标签保持不变；本轮仅做离线门禁、静态依赖抽查和文档校正，未启动真实模型/语音、服务或长时 soak；默认 Python launcher 缺失作为本机环境前提记录，不写成产品回退 |
| 31 | 2026-09-28 | 快（AI 边界 / 流水线交叉审查） | PASS（local；`OCLIVE_VOICE_PYTHON` 指向已安装 Python 3.12） | 未评分（快档） | 受检 HEAD `876b95d1a601499db86bc942dd4978c86d8b53fe`：Dimension 5 **PASS (28 checks)**；`cargo test -p oclive_kernel_host --lib --locked --offline -j 1` **634/634**；分层 use-import **3/3**、FQ **1/1**，host re-export **75/75**；`check-stale-paths` docs+code PASS。对照 AI 边界、核实协议与工程流水线发现本手册两组 ratchet 数字过期、发现即入债和历史评分口径过宽，已只改手册流程。 | `AGENTS.md`、`.cursor/rules` 与 `handoff/README.md` 的入口链接可达；[`ci.yml` run 36326312754](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36326312754) 对**受检旧 HEAD** 为 success，不证明本次未推送的手册修改；未跑真实模型/语音性能、完整架构/债务复核或长时 soak；未新增技术债或更改旧结论。 |
| 32 | 2026-09-28 | 快档基线＋架构/队列/文档专项 | PASS（本地工程）/ HOLD（债务自动派工） | 未评分（限定审查） | 受检代码起点 `6da23955`：`check:ci-local` exit 0，shared/Chat Pro 单测 **243/92** 通过，模块兼容与债务结构检查通过；静态依赖方向和分层 ratchet 未回退。修正本手册 D-PORT-02、`extra_sections` 的过期表述及核实协议的档位冲突；队列 5 条 `pr-open` 与当前远端 PR 状态待对账。 | 详见 [`ROUND-02-PLAN.md`](./debt-marathon/ROUND-02-PLAN.md#启动前基线审查2026-09-28限定范围)；本轮未给全档评分、未启真实模型/语音、未重跑 Chat Pro live，也未变更债务状态。通用 `dev-pipeline/SKILL.md` 本机缺失；父提交的远端 CI success 不代表本轮文档 HEAD。 |

---

## 9. 元纪律（最重要,每轮读一遍）

> **这份手册本身就是"向内打磨"的最大诱因。** 它精致、系统、令人安心,而且把你拉回最舒适的区域——审查内核、追求最优。

**约束**：

1. **历史全档曾获 A 级，不代表当前 HEAD 的全档评分。** 巡检的目的是**防回退**，不是**追完美**；未实测维度保持未评分，旧轮次不能补当前证据。
2. **硬门禁、安全、兼容或确定性回退先处理。** 其余不直接服务当前“惊喜”（官方剧场 demo / 发行版上线 / Apache-2.0 落地）的优化，默认 Deferred，并按 §4 的证据等级记录；不能用“愿景优先”略过阻断项。
3. **三类发现优先保留并处理**（因其直接服务愿景）：
   - 维度一【V2】槽态真实性缺口（可替换性是核心卖点的实现质量）
   - 维度二【剧场实时 / V1 低算力】性能预算（直接决定 demo 体感）
   - 维度六【V4】创作者黄金路径（直接决定 30 分钟创造能否兑现）
4. **默认走快档。** 全档稀缺、刻意。别让巡检频率变成逃避"把它推到陌生人面前"的借口。
5. **文档不增殖。** 模块定义只改 [`MODULE_MAP_AND_HANDOFF.md`](./MODULE_MAP_AND_HANDOFF.md)；无 RFC/关键决策不新建 handoff 顶层文；动文档前读 [`handoff/README.md`](./README.md) §文档分责（**可以慢，读对 SSOT**）；遵循 [`AI_CHANGE_BOUNDARIES.md`](./AI_CHANGE_BOUNDARIES.md) G10–G16 · §文档编写纪律。**效率源于限制。**

> 一句话：**先守住硬门禁和安全边界；其余非阻断打磨让位于“惊喜”，用证据保留到合适的巡检或开发轮次。**
