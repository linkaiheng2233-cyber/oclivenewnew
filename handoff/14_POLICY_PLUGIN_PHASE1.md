# 策略插件化第 1 步对接说明（已落地）

## 本次目标

在不改变现有业务行为的前提下，完成策略骨架解耦：

- 抽象 `EmotionPolicy` / `EventPolicy` / `MemoryPolicy` 接口，支持可插拔。
- 将关键策略参数集中配置，默认值与现网行为一致。
- 收敛 `DbManager` 职责，聚焦数据访问与事务执行，策略决策上移到编排层。

## 已完成内容

### 1) 策略接口与默认实现

- 新增：`src-tauri/src/domain/policy.rs`
  - `EmotionPolicy`：负责“当前情绪”解析与平滑。
  - `EventPolicy`：负责事件检测、影响系数、置信度。
  - `MemoryPolicy`：负责记忆拼装、是否持久化、重要度、FIFO 上限。
- 默认实现：
  - `DefaultEmotionPolicy`
  - `DefaultEventPolicy`
  - `DefaultMemoryPolicy`

### 2) 策略参数集中配置

- 新增配置结构：
  - `PolicyConfig`
  - `EmotionPolicyConfig`
  - `MemoryPolicyConfig`
- 默认配置（保持旧行为）：
  - 情绪平滑：`neutral_hold_enabled = true`
  - 低置信保持阈值：`low_confidence_hold_threshold = 0.6`
  - 记忆重要度：`default_importance = 0.5`
  - FIFO 上限：`fifo_limit = 500`
  - Ignore 单字过滤：`ignore_single_char_filter = true`

### 3) AppState 注入策略

- 修改：`src-tauri/src/state/mod.rs`
  - `AppState` 新增策略依赖字段：
    - `emotion_policy: Arc<dyn EmotionPolicy>`
    - `event_policy: Arc<dyn EventPolicy>`
    - `memory_policy: Arc<dyn MemoryPolicy>`
  - 运行时从环境变量加载策略配置并注入默认策略实现。
  - 测试环境使用 `PolicyConfig::default()`，确保测试稳定。

### 4) chat_engine 改为依赖策略接口

- 修改：`src-tauri/src/domain/chat_engine/`
  - 事件检测、影响系数、置信度由 `event_policy` 提供。
  - 情绪平滑由 `emotion_policy` 提供。
  - 记忆构造/过滤/重要度/FIFO 由 `memory_policy` 提供。
  - 编排逻辑不再硬编码策略细节，实现低耦合。

### 5) DbManager 职责收敛

- 修改：`src-tauri/src/infrastructure/db.rs`
  - `ChatTurnTxInput` 增加 `memory_fifo_limit`。
  - FIFO 裁剪阈值由调用层传入，不再在 DB 层硬编码策略值。

## 环境变量（可选覆盖）

- `POLICY_EMOTION_NEUTRAL_HOLD_ENABLED`
- `POLICY_EMOTION_LOW_CONFIDENCE_HOLD_THRESHOLD`
- `POLICY_MEMORY_IGNORE_SINGLE_CHAR_FILTER`
- `POLICY_MEMORY_DEFAULT_IMPORTANCE`
- `POLICY_MEMORY_FIFO_LIMIT`

> 未配置时使用默认值，不影响现有行为。

## 质量门禁结果

- `cargo fmt --check`：通过
- `cargo clippy --all-targets -- -D warnings`：通过
- `cargo test --tests`：通过

## 对 4.6 的接手建议

- 若要新增策略，不要改 `chat_engine` 主流程，直接新增策略实现并在 `AppState` 注入。
- 若要调参数，优先用配置对象/环境变量，不要回退为硬编码常量。
- `DbManager` 保持“纯数据访问 + 事务原子”，避免回流业务规则。

