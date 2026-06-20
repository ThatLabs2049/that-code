use crate::ai::types::{
    ApiErrorResponse, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, AiTestResult,
    ExecutorCompletion,
};
use crate::settings::AiSettings;
use futures_util::StreamExt;
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("AI provider is not configured. Add an API key in settings.")]
    NotConfigured,
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("Unexpected API response: {0}")]
    InvalidResponse(String),
}

pub fn normalize_base_url(base_url: &str) -> String {
    base_url.trim().trim_end_matches('/').to_string()
}

pub fn chat_completions_url(base_url: &str) -> String {
    format!("{}/chat/completions", normalize_base_url(base_url))
}

pub async fn chat_completion(
    settings: &AiSettings,
    request: ChatCompletionRequest,
) -> Result<String, ClientError> {
    ensure_configured(settings)?;

    let url = chat_completions_url(&settings.base_url);
    let client = build_client()?;

    let body = build_completion_body(&request);
    let body = attach_tools(body, request.tools.as_ref());

    let response = client
        .post(url)
        .json(&body);

    let response = auth_request(response, settings.api_key.trim())
        .send()
        .await?;

    parse_completion_response(response).await
}

pub async fn chat_completion_stream<F>(
    settings: &AiSettings,
    request: ChatCompletionRequest,
    mut on_delta: F,
) -> Result<String, ClientError>
where
    F: FnMut(&str),
{
    ensure_configured(settings)?;

    let url = chat_completions_url(&settings.base_url);
    let client = build_client()?;

    let mut body = build_completion_body(&request);
    body["stream"] = serde_json::Value::Bool(true);

    let response = auth_request(client.post(url).json(&body), settings.api_key.trim())
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        let message = serde_json::from_str::<ApiErrorResponse>(&body)
            .map(|parsed| parsed.error.message)
            .unwrap_or(body);
        return Err(ClientError::Api {
            status: status.as_u16(),
            message,
        });
    }

    parse_stream_response(response, &mut on_delta).await
}

async fn parse_stream_response<F>(
    response: reqwest::Response,
    on_delta: &mut F,
) -> Result<String, ClientError>
where
    F: FnMut(&str),
{
    let mut full = String::new();
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        buffer.push_str(&String::from_utf8_lossy(&chunk.map_err(ClientError::Http)?));

        while let Some(newline) = buffer.find('\n') {
            let line = buffer[..newline].trim().to_string();
            buffer = buffer[newline + 1..].to_string();

            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }

            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };

            if let Ok(parsed) = serde_json::from_str::<StreamChunk>(data) {
                if let Some(delta) = parsed
                    .choices
                    .into_iter()
                    .next()
                    .and_then(|choice| stream_delta_text(&choice.delta))
                {
                    full.push_str(&delta);
                    on_delta(&delta);
                }
            }
        }
    }

    if full.trim().is_empty() {
        return Err(ClientError::InvalidResponse(
            "missing streamed completion content".into(),
        ));
    }

    Ok(full)
}

