# 蓝图目录布局（`blueprint/` · 拉取式 SSOT）

**状态**：Stable v4 扩展外壳已落地；v2 兼容、v3 双核 Beta 并行；**宿主加载路径以源码为准**（今日仍读 `distros/chat-pro/roles/{id}/pipeline.ocblueprint`）。
**读者**：创作者、编写器、Cursor / Agent。  
**关联**：[ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) · [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md)

---

## 1. 目标

| 原则 | 说明 |
|------|------|
| **蓝图本体干净** | `pipeline.ocblueprint` 只保留 `schema_version`、`meta`（门面子集或指针）、`slot_registry`、`groups`、`runtime_config`、**`includes`** 与 v4 最小 **`extensions`** 外壳；禁止大段 Markdown、脚本、Comfy 式 `steps[]`。 |
| **文本与代码外置** | 性格长文与角色文案留在包根 `core_personality.txt` / `prompts/`，由编写器或 Prompt 链管理；专家表单结果、LoRA 说明、修订历史进入 **`blueprint/` 子目录**，只有契约白名单内的 JSON 片段由蓝图 **拉取（include）** 合并。 |
| **专家模型设施子模块（蓝图侧）独立** | 专家路由 UI、revision、说明文档 → **`blueprint/expert/`**（或 `includes/expert_routing.json`），**不**参与默认 `pack validate` 失败条件。 |
| **第三方扩展载荷隔离** | Stable v4 只在本体保留最小 `extensions` 外壳；载荷进入 `blueprint/extensions/<instance>/` 并由扩展作者维护。v2/v3 严格拒绝该字段。 |
| **弃用类 ComfyUI** | 无节点图执行器；「按钮 → 表单 → 生成 revision → apply」即可。 |

---

## 2. 角色包目录（推荐）

```text
distros/chat-pro/roles/{role_id}/
├── pipeline.ocblueprint      # 蓝图本体（瘦）· 宿主当前 SSOT 入口
├── blueprint/                # 蓝图相关卫星文件（本约定）
│   ├── README.md             # 本目录说明（可选）
│   ├── includes/             # 可被拉取的 JSON 片段（patch、片段 meta）
│   │   └── meta.personality.expert.json
│   ├── overlays/             # 叠加定义（不直接进本体）
│   │   └── expert.facility.blueprint
│   ├── revisions/            # 专家/向导 apply 前后快照（降级栈）
│   │   └── 20260520-143022.json
│   ├── extensions/           # v4 第三方扩展载荷；核心只校验/保留，不解释载荷
│   │   └── com.example.live2d.main/
│   │       └── config.json
│   └── docs/                 # 给人看的说明（宿主可忽略）
│       └── expert-complex-emotion.md
├── prompts/                  # 角色文案（非蓝图；由编写器/Prompt 链直接管理）
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
| `schema_version` | `2`、冻结的双核 Beta `3`，或 Stable `4` |
| `meta` | 创作者门面；v2 旧包可含兼容引擎字段，Stable v4 禁止与 `runtime_config` 双写 |
| `slot_registry` | 槽位实例（管理员） |
| `groups` | 架构图分组（可选） |
| `runtime_config` | v4 Stable 引擎策略；v3 仅为双核 Beta 兼容；未知子键按严格契约拒绝 |
| `pipeline` | v3 双核 `stable` / `experimental`（Proposed） |
| `includes` | **拉取清单**（见 §4） |
| `expert_overlay` | 可选指针：`active_revision`、`facility_path`（≤ 少量字段） |
| `extensions` | **仅 v4**：最小声明外壳，载荷由安全 `config_ref` 外置；当前核心不执行未知 Provider |

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
    }
  ],
  "meta": {
    "id": "demo.role",
    "name": "Demo",
    "personality": { "warmth": 0.5 }
  },
  "slot_registry": {
    "llm": {
      "type": "llm",
      "label": "LLM",
      "backend": "ollama",
      "position": 1
    }
  }
}
```

| `mode` | 行为 |
|--------|------|
| `merge` | JSON 深合并到 `target` 路径 |
| `replace` | 替换 `target` 整段 |

**路径规则**：相对 **`distros/chat-pro/roles/{role_id}/`**，只使用 ASCII 字母、数字、`_`、`.`、`/`、`-`；禁止 `..`、绝对路径、反斜杠、空路径段和符号链接逃逸包根。

**合并顺序**：按 `includes` 数组顺序；同 `target` 后项覆盖前项。  
**失败语义**：已声明文件缺失、不可读、JSON 非法或合并后蓝图无效时，`pack validate` 与角色激活均失败；best-effort helper 只可用于不激活的预览。
**降级**：apply 前将当前生效片段写入 `blueprint/revisions/`；回滚 = 改 `expert_overlay.active_revision` 或恢复 revision 快照，**不必**在本体堆叠多版历史。

