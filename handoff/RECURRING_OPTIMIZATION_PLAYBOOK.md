# OClive 多轮优化巡检手册（Recurring Optimization Playbook）

> **定位**：一份**可反复运行**的地基巡检流程。不是一次性审查报告，而是每隔一段时间 / 关键节点照着跑一遍的"体检套餐"。
>
> **核心信条**：保证地基稳固才能走得远。但**地基是为了承载"惊喜"（官方剧场 demo / 发行版），不是为了自身完美**——见文末「§9 元纪律」。
>
> **创建**：2026-06-09 · **维护者**：内核作者本人 · **状态**：活跃手册（每轮在 §8 追加一行记录）

---

## 0. 如何使用本手册

1. 不要每次都全跑。按 **§1 触发条件** 决定本轮跑「快档」还是「全档」。
2. 永远从 **基线门禁（§2）** 开始；基线 FAIL 则**中止**，先修地基再谈优化。
3. 按固定顺序走维度：**基线 → 一架构 → 二性能 → 三设计 → 四技术债 → 六文档**。
4. 每个维度用**两把尺子**：① 传统正确性（能跑/对不对）；② **愿景对齐**（V1–V4，见 §3）。
5. 收尾在 **§7 综合输出** 出评分，在 **§8 巡检日志** 追加一行,新债按编号入 `TECHNICAL_DEBT_INVENTORY.md`。

---

## 1. 触发条件与档位

| 档位 | 何时跑 | 范围 |
|------|--------|------|
| **快档（Smoke,~15min）** | 每次合并涉及 `process_message` / `plugin_host` / `host_profile` / CI 配置 / 迁移的改动后 | 仅 §2 基线 + 受影响维度的对应 checklist |
| **半档（~1h）** | 每完成一个发行版里程碑、或每 2–4 周 | §2 基线 + 维度一 + 维度二 |
| **全档（半天）** | 每个 minor 版本发布前、或重大架构变更后 | 全部六维 + §7 评分 + §8 记录 |

> 经验法则：**默认走快档**。全档稀缺、刻意,别让巡检变成日常逃避区（见 §9）。

---

## 2. 基线门禁（每轮必跑，FAIL 即中止）

PowerShell 下逐条跑（**不要用 `&&`**）：

```powershell
node scripts/dimension5-acceptance.mjs --ci   # 必须 PASS (9 checks)
cargo test -p oclive_kernel_host --lib         # 必须全绿
node scripts/check-domain-layering.mjs         # ratchet 数值不得上涨
git status                                      # 确认工作树状态 / 与 origin 差距
```

**判定**：九项门禁含 layering ratchet / cargo-audit / lockfile / ensure-plan / CHANGELOG 中英 parity / host re-export ratchet / **verify:ui** / **vite build**。任一 FAIL → **本轮停止所有优化,先恢复基线**。

**ratchet 锚点**（只降不升）：
- `domain→infrastructure`：use-import ≤ 4（全 test cfg）+ FQ ≤ 5 → 见 `handoff/LAYERING_BASELINE.json`
- host/runtime re-export import baseline ≤ 76 → `scripts/check-host-reexport-imports.mjs`

---

## 3. 愿景对齐四主轴（贯穿每个维度的第二把尺子）

> 这是本手册区别于通用「五维审查」的关键。每条发现都要标注它影响哪条主轴。

| 代号 | 愿景主轴 | 每轮拷问 |
|------|----------|----------|
| **V1 普适性** | 内核可去任何地方（牙刷/机器人/游戏/直播/嵌入式） | 这个设计有没有把内核钉死在桌面 / Tauri 上？低算力设备能扛吗？ |
| **V2 可替换性** | 六槽 + 记忆/情绪/性格/编排皆可换 | 这个槽是**真**可换,还是只在文档里可换（枚举占位）？ |
| **V3 可携带性** | 灵魂可跨发行版携带 | 这份数据/格式能不能活着走到下一个宿主？是已验证还是仅设计？ |
| **V4 创造坍缩** | 开发坍缩成创造力（30 分钟造灵魂） | 这一层有没有降低普通人的创造成本？第三方模块作者友好吗？ |

