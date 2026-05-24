# 创作者指南：用户身份模式 `identity_binding`

**完整 manifest 所有区块与字段表**见同仓库 **`roles/README_MANIFEST.md`**；可复制模板见 **`roles/manifest.template.json`**。

面向 **角色包作者**：在 `roles/{角色id}/manifest.json` 中配置「玩家相对 AI 是谁」与 **场景** 的关系。  
应用行为以仓库实现为准：`domain/user_identity.rs`、`api/role/runtime.rs`、`api/role/mod.rs`（`set_user_relation` / `set_scene_user_relation`）。

---

## 1. 字段放哪里

在 **顶层 manifest** 与 `default_relation`、`user_relations` 同级增加：

```json
"identity_binding": "global"
```

或：

```json
"identity_binding": "per_scene"
```

- **省略不写**：与 **`per_scene`** 相同（与历史版本兼容：支持按场景覆盖身份）。
- 合法值：**`global`** | **`per_scene`**（JSON 中为蛇形命名，与工程内枚举一致）。

---

## 2. 两种模式分别解决什么问题

| 模式 | 一句话 | 适合什么故事 |
|------|--------|----------------|
| **`global`** | 全剧 **一条** 玩家身份，**换场景不改变**「你是谁」 | 单线关系、同居、固定「同学/恋人/家人」一条线走到底 |
| **`per_scene`** | **不同场景可以不同身份**（家里家人、学校同学等） | 多场景多面具、地点决定关系、RPG 式「场合」切换 |

注意：**场景切换**（地图/剧情换地点）本身只改当前场景 id；差别在于 **计算当前有效身份时**，是否读取 **`role_scene_identity`（按场景存的身份覆盖表）**。

叙事上「哪些场合谁应在场、谁不应出场」由创作者在角色包中约定（description、`scene.json`、时间窗等），见 **`roles/README_MANIFEST.md`** 中 **「场景叙事与出场」** 小节。

---

## 3. 与 `user_relations`、`default_relation` 的关系

- **`user_relations`**：定义可选的身份键（如 `friend`、`family`）、展示名、`prompt_hint`、好感倍率、初始好感等。两种模式 **都需要**。
- **`default_relation`**：manifest 里的「默认关系键」。当玩家选择「跟随创作者默认身份」时，有效身份为该键；两种模式 **都适用**。
- **`identity_binding`**：只决定 **身份是否按场景分别存储/解析**，不改变 `user_relations` 里各键的含义。

---

## 4. 玩家在前端改身份时会发生什么（概念）

- **`global`**  
  - 改身份 = 写 **全局**关系，**不**再按场景写覆盖。  
  - 应用会在合适的时机 **清理** 该角色在库里的按场景覆盖，避免残留数据在将来改配置时误生效。

- **`per_scene`**  
  - 改身份（非「默认身份」选项）往往对应 **当前场景** 一条覆盖；  
  - 换到另一场景时，若该场景曾单独设过身份，会显示那条；否则回退到全局默认 / manifest 默认逻辑。

（具体以当前 UI：顶栏与开发面板为准，二者已与 `identity_binding` 对齐。）

---

## 5. 创作者怎么选

- 希望玩家 **从头到尾扮演同一种关系** → 用 **`global`**（示例：内置角色「沐沐」`mumu` 使用 `global`）。
- 希望 **同一存档里不同地点关系不同** → 用 **`per_scene`**（或不写该字段）。

若从 `per_scene` 改为 `global`，建议在文档中提醒玩家：**重新在设置里选一次身份**，以便全局状态与预期一致（旧按场景数据可能被应用清理或忽略，取决于版本与操作路径）。

---

## 6. 与导出/复制角色包

导出或复制 manifest 时，请 **一并带上 `identity_binding`**，否则接收方按默认 **`per_scene`** 理解，行为可能与你的设计不一致。

---

## 7. 相关代码索引（给二次开发）

| 主题 | 路径 |
|------|------|
| 有效身份键解析 | `src-tauri/src/domain/user_identity.rs` |
| UI 用运行时快照 | `src-tauri/src/api/role/runtime.rs` |
| 全局 / 按场景 写库命令 | `src-tauri/src/api/role/mod.rs` |
| 对话回合使用身份键 | `src-tauri/src/domain/chat_engine/mod.rs` |
| 磁盘 manifest 映射 | `src-tauri/src/models/role_manifest_disk.rs` |
| 枚举定义 | `src-tauri/src/models/role.rs`（`IdentityBinding`） |

---

*文档版本：与仓库 `identity_binding` 实现同步；若接口或默认值变更，以源码与迁移为准。*
