# RFC：后处理链（Post-Process Chain）— 预留

| 元数据 | 值 |
|--------|-----|
| 状态 | **Draft / 不在 v0.2 范围** |
| 关联 | [DISTRO_CAPABILITY_PROFILE.md](../kernel/DISTRO_CAPABILITY_PROFILE.md) `[post_process]` · [NAMING_CONVENTIONS.md](../NAMING_CONVENTIONS.md) §1.2 |
| 受众 | 内核 / 发行版集成方 |

---

## 1. 问题陈述

LLM 生成 **`reply`** 之后、返回用户之前，可能需要可插拔的**后处理**（格式化、安全过滤、TTS 分段、发行版 overlay 等）。今日部分逻辑硬编码在 `turn_pipeline/post.rs` 与设施模块中，缺少统一扩展点。

**本 RFC 仅预留命名与边界**；v0.2.x **不落地**运行时代码。

---

## 2. 术语（SSOT）

| 权威名 | English | 说明 |
|--------|---------|------|
| **后处理链** | **post-process chain** | LLM 输出 → 用户可见回复之间的有序步骤序列 |
| **内置后处理** | **built-in post-process** | 宿主 `turn_pipeline/post.rs` 中不可关闭的核心逻辑 |
| **发行版后处理 profile** | **distro post-process profile** | `distro.oclive.toml` → `[post_process].chain`（P4 草案字段） |

**消歧**：

- **不是**蓝图文件 `pipeline.ocblueprint` 的 `steps[]` DSL（已废弃，不作主路径调度）。
- **不是** `dual_pipeline` 实验核编排（见 [RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)）。
- **不是** 六宿主槽 `plugin_backends` / `slot_registry` 的后端替换。

---

## 3. 边界（草案）

```mermaid
flowchart LR
  LLM[LLM reply] --> BuiltIn[turn_pipeline/post.rs built-in]
  BuiltIn --> Chain{post-process chain}
  Chain --> User[SendMessageResponse.reply]
  Facilities[facility modules e.g. complex_emotion] -.-> BuiltIn
  Distro[distro.oclive.toml post_process.chain] -.-> Chain
```

| 层 | 职责 | 配置来源 |
|----|------|----------|
| `turn_pipeline/post.rs` | 会话写入、DTO 回填、与记忆/好感等已有副作用 | 代码 |
| **后处理链扩展点**（未实现） | 纯函数或 trait 链：输入 `reply` + 上下文 → 输出 `reply` | `distro.oclive.toml` / 未来 blueprint 只读段 |
| 设施模块 | 回合内叙事提示等，**在** Prompt 构建或 pre-LLM 阶段 | 蓝图 / 代码 |
| Experimental 核 | 双核降级前的实验步骤 | `pipeline.experimental` JSON |

---

## 4. 非目标（v0.2）

- 不新增 `post_process` trait 或插件槽位。
- 不扩展 blueprint v3 `runtime_config` 承载链定义。
- 不改变 `SendMessageResponse` 字段形状。

---

## 5. 落地前置条件（未来 PR）

1. 与 [DISTRO_CAPABILITY_PROFILE.md](../kernel/DISTRO_CAPABILITY_PROFILE.md) §3 `[post_process]` 字段对齐 Schema。
2. 明确与 `HostProfile`（P4）的合并优先级：发行版 > 角色包 > 会话。
3. OOCP 场景：至少一条「链 step 失败 → 降级为 built-in」黑盒用例。
4. Breaking 流程见 [BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md)。

---

## 6. 参考实现锚点（只读）

- 今日内置逻辑：`kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/post.rs`
- 命名 SSOT：[NAMING_CONVENTIONS.md](../NAMING_CONVENTIONS.md) §1.2「后处理链」
