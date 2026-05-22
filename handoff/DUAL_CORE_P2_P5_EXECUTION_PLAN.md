# 给 Cursor：双核 P2–P5 完整执行计划

**前置**：P0 文档 + Q1–Q20 已决；P1 `oclive_validation`（`blueprint_v3`、`runtime_config`、`--profile creator`）已入库。  
**SSOT**： [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) · [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../creator-docs/rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md) · [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md)

**本文**：对 DeepSeek 草案的 **仓库对齐版**；标出路径修正、分阶段 MVP 与 **开工前须答复** 的问题（§八）。

---

## 当前状态

| 阶段 | 状态 | 说明 |
|------|------|------|
| P0 | ✅ | 文档 + Q1–Q20 |
| P1 | ✅ | 校验 / Schema / creator profile |
| **P2** | ❌ | 宿主加载 + 调度器 + `process_message` |
| **P3** | ❌ | `oclive init --dual-core` |
| **P4** | ❌ | OOCP 降级场景 |
| **P5** | ❌ | Monolith + `--dual-core`（**仅 `oclive-cli` 生成工程**） |

**今日事实**：

- 角色加载在 **`src-tauri/src/infrastructure/storage.rs`**（`load_role_from_blueprint_v2_dir`），**无** `infrastructure/role_pack.rs`。
- `load_blueprint_v2_for_role_dir` **仅接受 `schema_version: 2`**；v3 须新增 **`load_blueprint_v3_for_role_dir`**（`oclive_validation`）。
- 仓库 **无** `SessionState` 类型；快照对象须另行定义（§八 Q21）。
- 主编排仍是 **`process_message` → `co_present::process_co_present`**（Stable = Q19 硬编码路径）。

---

## 核心设计（执行期不变）

| 项 | 决议 |
|----|------|
| 开关 | `runtime_config.dual_core.enabled`，默认 `false` |
| Stable | **始终** 可回落到 **`process_co_present`**（与现网一致） |
| Experimental | `pipeline.experimental` + `slot.<registry_key>.<method>`；P4 运行时仅七种 PluginHost `type` |
| 降级 | 实验失败 → 快照恢复 → Stable；**用户静默**（Q7） |
| Monolith | 无 `--dual-core` → 编译期无实验链路；`--monolith --dual-core` → 焊接 + 保留 Runner（Q14） |

---

## 依赖关系（须按序）

```text
P2-T1  load v3 + Role.runtime_config
  ↓
P2-T2  DualPipelineRunner + 快照 MVP
  ↓
P2-T3  process_message 门控
  ↓
P3     oclive init --dual-core
  ↓
P4     v3 夹具角色包 + OOCP S13
  ↓
P5     oclive-cli Monolith 生成（独立）
```

---

## 阶段一（P2）：宿主接线

### P2-T1：加载 `runtime_config`（修正路径）

**目标**：v3 蓝图加载 `runtime_config` + `pipeline`；v2 忽略 `runtime_config`（已有校验警告）。

| 步骤 | 位置 | 动作 |
|------|------|------|
| 1 | `crates/oclive_validation/src/blueprint_v3.rs` | 新增 `BlueprintV3LoadResult`（`disk` + `slot_registry` + `runtime_config` + `pipeline` + `groups`） |
| 2 | 同上 | `pub fn load_blueprint_v3_for_role_dir(role_dir, host_version)` |
| 3 | `crates/oclive_kernel_types/src/models/role.rs` | `Role` 增加 `runtime_config: Option<RuntimeConfig>`、`dual_pipeline: Option<DualPipelineDef>`（或扁平字段） |
| 4 | `src-tauri/src/infrastructure/storage.rs` | `load_role_from_dir`：读 blueprint 先判 `schema_version` → v3 走新 loader，v2 走现路径 |
| 5 | 过渡期 | v3 的 `interaction_mode` / `reply_quality_anchor` 等：**优先 `runtime_config`**，缺省再读 `meta`（与 ROLE_PACK_BOUNDARY 一致） |

**验收**：

- `roles/mumu`（v2）加载无回归。
- 手写 v3 夹具（见 P4）`load_role` 后 `role.runtime_config.dual_core.enabled` 正确。

**测试**：

- `cargo test -p oclive_validation`（已有 v3 单测 + 新增 load 夹具）
- `cargo test -p oclivenewnew-tauri --lib`（`RoleStorage` 相关）

**提交**：`feat(p2): load runtime_config and v3 blueprint in RoleStorage`

---

### P2-T2：`DualPipelineRunner`（修正抽象）

**目标**：实验优先 + 失败降级；**不**引入不存在的 `SessionState`。

**建议模块**：`src-tauri/src/domain/chat_engine/dual_pipeline.rs`（或 `oclive_kernel_runtime` 若 Monolith 须链接 — 见 §八 Q26）。

**建议 API（与现网对齐）**：