#[derive(Debug, serde::Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, serde::Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, serde::Deserialize)]
struct StreamDelta {
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

fn stream_delta_text(delta: &StreamDelta) -> Option<String> {
    delta
        .content
        .as_ref()
        .filter(|content| !content.is_empty())
        .cloned()
}

pub async fn test_connection(settings: &AiSettings) -> Result<AiTestResult, ClientError> {
    ensure_configured(settings)?;

    let model = settings.companion_model.clone();
    let started = Instant::now();
    let content = chat_completion(
        settings,
        ChatCompletionRequest {
            model: model.clone(),
            messages: vec![ChatMessage::user("Reply with the single word: ok")],
            temperature: 0.0,
            max_tokens: Some(256),
            tools: None,
            json_object_mode: false,
        },
    )
    .await?;

    Ok(AiTestResult {
        ok: true,
        model,
        message: content.trim().to_string(),
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

fn ensure_configured(settings: &AiSettings) -> Result<(), ClientError> {
    if settings.api_key.trim().is_empty() && !is_local_provider(&settings.base_url) {
        return Err(ClientError::NotConfigured);
    }

    Ok(())
}

pub fn is_local_provider(base_url: &str) -> bool {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return false;
    }

    let without_scheme = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .unwrap_or(trimmed);
    let host_part = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .trim();
    let host = if host_part.starts_with('[') {
        host_part
            .strip_prefix('[')
            .and_then(|s| s.split(']').next())
            .unwrap_or(host_part)
    } else {
        host_part.split(':').next().unwrap_or(host_part)
    }
    .to_lowercase();

    host == "localhost"
        || host.ends_with(".localhost")
        || host == "127.0.0.1"
        || host == "::1"
        || host == "0.0.0.0"
}

pub fn auth_request(
    request: reqwest::RequestBuilder,
    api_key: &str,
) -> reqwest::RequestBuilder {
    if api_key.trim().is_empty() {
        request
    } else {
        request.bearer_auth(api_key.trim())
    }
}

fn build_client() -> Result<reqwest::Client, ClientError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(ClientError::Http)
}

pub async fn chat_completion_executor(
    settings: &AiSettings,
    request: ChatCompletionRequest,
) -> Result<ExecutorCompletion, ClientError> {
    ensure_configured(settings)?;

    let url = chat_completions_url(&settings.base_url);
    let client = build_client()?;

    let body = build_completion_body(&request);
    let body = attach_tools(body, request.tools.as_ref());

    let response = auth_request(client.post(url).json(&body), settings.api_key.trim())
        .send()
        .await?;

    parse_executor_completion(response).await
}

pub fn tools_api_unsupported(message: &str) -> bool {
    let lower = message.to_lowercase();
    [
        "tool",
        "function",
        "unsupported",
        "not supported",
        "unknown parameter",
        "invalid parameter",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn attach_tools(mut body: serde_json::Value, tools: Option<&Vec<serde_json::Value>>) -> serde_json::Value {
    if let Some(tools) = tools {
        if !tools.is_empty() {
            body["tools"] = serde_json::Value::Array(tools.clone());
            body["tool_choice"] = serde_json::Value::String("auto".into());
        }
    }
    body
}

async fn parse_executor_completion(
    response: reqwest::Response,
) -> Result<ExecutorCompletion, ClientError> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        let message = serde_json::from_str::<ApiErrorResponse>(&body)
            .map(|parsed| parsed.error.message)
            .unwrap_or(body);
        return Err(ClientError::Api {
            status: status.as_u16(),
            message,
        });
    }

    let parsed: ChatCompletionResponse = serde_json::from_str(&body).map_err(|err| {
        ClientError::InvalidResponse(format!("{err}; body: {body}"))
    })?;

    let message = parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message)
        .ok_or_else(|| ClientError::InvalidResponse("missing completion choice".into()))?;

    if let Some(refusal) = message.refusal.as_ref().filter(|text| !text.trim().is_empty()) {
        return Err(ClientError::InvalidResponse(format!("model refused: {refusal}")));
    }

    let content = message.text_content();
    let tool_calls = message.tool_calls.filter(|calls| !calls.is_empty());

    if tool_calls.is_none() && content.is_none() {
        return Err(ClientError::InvalidResponse(
            "missing completion content and tool_calls".into(),
        ));
    }

    Ok(ExecutorCompletion {
        content,
        tool_calls,
    })
}

async fn parse_completion_response(
    response: reqwest::Response,
) -> Result<String, ClientError> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        let message = serde_json::from_str::<ApiErrorResponse>(&body)
            .map(|parsed| parsed.error.message)
            .unwrap_or(body);
        return Err(ClientError::Api {
            status: status.as_u16(),
            message,
        });
    }

    let parsed: ChatCompletionResponse = serde_json::from_str(&body).map_err(|err| {
        ClientError::InvalidResponse(format!("{err}; body: {body}"))
    })?;

    let message = parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message)
        .ok_or_else(|| ClientError::InvalidResponse("missing completion choice".into()))?;

    if let Some(refusal) = message.refusal.as_ref().filter(|text| !text.trim().is_empty()) {
        return Err(ClientError::InvalidResponse(format!("model refused: {refusal}")));
    }

    message
        .text_content()
        .ok_or_else(|| ClientError::InvalidResponse("missing completion content".into()))
}