---

## 4. 维度执行清单（按顺序）

### 维度一 · 架构全景 + 模块边界 ★每轮重点

**正确性 checklist**
- [ ] 12 crate 依赖图仍严格单向（`types→contracts→runtime→host→{server,tauri}`）
- [ ] `domain→infrastructure` 反向依赖无新增（对照 `LAYERING_BASELINE.json`）
- [ ] `process_message`（`crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`）仍是唯一编排 SSOT,业务逻辑未泄漏到 `src-tauri/src/api/*`
- [ ] 冻结项（dual_core / blueprint v3 / expert_routing）仍 feature-gated 默认不编译

**愿景拷问**
- [ ] 【V1】纯内核 `library`（host-independent）抽离进度——本轮裁决：阻塞 / 非阻塞？（§3.1）
- [ ] 【V2】**24 格槽态真实性矩阵**：6 槽 × {builtin/remote/directory/none},每格标 ✅真跑通 / ⚠️占位 / ❌未实现
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
- [ ] D-PORT-02：`PluginBackendRegistryPort` god-port 现状（22 方法 / 纯转发）
- [ ] D-SLOT-01：各槽 `BuiltinV1/V2/Placeholder` 并行 + 选择逻辑散落 `BackendRegistry`
- [ ] 错误模型一致性（`AppError / TurnError / ProcessMessageError`）
- [ ] 单实现 trait 普查（~24 contracts trait）：保留为 DI 端口 / 降级具体类型
- [ ] `resolve_*` 命名混淆度 + rustdoc 覆盖率

**愿景拷问**
- [ ] 【V2】god-port / 槽并行是"可替换性"愿景的实现质量问题——本轮是否升级优先级？
- [ ] 【V4】第三方模块作者面对的接口面是否清爽（决定"别人写得好我直接抄"能否转起来）？

**方法**：全仓统计 trait 实现数,列单实现 trait 表逐个标注处置；审 god-port 真实调用面 vs 暴露面。
**产物**：发现清单 + 单实现 trait 处置表 + D-PORT-02/D-SLOT-01 优先级重裁。

---

### 维度四 · 技术债务清单更新

**checklist**
- [ ] 逐条核对 `TECHNICAL_DEBT_INVENTORY.md`（K-PERF / K-PROFILE / K-DOC / D-LAYER / D-PORT / D-SLOT / D-POLICY）现状
- [ ] 已 Deferred 项：仍合理延后,还是因愿景推进需激活？
- [ ] 遗留 TODO/FIXME 扫描（已知仅 `plugin_scaffold.rs` 模板占位,确认无新增）
- [ ] 新发现按编号入库；**愿景对齐类新增用 `V-*` 前缀**（如 `V-EMBED-01` 纯内核抽离、`V-PORTABLE-01` 跨宿主携带验证、`V-LICENSE-01` Apache-2.0 落地）

**产物**：更新后的 `TECHNICAL_DEBT_INVENTORY.md`（更新 header 时间戳与 Verification 行）+ 新 `V-*` 段。

---

### 维度六 · 文档漂移修复

**checklist**
- [ ] `OCLIVE_ARCHITECTURE_OVERVIEW.md` 模块描述 vs 代码
- [ ] `ARCHITECTURE_LAYERING.md` 依赖规则 + 反向依赖标注
- [ ] `ROLE_PACK_SPEC.md` 字段 vs schema；`DISTRO_CAPABILITY_PROFILE.md` vs `host_profile.rs`
- [ ] `NAMING_CONVENTIONS.md` canonical 路径 vs 实际 import
- [ ] `CHANGELOG.md` + `CHANGELOG.en.md` `[Unreleased]` **中英 parity**（门禁项）
- [ ] 姊妹仓：`oclive-vscode/ROADMAP.md`、`VSCODE_DISTRIBUTION.md`（"能力优先"）、pack-editor README deprecated 状态
- [ ] 许可证迁移后无残留 AGPL（除历史性引用）

