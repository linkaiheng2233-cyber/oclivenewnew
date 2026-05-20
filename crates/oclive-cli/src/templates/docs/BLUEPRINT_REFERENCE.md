# 蓝图参考（`pipeline.ocblueprint`）

角色包可选携带 **`pipeline.ocblueprint`**（JSON），描述**纯内核定制**时的理想编排步骤图。本文件由 `oclive init` 写入生成项目，供无桌面宿主场景参考。

> **重要**：当前 **oclivenewnew 桌面宿主** 主路径已移除蓝图 DSL，首轮对话固定走 `process_message` 顺序编排。蓝图与 `oclive blueprint validate` 为**预备工具链**，不控制桌面版运行时。

## 文件形状

```json
{
  "schema_version": 1,
  "entry": "step_load",
  "steps": [
    { "id": "step_load", "type": "load_context", "next": "step_emotion" },
    { "id": "step_emotion", "type": "analyze_emotion", "next": "step_llm" },
    { "id": "step_llm", "type": "call_llm" }
  ]
}
```

## 字段

| 字段 | 必填 | 说明 |
|------|------|------|
| `schema_version` | 否 | 版本号，默认 0 |
| `entry` | 否 | 入口步骤 `id`，须存在于 `steps` |
| `steps` | 是 | 步骤数组，至少一项 |
| `steps[].id` | 是 | 唯一步骤标识 |
| `steps[].type` | 是 | 步骤类型（见下表） |
| `steps[].next` | 否 | 下一跳 `id`，须存在 |

## 已知步骤类型（内核枚举）

| `type` | 含义 |
|--------|------|
| `load_context` | 加载会话/角色上下文 |
| `analyze_emotion` | 用户情绪分析 |
| `detect_event` | 事件检测 |
| `retrieve_memory` | 记忆检索 |
| `build_prompt` | 组装 Prompt |
| `call_llm` | 调用主 LLM |
| `post_process` | 后处理（落库、情绪回写等） |

## 校验命令

在已安装 `oclive-cli` 的环境：

```bash
cargo run -p oclive-cli -- blueprint validate path/to/pipeline.ocblueprint
cargo run -p oclive-cli -- blueprint validate path/to/pipeline.ocblueprint --json
```

通过时退出码 0；失败时打印错误列表并以非零退出。

## 与 Monolith 的关系

- **蓝图**：运行时/设计期「步骤图」契约（JSON）。
- **Monolith**：编译期 `monolith.toml` 的 `weld_modules`，决定七槽是否静态焊接。

二者可并行用于定制内核，但互不替代。编排代码参考见 **`ORCHESTRATION_REFERENCE.md`**。