fn build_completion_body(request: &ChatCompletionRequest) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": request.model,
        "messages": request.messages,
    });

    if should_send_temperature(&request.model) {
        body["temperature"] = serde_json::json!(request.temperature);
    }

    attach_token_limit(&mut body, resolve_token_limit(request), &request.model);

    if request.json_object_mode {
        body["response_format"] = serde_json::json!({ "type": "json_object" });
    }

    body
}

fn uses_max_completion_tokens(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.starts_with("o1")
        || lower.starts_with("o3")
        || lower.starts_with("o4")
        || lower.contains("gpt-5")
        || lower.contains("-reasoning")
}

fn should_send_temperature(model: &str) -> bool {
    !uses_max_completion_tokens(model)
}

fn resolve_token_limit(request: &ChatCompletionRequest) -> Option<u32> {
    request.max_tokens.or_else(|| {
        if uses_max_completion_tokens(&request.model) {
            Some(8192)
        } else {
            None
        }
    })
}

fn attach_token_limit(body: &mut serde_json::Value, limit: Option<u32>, model: &str) {
    let Some(limit) = limit else {
        return;
    };

    if uses_max_completion_tokens(model) {
        body["max_completion_tokens"] = serde_json::json!(limit);
    } else {
        body["max_tokens"] = serde_json::json!(limit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_base_url() {
        assert_eq!(
            chat_completions_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn rejects_missing_api_key_for_remote_provider() {
        let settings = AiSettings::default();
        let err = ensure_configured(&settings).unwrap_err();
        assert!(matches!(err, ClientError::NotConfigured));
    }

    #[test]
    fn detects_unsupported_tools_errors() {
        assert!(tools_api_unsupported("tools parameter is not supported"));
        assert!(!tools_api_unsupported("rate limit exceeded"));
    }

    #[test]
    fn local_provider_matches_loopback_hosts_only() {
        assert!(is_local_provider("http://localhost:11434/v1"));
        assert!(is_local_provider("http://127.0.0.1:8080"));
        assert!(is_local_provider("http://[::1]:1234"));
        assert!(!is_local_provider("https://evil.localhost.attacker.com/v1"));
        assert!(!is_local_provider("https://127.0.0.1.evil.com/v1"));
    }

    #[test]
    fn reasoning_models_use_max_completion_tokens() {
        let body = build_completion_body(&ChatCompletionRequest {
            model: "gpt-5-mini".into(),
            messages: vec![ChatMessage::user("hi")],
            temperature: 0.6,
            max_tokens: None,
            tools: None,
            json_object_mode: false,
        });

        assert!(body.get("temperature").is_none());
        assert_eq!(body["max_completion_tokens"], 8192);
        assert!(body.get("max_tokens").is_none());
    }

    #[test]
    fn classic_models_use_max_tokens_and_temperature() {
        let body = build_completion_body(&ChatCompletionRequest {
            model: "gpt-4o-mini".into(),
            messages: vec![ChatMessage::user("hi")],
            temperature: 0.6,
            max_tokens: Some(128),
            tools: None,
            json_object_mode: false,
        });

        assert!((body["temperature"].as_f64().unwrap() - 0.6).abs() < 0.001);
        assert_eq!(body["max_tokens"], 128);
        assert!(body.get("max_completion_tokens").is_none());
    }
}