---

### 4.1 通用扩展载荷（Stable v4）

通用扩展不复用 `includes` 把任意载荷合并进核心结构。Stable v4 由 `extensions.<instance>.config_ref` 指向 `blueprint/extensions/<instance>/config.json`：

- 核心只校验外壳、路径、必需/可选语义并保持 round-trip。
- 扩展 Provider 校验和解释自己的载荷。
- 蓝图不得写显存卸载、进程终止或固定资源分配命令。
- 资源敏感 Provider 通过 Resource Adapter 接入宿主统一协调；不消耗共享资源的扩展无需实现。
- v2/v3 严格拒绝该外壳；v4 声明进入宿主只读能力计划：消费者/Provider/依赖/权限齐备才为 ready，可选缺失降级，必需缺失阻止角色激活。

字段与 `ExecutionPlan` / Resource Coordinator 分责见 [RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)。

---

## 5. 专家模型设施子模块（`blueprint/expert/` 可选别名）

与 [GITHUB_PLUGIN_INDEX_LINE.md](./GITHUB_PLUGIN_INDEX_LINE.md) 插件市场线无关；专指 **第 2 设施子模块**（**专家模型**专名 · 默认实现 **专家路由**）的蓝图/UI 材料。命名见 [OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md#设施模块命名规范规定)。

| 文件 | 用途 |
|------|------|
| `blueprint/overlays/expert.facility.blueprint` | 按钮 / 表单 `nodes` 定义（UI 元数据） |
| `blueprint/revisions/*.json` | 用户填表结果 + `patches`；可打开给用户核对 |
| `blueprint/docs/*.md` | 说明文档；**不参与** `load_role` |

**当前 `includes` 白名单**：`meta.personality`、`meta.life_trajectory`、`meta.life_schedule`、`expert_overlay`、`slot_registry.<key>`。`prompts/*.md` 由编写器直接管理，不通过 JSON include；`expert_routing.json` 由专家设施自己的 loader 读取。
**专家 apply UI 默认禁止**直接改 `slot_registry`（LoRA 写 llm 槽须「高级」二次确认）；这是创作工具策略，不改变上面的运行时白名单。

---

## 6. 校验与加载（今日 vs 目标）

| 项 | 今日 | 目标 |
|----|------|------|
| SSOT 路径 | `distros/chat-pro/roles/{id}/pipeline.ocblueprint` | 可迁至 `blueprint/pipeline.ocblueprint`，根路径保留兼容或 symlink 文档 |
| `pack validate` | 校验本体、已声明 include 文件及合并后的有效蓝图；`blueprint/docs` 忽略 | 编写器复用同一 Rust/WASM 校验链并显示结构化定位 |
| `includes` 解析 | **已实现**；现行目录校验要求文件存在，mode 仅 `merge` / `replace`，缺失会阻止 load | 若未来改为可选 include，须新增显式 required/optional 契约，不能靠 lenient helper 猜测 |
| `extensions` | **v4 外壳与能力计划已实现**：声明/路径/载荷 JSON 校验、round-trip、目录 Provider Registry、required/optional 激活门禁及跨发行版结构化诊断 | Provider 自有载荷语义校验；Resource Coordinator 与资源适配 |
| 专家目录缺失 | 不阻塞加载 | 保持 |

实现前：专家人格等白名单片段放 `blueprint/`，**手动** merge 或由 CLI `expert apply` 写入对应 `includes` 目标文件；`expert_routing.json` 仍由专家设施专用 loader 读取。

---

## 7. 与编写器 / CLI

- **编写器**：「蓝图」视图编辑瘦本体 + `slot_registry`；「专家」视图编辑 `blueprint/overlays` + 向导；「角色」视图编辑 `prompts/` / `meta` 门面。
- **CLI**：`pack validate` 不变；新增（路线图）`oclive blueprint resolve`（打印合并后有效配置）、`oclive expert apply|rollback`。

---

## 8. 迁移建议

1. 新建 `distros/chat-pro/roles/{id}/blueprint/` 子目录，外移冗长片段到 `includes/` 或 `revisions/`。
2. 在本体增加 `includes` 指向外置文件；删除本体中已外移的大块 JSON。
3. 校验通过后提交；旧包无 `blueprint/` 仍合法。

---

## 相关文档

| 主题 | 文档 |
|------|------|
| 角色 vs 蓝图职责 | [ROLE_PACK_BOUNDARY.md](./ROLE_PACK_BOUNDARY.md) |
| 角色包字段 | [ROLE_PACK_SPEC.md](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |
| 第 1–2 设施子模块（复杂情感 / 专家模型） | [OCLIVE_ARCHITECTURE_OVERVIEW.md](../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) |
| 通用蓝图扩展 / 资源协调 | [RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md) |