```rust
// 快照：Q8 仅内存 — 首轮至少覆盖 AppState 会话级可回滚字段
pub struct TurnRollbackSnapshot {
    pub narrative_hint: String,
    // 按需扩展：本轮未落库的编排中间变量
}

pub struct DualPipelineRunner { /* 持有 pipeline + slot_registry 视图 */ }

impl DualPipelineRunner {
    /// Stable 核 = 今日 co_present（Q19）
    pub async fn run_stable_via_co_present(...) -> Result<SendMessageResponse, ProcessMessageError>;

    /// Experimental：按 DAG 拓扑执行 pipeline.experimental（P2 MVP 见下）
    pub async fn run_experimental(...) -> Result<SendMessageResponse, ProcessMessageError>;

    /// 快照 → experimental → 成功返回 / 失败恢复 → stable
    pub async fn run_with_fallback(...) -> Result<SendMessageResponse, ProcessMessageError>;
}
```

**P2 MVP 范围（建议，避免一次做完解释器）**：

| 层级 | 内容 |
|------|------|
| **MVP-0** | `run_with_fallback`：若 experimental 为空 → 直接 `run_stable`；若 experimental 非空但 **解释器未实现** → 记日志 + 降级 stable（满足「可测降级」） |
| **MVP-1** | 实现 **部分 `action` → 现有 SlotRunner / PluginHost 调用**（与 `co_present` 子步骤对齐的 3–5 个 method） |
| **MVP-2** | 全 DAG + 七种类 type 门禁（Q20） |

**快照**：

- `TurnRollbackSnapshot::capture(state, srid)` / `restore(state, srid, snap)`  
- **不**回滚 DB 已提交记忆/事件（Q8）

**日志**：`tracing` 字段 `degraded_from=experimental`；**无** `reply` 字段（Q7）

**测试**：

- `dual_pipeline.rs` 单元测：DAG 排序、失败触发 fallback、快照恢复 `narrative_hint`

**提交**：`feat(p2): add DualPipelineRunner with snapshot and fallback`

---

### P2-T3：`process_message` 门控

**目标**：不开双核 = **零 diff**。

**条件（建议写死）**：

```text
dual_core.enabled == true
AND pipeline.experimental 非空
→ DualPipelineRunner::run_with_fallback
否则
→ 现有 run() 末尾 co_present 路径（不变）
```

**注意**：

- `complex_emotion` **不**进 pipeline（Q1）；仍在 `co_present` 固定点。
- Agent 短路、异地分支：**保持现有顺序**，仅在进入共景主路径前分支（与今日 `process_message.rs` 结构一致）。

**验收**：

- `cargo test -p oclivenewnew-tauri --lib`
- `cargo test -p oclivenewnew-tauri --test invoke_hotpath_matrix`（若适用）
- 默认 `roles/mumu` 手动/OOCP S0–S12 **无回归**

**提交**：`feat(p2): wire DualPipelineRunner into process_message`

---

## 阶段二（P3）：脚手架

### P3-T1：`oclive init --dual-core`

| 项 | 说明 |
|----|------|
| CLI | `crates/oclive-cli/src/main.rs` / `init` 子命令增加 `--dual-core` |
| 产物 | 生成 `roles/<id>/pipeline.ocblueprint`：`schema_version: 3`，`runtime_config.dual_core.enabled: true`，`pipeline.stable` / `pipeline.experimental`（experimental 默认可 copy stable 骨架） |
| 文档 | `CONFIG_REFERENCE.md`、`OCLIVE_CLI_GUIDE.md` |

**勿改**：默认 `init`（无 flag）仍生成 **v2** 或 v2 无 pipeline — 与今日一致。

**测试**：`cargo test -p oclive-cli` + 临时目录 `init --dual-core` 后 `pack validate`

**提交**：`feat(p3): add --dual-core flag to oclive init`

---

## 阶段三（P4）：标准构建 + OOCP

### P4-T1：v3 夹具角色包

路径建议：`examples/dual-core-fallback-role/` 或 `roles/__oocp_dual_core_fallback__/`（**勿**破坏 `mumu`）。

| 内容 | 说明 |
|------|------|
| `schema_version: 3` | `dual_core.enabled: true` |
| `pipeline.experimental` | 含 **必失败** 步骤（见 §八 Q24） |
| `slot_registry` | 合法七类 type + 对应 backend（directory 需 CI 可跑或 mock） |

### P4-T2：OOCP S13

| 项 | 说明 |
|----|------|
| 场景名 | **S13_dual_core_fallback**（当前套件已有 **S12** 内核码形态，勿混淆） |
| 脚本 | `examples/oocp-test-suite/run.mjs` + `creator-docs/testing/OOCP_TEST_SUITE.md` |
| CI | `.github/workflows/ci.yml`：**可选** 单独 job 或 `OCLIVE_DUAL_CORE_OOCP=1` 才跑（避免默认 CI 依赖双核） |
| 启动 | `OCLIVE_HTTP_API_MOCK_LLM=1` + 指向 v3 夹具 `role_path` |

**断言**：HTTP 200 + 有 `reply`；日志或内部标记可选断言 `degraded_from`（不对玩家可见）。

**提交**：`test(p4): add OOCP dual-core fallback scenario`

---

## 阶段四（P5）：Monolith（独立里程碑）

