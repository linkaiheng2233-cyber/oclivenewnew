# 角色包定制指南

本页说明如何直接维护当前 OCLive 角色包。完整字段契约以
[ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) 为准；这里提供最短、可执行的创作路径。
用户关系细节见 [CREATOR_USER_RELATIONS.md](CREATOR_USER_RELATIONS.md)，场景写法见
[CREATOR_SCENE_GUIDE.md](CREATOR_SCENE_GUIDE.md)；本页不替代这些专项说明。

## 1. 当前格式

新角色包只使用 **`pipeline.ocblueprint`** 作为主清单（SSOT），支持
`schema_version: 2` 或 `3`。不要在同一目录新增 `manifest.json` /
`settings.json`；这两个文件只属于 legacy 迁移路径。

推荐目录：

```text
roles/<角色 id>/
├── pipeline.ocblueprint        # 必填：角色元数据、关系、场景和模块槽位
├── core_personality.txt        # 必填：Tier 0 核心人设
├── config.json                 # 可选：时间、记忆、立绘、思考节奏等运行策略
├── memory_seed.json            # 可选：只读初始记忆种子
├── portrait_catalog.json       # Portable Core 需要
├── user_identities/
│   ├── index.json              # 可选：用户身份目录
│   └── <identity>.md
├── scenes/
│   └── <scene id>/
│       ├── scene.json
│       └── description.txt
├── knowledge/
│   └── *.md
└── assets/
    └── images/
```

`<角色 id>` 必须与 `pipeline.ocblueprint` 的 `meta.id` 一致。请使用稳定名称；
宿主会拒绝路径分隔符、`.` / `..`、控制字符、Windows 保留设备名和首尾空白。

## 2. 人设与关系

- `core_personality.txt` 是不可被运行时演化覆盖的核心人设。
- `meta.personality` 是七维数值参考；非空时必须恰好七项且每项在 `0.0–1.0`。
- `meta.relations` 定义可选关系，`meta.default_relation` 必须引用其中一项。
- `user_identities/index.json` 可把更完整的用户身份模板映射到关系。
- 运行时可变人格存数据库，不写回角色包。

关系和身份应该一一可解释。身份模板描述“用户是谁”和互动边界，不要复制角色的
整份核心人设，也不要让角色替用户发言或擅自升级关系。

## 3. 场景

`meta.scenes` 与 `scenes/<scene id>/` 共同形成场景集合。每个场景可包含：

- `description.txt`：该场景的气氛、行为和话题约束；
- `scene.json`：展示名、时间窗口、异地素材和可选叙事连续性；
- `continuity.initial_states`：创作期确定的初始状态候选；
- `continuity.transitions`：只由最终可见回复中的明确动作标记触发。

连续性状态用于保持位置、姿态、活动等微状态，不替代长期记忆、短期情绪或核心人设。

## 4. 立绘与 Portable Core

需要跨发行版携带基础视觉能力时：

1. 在 `config.json` 设置 `portrait_catalog.enabled: true`；
2. 创建 `portrait_catalog.json`；
3. 提供七个固定 ID：
   `happy_default`、`sad_default`、`angry_default`、`neutral_default`、
   `excited_default`、`confused_default`、`shy_default`；
4. 确保每个 `path` 都是包内安全相对路径且文件真实存在。

Portable Core 只定义通用人格与七张基础立绘，不限制角色还可以携带多少场景、语音、
知识或发行版专属能力。

## 5. 语音与其它侧通道

`voice_profile.json` 是可选语音侧通道。角色级语音配置只覆盖该角色的播报任务，
不得在切换角色时改写用户的全局语音设置。

`memory_seed.json` 只是随包分发的初始事件，不是运行时长期记忆数据库。
`.ocpersona` 与 `.ocmemory` 是独立迁移格式，也不应塞回角色包主清单。

## 6. 创建与验收

可以复制结构接近的正式角色，再逐项替换内容；不要把沐沐视为所有角色的能力上限，
它是针对 Chat Pro 体验定制的完整角色。较轻量的跨发行版结构可参考
`distros/chat-pro/roles/deepseek/`。

每次修改至少执行：

```powershell
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<角色 id>
```

需要 Portable Core 时再执行：

```powershell
cargo run -p oclive-cli -- pack validate .\distros\chat-pro\roles\<角色 id> --profile portable-core
```

验证通过只代表文件与契约正确；新增语音、视觉后端或发行版专属功能仍需各自的运行时
验收。导入压缩包建议保持 `{角色 id}/...` 单一顶层目录。

## 7. Legacy 包

仅维护旧包时才使用 `manifest.json` / `settings.json` 和
`pack validate --profile legacy`。迁移新格式请阅读
[V1_TO_V2_MIGRATION.md](V1_TO_V2_MIGRATION.md)。迁移完成后删除 legacy 双文件，
避免同一角色出现两套事实来源。

## 8. 延伸阅读

- [CREATOR_LEARNING_PATH.md](CREATOR_LEARNING_PATH.md)：按创作任务选择能力；
- [PACK_VERSIONING.md](PACK_VERSIONING.md)：版本升级与兼容策略；
- [CROSS_HOST_MEMORY.md](CROSS_HOST_MEMORY.md)：跨发行版人格与记忆边界；
- [docs/personality-archive-notes.md](../../docs/personality-archive-notes.md)：核心与可变人格档案。
