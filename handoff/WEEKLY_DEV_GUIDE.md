# oclivenewnew 周开发指南

**目标**：按周迭代，每周交付可验证的功能增量；Cursor 长期遵守的开发节奏与质量标准。

---

## 快速导航

- **当前进度**：WEEK3-003 完成（事件落库 + 集成测试 + 前端封装规划）
- **下一步**：WEEK3-004 ~ WEEK3-007（API 命令 → 前端集成 → 事务 + 情绪持久化 → 可选优化）
- **详细计划**：见 `02_DEVELOPMENT_PLAN_v3.8.md`
- **项目认知**：见 `04_4.6_PROJECT_TRUTH_CHECKLIST.md`
- **开发规范**：见 `03_DEVELOPMENT_STANDARDS.md`
- **进度快照**：见 `05_4.6_对接报告.md`

---

## WEEK3 周期（当前迭代）

### WEEK3-001 ✅ 完成
**主题**：编排 + Repository + send_message 命令

**交付**：
- ✅ `domain/chat_engine::process_message()` 完整串联
- ✅ Repository trait + SqliteMemoryRepository / SqliteFavorabilityRepository
- ✅ LlmClient trait + OllamaClient 实现
- ✅ `api/chat.rs::send_message` 命令（薄封装）
- ✅ DTO 定版（SendMessageRequest / SendMessageResponse）
- ✅ 状态注入（sqlx migrate + repos + LLM + RoleStorage）

**验收**：
```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo build
```

---

### WEEK3-002 ✅ 完成
**主题**：事件落库 + 集成测试 + 前端封装规划

**交付**：
- ✅ `process_message()` 返回 `ProcessMessageResult { response, emotion, event, favorability_delta }`
- ✅ `api/chat.rs::send_message` 调用 `EventRepository::save_event()`（原子性）
- ✅ SendMessageResponse 新增 `events: Vec<Event>` 字段
- ✅ `tests/chat_integration.rs` 集成测试（5+ 用例：happy path / 事件触发 / LLM 超时 / 好感度溢出 / 无事件）
- ✅ MockLlm 实现（可控返回值，便于测试）
- ✅ `src/utils/tauri-api.ts` 封装规划（类型定义）

**验收**：
```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --tests
cargo build
```

---

### WEEK3-003 ⏳ 进行中
**主题**：前端 Tauri API 封装 + 页面集成

#### 任务 A：前端 tauri-api.ts 完整封装（1～1.5h）
**目标**：一次性封装所有计划命令，前端可直接调用

**位置**：`src/utils/tauri-api.ts`

**封装命令**：
```typescript
// 已实现
export async function sendMessage(req: SendMessageRequest): Promise<SendMessageResponse>

// 本周实现
export async function loadRole(roleId: string): Promise<RoleData>
export async function getRoleInfo(roleId: string): Promise<RoleInfo>
export async function queryMemories(roleId: string, limit: number): Promise<Memory[]>
export async function queryEvents(roleId: string, limit: number): Promise<Event[]>
export async function createEvent(req: CreateEventRequest): Promise<Event>
```

**工作项**：
1. 定义 TypeScript 接口（与后端 DTO 对齐）
2. 使用 `@tauri-apps/api/tauri` 的 `invoke()` 包装每个命令
3. 错误处理（统一 try-catch）
4. 类型导出（便于页面导入）

**验收**：
- TypeScript 编译无错（`npm run build` 或 `tsc --noEmit`）
- 类型与后端 DTO 完全对齐
- 所有命令可被页面导入

---

#### 任务 B：前端页面集成（1～1.5h）
**目标**：页面调用 sendMessage，展示对话流

**位置**：`src/components/ChatWindow.vue`（或类似）

**工作项**：
1. 导入 `tauri-api.ts` 中的 `sendMessage()`
2. 表单输入 → 调用 `sendMessage()` → 展示 reply
3. 展示情绪、好感度变化、事件列表
4. 错误处理与加载状态

**验收**：
- 页面可调用 send_message 并正确展示响应
- 情绪、好感度、事件字段正确渲染
- 无 TypeScript 错误

---

### WEEK3-004 📋 规划中
**主题**：其他 API 命令实现（后端）

#### 任务 A：load_role 命令（0.5～1h）
**目标**：加载角色配置 + 初始化 runtime

**工作项**：
1. 在 `api/role.rs` 新建 `load_role` 命令
2. 签名：`load_role(role_id: String) -> Result<RoleData, String>`
3. 逻辑：
   - 从 `RoleStorage` 加载角色配置
   - 初始化 `role_runtime`（如果不存在）
   - 返回 RoleData（含 personality / favorability / emotion 等）
4. 在 `lib.rs::generate_handler!` 注册

**验收**：
- `cargo test --lib` 绿
- 手动测试：`load_role("mumu")` 返回完整 RoleData

---

#### 任务 B：get_role_info 命令（0.5～1h）
**目标**：查询角色当前状态（不初始化）

