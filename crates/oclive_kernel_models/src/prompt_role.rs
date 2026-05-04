//! 从宿主角色模型提取的、仅用于 Prompt 组装的只读切片（由 runtime 在调用处填充）。

use crate::{EvolutionConfig, MemoryConfig, UserRelation};

#[derive(Debug, Clone, Copy)]
pub struct PromptRolePromptSlice<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub core_personality: &'a str,
    pub evolution_config: &'a EvolutionConfig,
    pub user_relations: &'a [UserRelation],
    pub memory_config: Option<&'a MemoryConfig>,
    pub reply_quality_anchor: Option<&'a str>,
}
