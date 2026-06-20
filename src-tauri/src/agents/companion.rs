use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskSpec {
    pub objective: String,
    pub context: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub expected_output: String,
}

impl TaskSpec {
    pub fn validate(&self) -> Result<(), String> {
        if self.objective.trim().is_empty() {
            return Err("objective cannot be empty".into());
        }
        if self.context.trim().is_empty() {
            return Err("context cannot be empty".into());
        }
        if self.expected_output.trim().is_empty() {
            return Err("expected_output cannot be empty".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompanionReply {
    Direct { message: String },
    Delegate { message: String, task_spec: TaskSpec },
}

#[derive(Debug, Deserialize)]
struct RawCompanionResponse {
    mode: String,
    message: String,
    #[serde(default)]
    task_spec: Option<TaskSpec>,
}

#[derive(Debug, thiserror::Error)]
pub enum CompanionError {
    #[error("AI client error: {0}")]
    Client(#[from] crate::ai::ClientError),
    #[error("Could not parse companion response: {0}")]
    Parse(String),
}

pub fn parse_companion_response(raw: &str) -> Result<CompanionReply, CompanionError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CompanionError::Parse("empty companion response".into()));
    }

    let json_str = extract_json_object(trimmed);
    let parsed: RawCompanionResponse = match serde_json::from_str(json_str) {
        Ok(parsed) => parsed,
        Err(err) => {
            if json_str.starts_with('{') {
                return Err(CompanionError::Parse(format!("{err}; raw: {trimmed}")));
            }
            return Ok(CompanionReply::Direct {
                message: trimmed.to_string(),
            });
        }
    };

    match parsed.mode.as_str() {
        "direct" => Ok(CompanionReply::Direct {
            message: parsed.message.trim().to_string(),
        }),
        "delegate" => {
            let task_spec = parsed.task_spec.ok_or_else(|| {
                CompanionError::Parse("delegate mode requires task_spec".into())
            })?;
            Ok(CompanionReply::Delegate {
                message: parsed.message.trim().to_string(),
                task_spec,
            })
        }
        other => Err(CompanionError::Parse(format!("unknown mode: {other}"))),
    }
}

fn extract_json_object(input: &str) -> &str {
    if let Some(start) = input.find("```json") {
        let after = &input[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim();
        }
    }

    if let Some(start) = input.find("```") {
        let after = &input[start + 3..];
        if let Some(end) = after.find("```") {
            return after[..end].trim();
        }
    }

    if let Some(start) = input.find('{') {
        if let Some(end) = input.rfind('}') {
            return &input[start..=end];
        }
    }

    input
}

pub async fn respond(
    settings: &crate::settings::AiSettings,
    history: &[crate::db::Message],
    memories: &[crate::db::Memory],
) -> Result<CompanionReply, CompanionError> {
    use crate::ai::{chat_completion, prompts, ChatCompletionRequest, ChatMessage};

    let mut system = prompts::companion_system_message_for_settings(settings);
    if !memories.is_empty() {
        system.content.push_str("\n\n## User memories\n");
        for memory in memories {
            system.content.push_str("- ");
            system.content.push_str(&memory.content);
            system.content.push('\n');
        }
        system.content.push_str(
            "\nReference memories naturally in your personality's voice.\n",
        );
    }

    let mut messages = vec![system];

    for message in history {
        let role = match message.role.as_str() {
            "user" => "user",
            "companion" => "assistant",
            _ => continue,
        };

        messages.push(ChatMessage {
            role: role.into(),
            content: message.content.clone(),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    let raw = chat_completion(
        settings,
        ChatCompletionRequest {
            model: settings.companion_model.clone(),
            messages,
            temperature: settings.companion_temperature,
            max_tokens: None,
            tools: None,
            json_object_mode: true,
        },
    )
    .await?;

    parse_companion_response(&raw)
}

pub async fn delegate_plan_message(
    settings: &crate::settings::AiSettings,
    history: &[crate::db::Message],
    objective: &str,
    on_delta: Option<&(dyn Fn(&str) + Send + Sync)>,
) -> Result<String, CompanionError> {
    use crate::ai::{chat_completion, chat_completion_stream, prompts, ChatCompletionRequest, ChatMessage};

    let mut messages = vec![prompts::plan_stream_system_message(settings)];

    for message in history.iter().rev().take(6).collect::<Vec<_>>().into_iter().rev() {
        let role = match message.role.as_str() {
            "user" => "user",
            "companion" => "assistant",
            _ => continue,
        };
        messages.push(ChatMessage {
            role: role.into(),
            content: message.content.clone(),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    messages.push(ChatMessage {
        role: "user".into(),
        content: format!("User request:\n{objective}"),
        tool_calls: None,
        tool_call_id: None,
    });

    let request = ChatCompletionRequest {
        model: settings.companion_model.clone(),
        messages,
        temperature: 0.75,
        max_tokens: Some(220),
        tools: None,
        json_object_mode: false,
    };

    let raw = if let Some(on_delta) = on_delta {
        chat_completion_stream(settings, request, |delta| on_delta(delta)).await?
    } else {
        chat_completion(settings, request).await?
    };

    Ok(raw.trim().to_string())
}

pub async fn format_executor_result(
    settings: &crate::settings::AiSettings,
    history: &[crate::db::Message],
    task_spec: &TaskSpec,
    executor_result: &crate::agents::executor::ExecutorResult,
    on_delta: Option<&(dyn Fn(&str) + Send + Sync)>,
) -> Result<String, CompanionError> {
    use crate::ai::{chat_completion, chat_completion_stream, prompts, ChatCompletionRequest, ChatMessage};

    let task_json = serde_json::to_string(task_spec).map_err(|err| {
        CompanionError::Parse(format!("task spec serialize: {err}"))
    })?;
    let result_json = serde_json::to_string(executor_result).map_err(|err| {
        CompanionError::Parse(format!("executor result serialize: {err}"))
    })?;

    let mut messages = vec![prompts::format_stream_system_message(settings)];

    for message in history {
        let role = match message.role.as_str() {
            "user" => "user",
            "companion" => "assistant",
            _ => continue,
        };
        messages.push(ChatMessage {
            role: role.into(),
            content: message.content.clone(),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    messages.push(ChatMessage {
        role: "user".into(),
        content: format!(
            "Present these executor results to the user.\n\nTask:\n{task_json}\n\nExecutor output:\n{result_json}"
        ),
        tool_calls: None,
        tool_call_id: None,
    });

    let request = ChatCompletionRequest {
        model: settings.companion_model.clone(),
        messages,
        temperature: settings.companion_temperature,
        max_tokens: None,
        tools: None,
        json_object_mode: false,
    };

    let raw = if let Some(on_delta) = on_delta {
        chat_completion_stream(settings, request, |delta| on_delta(delta)).await?
    } else {
        chat_completion(settings, request).await?
    };

    Ok(raw.trim().to_string())
}

pub fn not_configured_message() -> &'static str {
    "I'd love to chat with you properly — but I need an API key first.\n\nOpen Settings (top right), add your key and provider, then come back. I'll be right here."
}

pub fn error_message(_err: &str) -> String {
    "Something went wrong on my end — I'm sorry about that. \
     If this keeps happening, check your connection in Settings and try again in a moment."
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_task_spec() {
        let spec = TaskSpec {
            objective: "".into(),
            context: "ctx".into(),
            constraints: vec![],
            expected_output: "out".into(),
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn parses_direct_json() {
        let raw = r#"{"mode":"direct","message":"Hey there!"}"#;
        let reply = parse_companion_response(raw).unwrap();
        assert_eq!(
            reply,
            CompanionReply::Direct {
                message: "Hey there!".into()
            }
        );
    }

    #[test]
    fn parses_delegate_json() {
        let raw = r#"{
            "mode": "delegate",
            "message": "Give me a moment.",
            "task_spec": {
                "objective": "Plan week",
                "context": "User wants help",
                "constraints": ["actionable"],
                "expected_output": "weekly plan"
            }
        }"#;
        let reply = parse_companion_response(raw).unwrap();
        assert!(matches!(reply, CompanionReply::Delegate { .. }));
    }

    #[test]
    fn extracts_json_from_fences() {
        let raw = "```json\n{\"mode\":\"direct\",\"message\":\"Hi\"}\n```";
        let reply = parse_companion_response(raw).unwrap();
        assert_eq!(
            reply,
            CompanionReply::Direct { message: "Hi".into() }
        );
    }

    #[test]
    fn treats_plain_text_as_direct_reply() {
        let raw = "Hey — good to see you.";
        let reply = parse_companion_response(raw).unwrap();
        assert_eq!(
            reply,
            CompanionReply::Direct {
                message: "Hey — good to see you.".into()
            }
        );
    }
}