**愿景拷问**
- [ ] 【V4】是否需为普通创作者抽一条与内核文档**物理分离**的"会发光黄金路径"（解决"文档只有内核开发者一种声音"）？

**产物**：漂移清单（路径 + 不一致 + 修复建议）+ 直接修正 + 创作者黄金路径结构建议。

---

## 5. 常用命令速查（PowerShell,逐条跑勿用 `&&`）

```powershell
node scripts/dimension5-acceptance.mjs --ci      # 维度五门禁（9 checks）
cargo test -p oclive_kernel_host --lib           # 核心单测
node scripts/check-domain-layering.mjs           # 分层 ratchet
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
| 编排 SSOT | `crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` |
| Prompt 公式 | `crates/oclive_kernel_runtime/src/domain/prompt_builder/` |
| 分层基线 | `handoff/LAYERING_BASELINE.json` |
| 技术债总账 | `handoff/TECHNICAL_DEBT_INVENTORY.md` |
| 迁移 SSOT | `crates/oclive_kernel_host/migrations/*.sql` |
| crate 速查 | `crates/README.md` |
| 命名 SSOT | `creator-docs/NAMING_CONVENTIONS.md` |

---

## 7. 综合输出模板（全档收尾必填）

```
## 巡检轮次 N（YYYY-MM-DD,档位：快/半/全）

### 基线：PASS / FAIL（FAIL 则只记此行）

### 发现清单（按优先级）
| # | 现状 | 问题 | 建议 | 工作量 | 愿景影响(V1-4) | 处置 |

### 本轮修复（Done）
### 本轮延后（Deferred,已入技术债编号）

### 六维健康度评分（双栏：正确性 / 愿景最优性）
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

---

## 8. 巡检日志（每轮追加一行）

| 轮次 | 日期 | 档位 | 基线 | 综合评分 | 关键发现 / 新增债 | 备注 |
|------|------|------|------|----------|-------------------|------|
| 0 | 2026-06-08 | 全 | PASS | A− | Opus 4.8 收尾,无新债 | 本手册前置基线 |
| 1 | 2026-06-09 | 全 | PASS | A− | 无新债;槽态矩阵 24 格全有路径,缺口=remote 缺 env 静默回退;D-SLOT-01/D-PORT-02 维持 Deferred(冻结期) | Theater v0 冻结期巡检,只防回退 |
| 2 | 2026-06-09 | 半 | PASS | A− | Wave1–3: D-ERR-01/K-PROFILE-04/D-CLEAN-01 Done; V-THEATER-PERF-01/V-SLOT-HONEST-01 Done; re-export 77; layering FQ 1 | 工程夯实轨 Wave 1–3 合并 |
| 3 | 2026-06-09 | 全 | PASS | A− | Wave4 条件门: 陌生人测试未执行; C 档维持 Deferred; Phase 5 解冻 **不启动** | 见 TECHNICAL_DEBT §巡检债 Wave |
| 4 | 2026-06-10 | 半 | PASS* | A− | oclive-vscode IA 统一 + ensureReady 缓存 + 轮询退避 + 占位清理; V-VSCODE-PERF-05 F5/.vsix 仍 Pending | *姊妹仓 lint/compile/smoke; 主仓基线未重跑 |
| 5 | 2026-06-10 | 快 | n/a* | — | oclive-vscode 用户报障修复: 设置内即时切角色卡死(handleMessage 串行化 + switchRole guard 全程保持 + 去重 pushState) / 模型调用不稳(ensureReady 三态 trust·revalidate·replan, 健康连接不再整轮重规划/mock 杀端口) / 角色下拉栏改 Cursor 配色 / 新增 ensureReadyPolicy+serialQueue 单测; V-VSCODE-FIX-01·02 / UI-01 / QA-01 Done | *姊妹仓 lint/compile/test:unit/webview build 通过; 主仓基线未跑 |
| 6 | 2026-06-10 | 快 | n/a* | — | oclive-vscode 设置落地: 角色区改只读去重(IA-02) / 内核「重新发现」触发 autoDiscover(LAND-01) / 移除高级实验性死占位(HONEST-02) | *姊妹仓 lint/test:unit/webview build 通过; F5 实机待开发者确认 |
| 7 | 2026-06-10 | 快 | n/a* | — | VS Code 聊天体验: LATENCY(停止/预热/计时) + UNDO(四形态/meta_action_templates) + STREAM(/chat/stream Gate 批准); 主仓 validation + oclive_kernel_host 流式 | *姊妹仓 lint/build/test:unit; 主仓 cargo test validation+host |
| 8 | 2026-06-10 | 半 | FAIL→PASS | A− | 基线红: role_manager FQ 2>1 → 插件注入化修复; 删 `oclive_runtimed`+`crates/models`; deny.toml 去 AGPL; D-ORPHAN-01/02、D-NAME-01(104 resolve_*) 入账 | dimension5+host lib 182 绿; 含轮次7流式/撤销/meta_action_templates 提交 |
| 9 | 2026-06-10 | 全 | PASS* | A− | *门禁 PASS 但场外两处红: K-BUILD-02 TheaterShell 导入错层致 `vite build` HEAD 失败(修复) + D-SCRIPT-01 verify:ui 锚点全过时崩溃(重写); D-ORPHAN-03 删 V1 孤儿组件 23KB; K-DOC-08 runtimed 幽灵引用清掉; D-TRAIT-01(16 单实现 trait) 入账 Deferred; D-PORT-02 计数 22→24 | 过度工程普查轮; 大刀(god-port/V2 槽/错误四层)维持冻结 Deferred |
| 10 | 2026-06-10 | 半 | PASS | A− | K-GATE-01 九检; D-ORPHAN-03b/c; D-PORT-02/D-SLOT-01/D-TRAIT-01; Theater 自动化 9 测绿·人工陌生人待执行; beat patch 单测绿; cargo build ~81s dev | [SLOT_BACKEND_REALITY_MATRIX.md](./SLOT_BACKEND_REALITY_MATRIX.md) |
| 11 | 2026-06-11 | 半 | PASS | A− | Batch 1–3 入库; Phase 1 D-ERR-02/034/棘轮76/K-DOC; Phase 2 K-PERF-20/21 快照+settings 批量; Phase 3 会话索引/RoleRuntimeRepo/前端 follow-up; Phase 4 Deferred 登记 | Theater v0 冻结期; 陌生人测试仍 Pending |

---

## 9. 元纪律（最重要,每轮读一遍）

> **这份手册本身就是"向内打磨"的最大诱因。** 它精致、系统、令人安心,而且把你拉回最舒适的区域——审查内核、追求最优。

**约束**：

1. **地基已是 A 级。** 巡检的目的是**防回退**,不是**追完美**。边际价值在 A 级之后快速趋零。
2. **凡不直接服务于当前"惊喜"（官方剧场 demo / 发行版上线 / Apache-2.0 落地 / push 上线）的发现,默认 Deferred,本轮只记录不动手。**
3. **三类发现优先保留并处理**（因其直接服务愿景）：
   - 维度一【V2】槽态真实性缺口（可替换性是核心卖点的实现质量）
   - 维度二【剧场实时 / V1 低算力】性能预算（直接决定 demo 体感）
   - 维度六【V4】创作者黄金路径（直接决定 30 分钟创造能否兑现）
4. **默认走快档。** 全档稀缺、刻意。别让巡检频率变成逃避"把它推到陌生人面前"的借口。

> 一句话:**地基稳是为了让"惊喜"走得远;当地基稳与做惊喜冲突时,先做惊喜,把地基发现记进 §8 等下一轮。**
