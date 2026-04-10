# 角色包「用户自定义」创作者教学

本文面向**准备自制或维护角色包**的创作者，说明如何在包内配置**用户身份、展示名、好感与记忆相关选项**，使玩家在应用里看到符合你设计的关系称谓与初始体验。  
更细的场景目录与 `scene.json` 写法，请参阅 [《角色包场景系统 — 创作者使用指南》](./CREATOR_SCENE_GUIDE.md)；身份字段与校验规则的细节摘要见 [《创作者说明：用户身份与初始好感》](./CREATOR_USER_RELATIONS.md)。

---

## 1. 角色包是什么？

**角色包**是应用从磁盘加载的一个文件夹，路径形如：

```text
roles/<角色 id>/
```

其中 `<角色 id>` 必须与 `manifest.json` 顶层的 **`id`** 一致，且建议使用**稳定的小写英文**（可含数字、下划线），避免随意改名，以免与存档、导入导出路径冲突。

---

## 2. 推荐目录结构

```text
roles/<角色 id>/
├── manifest.json           # 必填：角色元数据、用户身份、场景列表、记忆策略等
├── core_personality.txt    # 强烈建议：核心性格档案（包内长文），供主模型阅读；运行时不可由模型改写
├── config.json             # 可选：虚拟时间等
└── scenes/                 # 推荐：按场景分子目录
    ├── <scene_id>/
    │   ├── scene.json
    │   └── description.txt
    └── ...
```

加载时，程序会读取 `manifest.json` 并校验；再通过 `core_personality.txt`、`scenes/` 补全对话与展示信息。若 `settings.json` 中 **`evolution.personality_source`** 为 **`profile`**，对话后的 **可变性格档案**仅存本地数据库并由模型维护，包内不可手写；详见 **[docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)**。

---

## 3. `manifest.json` 里与「用户自定义」最相关的部分

### 3.1 顶层常用字段（与展示相关）

| 字段 | 说明 |
|------|------|
| `id` | 角色唯一标识，与文件夹名一致。 |
| `name` | 角色在列表里显示的名称（可与内部称呼不同）。 |
| `version` / `author` / `description` | 版本、作者、简介。 |
| `model` | 可选；指定本角色默认使用的 Ollama 模型名（也可在应用环境或界面中选择）。 |
| `default_personality` | 可选；七维性格初值（倔强、黏人、敏感、强势、宽容、话多、温暖），每项约 0～1。`profile` 人格来源下多为视图，仍建议填写。 |
| `scenes` | 场景 id 列表；会与 `scenes/` 下子目录**合并去重**，详见场景指南。 |

### 3.2 `user_relations`：玩家「身份」的核心配置

每个**用户身份**对应 `user_relations` 里的一个**键值对**：

- **键（key）**：程序内部使用的 **英文关系 id**（如 `friend`、`classmate`、`parent`）。存档、接口、默认关系都引用它，**发布后请尽量保持稳定**。
- **值（对象）**：该身份下的提示、倍率与初始好感等。

示例：

```json
"user_relations": {
  "classmate": {
    "display_name": "同学",
    "prompt_hint": "你和角色是同班同学，说话随意，会聊功课与课间琐事",
    "favor_multiplier": 1.0,
    "initial_favorability": 30
  },
  "parent": {
    "display_name": "父母",
    "prompt_hint": "你扮演孩子的家长，角色会嘴硬但在意你们的感受",
    "favor_multiplier": 1.15,
    "initial_favorability": 70
  }
}
```

字段含义简述：

| 字段 | 是否必填 | 说明 |
|------|----------|------|
| `display_name` | 否 | **界面里展示的名称**（中文或其它文案）。不写或留空时，展示会退回为英文键；应用对常见英文键（如 `classmate`、`friend`）还提供**界面级中文 fallback**，方便测试，但**正式作品仍建议写明 `display_name`**，以免与默认翻译不一致。 |
| `prompt_hint` | 否 | 给模型看的**关系提示**，说明当前身份下用户与角色如何相处。 |
| `favor_multiplier` | 有默认值 | 好感变化倍率，须为**正数**；加载时会校验。 |
| `initial_favorability` | 有默认值 | 该身份下**首次建立关系**时的初始好感，范围 **0～100**；加载时会校验。 |

### 3.3 `default_relation`

填写一个**必须存在于 `user_relations` 键**中的英文 id，表示新对话或未单独指定时的默认身份。若写错键名，角色包**无法加载**。

### 3.4 `memory_config` 与场景一致

若使用 `memory_config.topic_weights`，其**顶层键必须是场景 id**，且该场景须出现在 `manifest.scenes` 或 `scenes/` 子目录合并后的列表中，否则加载会失败并提示**中文错误信息**。详见 [《创作者说明：用户身份与初始好感》](./CREATOR_USER_RELATIONS.md)。

### 3.5 `evolution`（可选）

与事件影响、**人格来源**（`personality_source`）、可变档案更新步长（`max_change_per_event`）等相关；可按作品节奏调整，缺省亦有合理默认。摘要见 [roles/README_MANIFEST.md](../../roles/README_MANIFEST.md) §5.3。

---

## 4. `core_personality.txt`：核心性格档案与用户身份的配合

`core_personality.txt` 是包内 **核心性格档案**：描述**角色本身**是谁、如何说话、有哪些禁区；运行时 **不得**由模型改写该正文。  
**用户身份**（父母 / 同学 / 恋人等）主要在 `user_relations` 与 `prompt_hint` 里定义。两者应一致：例如「用户扮演父母」的包，档案侧应写子女视角，避免与 `prompt_hint` 冲突。

---

## 5. 加载校验与排错

在从目录加载角色时，程序会对 `manifest.json` 做校验（非空 id、非空 `name`、`user_relations` 非空、`default_relation` 合法、`topic_weights` 与场景一致、数值合法等）。**失败时会返回明确的中文说明**，请按提示修改 JSON 后重试。

---

## 6. 创作者工作流建议

1. 新建 `roles/<你的角色 id>/`，先写好 **`manifest.json`** 的 `id`、`name`、`user_relations`、`default_relation`。  
2. 为每个身份写好 **`display_name`** 与 **`prompt_hint`**，并设好 **`initial_favorability`** 与 **`favor_multiplier`**。  
3. 配置 **`scenes`** 与 `scenes/<scene_id>/`，需要时再填 **`topic_weights`**。  
4. 撰写 **`core_personality.txt`**（核心性格档案），与身份设定对齐；若使用 **`profile`** 人格来源，在 `settings.json` 的 **`evolution`** 中配置 `max_change_per_event` 等，勿尝试在包内手写运行时可变档案。  
5. 在应用内加载角色，检查**身份下拉框**称谓、场景列表与对话是否符合预期。  

---

## 7. 延伸阅读

- [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md) — 核心/可变档案与 `personality_source` 设计轴心。  
- [《创作者说明：用户身份与初始好感》](./CREATOR_USER_RELATIONS.md) — `display_name`、`default_relation`、好感与 `topic_weights` 校验要点。  
- [《角色包场景系统 — 创作者使用指南》](./CREATOR_SCENE_GUIDE.md) — 场景目录、`scene.json`、`description.txt` 与场景切换。  

若在遵守上述规范的前提下仍遇到加载失败或界面显示异常，请把**完整报错文案**与 `manifest.json` 相关片段一并反馈，便于定位问题。