**范围**：**仅** `oclive-cli` 生成的 kernel_server / monolith 工程，**非**主仓 `src-tauri` 默认桌面路径。

| 组合 | 行为 |
|------|------|
| `--monolith` | 无 `DualPipelineRunner`、无 experimental（与 RFC 一致） |
| `--monolith --dual-core` | `process_message_monolith.rs` 焊接 stable/experimental 步骤 + **保留** Runner 与快照 |

**实现触点**：

- `oclive-cli` init / build 模板：`monolith.toml`、`process_message_monolith.rs` 生成逻辑
- 检测 init 时是否 `--dual-core`（或 `monolith.toml` 增 `[dual_core] enabled`）

**文档**：`RFC_OCLIVE_MONOLITH_MODE.md`、`OCLIVE_CLI_GUIDE.md`

**提交**：`feat(p5): support --dual-core in Monolith build`

---

## 阶段五（P7）：全量验证

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
npm run test:unit
npm run build
# 可选：OCLIVE_DUAL_CORE_OOCP=1 node examples/oocp-test-suite/run.mjs
```

更新 [DUAL_CORE_ALIGNMENT.md](DUAL_CORE_ALIGNMENT.md) / [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) 进度表。

---

## 与 DeepSeek 草案的差异（已修正）

| 草案 | 仓库对齐 |
|------|----------|
| `infrastructure/role_pack.rs` | **`storage.rs` + `oclive_validation` loader** |
| `SessionState` clone | **`TurnRollbackSnapshot` + `AppState` 字段**（Q8） |
| `run_stable` 独立六槽表 | **Q19：Stable = `process_co_present`** |
| OOCP「12 场景」 | 现行 **S0–S12**；双核新增 **S13** |
| P2 一次实现完整 experimental | 建议 **MVP-0→1→2** 分 PR |

---

## 八、开工前须你答复的问题

答复后写入 RFC §10 / 本页 §九，再动 P2 代码。

### 阻塞 P2-T2 / T3

**Q21 — 快照对象到底是什么？**  
仓库无 `SessionState`。建议首轮仅回滚 `AppState::last_complex_emotion_narrative_hint`（按 `srid`）。是否同意？还要覆盖哪些键？

**Q22 — P2 Experimental 解释器 MVP 边界？**  
- **A**：MVP-0 仅「experimental 非空 → 故意返回 Err → 降级 stable」（验证降级链，不真跑新编排）  
- **B**：MVP-1 实现 3–5 个 `action` 到现有 `co_present` 子步骤映射  
- **C**：P2 一次实现完整 DAG + 全部可映射 method  

**Q23 — `pipeline.stable` 在 `enabled=true` 时是否执行？**  
- **A**：永不执行 stable pipeline；Stable 恒 `co_present`（Q19）  
- **B**：experimental 成功后仍可用 `pipeline.stable` 作收尾（与 Q19 冲突，需否决）

### 阻塞 P4

**Q24 — OOCP「故意失败」如何造？**  
- **A**：experimental 某 `action` 指向不存在的 registry 键（校验应拦 — 不适合运行时）  
- **B**：指向合法键但 `method` 未实现 → Runner 返回 Err  
- **C**：directory 插件 stub 返回 JSON-RPC error（需夹具插件）

**Q25 — 默认 CI 是否跑 S13？**  
- **A**：仅手动 /  nightly  
- **B**：并入现有 `oocp-test-suite` job（须 v3 夹具 + 环境变量开双核）

### 阻塞 P5 / 结构

**Q26 — `DualPipelineRunner` 放哪 crate？**  
- **A**：仅 `src-tauri`（桌面先落地）  
- **B**：`oclive_kernel_runtime`（Monolith / HTTP 共用，工作量大）

**Q27 — Monolith 双核开关来源？**  
- **A**：仅 `init --monolith --dual-core` 写入模板  
- **B**：`monolith.toml` 增加 `[dual_core] enabled = true` 可手改

### 过渡期数据

**Q28 — v3 角色包是否进 `roles/mumu`？**  
- **A**：保持 mumu 为 v2；双核仅 examples 夹具  
- **B**：另建 `roles/mumu-dual-core-v3` 用于人工试玩

**Q29 — `enabled=true` 但 `experimental` 为空？**  
建议：**视为未开启双核**，走 `co_present`（与 HANDOFF 一致）。是否确认？

---

## 九、建议 PR 切分（便于 review）

| PR | 范围 |
|----|------|
| 1 | P2-T1 loader + `Role` 字段 |
| 2 | P2-T2 Runner MVP-0（仅降级链） |
| 3 | P2-T2 MVP-1（部分 action） + P2-T3 门控 |
| 4 | P3 init |
| 5 | P4 OOCP S13 + 夹具 |
| 6 | P5 Monolith（可选延后） |

---

## 给 Cursor 的指令

1. **先等 §八 Q21–Q29 答复**（至少 Q21–Q23、Q24、Q25）。  
2. 不得在未开 `dual_core.enabled` 时改变 `co_present` 行为。  
3. 校验以 `validate_blueprint_v3_json` 为准；宿主加载与校验字段一致。  
4. P5 **不**与 P2–P4 同 PR。
