//! 情绪 / 事件 / 记忆策略 trait。

use oclive_kernel_types::{
    Emotion, EmotionResult, Event, EventType, PolicyContext, Result,
};

/// Maps analyzed user emotion into the role's displayed [`Emotion`].
pub trait EmotionPolicy: Send + Sync {
    /// 将分析得到的用户情绪映射为角色当前展示情绪。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn resolve_current_emotion(&self, previous: Option<&str>, analyzed: &EmotionResult) -> Emotion;
}

/// Detects in-turn events and supplies impact/confidence weights per [`EventType`].
pub trait EventPolicy: Send + Sync {
    /// 检测本回合对话事件类型。
    ///
    /// # Errors
    ///
    /// Returns an error when the policy cannot classify the message into an [`Event`].
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn detect(&self, text: &str, user_emotion: &Emotion, bot_emotion: &Emotion) -> Result<Event>;

    /// 返回指定事件类型的叙事影响权重。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn impact(&self, event_type: &EventType) -> f64;

    /// 返回指定事件类型的检测置信度。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn confidence(&self, event_type: &EventType) -> f32;
}

/// Decides what to persist as long-term memory and with what importance.
pub trait MemoryPolicy: Send + Sync {
    /// 根据策略上下文构建待持久化的记忆正文。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn build_memory_entry(&self, ctx: &PolicyContext<'_>) -> String;

    /// 判断本回合是否应写入长期记忆。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn should_persist(&self, ctx: &PolicyContext<'_>) -> bool;

    /// 计算本回合记忆的重要性分数。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn importance(&self, ctx: &PolicyContext<'_>) -> f64;

    /// 返回 FIFO 淘汰上限（条数）。
    ///
    /// # Errors
    ///
    /// 无；本方法不返回 `Result`。
    ///
    /// # Panics
    ///
    /// 不 panic。
    fn fifo_limit(&self) -> i32;
}
