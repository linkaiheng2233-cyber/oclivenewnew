# 蓝图目录布局（`blueprint/` · 拉取式 SSOT）

**状态**：架构约定（实验功能与 v3 双核可并行）；**宿主加载路径以源码为准**（今日仍读 `roles/{id}/pipeline.ocblueprint`）。  
**读者**：创作者、编写器、Cursor / Agent。  
**关联**：[ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) · [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md)

---

## 1. 目标

| 原则 | 说明 |
|------|------|
| **蓝图本体干净** | `pipeline.ocblueprint` 只保留 `schema_version`、`meta`（门面子集或指针）、`slot_registry`、`groups`、`runtime_config`、**`includes`**；禁止大段 Markdown、脚本、Comfy 式 `steps[]`。 |
| **文本与代码外置** | 性格长文、专家表单结果、LoRA 说明、修订历史 → **`blueprint/` 子目录或包根 `prompts/`**，由蓝图 **拉取（include）** 合并，不搅在 JSON 里。 |
| **专家设施独立** | 专家模型 UI 定义、revision、说明文档 → **`blueprint/expert/`**，**不**参与默认 `pack validate` 失败条件。 |
| **弃用类 ComfyUI** | 无节点图执行器；「按钮 → 表单 → 生成 revision → apply」即可。 |

---

## 2. 角色包目录（推荐）

```text
roles/{role_id}/
├── pipeline.ocblueprint      # 蓝图本体（瘦）· 宿主当前 SSOT 入口
├── blueprint/                # 蓝图相关卫星文件（本约定）
│   ├── README.md             # 本目录说明（可选）
│   ├── includes/             # 可被拉取的 JSON/YAML 片段（patch、片段 meta）
│   │   └── meta.personality.expert.json
│   ├── overlays/             # 叠加定义（不直接进本体）
│   │   └── expert.facility.blueprint
│   ├── revisions/            # 专家/向导 apply 前后快照（降级栈）
│   │   └── 20260520-143022.json
│   └── docs/                 # 给人看的说明（宿主可忽略）
│       └── expert-complex-emotion.md
├── prompts/                  # 角色文案（非蓝图；可被 includes 引用）
├── scenes/
├── core_personality.txt
├── ui.json
└── assets/
```

**角色内容**（`prompts/`、`scenes/`、`core_personality.txt`）留在包根，**不**挪进 `blueprint/`，避免与「引擎配置」混淆。

---

## 3. 瘦蓝图：`pipeline.ocblueprint`

### 3.1 允许出现在本体的键

| 键 | 说明 |
|----|------|
| `schema_version` | `2` 或 `3` |
| `meta` | 创作者门面 + 过渡期引擎字段（目标迁至 `runtime_config`） |
| `slot_registry` | 槽位实例（管理员） |
| `groups` | 架构图分组（可选） |
| `runtime_config` | v3 引擎策略（含 `expert_hints` 等实验键） |
| `pipeline` | v3 双核 `stable` / `experimental`（Proposed） |
| `includes` | **拉取清单**（见 §4） |
| `expert_overlay` | 可选指针：`active_revision`、`facility_path`（≤ 少量字段） |

### 3.2 禁止出现在本体

- 长字符串叙事、Markdown 正文、脚本、Base64
- `module_relations`、`steps`、`entry`（已有校验）
- 专家向导的完整 `answers` 大对象（应放 `blueprint/revisions/`）

---

## 4. 拉取：`includes[]`

蓝图在 **加载时**（或 `expert apply` 写盘后）按清单合并外置文件，而不是把外置内容粘贴进本体。

```json
{
  "schema_version": 2,
  "includes": [
    {
      "id": "expert-personality",
      "path": "blueprint/includes/meta.personality.expert.json",
      "target": "meta.personality",
      "mode": "merge"
    },
    {
      "id": "expert-prompt-fragment",
      "path": "prompts/personality_tune.md",
      "target": "runtime_config.expert_hints.prompt_fragment",
      "mode": "file_text"
    }
  ],
  "meta": {
    "id": "demo.role",
    "name": "Demo",
    "personality": { "warmth": 0.5 }
  },
  "slot_registry": {}
}
```

| `mode` | 行为 |
|--------|------|
| `merge` | JSON 深合并到 `target` 路径 |
| `replace` | 替换 `target` 整段 |
| `file_text` | 读取文本写入字符串字段（供 Prompt / 设施子模块） |

**路径规则**：相对 **`roles/{role_id}/`**；禁止 `..` 逃逸包根（实现时校验）。

**合并顺序**：按 `includes` 数组顺序；同 `target` 后项覆盖前项。  
**降级**：apply 前将当前生效片段写入 `blueprint/revisions/`；回滚 = 改 `expert_overlay.active_revision` 或恢复 revision 快照，**不必**在本体堆叠多版历史。

---

## 5. 专家模型（`blueprint/expert/` 可选别名）

与 [GITHUB_PLUGIN_INDEX_LINE.md](./GITHUB_PLUGIN_INDEX_LINE.md) 插件市场线无关；专指 **第 N 设施子模块** 配置向导。

| 文件 | 用途 |
|------|------|
| `blueprint/overlays/expert.facility.blueprint` | 按钮 / 表单 `nodes` 定义（UI 元数据） |
| `blueprint/revisions/*.json` | 用户填表结果 + `patches`；可打开给用户核对 |
| `blueprint/docs/*.md` | 说明文档；**不参与** `load_role` |

**apply 白名单**（合并目标）：`meta.personality`、`meta.life_*`、`prompts/*.md`、`runtime_config.expert_hints`。  
**默认禁止**：`slot_registry`（LoRA 写 llm 槽须「高级」二次确认）。

---

## 6. 校验与加载（今日 vs 目标）

| 项 | 今日 | 目标 |
|----|------|------|
| SSOT 路径 | `roles/{id}/pipeline.ocblueprint` | 可迁至 `blueprint/pipeline.ocblueprint`，根路径保留兼容或 symlink 文档 |
| `pack validate` | 只校验本体 JSON | `blueprint/includes` 可选 `oclive blueprint validate`；`blueprint/docs` 忽略 |
| `includes` 解析 | **已实现**（`load_blueprint_v2/v3_for_role_dir` + `resolve_blueprint_includes_lenient`） | 缺失卫星文件 warn 跳过 |
| 专家目录缺失 | 不阻塞加载 | 保持 |

实现前：专家材料放 `blueprint/`，**手动** merge 或 CLI `expert apply` 写入 `includes` 目标文件；本体仅增 `includes` 条目指向该文件。

---

## 7. 与编写器 / CLI

- **编写器**：「蓝图」视图编辑瘦本体 + `slot_registry`；「专家」视图编辑 `blueprint/overlays` + 向导；「角色」视图编辑 `prompts/` / `meta` 门面。
- **CLI**：`pack validate` 不变；新增（路线图）`oclive blueprint resolve`（打印合并后有效配置）、`oclive expert apply|rollback`。

---

## 8. 迁移建议

1. 新建 `roles/{id}/blueprint/` 子目录，外移冗长片段到 `includes/` 或 `revisions/`。
2. 在本体增加 `includes` 指向外置文件；删除本体中已外移的大块 JSON。
3. 校验通过后提交；旧包无 `blueprint/` 仍合法。

---

## 相关文档

| 主题 | 文档 |
|------|------|
| 角色 vs 蓝图职责 | [ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) |
| 角色包字段 | [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| 第 1 设施子模块 | [OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
