//! LLM function-calling parse port (implementation in host infrastructure).

use oclive_kernel_types::{ToolCall, ToolSchemaInput};
use serde_json::Value;

/// Parses OpenAI-style `tool_calls` / `function_call` from LLM text output.
pub trait FunctionCallingParserPort: Send + Sync {
    /// Extract tool calls from raw LLM response text.
    fn parse_from_llm_response(&self, text: &str) -> Vec<ToolCall>;

    /// Build function-calling schema JSON for the given tool list.
    fn to_function_calling_schema(&self, tools: &[ToolSchemaInput]) -> Value;
}
