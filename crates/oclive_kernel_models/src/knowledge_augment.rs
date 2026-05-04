//! 知识检索合并后的事件关键词补充（供事件检测等使用）。

use crate::event::EventType;
use std::collections::HashMap;

/// 知识驱动的额外事件关键词（B1：作为规则事件检测的补充输入）。
#[derive(Debug, Clone, Default)]
pub struct KnowledgeEventAugment {
    pub by_event: HashMap<EventType, Vec<String>>,
}

impl KnowledgeEventAugment {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_event.values().all(|v| v.is_empty())
    }
}