**工作项**：
1. 在 `api/role.rs` 新建 `get_role_info` 命令
2. 签名：`get_role_info(role_id: String) -> Result<RoleInfo, String>`
3. 逻辑：
   - 从 `role_runtime` 查询当前状态
   - 返回 RoleInfo（favorability / emotion / personality_vector 等）
   - 若角色未加载，返回错误
4. 在 `lib.rs::generate_handler!` 注册

**验收**：
- `cargo test --lib` 绿
- 手动测试：`get_role_info("mumu")` 返回当前状态

---

#### 任务 C：query_memories 命令（0.5～1h）
**目标**：分页查询长期记忆

**工作项**：
1. 在 `api/memory.rs` 新建 `query_memories` 命令
2. 签名：`query_memories(role_id: String, limit: i32, offset: i32) -> Result<Vec<Memory>, String>`
3. 逻辑：
   - 调用 `MemoryRepository::query_by_role(role_id, limit, offset)`
   - 返回 Memory 列表
4. 在 `lib.rs::generate_handler!` 注册

**验收**：
- `cargo test --lib` 绿
- 手动测试：`query_memories("mumu", 10, 0)` 返回最近 10 条记忆

---

#### 任务 D：query_events 命令（0.5～1h）
**目标**：分页查询事件历史

**工作项**：
1. 在 `api/event.rs` 新建 `query_events` 命令
2. 签名：`query_events(role_id: String, limit: i32, offset: i32) -> Result<Vec<Event>, String>`
3. 逻辑：
   - 调用 `EventRepository::query_by_role(role_id, limit, offset)`
   - 返回 Event 列表
4. 在 `lib.rs::generate_handler!` 注册

**验收**：
- `cargo test --lib` 绿
- 手动测试：`query_events("mumu", 10, 0)` 返回最近 10 条事件

---

#### 任务 E：create_event 命令（0.5～1h）
**目标**：手动创建事件（测试 / 调试用）

**工作项**：
1. 在 `api/event.rs` 新建 `create_event` 命令
2. 签名：`create_event(req: CreateEventRequest) -> Result<Event, String>`
3. 逻辑：
   - 验证 CreateEventRequest（role_id / event_type / timestamp）
   - 调用 `EventRepository::save_event()`
   - 返回创建的 Event
4. 在 `lib.rs::generate_handler!` 注册

**验收**：
- `cargo test --lib` 绿
- 手动测试：`create_event(...)` 成功创建事件

---

**WEEK3-004 总验收**：
```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --tests
cargo build
```
**结果**：所有命令可调用，无编译 / 测试错误

---

### WEEK3-005 📋 规划中
**主题**：前端页面完整集成 + 角色管理

#### 任务 A：角色加载与切换（1～1.5h）
**目标**：前端可加载角色、切换角色、展示角色信息

**工作项**：
1. 新建 `src/components/RoleSelector.vue`
2. 调用 `loadRole()` 加载角色
3. 调用 `getRoleInfo()` 展示当前状态
4. 支持角色切换（下拉菜单或列表）
5. 展示 favorability / emotion / personality_vector

**验收**：
- 页面可加载角色并展示信息
- 角色切换后信息正确更新
- 无 TypeScript 错误

---

#### 任务 B：记忆与事件查看（1～1.5h）
**目标**：前端可查看角色的记忆和事件历史

**工作项**：
1. 新建 `src/components/MemoryPanel.vue`（展示长期记忆）
2. 新建 `src/components/EventPanel.vue`（展示事件历史）
3. 调用 `queryMemories()` 和 `queryEvents()`
4. 支持分页（limit / offset）
5. 展示时间戳、内容、类型等

**验收**：
- 页面可查看记忆和事件
- 分页功能正常
- 无 TypeScript 错误

---

#### 任务 C：对话界面完善（1h）
**目标**：优化 ChatWindow，支持情绪、好感度、事件实时展示

**工作项**：
1. 改进 ChatWindow 布局（对话 + 侧边栏）
2. 侧边栏展示：当前情绪、好感度、最近事件
3. 对话框展示：用户输入 + bot 回复 + 情绪标签
4. 实时更新（每次 send_message 后刷新侧边栏）

**验收**：
- 对话界面美观、信息完整
- 情绪、好感度、事件实时更新
- 无 TypeScript 错误

---

**WEEK3-005 总验收**：
```bash
npm run build
npm run lint  # 如果有 ESLint
# 手动测试：完整对话流 + 角色切换 + 记忆查看
```

---

### WEEK3-006 📋 规划中
**主题**：事务支持 + bot 情绪持久化

#### 任务 A：事务框架（1～2h）
**目标**：send_message 中的多步操作（情绪分析 → 事件检测 → 好感度 → 记忆 → 事件写入）支持事务回滚

**工作项**：
1. 在 `infrastructure/db.rs` 中添加事务支持
   - 新增 `DbManager::begin_transaction()` / `commit()` / `rollback()`
   - 或使用 sqlx 的 `Transaction` 类型
2. 修改 `domain/chat_engine::process_message()` 签名
   - 接收 `&mut Transaction` 而非 `&DbManager`
   - 所有 DB 操作通过 transaction 执行
