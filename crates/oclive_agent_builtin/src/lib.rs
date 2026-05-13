//! Builtin ReAct Agent（`LlmClient` + `McpInvoke`）。

#[cfg(feature = "providers")]
mod builtin_react;

#[cfg(feature = "providers")]
pub use builtin_react::BuiltinReActAgent;
