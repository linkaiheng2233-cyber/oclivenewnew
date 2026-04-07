# 角色包 `manifest.json` 与 `settings.json`（创作者）

- **`manifest.json`** 路径：**`roles/{角色id}/manifest.json`**，与角色文件夹同名。门面与主要契约：**id、展示信息、七维、`scenes`、`user_relations`、`default_relation`** 等。完整示例见 **`manifest.template.json`**。
- **`settings.json`**（**可选**）路径：**`roles/{角色id}/settings.json`**。进阶引擎向配置：**`model`、`evolution`、`identity_binding`、`memory_config`**。完整示例见 **`settings.template.json`**。  
**推荐**创作者新包采用「manifest 门面 + settings 引擎」；应用内保存角色时也会写出这两份文件（manifest 存根 + settings 完整引擎段）。

**标准 JSON 不支持 `//` 或 `/* */` 注释**；说明性文字请用 **以下划线 `_` 开头的键**（加载时会**忽略**，不报错）。

---

## 在 oclive 中导入角色包（`.ocpak` / `.zip` / 文件夹）

运行时主界面提供 **「导入压缩包」** 与 **「从文件夹导入」**：

| 来源 | 要求 |
|------|------|
| **`.ocpak` 或 `.zip`** | 标准 ZIP 容器，解压后须与 **`roles/{角色id}/`** 目录结构一致（根目录或唯一子目录下含 `manifest.json`，与编写器导出一致）。 |
| **已解压目录** | 直接选择等同于 **`roles/{角色id}/`** 的那一文件夹（内含 `manifest.json`）；无需再打包。 |

导入前会读取 `manifest.json` 做预览；若本地已存在相同 **角色 ID**，会提示是否覆盖。与手动复制到 `roles/` 相比，应用内导入会走同一套校验与进度反馈。

**压缩包内多路径**：预览时优先 **`manifest.json`（ZIP 根）**，其次 **`{单层目录}/manifest.json`**，再才是更深路径下的 `manifest.json`（与 oclive 导出布局一致）。手工验收清单见 **[TESTING_ROLE_PACK_IMPORT.md](TESTING_ROLE_PACK_IMPORT.md)**。

---

## manifest 与 settings.json（加载顺序）

与源码一致（`RoleStorage::load_role_from_dir`、`DiskRoleSettings::apply_to_manifest`）：

1. 读取并解析 **`manifest.json`** → 内部 `DiskRoleManifest`。
2. 若存在 **`settings.json`** → 解析后**按字段覆盖** manifest 中同名项（仅 settings 里出现的字段参与覆盖）。
3. 合并场景 id、执行 **`validate_disk_manifest`**，再转为运行时 **`Role`**。

**兼容**：若角色目录**只有** `manifest.json`，且其中仍包含模型/演化/记忆等字段（旧版单文件），行为与过去一致，无需 `settings.json`。

**保存写回**：应用内保存时，`manifest.json` 中引擎相关字段为存根默认值，真实数值写在 **`settings.json`**。

---

## 开关与模式（先看这里）

与「玩家身份 × 场景」相关的模式开关字段名为 **`identity_binding`**（**推荐写在 `settings.json`**，亦可写在 manifest；若两处都有，**以 settings 为准**）。

| 字段 | 取值 | 效果 |
|------|------|------|
| **`identity_binding`** | **`global`** | 全剧 **一条** 玩家身份，**换场景不改变**「你是谁」（顶栏用全局身份）。 |
| | **`per_scene`** | **不同场景可不同身份**；省略不写时 manifest 反序列化默认为 **`per_scene`**（与旧版一致）。 |

更细的语义与玩家侧表现见 **`handoff/21_CREATOR_IDENTITY_BINDING.md`**。

---

## 用户叙事场景 vs 角色所在场景（双线 / 同行）

引擎区分两条线，避免「改顶栏叙事」就等价于拖角色一起走：

