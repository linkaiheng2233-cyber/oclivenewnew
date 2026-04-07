# 开发流程与标准（OCLive / oclivenewnew）

供人类开发者与 Cursor 统一遵循；与 `.cursor/rules/oclivenewnew.mdc` 及 `handoff/WEEKLY_DEV_GUIDE.md` 互补，冲突时以 **仓库内代码与迁移文件** 为准。

---

## 1. 角色包规范

### 1.1 目录结构（约定）

```
roles/{role_id}/
  manifest.json          # DiskRoleManifest，见 `src-tauri/src/models/role_manifest_disk.rs`
  core_personality.txt   # 可选
  assets/images/         # 情绪立绘 PNG（见下）
  scenes/{scene_id}/     # 可选：scene.json、description.txt
```

### 1.2 `send_message` 响应要点

- `reply`：模型回复正文
- `emotion`：`EmotionDto`，**用户输入**侧七维分析
- `bot_emotion`：**字符串**，本回合解析后的 bot 情绪标签（与 `models/emotion.rs` 的 `Display` 小写一致），供顶栏/立绘选用

### 1.3 `manifest.json` 要点

- 顶层含 `id`、`name`、`version`、`author`、`description`
- `default_personality`：七维浮点数组（顺序见 `PersonalityDefaults`）
- `evolution`、`user_relations`、`default_relation`、`memory_config`、`scenes`
- 磁盘与运行时 `Role` 的映射以 `DiskRoleManifest::to_role` 为准

### 1.4 情绪图片命名（与前端一致）

- 路径：`assets/images/{文件名}`
- 与 `src/utils/emotion-assets.ts` 中映射一致，例如：`happy.png`、`normal.png`、`disgust_light.png`
- 缺失或浏览器加载失败时，前端回退 emoji，不应导致白屏或崩溃

### 1.5 迁移旧包

- 可用 `scripts/migrate_role_manifest.py` 将旧嵌套 `personality` 结构转为新 manifest

---

## 2. 后端规范

### 2.1 分层

- **`api/`**：Tauri `command` 入口，薄封装，复杂逻辑下放 `domain/`
- **`domain/`**：编排、策略、引擎、纯业务规则
- **`infrastructure/`**：DB、存储、外部 HTTP/LLM、具体实现
- **`models/`**：DTO、领域模型；前后端契约以 `models/dto.rs` 为准（回复字段 **`reply`**）

### 2.2 错误与错误码

- 领域层优先 `crate::error::Result<T>` / `AppError`
- 暴露给前端的字符串由 `to_frontend_error()` 等统一格式化；事务类错误码以 `TXN_` 等为前缀（见 `tauri-api.ts` 中 `TransactionErrorMessages`）
- 参数错误使用 `AppError::InvalidParameter`，对应码 `INVALID_PARAMETER`

### 2.3 数据库

- **禁止**修改已合并的迁移文件；表结构变更必须 **新增** `migrations/00x_*.sql`
- 与 `001_init.sql` 等历史文件对照，避免虚构表名（以迁移为准）

### 2.4 测试

- 新逻辑优先补 **单元测试**（`#[cfg(test)]` 或同文件 `tests`）
- 跨模块行为补 **集成测试**（`src-tauri/tests/*.rs`）
- **覆盖率**：目标不低于 **80%**（新模块应尽量达标；未接 CI 时由评审与门禁命令兜底）。当前仓库**未**以覆盖率作为发布硬性门禁；提升至 80% 为后续迭代目标（见 `19_RELEASE_CHECKLIST.md` 已知限制）。

---

## 3. 前端规范

### 3.1 目录

- `components/`：可复用 UI
- `stores/`：Pinia（持久化策略与现有 `roleStore` / `chatStore` 等保持一致）
- `utils/tauri-api.ts`：**唯一**推荐调用 Tauri invoke 的入口；类型与后端 DTO 同步
- 全局样式变量：`theme.css` 等，避免魔法数色值散落

### 3.2 状态

- 按职责拆分 store；与角色相关的运行时状态以服务端 `get_role_info` / 各 command 回包为准，本地仅缓存展示

### 3.3 API

- 新增 command 必须在 `src-tauri/src/lib.rs` 注册，并在 `tauri-api.ts` 增加类型安全封装

### 3.4 情绪展示与扩展（前端）

- 统一维护 **`src/utils/emotion-assets.ts`**：
  - **`emotionToEmoji`**：小写 key → emoji；
  - **`emotionToImage`**：小写 key → `assets/images/` 下文件名；
  - **`emotionToLabelZh`**：顶栏中文标签；
  - **`emotionToAssetFilename(emotion)`**：供 `resolve_role_asset_path` 使用。
- **新增一种情绪**时（后端 `Emotion` 与角色包需同步规划）：
  1. 在以上映射中补充对应项；
  2. 在 `roles/{role_id}/assets/images/` 放置同名 PNG（可用占位图）；
  3. 后端枚举与策略变更需单独迁移/评审，不可仅改前端。

---

## 4. 测试与提交门禁（建议每次提交前）

```bash
cd src-tauri && cargo fmt && cargo clippy -- -D warnings && cargo test
cd .. && npm run build
```

- 新功能应带对应测试；仅文案/注释变更可酌情跳过部分命令

---

## 5. 发布流程

1. 更新版本号（前端 `package.json` 与 Tauri 配置一致）
2. 跑通第四节门禁与 `handoff/19_RELEASE_CHECKLIST.md`
3. 执行 `npm run tauri:build`
4. 在 `src-tauri/target/release/bundle/` 取产物并做安装冒烟（启动、加载角色、发一条消息、看日志 `oclive_roles`）

---

## 6. 后续迭代（路线图摘要）

| 阶段 | 方向 |
|------|------|
| 短期 | 用户导入角色包、对话导出增强 |
| 中期 | 场景 `scene.json` 独白/欢迎语、关键路径耗时日志 |
| 长期 | 插件扩展、可选云端同步 |

详细需求以产品/决策文档为准。