3. 修改 `api/chat.rs::send_message`
   - 开启事务 → 调用 `process_message()` → 提交或回滚
4. 补充集成测试（事务回滚场景）

**验收**：
- `cargo test --lib` 绿
- `cargo test --tests` 绿（含事务回滚测试）
- 手动测试：send_message 中途失败时，数据库无污染

---

#### 任务 B：bot 情绪持久化（1～1.5h）
**目标**：bot 的情绪状态（当前不是 Neutral）持久化到数据库

**工作项**：
1. 在 `role_runtime` 表中新增 `current_emotion` 字段（或新表 `emotion_history`）
2. 修改 `domain/emotion_analyzer.rs`
   - 返回 bot 的情绪（不仅是用户情绪）
3. 修改 `domain/chat_engine::process_message()`
   - 保存 bot 情绪到数据库
4. 修改 `api/role.rs::get_role_info()`
   - 返回 bot 当前情绪
5. 补充集成测试

**验收**：
- `cargo test --lib` 绿
- `cargo test --tests` 绿
- 手动测试：send_message 后查询 role_runtime，bot 情绪已更新

---

**WEEK3-006 总验收**：
```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --tests
cargo build
```

---

### WEEK3-007 📋 规划中
**主题**：可选优化 + 性能 + 文档

#### 任务 A：缓存优化（可选，1～1.5h）
**目标**：减少数据库查询，提升响应速度

**工作项**：
1. 在 `infrastructure/cache.rs` 中实现 LRU 缓存
2. 缓存 role_runtime / personality_vector / 最近记忆
3. send_message 后清除相关缓存
4. 补充缓存命中率测试

**验收**：
- 缓存功能正常
- 测试绿
- 性能基准测试（可选）

---

#### 任务 B：错误处理完善（可选，1h）
**目标**：统一错误类型、改进错误消息

**工作项**：
1. 在 `models/error.rs` 中定义统一的 `AppError` 枚举
2. 所有 API 命令返回 `Result<T, AppError>`
3. 前端处理 AppError 并展示用户友好的错误消息
4. 补充错误处理测试

**验收**：
- 所有命令使用统一错误类型
- 前端可正确处理错误
- 测试绿

---

#### 任务 C：文档与示例（可选，1h）
**目标**：补充 API 文档、使用示例、架构图

**工作项**：
1. 在 `handoff/` 中新增 `06_API_REFERENCE.md`（所有命令的签名、参数、返回值）
2. 新增 `07_FRONTEND_INTEGRATION_GUIDE.md`（前端调用示例）
3. 更新 `README.md`（项目概览、快速开始）
4. 可选：绘制架构图（编排 → 各 engine → Repository → DB）

**验收**：
- 文档完整、示例可运行
- 新人可按文档快速上手

---

**WEEK3-007 总验收**：
```bash
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --tests
cargo build
```

---

## 每周固定门禁

**执行时机**：每周五或交付前

**命令**（在 `src-tauri/` 目录下）：
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --tests
cargo build
```

**前端**（在项目根目录）：
```bash
npm run build
npm run lint  # 如果有 ESLint
```

**失败处理**：
- `fmt` 失败 → 运行 `cargo fmt` 自动修复
- `clippy` 失败 → 修改代码或添加 `#[allow(...)]`
- `test` 失败 → 修复测试或代码
- `build` 失败 → 检查依赖版本或编译错误

---

## 核心架构约束

**编排**：
- ❌ 业务逻辑散落在 API 层 → ✅ 所有逻辑在 `domain/chat_engine::process_message()`
- ❌ 直接调 infrastructure → ✅ 通过 Repository trait

**持久化**：
- ❌ 直接操作 SqlitePool → ✅ 通过 Repository 实现
- ❌ 虚构表名 → ✅ 以 `src-tauri/migrations/001_init.sql` 为准
- ❌ 事件不落库 → ✅ API 层调 `EventRepository::save_event()`

**Tauri 命令**：
- ❌ 命令定义散落 → ✅ 所有命令在 `api/*.rs`，唯一注册点 `lib.rs::generate_handler!`
- ❌ 删除 `.setup()` 或 `AppState::manage()` → ✅ 保留

**类型对齐**：
- ❌ EmotionResult 和 Emotion 混淆 → ✅ EmotionResult 是 7 维 f64，Emotion 是枚举
- ❌ SendMessageResponse 用 `response` 字段 → ✅ 用 `reply`
- ❌ PromptBuilder 第 5 参数用 `&Emotion` → ✅ 用 `&str`

**事务**（WEEK3-006 后）：
- ❌ 多步操作无事务保护 → ✅ send_message 中所有 DB 操作在事务内

---

## 一句话摘要

**WEEK3-003 完成**：前端 tauri-api 封装 + 页面集成；**WEEK3-004**：实现 load_role / get_role_info / query_memories / query_events / create_event；**WEEK3-005**：前端角色管理 + 记忆查看 + 对话完善；**WEEK3-006**：事务支持 + bot 情绪持久化；**WEEK3-007**：可选优化（缓存 / 错误处理 / 文档）；每周固定门禁（fmt / clippy / test / build）。
