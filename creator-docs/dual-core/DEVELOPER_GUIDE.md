# 双核双态开发者指南

## 概述

**双核双态**在运行时提供两条路径：

| 核 | 来源 | 行为 |
|----|------|------|
| **稳定核** | 始终为 `process_co_present` | 今日默认单核编排 |
| **实验核** | `pipeline.experimental` | 按 DAG 顺序执行已实现的 `action`；失败则**静默降级**到稳定核 |

门控（须同时满足）：

```text
runtime_config.dual_core.enabled == true
AND pipeline.experimental 非空
```

否则与未开双核时 **零差异**。

---

## 启用双核

### 脚手架

```bash
cargo run -p oclive-cli -- init --dual-core --preset full -o ./my-kernel
```

生成 `roles/default/pipeline.ocblueprint`（`schema_version: 3`，含 `runtime_config` 与 `pipeline`）。

### 手写蓝图

1. 使用 `schema_version: 3`。
2. 设置 `runtime_config.dual_core.enabled: true`。
3. 填写 `pipeline.experimental`（见 [METHOD_REGISTRY.md](./METHOD_REGISTRY.md)）。
4. `pipeline.stable` 可作文档参考，**宿主不执行**。

校验：

```bash
cargo run -p oclive-cli -- pack validate --profile creator ./roles/your_role
```

---

## 编写 experimental pipeline

1. 在 `slot_registry` 为每个 `action` 准备合法 `registry_key`。
2. 实例 `type` 必须与 method 匹配（如 `emotion` + `analyze`）。
3. 用 `depends_on` 声明同 pipeline 内依赖；不得成环。
4. 建议末尾包含 `slot.<llm_key>.generate`，或由 Agent `process` 短路。

示例：

```json
"pipeline": {
  "stable": [],
  "experimental": [
    { "action": "slot.emotion.analyze", "depends_on": [] },
    { "action": "slot.memory.retrieve", "depends_on": ["slot.emotion.analyze"] },
    { "action": "slot.llm.generate", "depends_on": ["slot.memory.retrieve"] }
  ]
}
```

---

## 注册新槽位实例

在 `slot_registry` 增加实例即可；`action` 引用 **键名** 而非 type。v3 可选 `zone: "experimental"` 限制仅出现在实验 pipeline（校验规则见 blueprint v3）。

---

## 调试与降级

实验失败时用户 **看不到** 降级提示；须通过 `tracing` 排查。

### 如何查看降级日志

PowerShell（桌面宿主或 `oclivenewnew-tauri --api`）：

```powershell
$env:RUST_LOG = "info,oclive_dual_core=info"
# 或仅双核：$env:RUST_LOG = "oclive_dual_core=info"
```

典型日志序列（`target=oclive_dual_core`）：

| 级别 | 含义 |
|------|------|
| `INFO` 开始执行实验核，`step_count=N` | 进入实验 pipeline |
| `INFO` 实验核执行成功 | 实验步全部完成（含 Agent 短路或即将交稳定核收尾） |
| `WARN` 实验核在第 X 步失败: …，正在降级到稳定核 | 某步 `action` / method / 共景子阶段失败 |
| `INFO` 稳定核执行完成（降级模式） | 快照已回滚，`co_present` 已返回回复 |

过滤示例：

```powershell
# 若使用 tracing 默认 fmt 层，在终端中搜：
# oclive_dual_core
```

- OOCP 可选场景 `S13_dual_core_fallback`（`OCLIVE_OOCP_INCLUDE_S13=1`）。

### 快照回滚（实验前捕获）

| 字段 | 说明 |
|------|------|
| `narrative_hint` | 复杂情感叙事缓存 |
| `emotion_state` | `get_current_emotion` |
| `active_scene_id` | `get_user_presence_scene` / `set_user_presence_scene` |

实验步修改上述内存态后若失败，回滚再跑稳定核。

---

## 贡献新 method 映射

1. 在 `src-tauri/src/domain/dual_pipeline_registry.rs` 注册 `(type, method)`。
2. 在 `dual_pipeline_steps.rs` 实现与共景子步骤对齐的调用。
3. 更新 [METHOD_REGISTRY.md](./METHOD_REGISTRY.md) 与 `oclive explain`。
4. 补充单元测试。

---

## FAQ

**Q：`pipeline.stable` 会执行吗？**  
A：不会。稳定核恒为 `co_present`。

**Q：`enabled=true` 但 `experimental=[]`？**  
A：视为未开双核，走 `co_present`。

**Q：Monolith 编译还能用双核吗？**  
A：`oclive init --monolith --dual-core` 在 `monolith.toml` 写入 `[dual_core]`；链入主仓时仍由 `DualPipelineRunner` 调度。

**Q：创作者包能默认开双核吗？**  
A：分发包勿单独 `enabled: true`；见 [ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)。

---

## 英文

[DEVELOPER_GUIDE.en.md](./DEVELOPER_GUIDE.en.md)
