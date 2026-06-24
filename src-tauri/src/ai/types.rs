use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<serde_json::Value>>,
    /// Ask the provider for a JSON object response (OpenAI-compatible `response_format`).
    pub json_object_mode: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutorCompletion {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChoice {
    pub message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionMessage {
    /// OpenAI-compatible APIs may return a string or a multimodal part array.
    pub content: Option<serde_json::Value>,
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default)]
    pub refusal: Option<String>,
    /// Some reasoning-model proxies expose chain-of-thought here instead of `content`.
    #[serde(default)]
    pub reasoning_content: Option<String>,
}

pub fn extract_content_value(content: &Option<serde_json::Value>) -> Option<String> {
    let value = content.as_ref()?;
    match value {
        serde_json::Value::String(text) if !text.trim().is_empty() => Some(text.clone()),
        serde_json::Value::Array(parts) => {
            let mut text = String::new();
            for part in parts {
                if let Some(part_text) = part.get("text").and_then(|v| v.as_str()) {
                    text.push_str(part_text);
                } else if let Some(part_text) = part.as_str() {
                    text.push_str(part_text);
                }
            }
            (!text.trim().is_empty()).then_some(text)
        }
        serde_json::Value::Null => None,
        _ => None,
    }
}

impl ChatCompletionMessage {
    pub fn text_content(&self) -> Option<String> {
        extract_content_value(&self.content)
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorBody,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiTestResult {
    pub ok: bool,
    pub model: String,
    pub message: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingTestResult {
    pub ok: bool,
    pub model: String,
    pub latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_string_and_multimodal_content() {
        assert_eq!(
            extract_content_value(&Some(serde_json::json!("hello"))),
            Some("hello".into())
        );
        assert_eq!(
            extract_content_value(&Some(serde_json::json!([
                {"type": "text", "text": "hi "},
                {"type": "text", "text": "there"}
            ]))),
            Some("hi there".into())
        );
        assert_eq!(extract_content_value(&Some(serde_json::Value::Null)), None);
    }
}
