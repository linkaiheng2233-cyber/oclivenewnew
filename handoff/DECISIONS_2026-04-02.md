# 角色包与运行时决策记录（2026-04-02）

本文档汇总「沐沐项目」在角色包规范、后端、前端、迁移与验收上的**已拍板决策**及**实现注意事项**，作为后续开发与 Cursor 协作的参考依据。若与代码冲突，以本文档为准更新代码或回写本文档。

---

## 一、角色包规范（目标形态）

### 1.1 目录结构（目标）

```
roles/{role_id}/
├── manifest.json          # 必需（新格式，见下节）
├── core_personality.txt   # 可选，覆盖 manifest 中的人设文本
├── Modelfile              # 可选，Ollama 模型定义
├── assets/
│   └── images/
│       ├── normal.png
│       ├── happy.png
│       ├── sad.png
│       ├── angry.png
│       ├── shy.png
│       ├── confused.png
│       ├── disgust_light.png
│       ├── disgust_mid.png
│       └── disgust_heavy.png
└── scenes/
    ├── {scene_id}/
    │   └── scene.json
    └── ...
```

### 1.2 `manifest.json`（磁盘新格式）

- 采用扁平/扩展字段，与现有 Rust `Role` 旧结构**不同**；由 **`models/role_manifest.rs`** 定义 `RoleManifest` 反序列化，在 **`infrastructure/storage.rs`** 中转换为内部 `Role`（可逐步收敛）。
- 字段要点（与产品讨论一致）：
  - `id`、`name`、`version`、`author`、`description`
  - `default_personality`：七维数组，顺序为 **[倔强, 黏人, 敏感, 强势, 宽容, 话多, 温暖]**
  - `evolution`：`event_impact_factor`、`ai_analysis_interval`、`max_change_per_event`、`max_total_change` 等
  - `scenes`：场景 id 列表
  - `user_relations`：如 `friend` / `family`，含 `prompt_hint`、`favor_multiplier`
  - `default_relation`：默认关系键
  - `memory_config`：`scene_weight_multiplier`、`topic_weights`（按场景）

### 1.3 `scene.json`

- `name`、`description`、`memory_weight`、`topic_weights` 等（与规范一致）。
- **旧包**中 `description.txt` 由迁移脚本合并进对应 `scenes/{id}/scene.json` 的 `description`。

---

## 二、数据库与运行时

### 2.1 `role_runtime` 扩展（决策 A：单表扩展）

- **不**新建 `user_personality` 表。
- 在 **`role_runtime`** 上增加（或约定）：
  - **`user_relation`**：`TEXT`，当前关系键（如 `friend`、`family`）。
  - **`event_impact_factor`**：`REAL`，默认 **1.0**；用户 UI 调节的是**运行时覆盖**。
- 好感度列名使用 **`current_favorability`**（与现有迁移一致，勿写成 `favorability`）。

### 2.2 首次加载与默认值

- 角色包中 **`evolution.event_impact_factor`** 作为默认值，在**首次创建/初始化**该角色运行时写入 `role_runtime`。
- 用户通过 UI 修改后**以运行时为准**，**不回写** `manifest.json`。

### 2.3 性格：旧七维 + core / delta

- **`personality_vector` 表**（迁移）：将原「新七维列」改为（或等价于）：
  - **`core_personality`**：`JSON` 数组，旧七维初始值（来自角色包 `default_personality`）。
  - **`delta_personality`**：`JSON` 数组，可变偏移；**每次事件后更新并持久化**；`AI 深度分析` 时也可调整 `delta`。
- **运行时有效性格** = `core + delta`，各分量 **clamp 到 [0,1]**。
- **历史表语义**：每条历史记录存 **有效性格向量（core+delta 合成后的七维）**，便于审计；`delta` 的权威来源以运行时持久化为准（`role_runtime` 或约定附表，实现时二选一写清）。
- 开发阶段若旧数据无意义，**可接受重置**为角色包默认。

### 2.4 记忆：`long_term_memory.scene_id`

