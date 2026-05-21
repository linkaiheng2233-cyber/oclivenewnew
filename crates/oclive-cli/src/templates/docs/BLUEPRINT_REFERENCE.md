# 蓝图参考（`pipeline.ocblueprint` v2）

> **已废弃**：下文 **schema_version 1 + `steps[]` DSL** 仅作史料；CLI `oclive blueprint validate` **仅接受 v2**（`meta` + `slot_registry`）。桌面宿主编排见 `process_message` → `co_present`，不执行蓝图步骤图。

## v2 形状（权威）

见仓库 [`creator-docs/role-pack/ROLE_PACK_SPEC.md`](../../../../creator-docs/role-pack/ROLE_PACK_SPEC.md) 与 [`handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md`](../../../../handoff/BLUEPRINT_V2_IMPLEMENTATION_PLAN.md)。

```json
{
  "schema_version": 2,
  "meta": {
    "id": "demo",
    "name": "Demo",
    "version": "0.1.0",
    "author": "t",
    "description": "d",
    "personality": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
    "relations": {
      "friend": { "initial_favorability": 50.0, "favor_multiplier": 1.0 }
    },
    "default_relation": "friend"
  },
  "slot_registry": {
    "llm": { "type": "llm", "label": "LLM", "backend": "ollama", "position": 0 }
  }
}
```

校验：`oclive blueprint validate <path>` · `oclive pack validate <role-dir>`（默认 v2）。

---

## 附录：legacy steps[] DSL（勿新建）

以下 **schema_version 1** 示例**不得**用于新包；校验将拒绝 `steps` / `entry` 顶层键。

```json
{
  "schema_version": 1,
  "entry": "step_load",
  "steps": [
    { "id": "step_load", "type": "load_context", "next": "step_emotion" }
  ]
}
```

历史步骤类型表见 git 史中的旧版 BLUEPRINT_REFERENCE；运行时不再实现。
