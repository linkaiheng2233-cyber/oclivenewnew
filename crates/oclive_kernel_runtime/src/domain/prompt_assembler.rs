//! Swappable prompt-assembler facade; default delegates to [`PromptBuilder`](super::prompt_builder::PromptBuilder).

use crate::domain::prompt_builder::{PromptBuilder, PromptInput};
use crate::error::Result;
use crate::models::Role;
use std::sync::atomic::{AtomicBool, Ordering};

pub use oclive_kernel_contracts::PromptAssembler;

pub struct BuiltinPromptAssembler;

impl PromptAssembler for BuiltinPromptAssembler {
    fn build_prompt(&self, input: &PromptInput<'_>) -> Result<String> {
        Ok(PromptBuilder::build_prompt(input))
    }

    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
        PromptBuilder::top_topic_hint(role, scene_id)
    }
}

pub struct RemotePromptAssemblerPlaceholder {
    inner: BuiltinPromptAssembler,
    warned: AtomicBool,
}

impl RemotePromptAssemblerPlaceholder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: BuiltinPromptAssembler,
            warned: AtomicBool::new(false),
        }
    }

    fn warn_once(&self) {
        if self
            .warned
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            tracing::warn!(
                target: "oclive_plugin",
                "prompt backend Remote is not connected; using builtin PromptBuilder"
            );
        }
    }
}

impl PromptAssembler for RemotePromptAssemblerPlaceholder {
    fn build_prompt(&self, input: &PromptInput<'_>) -> Result<String> {
        self.warn_once();
        self.inner.build_prompt(input)
    }

    fn top_topic_hint(&self, role: &Role, scene_id: &str) -> Option<String> {
        self.warn_once();
        self.inner.top_topic_hint(role, scene_id)
    }
}

impl Default for RemotePromptAssemblerPlaceholder {
    fn default() -> Self {
        Self::new()
    }
}