- 新增迁移：为 **`long_term_memory`** 增加 **`scene_id`**（可 `NULL`）。
- 写入长期记忆时记录**当前场景**；检索时对同场景记忆乘以 **`scene_weight_multiplier`**（来自 manifest `memory_config` 或场景配置，按实现挂钩）。
- 旧数据 `NULL`：检索时权重按 **1** 处理。

### 2.5 `memory_config.topic_weights`（阶段策略）

- **第一阶段**：仅用于 **prompt 提示**（例如「在{场景}下，你们可能会聊……」），**不对记忆做话题过滤或打标签**。
- **P1** 再考虑话题分类、检索加权。

### 2.6 用户关系 vs `relation_state`

- **并存、职责分离**：
  - **`user_relation`（manifest / 用户选择）**：影响 **prompt 中的关系描述**、好感度增量上的 **`favor_multiplier`**（在**增量计算时**应用，与现有 `relation_state` 逻辑不混用同一套状态机）。
  - **`relation_state`**：仍由好感度与事件驱动（陌生人→朋友→…），**不参与**用户手动选关系的演化。
- 两者在 prompt 中均可体现，**计算上互不替代**。

---

## 三、后端模块与代码路径

- 性格与演化：以 **`domain/personality_engine.rs`** 为主；可按需 **`domain/evolution.rs`** 抽离「事件影响系数、限幅」等逻辑。
- **勿**虚构不存在的 `domain/personality.rs`；向量类型在 **`models/personality.rs`** 演进（旧七维 + 与 `EvolutionBounds` 的衔接按实现调整）。
- 新增 **`models/role_manifest.rs`**（`RoleManifest`），**`storage.rs`** 负责读盘与映射。

---

## 四、API / DTO / 前端

- 新增命令示例：**`set_user_relation`**、**`set_evolution_factor`**（名称以最终实现为准）。
- **`get_role_info`** 返回：`user_relations`、`default_relation`、当前 `user_relation`、`event_impact_factor` 等前端面板所需字段。
- **必须同步**：`src-tauri/src/models/dto.rs` 与 **`src/utils/tauri-api.ts`**，避免面板拿不到数据。

---

## 五、情绪图片（Tauri）

- 使用 **`convertFileSrc`**（`@tauri-apps/api/tauri`）将**可访问的绝对路径**转为 WebView URL。
- **`tauri.conf.json`**：`allowlist.fs` 的 **`scope`** 需允许读取 **`roles/`**（及实际解析路径）；开发模式先跑通。
- 生产包若 `roles` 路径变化，阶段 4 再约定是否复制到 `resource` 目录。

---

## 六、工程与验收（务实版）

- **`cargo clippy`**：允许仓库内存量警告；**新增代码**尽量避免新警告；不强制全仓库 `-D warnings` 清零。
- 合并前：**`cargo test --lib --tests`** 通过，**`npm run build`** 成功。
- 提交信息建议使用 `feat:` / `fix:` / `refactor:` 等前缀。

---

## 七、分阶段实施（执行顺序）

1. **后端核心**：旧七维 + core/delta、演化系数、记忆 `scene_id` 与加权、`user_relation` / `event_impact_factor`、`RoleManifest` 读盘、prompt 与好感增量中的 `favor_multiplier`、单元测试。
2. **前端基础**：`RoleInfoPanel.vue`、场景中文名、情绪图 `convertFileSrc`、`roleStore` 状态与持久化字段对齐。
3. **迁移脚本**：旧 `roles/` → 新规范 `manifest` + `scene.json`，备份旧文件策略按脚本说明。
4. **联调与验收**：角色切换、对话、场景、时间跳转、关系切换、导出等端到端。

---

## 八、与仓库既有规则的衔接

- 编排主流程仍以 `chat_engine` 的 `process_message` 为准；契约以 **`models/dto.rs`** 为准，回复字段 **`reply`**。
- SQL 与表结构以 **`migrations/*.sql`** 为唯一来源；禁止虚构表名。
- Tauri 命令在 **`src-tauri/src/api/*.rs`** 注册于 **`lib.rs`** 的 `generate_handler!`。

---

## 九、修订记录

| 日期       | 说明 |
|------------|------|
| 2026-04-02 | 初版：汇总讨论决策与实现提醒 |