| 存储 / 字段 | 含义 |
|-------------|------|
| **`role_runtime.user_presence_scene`** | 用户当前对话与叙事视角；每条 **`send_message`** 会用请求里的 `scene_id` 同步；也可单独调用 **`set_user_presence_scene`**（顶栏选「仅我过去」时）。 |
| **`role_runtime.current_scene`** | 角色在故事世界中的位置；仅 **`switch_scene` 且 `together: true`（默认）**、或下文 **`autonomous_scene`** 命中规则时更新。 |

- **异地判定**：**`send_message` 的 `scene_id`**（叙事）与 **`current_scene`**（角色）不一致 → 走异地线（占位或异地心声，与开关一致）。与旧版「只认一个场景」相比，叙事 id 以请求与 `user_presence_scene` 为准，**不再**以前端强行把 UI 对齐 `current_scene` 为唯一来源。
- **玩家侧**：顶栏「叙事」场景、位移条与邀请同行确认中，可选 **仅更新叙事** 或 **同行前往**（角色与用户同场景）。

### `settings.json` 可选：`autonomous_scene`（虚拟时间驱动角色位移，二期）

在 **`jump_time`** 写入新虚拟时间之后，按 **`on_virtual_time` 数组顺序**匹配**第一条**满足以下条件的规则，将 **`current_scene`** 更新为 `to_scene`（**不**修改 `user_presence_scene`）：当前 `current_scene == when_scene`；本地小时 `hour` 落在 **`[hour_start, hour_end)`**（若 `hour_end < hour_start` 则视为跨午夜窗口）；`to_scene` 在角色包场景列表中，且该场景的 **`scene.json` → `time_windows`** 在 `storage.is_scene_time_allowed` 下允许当前虚拟时间切入。

**玩家侧**：若规则生效，主界面左下角会显示可关闭的系统提示（叙事顶栏不会自动跟随，避免误以为「你也到了该场景」）。

| 字段 | 说明 |
|------|------|
| **`on_virtual_time`** | 对象数组，每项含 **`when_scene`**、**`hour_start`**、**`hour_end`**（0–23）、**`to_scene`** |

---

## 异地「生活轨迹 / 心声」（`manifest.life_trajectory` + `settings.remote_presence` + 场景素材）

当 **`send_message` 的 `scene_id`**（用户叙事场景，并与 **`role_runtime.user_presence_scene`** 对齐）与数据库 **`role_runtime.current_scene`**（角色所在场景）**不一致**时，后端视为「异地」。

- **应用内开关**「异地心声」关闭 + 异地：只返回**占位文案**（可配置），**不**写入短期记忆 / 事件 / 好感事务，避免无对话却涨好感。
- **开关开启 + 异地：一次专用 LLM**，以生活轨迹与内心独白为主；正文由模型**依人设延伸现编**，并以 `summary` / `summary_lines` 与场景 `away_life*` 为**参考**（化用情境，非照抄）；仍走好感等持久化（事件列表对本回合返回空以降低噪声）。若主 LLM 调用失败，引擎会回退到与共景相同的「备用短回复」策略，并在界面轻提示「本次为备用回复」。
- **`stub_ooc` / `stub_messages`** 仅在**关闭**「异地心声」、走异地占位时使用；**不参与**异地心声 LLM 的正文生成。

### `manifest.json` 可选块：`life_trajectory`

创作者可规定异地时的语气、结构与示例；**开启异地心声时**，引擎仍用专用 LLM **现编**正文（延伸人设 + 参考下方素材与总述）。回复是否带固定括注、是否用轮换短句作**关异地心声时的占位**，**由创作者在包内自行决定**；下列字段只是引擎提供的配置方式，并非强制「唯一正确」形态。

