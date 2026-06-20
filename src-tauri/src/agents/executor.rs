use serde::{Deserialize, Serialize};

use crate::agents::companion::TaskSpec;
use crate::ai::{
    chat_completion, chat_completion_executor, prompts, tools_api_unsupported, ChatCompletionRequest,
    ChatMessage, ClientError, ToolCall,
};
use crate::changes::ChangeTracker;
use crate::db::Message;
use crate::run_state::RunState;
use crate::settings::{AiSettings, EXECUTOR_MAX_STEPS};
use crate::tools::{
    execute_tool, is_workspace_tool, openai_tool_definitions, resolve_verify_command, run_verify,
    tool_context_from_settings, validate_tool_call, ToolContext, ToolResult, MAX_VERIFY_RETRIES,
};
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityStep {
    pub step: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutorStatus {
    Success,
    NeedsClarification,
    Error,
}

impl ExecutorStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::NeedsClarification => "needs_clarification",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutorResult {
    pub status: ExecutorStatus,
    pub summary: String,
    pub content: String,
    #[serde(default)]
    pub activity_log: Vec<ActivityStep>,
    #[serde(default)]
    pub file_changes: Vec<crate::changes::FileChange>,
}

#[derive(Debug, Clone)]
pub struct ExecutorProgress {
    pub task_spec: TaskSpec,
    pub status: String,
    pub summary: String,
    pub activity_log: Vec<ActivityStep>,
}

pub type ProgressFn<'a> = dyn Fn(ExecutorProgress) + Send + Sync + 'a;

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("AI client error: {0}")]
    Client(#[from] crate::ai::ClientError),
    #[error("Invalid task specification: {0}")]
    InvalidTaskSpec(String),
    #[error("Could not parse executor response: {0}")]
    Parse(String),
    #[error("Executor exceeded maximum steps ({EXECUTOR_MAX_STEPS})")]
    MaxSteps,
    #[error("Run cancelled")]
    Cancelled,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ExecutorAction {
    ToolCall {
        tool: String,
        #[serde(default)]
        arguments: serde_json::Value,
    },
    FinalAnswer {
        status: String,
        #[serde(default)]
        summary: String,
        #[serde(default)]
        content: String,
    },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawExecutorResponse {
    status: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    activity_log: Vec<ActivityStep>,
}

#[allow(dead_code)]
pub fn parse_executor_response(raw: &str) -> Result<ExecutorResult, ExecutorError> {
    let trimmed = raw.trim();
    let json_str = extract_json_object(trimmed);

    let parsed: RawExecutorResponse = serde_json::from_str(json_str).map_err(|err| {
        ExecutorError::Parse(format!("{err}; raw: {trimmed}"))
    })?;

    let status = parse_status(&parsed.status)?;

    Ok(ExecutorResult {
        status,
        summary: parsed.summary.trim().to_string(),
        content: parsed.content.trim().to_string(),
        activity_log: parsed.activity_log,
        file_changes: vec![],
    })
}

fn parse_status(raw: &str) -> Result<ExecutorStatus, ExecutorError> {
    match raw {
        "success" => Ok(ExecutorStatus::Success),
        "needs_clarification" => Ok(ExecutorStatus::NeedsClarification),
        "error" => Ok(ExecutorStatus::Error),
        other => Err(ExecutorError::Parse(format!("unknown status: {other}"))),
    }
}

fn parse_executor_action(raw: &str) -> Result<ExecutorAction, ExecutorError> {
    let json_str = extract_json_object(raw.trim());
    serde_json::from_str(json_str).map_err(|err| ExecutorError::Parse(format!("{err}; raw: {raw}")))
}

fn extract_json_object(input: &str) -> &str {
    if let Some(start) = input.find("```json") {
        let after = &input[start + 7..];
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

fn recent_message_lines(history: &[Message], limit: usize) -> Vec<String> {
    history
        .iter()
        .rev()
        .take(limit)
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn emit_progress(
    progress: Option<&ProgressFn<'_>>,
    task_spec: &TaskSpec,
    activity_log: &[ActivityStep],
    status: &str,
    summary: &str,
) {
    if let Some(callback) = progress {
        callback(ExecutorProgress {
            task_spec: task_spec.clone(),
            status: status.into(),
            summary: summary.into(),
            activity_log: activity_log.to_vec(),
        });
    }
}

pub async fn execute(
    settings: &AiSettings,
    task_spec: &TaskSpec,
    history: &[Message],
    progress: Option<&ProgressFn<'_>>,
    cancel: Option<&AtomicBool>,
) -> Result<ExecutorResult, ExecutorError> {
    task_spec
        .validate()
        .map_err(ExecutorError::InvalidTaskSpec)?;

    let tool_ctx = tool_context_from_settings(
        &settings.workspace_path,
        settings.allow_file_overwrites,
        &settings.command_allowlist_extra,
    );
    let mut change_tracker = ChangeTracker::default();
    let mut verify_failures = 0usize;

    let task_json = serde_json::to_string(task_spec).map_err(|err| {
        ExecutorError::Parse(format!("could not serialize task spec: {err}"))
    })?;

    let recent = recent_message_lines(history, 8);
    let mut activity_log = vec![ActivityStep {
        step: "start".into(),
        detail: task_spec.objective.clone(),
    }];

    let mut tool_definitions = openai_tool_definitions();
    let mcp_session = if settings.mcp_enabled {
        settings
            .mcp_server_command
            .as_ref()
            .and_then(|command| match crate::mcp::McpSession::spawn(command) {
                Ok(session) => Some(session),
                Err(err) => {
                    activity_log.push(ActivityStep {
                        step: "mcp".into(),
                        detail: err,
                    });
                    None
                }
            })
    } else {
        None
    };

    if let Some(session) = &mcp_session {
        if let Ok(tools) = session.list_tools() {
            tool_definitions.extend(crate::mcp::openai_tools_from_mcp(&tools));
        }
    }

    let mut use_native_tools = true;

    emit_progress(
        progress,
        task_spec,
        &activity_log,
        "running",
        "Starting executor…",
    );

    let mut messages = vec![
        prompts::executor_system_message(settings.workspace_configured()),
        ChatMessage::user(format!(
            "Task specification:\n{task_json}\n\nRecent conversation:\n{}",
            recent.join("\n")
        )),
    ];

    for step in 0..EXECUTOR_MAX_STEPS {
        if cancel.is_some_and(RunState::is_cancelled) {
            return Err(ExecutorError::Cancelled);
        }

        if use_native_tools {
            let request = ChatCompletionRequest {
                model: settings.executor_model.clone(),
                messages: messages.clone(),
                temperature: settings.executor_temperature,
                max_tokens: None,
                tools: Some(tool_definitions.clone()),
                json_object_mode: false,
            };

            match chat_completion_executor(settings, request).await {
                Ok(completion) => {
                    if let Some(tool_calls) = completion.tool_calls {
                        if let Some(result) = handle_native_tool_calls(
                            tool_calls,
                            completion.content,
                            settings,
                            task_spec,
                            &tool_ctx,
                            &mut change_tracker,
                            &mut verify_failures,
                            &mut activity_log,
                            &mut messages,
                            progress,
                            cancel,
                            mcp_session.as_ref(),
                        )? {
                            return Ok(result);
                        }
                        continue;
                    }

                    if let Some(content) = completion.content {
                        if let Some(result) = handle_json_action(
                            &content,
                            settings,
                            task_spec,
                            &tool_ctx,
                            &mut change_tracker,
                            &mut verify_failures,
                            &mut activity_log,
                            &mut messages,
                            progress,
                            cancel,
                            mcp_session.as_ref(),
                        )? {
                            return Ok(result);
                        }
                        continue;
                    }
                }
                Err(ClientError::Api { message, .. }) if tools_api_unsupported(&message) => {
                    use_native_tools = false;
                    continue;
                }
                Err(err) => return Err(err.into()),
            }
        }

        let raw = chat_completion(
            settings,
            ChatCompletionRequest {
                model: settings.executor_model.clone(),
                messages: messages.clone(),
                temperature: settings.executor_temperature,
                max_tokens: None,
                tools: None,
                json_object_mode: false,
            },
        )
        .await?;

        if let Some(result) = handle_json_action(
            &raw,
            settings,
            task_spec,
            &tool_ctx,
            &mut change_tracker,
            &mut verify_failures,
            &mut activity_log,
            &mut messages,
            progress,
            cancel,
            mcp_session.as_ref(),
        )? {
            return Ok(result);
        }

        let _ = step;
    }

    Err(ExecutorError::MaxSteps)
}

#[allow(clippy::too_many_arguments)]
fn handle_native_tool_calls(
    tool_calls: Vec<ToolCall>,
    assistant_content: Option<String>,
    settings: &AiSettings,
    task_spec: &TaskSpec,
    tool_ctx: &Result<ToolContext, crate::tools::SandboxError>,
    change_tracker: &mut ChangeTracker,
    verify_failures: &mut usize,
    activity_log: &mut Vec<ActivityStep>,
    messages: &mut Vec<ChatMessage>,
    progress: Option<&ProgressFn<'_>>,
    cancel: Option<&AtomicBool>,
    mcp_session: Option<&crate::mcp::McpSession>,
) -> Result<Option<ExecutorResult>, ExecutorError> {
    for call in &tool_calls {
        if call.function.name == "final_answer" {
            let args = parse_tool_arguments(&call.function.arguments)?;
            validate_tool_call("final_answer", &args)
                .map_err(ExecutorError::Parse)?;
            let result = build_final_result(args, activity_log, tool_ctx, change_tracker)?;
            emit_progress(
                progress,
                task_spec,
                &result.activity_log,
                result.status.as_str(),
                &result.summary,
            );
            return Ok(Some(result));
        }
    }

    messages.push(ChatMessage {
        role: "assistant".into(),
        content: assistant_content.unwrap_or_default(),
        tool_calls: Some(tool_calls.clone()),
        tool_call_id: None,
    });

    for call in tool_calls {
        if cancel.is_some_and(RunState::is_cancelled) {
            return Err(ExecutorError::Cancelled);
        }

        if call.function.name == "final_answer" {
            continue;
        }

        if crate::mcp::is_mcp_tool(&call.function.name) {
            let arguments = parse_tool_arguments(&call.function.arguments)?;
            let tool_result = if let Some(session) = mcp_session {
                match session.call_tool(&call.function.name, &arguments) {
                    Ok(output) => ToolResult {
                        ok: true,
                        output,
                        error: None,
                    },
                    Err(err) => ToolResult {
                        ok: false,
                        output: String::new(),
                        error: Some(err),
                    },
                }
            } else {
                ToolResult {
                    ok: false,
                    output: String::new(),
                    error: Some("MCP is not connected".into()),
                }
            };
            record_tool_activity(activity_log, &call.function.name, &tool_result);
            messages.push(tool_result_message(
                &call.id,
                tool_observation(&call.function.name, &tool_result),
            ));
            continue;
        }

        if !is_workspace_tool(&call.function.name) {
            let err = format!("unknown tool: {}", call.function.name);
            activity_log.push(ActivityStep {
                step: format!("tool:{}", call.function.name),
                detail: err.clone(),
            });
            messages.push(tool_result_message(
                &call.id,
                serde_json::json!({ "ok": false, "error": err }),
            ));
            continue;
        }

        let arguments = parse_tool_arguments(&call.function.arguments)?;
        if let Err(err) = validate_tool_call(&call.function.name, &arguments) {
            activity_log.push(ActivityStep {
                step: format!("tool:{}", call.function.name),
                detail: err.clone(),
            });
            emit_progress(
                progress,
                task_spec,
                activity_log,
                "running",
                &format!("Invalid {} call", call.function.name),
            );
            messages.push(tool_result_message(
                &call.id,
                serde_json::json!({ "ok": false, "error": err }),
            ));
            continue;
        }

        let tool_result = match tool_ctx {
            Ok(ctx) => execute_tracked_tool(ctx, change_tracker, &call.function.name, &arguments),
            Err(_) => ToolResult {
                ok: false,
                output: String::new(),
                error: Some(
                    "Workspace is not configured. Ask the user to pick a project folder in Settings."
                        .into(),
                ),
            },
        };

        record_tool_activity(activity_log, &call.function.name, &tool_result);
        emit_progress(
            progress,
            task_spec,
            activity_log,
            "running",
            &format!("Ran {}", call.function.name),
        );

        if tool_result.ok
            && settings.verify_enabled
            && is_verify_trigger_tool(&call.function.name)
            && *verify_failures < MAX_VERIFY_RETRIES
        {
            if let Ok(ctx) = tool_ctx {
                if let Some(verify_output) =
                    run_auto_verify(settings, ctx, activity_log, verify_failures)
                {
                    messages.push(tool_result_message(
                        &call.id,
                        tool_observation(&call.function.name, &tool_result),
                    ));
                    messages.push(ChatMessage::user(verify_output));
                    emit_progress(
                        progress,
                        task_spec,
                        activity_log,
                        "running",
                        "Verify failed — fixing…",
                    );
                    return Ok(None);
                }
            }
        }

        messages.push(tool_result_message(
            &call.id,
            tool_observation(&call.function.name, &tool_result),
        ));
    }

    Ok(None)
}

#[allow(clippy::too_many_arguments)]
fn handle_json_action(
    raw: &str,
    settings: &AiSettings,
    task_spec: &TaskSpec,
    tool_ctx: &Result<ToolContext, crate::tools::SandboxError>,
    change_tracker: &mut ChangeTracker,
    verify_failures: &mut usize,
    activity_log: &mut Vec<ActivityStep>,
    messages: &mut Vec<ChatMessage>,
    progress: Option<&ProgressFn<'_>>,
    cancel: Option<&AtomicBool>,
    mcp_session: Option<&crate::mcp::McpSession>,
) -> Result<Option<ExecutorResult>, ExecutorError> {
    let action = parse_executor_action(raw)?;

    match action {
        ExecutorAction::FinalAnswer {
            status,
            summary,
            content,
        } => {
            let status = parse_status(&status)?;
            let mut result = ExecutorResult {
                status,
                summary: summary.trim().to_string(),
                content: content.trim().to_string(),
                activity_log: activity_log.clone(),
                file_changes: finalize_changes(tool_ctx, change_tracker),
            };

            if result.activity_log.is_empty() {
                result.activity_log.push(ActivityStep {
                    step: "complete".into(),
                    detail: result.summary.clone(),
                });
            }

            emit_progress(
                progress,
                task_spec,
                &result.activity_log,
                result.status.as_str(),
                &result.summary,
            );

            Ok(Some(result))
        }
        ExecutorAction::ToolCall { tool, arguments } => {
            if cancel.is_some_and(RunState::is_cancelled) {
                return Err(ExecutorError::Cancelled);
            }

            if crate::mcp::is_mcp_tool(&tool) {
                let tool_result = if let Some(session) = mcp_session {
                    match session.call_tool(&tool, &arguments) {
                        Ok(output) => ToolResult {
                            ok: true,
                            output,
                            error: None,
                        },
                        Err(err) => ToolResult {
                            ok: false,
                            output: String::new(),
                            error: Some(err),
                        },
                    }
                } else {
                    ToolResult {
                        ok: false,
                        output: String::new(),
                        error: Some("MCP is not connected".into()),
                    }
                };
                record_tool_activity(activity_log, &tool, &tool_result);
                messages.push(ChatMessage::assistant(raw));
                messages.push(ChatMessage::user(format!(
                    "Tool result:\n{}",
                    tool_observation(&tool, &tool_result)
                )));
                return Ok(None);
            }

            if let Err(err) = validate_tool_call(&tool, &arguments) {
                activity_log.push(ActivityStep {
                    step: format!("tool:{tool}"),
                    detail: err.clone(),
                });
                emit_progress(
                    progress,
                    task_spec,
                    activity_log,
                    "running",
                    &format!("Invalid {tool} call"),
                );
                messages.push(ChatMessage::user(format!("Tool validation error:\n{err}")));
                return Ok(None);
            }

            let tool_result = match tool_ctx {
                Ok(ctx) => execute_tracked_tool(ctx, change_tracker, &tool, &arguments),
                Err(_) => ToolResult {
                    ok: false,
                    output: String::new(),
                    error: Some(
                        "Workspace is not configured. Ask the user to pick a project folder in Settings."
                            .into(),
                    ),
                },
            };

            record_tool_activity(activity_log, &tool, &tool_result);
            emit_progress(
                progress,
                task_spec,
                activity_log,
                "running",
                &format!("Ran {tool}"),
            );

            if tool_result.ok
                && settings.verify_enabled
                && is_verify_trigger_tool(&tool)
                && *verify_failures < MAX_VERIFY_RETRIES
            {
                if let Ok(ctx) = tool_ctx {
                    if let Some(verify_output) =
                        run_auto_verify(settings, ctx, activity_log, verify_failures)
                    {
                        messages.push(ChatMessage::assistant(raw));
                        messages.push(ChatMessage::user(verify_output));
                        emit_progress(
                            progress,
                            task_spec,
                            activity_log,
                            "running",
                            "Verify failed — fixing…",
                        );
                        return Ok(None);
                    }
                }
            }

            messages.push(ChatMessage::assistant(raw));
            messages.push(ChatMessage::user(format!(
                "Tool result:\n{}",
                tool_observation(&tool, &tool_result)
            )));
            Ok(None)
        }
    }
}

fn parse_tool_arguments(raw: &str) -> Result<serde_json::Value, ExecutorError> {
    serde_json::from_str(raw).map_err(|err| ExecutorError::Parse(format!("{err}; raw: {raw}")))
}

fn tool_observation(tool: &str, tool_result: &ToolResult) -> serde_json::Value {
    serde_json::json!({
        "tool": tool,
        "ok": tool_result.ok,
        "output": tool_result.output,
        "error": tool_result.error,
    })
}

fn tool_result_message(tool_call_id: &str, observation: serde_json::Value) -> ChatMessage {
    ChatMessage {
        role: "tool".into(),
        content: observation.to_string(),
        tool_calls: None,
        tool_call_id: Some(tool_call_id.into()),
    }
}

fn build_final_result(
    args: serde_json::Value,
    activity_log: &[ActivityStep],
    tool_ctx: &Result<ToolContext, crate::tools::SandboxError>,
    change_tracker: &mut ChangeTracker,
) -> Result<ExecutorResult, ExecutorError> {
    let status = args
        .get("status")
        .and_then(|value| value.as_str())
        .ok_or_else(|| ExecutorError::Parse("final_answer requires status".into()))?;
    let summary = args
        .get("summary")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let content = args
        .get("content")
        .and_then(|value| value.as_str())
        .unwrap_or_default();

    let status = parse_status(status)?;
    let mut result = ExecutorResult {
        status,
        summary: summary.trim().to_string(),
        content: content.trim().to_string(),
        activity_log: activity_log.to_vec(),
        file_changes: finalize_changes(tool_ctx, change_tracker),
    };

    if result.activity_log.is_empty() {
        result.activity_log.push(ActivityStep {
            step: "complete".into(),
            detail: result.summary.clone(),
        });
    }

    Ok(result)
}

fn record_tool_activity(activity_log: &mut Vec<ActivityStep>, tool: &str, tool_result: &ToolResult) {
    let detail = if tool_result.ok {
        truncate_for_log(&tool_result.output, 240)
    } else {
        tool_result
            .error
            .clone()
            .unwrap_or_else(|| "tool failed".into())
    };

    activity_log.push(ActivityStep {
        step: format!("tool:{tool}"),
        detail,
    });
}

fn finalize_changes(
    tool_ctx: &Result<ToolContext, crate::tools::SandboxError>,
    tracker: &mut ChangeTracker,
) -> Vec<crate::changes::FileChange> {
    match tool_ctx {
        Ok(ctx) => tracker.finalize(&ctx.sandbox),
        Err(_) => Vec::new(),
    }
}

fn is_verify_trigger_tool(tool: &str) -> bool {
    matches!(tool, "write_file" | "edit_file" | "delete_file")
}

fn extract_tool_path(tool: &str, args: &serde_json::Value) -> Option<String> {
    match tool {
        "write_file" | "edit_file" | "delete_file" | "read_file" | "file_info" | "create_dir" => {
            args.get("path").and_then(|v| v.as_str()).map(str::to_string)
        }
        _ => None,
    }
}

fn execute_tracked_tool(
    ctx: &ToolContext,
    tracker: &mut ChangeTracker,
    tool: &str,
    args: &serde_json::Value,
) -> ToolResult {
    if is_mutating_tool(tool) {
        if let Some(path) = extract_tool_path(tool, args) {
            tracker.capture_before(&ctx.sandbox, &path);
        }
    }

    let result = execute_tool(ctx, tool, args);

    if result.ok {
        if let Some(path) = extract_tool_path(tool, args) {
            tracker.note_touched(&path);
        }
    }

    result
}

fn is_mutating_tool(tool: &str) -> bool {
    matches!(
        tool,
        "write_file" | "edit_file" | "delete_file" | "create_dir"
    )
}

fn run_auto_verify(
    settings: &AiSettings,
    ctx: &ToolContext,
    activity_log: &mut Vec<ActivityStep>,
    verify_failures: &mut usize,
) -> Option<String> {
    let command = resolve_verify_command(
        ctx.sandbox.root(),
        settings.verify_command.as_deref(),
    )?;

    let result = run_verify(&ctx.sandbox, &command, &settings.command_allowlist_extra);
    if result.ok {
        activity_log.push(ActivityStep {
            step: "verify".into(),
            detail: format!("{command} passed"),
        });
        return None;
    }

    *verify_failures += 1;
    let detail = truncate_for_log(&result.output, 400);
    activity_log.push(ActivityStep {
        step: "verify".into(),
        detail: format!("{command} failed ({verify_failures}/{MAX_VERIFY_RETRIES}): {detail}"),
    });

    Some(format!(
        "Verify command failed ({command}). Fix the code and try again.\n{}",
        result.output
    ))
}

fn truncate_for_log(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max).collect();
    format!("{truncated}…")
}

pub fn should_skip_format_pass(result: &ExecutorResult) -> bool {
    matches!(result.status, ExecutorStatus::Success)
        && result.content.len() <= 600
        && !result.content.contains("```")
        && result
            .activity_log
            .iter()
            .any(|step| step.step.starts_with("tool:"))
}

pub fn template_format(result: &ExecutorResult) -> String {
    if result.content.trim().is_empty() {
        result.summary.clone()
    } else if result.summary.trim().is_empty() {
        result.content.clone()
    } else {
        format!("{}\n\n{}", result.summary.trim(), result.content.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::companion::TaskSpec;

    #[test]
    fn parses_success_response() {
        let raw = r#"{"status":"success","summary":"Done","content":"Plan here","activity_log":[]}"#;
        let result = parse_executor_response(raw).unwrap();
        assert_eq!(result.status, ExecutorStatus::Success);
        assert_eq!(result.content, "Plan here");
    }

    #[test]
    fn parses_tool_call_action() {
        let raw = r#"{"action":"tool_call","tool":"list_dir","arguments":{"path":"."}}"#;
        let action = parse_executor_action(raw).unwrap();
        assert!(matches!(action, ExecutorAction::ToolCall { .. }));
    }

    #[test]
    fn rejects_invalid_task_spec() {
        let spec = TaskSpec {
            objective: "".into(),
            context: "ctx".into(),
            constraints: vec![],
            expected_output: "out".into(),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(execute(&AiSettings::default(), &spec, &[], None, None))
            .unwrap_err();
        assert!(matches!(err, ExecutorError::InvalidTaskSpec(_)));
    }

    #[test]
    fn builds_final_answer_from_native_args() {
        let args = serde_json::json!({
            "status": "success",
            "summary": "Updated add()",
            "content": "Tests pass."
        });
        let mut log = vec![];
        let mut tracker = ChangeTracker::default();
        let result = build_final_result(
            args,
            &mut log,
            &Err(crate::tools::SandboxError::NotConfigured),
            &mut tracker,
        )
        .unwrap();
        assert_eq!(result.status, ExecutorStatus::Success);
        assert_eq!(result.content, "Tests pass.");
    }

    #[test]
    fn skip_format_for_short_tool_results() {
        let result = ExecutorResult {
            status: ExecutorStatus::Success,
            summary: "Found files".into(),
            content: "src/main.rs".into(),
            activity_log: vec![ActivityStep {
                step: "tool:list_dir".into(),
                detail: "src".into(),
            }],
            file_changes: vec![],
        };
        assert!(should_skip_format_pass(&result));
    }
}
