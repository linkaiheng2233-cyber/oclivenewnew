use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchemaInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// 从 LLM 文本输出中提取 OpenAI 风格函数调用：
/// - `{"tool_calls":[{"id":"..","function":{"name":"x","arguments":{...}}}]}`
/// - `{"function_call":{"name":"x","arguments":"{\"city\":\"深圳\"}"}}`
pub fn parse_from_llm_response(text: &str) -> Vec<ToolCall> {
    let t = text.trim();
    if t.is_empty() {
        return Vec::new();
    }
    let Ok(v) = serde_json::from_str::<Value>(t) else {
        return Vec::new();
    };
    if let Some(arr) = v.get("tool_calls").and_then(|x| x.as_array()) {
        let mut out = Vec::new();
        for (i, item) in arr.iter().enumerate() {
            let id = item
                .get("id")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("tool-{}", i + 1));
            let fn_obj = item.get("function").cloned().unwrap_or(Value::Null);
            let name = fn_obj
                .get("name")
                .and_then(|x| x.as_str())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let args = match fn_obj.get("arguments") {
                Some(Value::String(s)) => {
                    serde_json::from_str::<Value>(s).unwrap_or_else(|_| json!({}))
                }
                Some(v) => v.clone(),
                None => json!({}),
            };
            out.push(ToolCall {
                id,
                function: FunctionCall {
                    name,
                    arguments: args,
                },
            });
        }
        return out;
    }
    if let Some(fc) = v.get("function_call") {
        let name = fc
            .get("name")
            .and_then(|x| x.as_str())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            return Vec::new();
        }
        let args = match fc.get("arguments") {
            Some(Value::String(s)) => serde_json::from_str::<Value>(s).unwrap_or_else(|_| json!({})),
            Some(v) => v.clone(),
            None => json!({}),
        };
        return vec![ToolCall {
            id: "tool-1".to_string(),
            function: FunctionCall {
                name,
                arguments: args,
            },
        }];
    }
    Vec::new()
}

/// 将 MCP 工具列表转换成 Function Calling 兼容 schema（最小参数对象）。
pub fn to_function_calling_schema(tools: &[ToolSchemaInput]) -> Value {
    let items: Vec<Value> = tools
        .iter()
        .map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description.clone().unwrap_or_default(),
                    "parameters": {
                        "type": "object",
                        "additionalProperties": true
                    }
                }
            })
        })
        .collect();
    Value::Array(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tool_calls_array() {
        let s = r#"{"tool_calls":[{"id":"a1","function":{"name":"get_weather","arguments":{"city":"深圳"}}}]}"#;
        let out = parse_from_llm_response(s);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].function.name, "get_weather");
        assert_eq!(out[0].function.arguments["city"], "深圳");
    }

    #[test]
    fn parse_function_call_single() {
        let s = r#"{"function_call":{"name":"get_weather","arguments":"{\"city\":\"北京\"}"}}"#;
        let out = parse_from_llm_response(s);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].function.arguments["city"], "北京");
    }
}