| 字段 | 说明 |
|------|------|
| **`summary`** | 可选字符串；与 `summary_lines` 二选一（或并存时以 `summary_lines` 优先）。创作者对异地时角色节奏与碎碎念的说明，注入异地心声 LLM。 |
| **`summary_lines`** | 可选字符串数组；多段总述，加载时用空行拼接为一段，**在 manifest 里分行书写更易读**；与 `summary` 同时存在且本数组非空时优先使用本数组。 |
| **`stub_ooc`** | 可选；若与 **`stub_messages`** 联用，引擎会把「固定括注 + 中文逗号 + 一条旁白」拼成关异地心声时的占位（旁白在 `stub_messages` 轮换）。**不需要这种固定结构时**，不配置 `stub_ooc`，仅用 `stub_messages` 写整段即可。 |
| **`stub_messages`** | 字符串数组。未配置 `stub_ooc`：**整段**占位轮换（创作者爱写多长都可以）。配置了 `stub_ooc`：数组里为**仅旁白句**（轮换）。**旧包**若仍把整段写在 `settings.json` 的 `remote_presence.stub_messages`，加载时仍有效。 |

### `settings.json` 可选块：`remote_presence`（仅模式开关）

| 字段 | 说明 |
|------|------|
| **`default_enabled`** | 可选布尔；UI 可提示「包作者建议默认勾选异地心声」；**持久化仍以用户开关（数据库）为准**。 |

### 场景素材（角色在 **`current_scene`**、用户从**另一场景**发消息时注入）

优先级（`RoleStorage::away_life_material`）：

1. **`scenes/{scene_id}/away_life.txt`**（长文，与 `description.txt` 同级）
2. 否则 **`scene.json`** 里 **`away_life_by_user_scene`**：键为**用户上下文场景 id**，值为该组合下的覆盖文案
3. 否则 **`away_life_notes`**：字符串数组合并为一段

入库前单段素材在存储层会按 UTF-8 字符数**截断到约 8000 字**；注入异地心声主 LLM 时还会再截到约 **4000 字**以控制上下文长度（过长会带「已截断」提示）。

示例见 **`roles/mumu`**（`manifest.json` 的 `life_trajectory`、`settings.json` 的 `remote_presence.default_enabled`、`scenes/home/scene.json`、`scenes/company/away_life.txt`、**`knowledge/*.md` 世界观示例**）。

### `manifest.json` 可选块：`life_schedule`（虚拟时间日程 / 生活轨迹引擎）

与上文的 **`life_trajectory`**（异地心声气质与占位）不同：**`life_schedule`** 按 **虚拟时间** 推断角色「此刻在做什么」，供对话提示与 UI 一行状态展示；**不**自动修改好感数值。

| 字段 | 说明 |
|------|------|
| **`timezone_offset_minutes`** | 可选。相对 UTC 的**分钟**偏移，用于把 `role_runtime.virtual_time_ms` 换算成角色本地**星期几 + 时刻**；省略则按 UTC。例：东八区 `480`。 |
| **`entries`** | 对象数组；**按顺序**匹配，**首条命中**即采用（与 `autonomous_scene.on_virtual_time` 类似）。 |

每条 **`entries[]`**：

| 字段 | 说明 |
|------|------|
| **`weekday`** | `1` = 周一 … `7` = 周日（与引擎内部星期编号一致）。 |
| **`time_start` / `time_end`** | `HH:MM`（24 小时制）。若 **`time_end` &lt; `time_start`**，表示时段**跨午夜**（左闭右开 `[start, end)`）。 |
| **`activity_id`** | 机器可读键，如 `work` / `school`。 |
| **`label`** | 人类可读短标签，注入提示与 UI。 |
| **`preferred_scene_id`** | 可选；倾向的场景 id，**须**出现在本角色合并后的场景列表中。 |
| **`availability`** | 可选：`busy` / `distracted` / `free`，映射为忙碌度供模型与 UI 参考。 |

校验在合并 `settings.json` 后执行（`validate_disk_manifest`）：未知场景 id、非法时刻、空 `activity_id`/`label` 等会报错。

仓库示例：**`roles/mumu/manifest.json`** 内含片段。

---

## 其它开关与调试入口（聚合）

