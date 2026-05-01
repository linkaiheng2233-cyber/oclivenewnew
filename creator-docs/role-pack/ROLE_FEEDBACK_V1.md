## 角色包使用后反馈（半私密）v1

目标：让用户在使用角色包后提交结构化反馈，帮助创作者迭代；默认 **本机私有**，不公开展示、不依赖社区站�?
### 1. 用户在哪里反馈？

- **主程序（oclivenewnew�?*：在角色运行时信息区域点�?**「反馈此角色包�?* 提交�?- 反馈写入本机 SQLite（运行时 app data 目录），不会自动上传�?
### 2. 创作者在哪里收到反馈�?
- **编写器（oclive-pack-editor�?*：试聊面板里点击 **「查看反馈（半私密）�?* 查看与处理�?- 编写器通过运行时的本机 `--api` 读取（`/role-feedback`）�?
> 说明：这是一条“本机闭环”路径，适合创作者在同一台电脑上创作与测试。若未来要远程收件，需要额外设计同�?上传机制（不�?v1 内）�?
---

### 3. 数据库（SQLite）表结构

由迁移文件维护：

- `crates/oclive_kernel_runtime/migrations/016_role_feedback.sql`
- `crates/oclive_kernel_runtime/migrations/017_role_feedback_governance.sql`

核心字段（简述）�?
- **身份**：`role_id`、`session_id?`
- **内容**：`mood_tag?`、`message`
- **治理**：`status=open|handled`、`read_at?`、`handled_at?`、`handled_note?`
- **上下�?*：`scene_id?`、`presence_mode?`
- **版本/来源**：`role_version?`、`runtime_version?`、`client_version?`、`source?`

---

### 4. 本机 HTTP API�?-api�?
运行时以 `--api` 启动（仅绑定 `127.0.0.1`）�?
- **提交反馈**：`POST /role-feedback`
- **查询列表**：`GET /role-feedback?role_id=...&limit=...&offset=...`
- **标记已读**：`POST /role-feedback/mark-read`
- **标记已处�?*：`POST /role-feedback/set-handled`

返回字段以代码为准（`src-tauri/src/http_api.rs`）�?
---

### 5. 隐私与内容约束（强制�?
- 不要在反馈中填写真实姓名、电话、账号、地址等个人隐私信息�?- 运行时与编写器仅做“内容存储与展示”，不做自动公开发布�?