下列项**不在** `manifest.json` 顶层与 `identity_binding` 同一层级，但都属于「调行为 / 省 LLM / 控场景」时常改的地方；集中列出方便创作者**对照代码排查**（路径以仓库为准）。

### A. 场景 `roles/{id}/scenes/{scene_id}/scene.json`（随角色包）

| 配置 | 作用 |
|------|------|
| **`time_windows`** | **空数组**：任意**虚拟时间**下，该场景都可作为自动/手动切换目标。**非空**：仅当虚拟时间落在 `start`～`end`（`HH:MM`）任一窗口内，才允许切入（见 `storage.is_scene_time_allowed`，与 `chat_engine/scene` 一致）。 |
| **`keywords` / `events`** | 与位移动词一起参与「位移意图」规则命中。**实际移动角色**以用户确认后 **`switch_scene`（`together: true`）** 为准；仅改叙事用 **`set_user_presence_scene`** 或 `switch_scene` **`together: false`**。`send_message` 不会自动写入 `current_scene`，可返回 **`offer_destination_picker`**（选目的地）或 **`offer_together_travel`**（邀请同行确认）。 |
| **`away_life_notes` / `away_life_by_user_scene`** | 异地心声素材（与 `away_life.txt` 二选一或组合）；见上文「异地生活轨迹」。 |

#### 场景叙事与出场（设定延伸）

引擎**不**强制「某场景里角色是否在场」：多场景只切换当前 `scene_id`，叙事上「哪些场合谁会出现、谁不应出现」由创作者在角色包里体现，**不是**引擎硬规则（除非你后续单独做「禁止对话」类产品能力）。

常见写法：

- 在 **`scenes/{scene_id}/description.txt`**、**`scene.json`** 或欢迎语里约定场合、在场人物与边界（例如家人通常不出现在公司等）；
- 用 **`time_windows`** 约束「只有某时段才合理出现在该地点」；
- 与 **`identity_binding`**、身份文案配合，区分「场合」与「你是谁」（后者见 **`handoff/21_CREATOR_IDENTITY_BINDING.md`**）。

### B. 应用策略 `src-tauri/config/policy.toml`（**整应用**，非单个角色包）

| 概念 | 说明 |
|------|------|
| **`default_profile`** | 未在 `scene_bindings` 中出现的场景使用的策略档名。 |
| **`[profiles.*]`** | 各档下的 `emotion` / `memory`（如 neutral 持有、记忆 FIFO 上限等）。 |
| **`[scene_bindings]`** | 场景 id → 使用哪一套 `profiles` 名。 |

### C. 环境变量（本机调试，**不写进** manifest）

| 变量 | 默认倾向 | 作用 |
|------|------------|------|
| **`OCLIVE_EVENT_IMPACT_LLM`** | 开启 | 设为 `0` / `false` / `off` / `no` 时**关闭**事件影响 LLM，回退规则 `EventDetector`（`event_impact_ai.rs`）。 |
| **`OCLIVE_PORTRAIT_EMOTION_LLM`** | 开启 | 同上关闭时**跳过**立绘第二次 LLM，仅用启发式（`portrait_emotion_engine.rs`）。 |
| **`POLICY_EMOTION_NEUTRAL_HOLD_ENABLED`** | 读 toml 再可被覆盖 | 覆盖 `policy.toml` 中 emotion.neutral_hold |
| **`POLICY_EMOTION_LOW_CONFIDENCE_HOLD_THRESHOLD`** | 同上 | 浮点阈值 |
| **`POLICY_MEMORY_IGNORE_SINGLE_CHAR_FILTER`** | 同上 | 是否忽略单字记忆等 |
| **`POLICY_MEMORY_DEFAULT_IMPORTANCE`** | 同上 | 记忆默认重要性 |
| **`POLICY_MEMORY_FIFO_LIMIT`** | 同上 | 短期记忆条数上限等 |
| **`OCLIVE_LLM_TEMPERATURE`** | `0.8` | 主对话采样温度（`llm_params.rs`） |
| **`OCLIVE_LLM_TOP_P`** | `0.9` | 主对话 top_p |
| **`OCLIVE_LLM_TAG_TEMPERATURE`** | `0.28` | 标签/短输出类任务 |
| **`OCLIVE_LLM_TAG_TOP_P`** | `0.85` | 同上 |
| **`OLLAMA_BASE_URL`** | `http://localhost:11434` | Ollama API |
| **`OLLAMA_MODEL`** | `qwen2.5:7b` | 全局默认模型（**角色 `settings.json` / `manifest.json` 的 `model` 仍优先**） |
| **`OCLIVE_ROLES_DIR`** | 无 | 指向外部 `roles` 根目录，便于不拷仓库就换包 |

### D. 怎么选用

| 目的 | 优先改 |
|------|--------|
| 身份是否随场景变 | **`identity_binding`**（见上节；推荐 **`settings.json`**） |
| 某场景是否「只有晚上能去」 | **`scene.json` 的 `time_windows`** |
| 记忆/情绪策略随场景变 | **`policy.toml` 的 `scene_bindings` + profiles** |
| 本机对照规则、减 LLM 调用 | **`OCLIVE_EVENT_IMPACT_LLM` / `OCLIVE_PORTRAIT_EMOTION_LLM`** |
| 对话发散度 | **`OCLIVE_LLM_*`** |

---

## 阅读顺序（推荐）

0. [manifest 与 settings.json](#manifest-与-settingsjson加载顺序)  
0.5. [开关与模式（先看这里）](#开关与模式先看这里)  
0.6. [其它开关与调试入口（聚合）](#其它开关与调试入口聚合)  
1. [一、基础元数据](#一基础元数据)  
2. [二、默认性格七维](#二默认性格七维-default_personality)  
3. [三、场景 scenes](#三场景-scenes)  
4. [四、用户身份（manifest）](#四用户身份-manifest)  
5. [五、settings.json 引擎字段](#五settingsjson-引擎字段推荐)  
6. [六、与 `scenes/` 子目录的关系](#六与-scenes-子目录的关系)  

---

## 一、基础元数据

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | **必填**。角色唯一 id，**必须与**文件夹名 `roles/{id}` **完全一致**。 |
| `name` | string | **必填**。展示用角色名。 |
| `version` | string | 版本号，建议语义化（如 `1.0.2`）。 |
| `author` | string | 作者/社团名。 |
| `description` | string | 短简介；会出现在 UI 与部分提示上下文。 |

---

## 二、默认性格七维 `default_personality`

| 字段 | 类型 | 说明 |
|------|------|------|
| `default_personality` | number[]，长度 7 | **可选**；缺省由应用内部默认。顺序固定为：**倔强、黏人、敏感、强势、宽容、话多、温暖**（与内部 `PersonalityVector` / 旧七维一致）。每项约 `0.0～1.0`。 |

---

## 三、场景 `scenes`

| 字段 | 类型 | 说明 |
|------|------|------|
| `scenes` | string[] | **场景 id 列表**。可与 `roles/{id}/scenes/` 下子目录**合并**；若都为空，应用会回退为至少一个默认场景（见代码 `merge_scene_ids`）。 |

场景展示名、独白等可在各场景的 `scene.json` 中配置（不在本清单展开）。

---

## 四、用户身份（manifest）

以下为 **`manifest.json`** 中的关系契约；**`identity_binding`** 见下一节「settings」。

### 4.1 `user_relations`（对象，键为关系 id）

每个身份键对应一个对象：

| 字段 | 说明 |
|------|------|
| `display_name` | **可选**。UI 与提示用中文名；省略则多用 id 展示。 |
| `prompt_hint` | **建议填写**。注入提示词的身份语气要点。 |
| `favor_multiplier` | 好感变化倍率（正数，常见约 `0.8～1.3`）。 |
| `initial_favorability` | 选中该身份时的起始好感 **0～100**；未写则按默认规则。 |

### 4.2 `default_relation`

| 字段 | 说明 |
|------|------|
| `default_relation` | string，**必须在** `user_relations` **的键中存在**。表示创作者推荐的默认关系键；玩家选「跟随创作者默认身份」时使用。 |

### 4.3 `dev_only`（可选，默认 `false`）

| 字段 | 说明 |
|------|------|
| `dev_only` | 为 **`true`** 时，该包**默认不出现在**应用 `list_roles` 角色列表中（适合创作者本地调试包、未公开样例，避免与正式上架角色混在一起）。仍可通过 **`load_role` / `switch_role` 按 `id` 加载**。需要在本机列表里看到时，设置环境变量 **`OCLIVE_LIST_DEV_ROLES=1`**（或 `true` / `yes` / `on`）。 |

本仓库 **`roles/`** 下不再附带独立的「test_*」示例目录；身份与关系契约请在正式角色包的 **`user_relations`** 中编写。玩家可选身份仍由 **`user_relations` + `default_relation`** 定义；`identity_binding` 控制身份是否按场景切换（见第五节）。

---

## 五、settings.json 引擎字段（推荐）

下列字段位于 **`settings.json`**（或旧版全部写在 **manifest** 亦可；合并后 **`settings` 覆盖 manifest**）。结构体见源码 `DiskRoleSettings`、`DiskRoleManifest` 中对应段。

### 5.1 `schema_version`

| 字段 | 说明 |
|------|------|
| `schema_version` | 数字，当前为 **`1`**，便于日后迁移。 |

### 5.2 模型（LLM）

| 字段 | 类型 | 说明 |
|------|------|------|
| `model` 或 `ollama_model` | string（可选） | **二选一**，等价别名（JSON 键常用 `model`）。本角色默认 **Ollama 模型名**。未写则跟随应用环境变量 `OLLAMA_MODEL` 与全局默认。 |

### 5.3 演化参数 `evolution`

控制事件对性格演化等（细项以源码 `EvolutionConfig` / `EvolutionConfigDisk` 为准）。

| 字段 | 说明 |
|------|------|
| `event_impact_factor` | 事件对性格演化影响的总系数（默认常见为 `1.0`）。 |
| `ai_analysis_interval` | 与 AI 分析节奏相关的间隔参数（默认如 `15`）。 |
| `max_change_per_event` | 单次事件性格变化上限。 |
| `max_total_change` | 累计变化上限。 |

### 5.4 `identity_binding`

| 取值 | 说明 |
|------|------|
| `global` | **全剧一个身份**：换场景不改变「你是谁」；详见 `handoff/21_CREATOR_IDENTITY_BINDING.md`。 |
| `per_scene` | **可按场景覆盖身份**（manifest 反序列化默认亦为 `per_scene`，与旧行为一致）。 |

### 5.5 记忆 `memory_config`

| 字段 | 说明 |
|------|------|
| `scene_weight_multiplier` | 场景相关记忆权重总系数（默认常见如 `1.2`）。 |
| `topic_weights` | 对象：`场景 id` → `主题名` → **权重**（非负小数）。场景 id 须存在于本角色合并后的场景列表，否则**校验会报错**。 |

---

## 六、与 `scenes/` 子目录的关系

- `manifest.scenes` 与 `roles/{id}/scenes/<scene_id>/` **目录名**会合并成完整场景列表。  
- 每个场景可有 `scene.json`（名称、时段、独白模板等）。  
- **`memory_config.topic_weights` 的键**必须对应**真实存在的场景 id**。

---

## 校验与维护提示

- 在 **merge `settings.json` 之后** 执行 `validate_disk_manifest`（见 `src-tauri/src/domain/role_manifest_validate.rs`）：如 `default_relation` 不在 `user_relations` 中、`topic_weights` 出现未声明场景等会**报错**。  
- 修改字段后建议本地跑一次应用或 `cargo test` 相关用例，确认能通过。  
- 身份模式与玩家侧行为：见 **`handoff/21_CREATOR_IDENTITY_BINDING.md`**。
