use serde::{Deserialize, Serialize};

use crate::agents::profile::{
    build_scout_briefing, is_implementation_task, maybe_promote_to_editor_for_greenfield,
    should_escalate_after_empty_listing, should_escalate_after_exploration_loop,
    should_escalate_after_read_failures, AgentPhase, RunConfig,
};
use crate::agents::task::TaskSpec;
use crate::ai::{
    chat_completion, chat_completion_executor, prompts, tools_api_unsupported, ChatCompletionRequest,
    ChatMessage, ClientError, ToolCall,
};
use crate::changes::ChangeTracker;
use crate::db::Message;
use crate::run_state::RunState;
use crate::settings::AiSettings;
use std::time::{Duration, Instant};
use crate::tools::{
    execute_tool_async, is_workspace_tool, openai_tool_definitions_filtered, resolve_verify_command,
    run_verify, tool_context_from_settings, validate_tool_call, ToolContext, ToolResult,
    MAX_VERIFY_RETRIES,
};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

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
    #[error("Run cancelled")]
    Cancelled,
    #[error("Plan approval required")]
    PlanApprovalRequired(Box<PendingPlanApproval>),
}

impl ExecutorError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Client(err) => err.user_message(),
            Self::InvalidTaskSpec(detail) => {
                format!("Could not start the agent task: {detail}")
            }
            Self::Parse(detail) => {
                if detail.len() > 120 {
                    "The model returned a format ThatCode could not parse. In Settings, pick a model with tool/function support (e.g. GPT-4o or Claude 3.5), or try the Deep tier.".into()
                } else {
                    format!("The model returned a response ThatCode could not parse: {detail}")
                }
            }
            Self::Cancelled => "Run stopped — you can ask again anytime.".into(),
            Self::PlanApprovalRequired(_) => "Waiting for plan approval.".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingPlanApproval {
    pub briefing: String,
    pub task_spec: TaskSpec,
    pub activity_log: Vec<ActivityStep>,
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

fn parse_executor_action_flexible(raw: &str, allow_plain_text: bool) -> Result<ExecutorAction, ExecutorError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ExecutorError::Parse("empty response".into()));
    }

    match parse_executor_action(trimmed) {
        Ok(action) => Ok(action),
        Err(primary_err) => {
            if let Ok(result) = parse_executor_response(trimmed) {
                return Ok(final_answer_from_result(result));
            }

            if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
                if allow_plain_text {
                    return Ok(plain_text_final_answer(trimmed));
                }
                return Err(ExecutorError::Parse(PLAIN_TEXT_RESPONSE.into()));
            }

            Err(primary_err)
        }
    }
}

pub(crate) const PLAIN_TEXT_RESPONSE: &str = "plain_text_response";

fn final_answer_from_result(result: ExecutorResult) -> ExecutorAction {
    let status = match result.status {
        ExecutorStatus::Success => "success",
        ExecutorStatus::NeedsClarification => "needs_clarification",
        ExecutorStatus::Error => "error",
    };
    ExecutorAction::FinalAnswer {
        status: status.into(),
        summary: result.summary,
        content: result.content,
    }
}

fn plain_text_final_answer(text: &str) -> ExecutorAction {
    let summary = text
        .lines()
        .next()
        .unwrap_or(text)
        .chars()
        .take(120)
        .collect::<String>();
    ExecutorAction::FinalAnswer {
        status: "success".into(),
        summary,
        content: text.to_string(),
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
        let after = input[start + 3..].trim_start();
        if after.starts_with('{') {
            if let Some(end) = after.find("```") {
                return after[..end].trim();
            }
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

#[derive(Debug, Default)]
struct StepCounters {
    scout: usize,
    editor: usize,
}

impl StepCounters {
    fn count(&self, phase: AgentPhase) -> usize {
        match phase {
            AgentPhase::Scout => self.scout,
            AgentPhase::Editor => self.editor,
        }
    }

    fn increment(&mut self, phase: AgentPhase) {
        match phase {
            AgentPhase::Scout => self.scout += 1,
            AgentPhase::Editor => self.editor += 1,
        }
    }
}

#[derive(Default)]
struct LoopGuard {
    targets: HashMap<String, usize>,
}

impl LoopGuard {
    fn target_key(tool: &str, arguments: &serde_json::Value) -> Option<String> {
        match tool {
            "read_file" | "file_info" => arguments
                .get("path")
                .and_then(|value| value.as_str())
                .map(|path| format!("read:{path}")),
            "list_dir" => arguments
                .get("path")
                .and_then(|value| value.as_str())
                .map(|path| format!("list:{path}")),
            "grep" => {
                let path = arguments
                    .get("path")
                    .and_then(|value| value.as_str())
                    .unwrap_or(".");
                let pattern = arguments
                    .get("pattern")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                Some(format!("grep:{path}:{pattern}"))
            }
            "search_files" => arguments
                .get("query")
                .and_then(|value| value.as_str())
                .map(|query| format!("search:{query}")),
            _ => None,
        }
    }

    fn note(&mut self, tool: &str, arguments: &serde_json::Value) -> Option<String> {
        let key = Self::target_key(tool, arguments)?;
        let count = self.targets.entry(key).or_insert(0);
        *count += 1;
        match *count {
            2 => Some(format!(
                "System: You already called `{tool}` on the same target. Do not repeat — edit, write, run a command, or return final_answer."
            )),
            n if n >= 3 => Some(format!(
                "System: Stop repeating `{tool}`. Make a concrete change with edit_file/write_file or return final_answer with your best result."
            )),
            _ => None,
        }
    }

    fn read_only_streak(activity_log: &[ActivityStep]) -> usize {
        activity_log
            .iter()
            .rev()
            .take_while(|step| {
                step.step.starts_with("tool:")
                    && step
                        .step
                        .strip_prefix("tool:")
                        .is_some_and(|name| crate::agents::profile::READ_ONLY_TOOLS.contains(&name))
            })
            .count()
    }
}

fn loop_guard_nudge(
    loop_guard: &mut LoopGuard,
    run_config: &RunConfig,
    tool: &str,
    arguments: &serde_json::Value,
    activity_log: &[ActivityStep],
    objective: &str,
) -> Option<String> {
    loop_guard.note(tool, arguments).or_else(|| {
        if run_config.phase == AgentPhase::Editor && is_implementation_task(objective) {
            let streak = LoopGuard::read_only_streak(activity_log);
            if streak >= 4 {
                return Some(
                    "System: Enough exploration — use edit_file, write_file, or run_command next, then final_answer."
                        .into(),
                );
            }
        }
        if run_config.phase == AgentPhase::Scout {
            let streak = LoopGuard::read_only_streak(activity_log);
            if streak >= 6 {
                return Some(
                    "System: Summarize findings in final_answer now, or call a mutating tool to implement."
                        .into(),
                );
            }
        }
        None
    })
}

fn should_force_escalation(
    run_config: &RunConfig,
    activity_log: &[ActivityStep],
    tool: &str,
    tool_result: &ToolResult,
    objective: &str,
) -> bool {
    should_escalate_after_empty_listing(run_config, tool, tool_result, objective)
        || should_escalate_after_read_failures(run_config, activity_log, objective)
        || should_escalate_after_exploration_loop(run_config, activity_log, objective)
}

fn phase_running_summary(run_config: &RunConfig, phase: AgentPhase) -> String {
    format!(
        "{} ({})",
        run_config.phase_label(phase),
        run_config.model_for_phase(phase)
    )
}

fn try_escalate_to_editor(
    settings: &AiSettings,
    run_config: &mut RunConfig,
    counters: &StepCounters,
    task_spec: &TaskSpec,
    activity_log: &mut Vec<ActivityStep>,
    messages: &mut Vec<ChatMessage>,
    reason: &str,
) -> Result<(), ExecutorError> {
    if settings.plan_before_edit {
        let briefing = build_scout_briefing(activity_log, task_spec);
        return Err(ExecutorError::PlanApprovalRequired(Box::new(PendingPlanApproval {
            briefing,
            task_spec: task_spec.clone(),
            activity_log: activity_log.clone(),
        })));
    }

    escalate_to_editor(
        run_config,
        counters,
        settings,
        task_spec,
        activity_log,
        messages,
        reason,
    );
    Ok(())
}

fn escalate_to_editor(
    run_config: &mut RunConfig,
    counters: &StepCounters,
    settings: &AiSettings,
    task_spec: &TaskSpec,
    activity_log: &mut Vec<ActivityStep>,
    messages: &mut Vec<ChatMessage>,
    reason: &str,
) {
    activity_log.push(ActivityStep {
        step: "phase:escalate".into(),
        detail: format!(
            "Scout → Editor ({}) — {reason}",
            run_config.strong_model
        ),
    });
    run_config.phase = AgentPhase::Editor;
    let briefing = build_scout_briefing(activity_log, task_spec);
    messages.clear();
    messages.push(prompts::agent_system_message_for_phase(
        settings.workspace_configured(),
        AgentPhase::Editor,
    ));
    messages.push(ChatMessage::user(briefing));
    let _ = counters;
}

pub async fn execute(
    settings: &AiSettings,
    run_config: &mut RunConfig,
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
        detail: format!(
            "{} — {}",
            run_config.tier.as_str(),
            task_spec.objective
        ),
    }];

    let mcp_command: Option<&str> = if settings.mcp_enabled {
        settings.mcp_server_command.as_deref()
    } else {
        None
    };

    let mcp_session = if let Some(command) = mcp_command {
        match tokio::time::timeout(
            Duration::from_secs(30),
            crate::mcp::spawn_async(command),
        )
        .await
        {
            Ok(Ok(session)) => Some(session),
            Ok(Err(err)) => {
                activity_log.push(ActivityStep {
                    step: "mcp".into(),
                    detail: err,
                });
                None
            }
            Err(_) => {
                activity_log.push(ActivityStep {
                    step: "mcp".into(),
                    detail: "MCP server did not respond within 30 seconds".into(),
                });
                None
            }
        }
    } else {
        None
    };

    let mut mcp_tool_definitions = Vec::new();
    if let Some(session) = &mcp_session {
        if let Ok(tools) = crate::mcp::list_tools_async(session).await {
            mcp_tool_definitions = crate::mcp::openai_tools_from_mcp(&tools);
        }
    }

    let mut use_native_tools = true;
    let mut counters = StepCounters::default();
    let mut loop_guard = LoopGuard::default();
    let run_started = Instant::now();
    const MAX_RUN_SECS: u64 = 1200;
    maybe_promote_to_editor_for_greenfield(settings, run_config, &task_spec.objective);
    let phase = run_config.phase;

    emit_progress(
        progress,
        task_spec,
        &activity_log,
        "running",
        &phase_running_summary(run_config, phase),
    );

    let mut messages = vec![
        prompts::agent_system_message_for_phase(settings.workspace_configured(), run_config.phase),
        ChatMessage::user(format!(
            "Task specification:\n{task_json}\n\nRecent conversation:\n{}",
            recent.join("\n")
        )),
    ];

    loop {
        if cancel.is_some_and(RunState::is_cancelled) {
            return Err(ExecutorError::Cancelled);
        }

        if run_started.elapsed() >= Duration::from_secs(MAX_RUN_SECS) {
            return Ok(build_step_limit_result(
                task_spec,
                &activity_log,
                &change_tracker,
                &tool_ctx,
            ));
        }

        cap_message_history(&mut messages);

        let phase = run_config.phase;
        let max_steps = run_config.max_steps_for_phase(phase);
        if max_steps == 0 || counters.count(phase) >= max_steps {
            if run_config.should_escalate_on_scout_exhausted(phase, counters.scout) {
                try_escalate_to_editor(
                    settings,
                    run_config,
                    &counters,
                    task_spec,
                    &mut activity_log,
                    &mut messages,
                    "scout step limit reached",
                )?;
                emit_progress(
                    progress,
                    task_spec,
                    &activity_log,
                    "running",
                    &phase_running_summary(run_config, run_config.phase),
                );
                continue;
            }
            return Ok(build_step_limit_result(
                task_spec,
                &activity_log,
                &change_tracker,
                &tool_ctx,
            ));
        }

        counters.increment(phase);

        let mut tool_definitions = openai_tool_definitions_filtered(|name| {
            run_config.is_tool_allowed(phase, name)
        });
        tool_definitions.extend(
            mcp_tool_definitions
                .iter()
                .filter(|tool| {
                    tool.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .is_some_and(|name| run_config.is_tool_allowed(phase, name))
                })
                .cloned(),
        );

        let model = run_config.model_for_phase(phase).to_string();
        let step_no = counters.count(phase);
        activity_log.push(ActivityStep {
            step: "model".into(),
            detail: format!(
                "Calling {} ({step_no}/{})",
                model,
                run_config.max_steps_for_phase(phase)
            ),
        });
        emit_progress(
            progress,
            task_spec,
            &activity_log,
            "running",
            &phase_running_summary(run_config, phase),
        );
        activity_log.pop();

        if use_native_tools {
            let request = ChatCompletionRequest {
                model: model.clone(),
                messages: messages.clone(),
                temperature: run_config.temperature,
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
                            run_config,
                            &counters,
                            task_spec,
                            &tool_ctx,
                            &mut change_tracker,
                            &mut verify_failures,
                            &mut activity_log,
                            &mut messages,
                            &mut loop_guard,
                            progress,
                            cancel,
                            mcp_session.as_ref(),
                        )
                        .await? {
                            return Ok(result);
                        }
                        continue;
                    }

                    if let Some(content) = completion.content {
                        match handle_json_action(
                            &content,
                            settings,
                            run_config,
                            &counters,
                            task_spec,
                            &tool_ctx,
                            &mut change_tracker,
                            &mut verify_failures,
                            &mut activity_log,
                            &mut messages,
                            &mut loop_guard,
                            progress,
                            cancel,
                            mcp_session.as_ref(),
                            false,
                        )
                        .await
                        {
                            Ok(Some(result)) => return Ok(result),
                            Ok(None) => continue,
                            Err(ExecutorError::Parse(ref msg)) if msg == PLAIN_TEXT_RESPONSE => {
                                activity_log.push(ActivityStep {
                                    step: "model".into(),
                                    detail: "Model replied in plain text — nudging to use tools".into(),
                                });
                                messages.push(ChatMessage::assistant(content));
                                messages.push(ChatMessage::user(
                                    "Use the available tools or call final_answer when done. Plain text alone cannot complete the task.",
                                ));
                                continue;
                            }
                            Err(err) => return Err(err),
                        }
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
                model,
                messages: messages.clone(),
                temperature: run_config.temperature,
                max_tokens: None,
                tools: None,
                json_object_mode: false,
            },
        )
        .await?;

        if let Some(result) = handle_json_action(
            &raw,
            settings,
            run_config,
            &counters,
            task_spec,
            &tool_ctx,
            &mut change_tracker,
            &mut verify_failures,
            &mut activity_log,
            &mut messages,
            &mut loop_guard,
            progress,
            cancel,
            mcp_session.as_ref(),
            true,
        )
        .await? {
            return Ok(result);
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_native_tool_calls(
    tool_calls: Vec<ToolCall>,
    assistant_content: Option<String>,
    settings: &AiSettings,
    run_config: &mut RunConfig,
    counters: &StepCounters,
    task_spec: &TaskSpec,
    tool_ctx: &Result<ToolContext, crate::tools::SandboxError>,
    change_tracker: &mut ChangeTracker,
    verify_failures: &mut usize,
    activity_log: &mut Vec<ActivityStep>,
    messages: &mut Vec<ChatMessage>,
    loop_guard: &mut LoopGuard,
    progress: Option<&ProgressFn<'_>>,
    cancel: Option<&AtomicBool>,
    mcp_session: Option<&Arc<crate::mcp::McpSession>>,
) -> Result<Option<ExecutorResult>, ExecutorError> {
    let phase = run_config.phase;

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

        if run_config.should_escalate_on_tool(phase, &call.function.name) {
            try_escalate_to_editor(
                settings,
                run_config,
                counters,
                task_spec,
                activity_log,
                messages,
                "edit required",
            )?;
            emit_progress(
                progress,
                task_spec,
                activity_log,
                "running",
                &phase_running_summary(run_config, run_config.phase),
            );
            return Ok(None);
        }

        if !run_config.is_tool_allowed(phase, &call.function.name) {
            let err = format!("tool `{}` is not allowed in {} phase", call.function.name, run_config.phase_label(phase));
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

        if crate::mcp::is_mcp_tool(&call.function.name) {
            let arguments = parse_tool_arguments(&call.function.arguments)?;
            let tool_result = if let Some(session) = mcp_session {
                match crate::mcp::call_tool_async(session, &call.function.name, &arguments).await {
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
            Ok(ctx) => execute_tracked_tool_async(ctx, change_tracker, &call.function.name, &arguments).await,
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

        if should_force_escalation(
            run_config,
            activity_log,
            &call.function.name,
            &tool_result,
            &task_spec.objective,
        ) {
            messages.push(tool_result_message(
                &call.id,
                tool_observation(&call.function.name, &tool_result),
            ));
            let reason = if tool_result.output.contains("empty directory") {
                "empty workspace — creating files"
            } else if should_escalate_after_exploration_loop(
                run_config,
                activity_log,
                &task_spec.objective,
            ) {
                "exploration complete — implementing"
            } else {
                "repeated read failures — switching to editor"
            };
            try_escalate_to_editor(
                settings,
                run_config,
                counters,
                task_spec,
                activity_log,
                messages,
                reason,
            )?;
            emit_progress(
                progress,
                task_spec,
                activity_log,
                "running",
                &phase_running_summary(run_config, run_config.phase),
            );
            return Ok(None);
        }

        if tool_result.ok
            && run_config.verify_enabled
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
        if let Some(nudge) = loop_guard_nudge(
            loop_guard,
            run_config,
            &call.function.name,
            &arguments,
            activity_log,
            &task_spec.objective,
        ) {
            messages.push(ChatMessage::user(nudge));
        }
    }

    Ok(None)
}

#[allow(clippy::too_many_arguments)]
async fn handle_json_action(
    raw: &str,
    settings: &AiSettings,
    run_config: &mut RunConfig,
    counters: &StepCounters,
    task_spec: &TaskSpec,
    tool_ctx: &Result<ToolContext, crate::tools::SandboxError>,
    change_tracker: &mut ChangeTracker,
    verify_failures: &mut usize,
    activity_log: &mut Vec<ActivityStep>,
    messages: &mut Vec<ChatMessage>,
    loop_guard: &mut LoopGuard,
    progress: Option<&ProgressFn<'_>>,
    cancel: Option<&AtomicBool>,
    mcp_session: Option<&Arc<crate::mcp::McpSession>>,
    allow_plain_text_final: bool,
) -> Result<Option<ExecutorResult>, ExecutorError> {
    let action = parse_executor_action_flexible(raw, allow_plain_text_final)?;
    let phase = run_config.phase;

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

            if run_config.should_escalate_on_tool(phase, &tool) {
                try_escalate_to_editor(
                    settings,
                    run_config,
                    counters,
                    task_spec,
                    activity_log,
                    messages,
                    "edit required",
                )?;
                emit_progress(
                    progress,
                    task_spec,
                    activity_log,
                    "running",
                    &phase_running_summary(run_config, run_config.phase),
                );
                return Ok(None);
            }

            if !run_config.is_tool_allowed(phase, &tool) {
                let err = format!("tool `{tool}` is not allowed in {} phase", run_config.phase_label(phase));
                activity_log.push(ActivityStep {
                    step: format!("tool:{tool}"),
                    detail: err.clone(),
                });
                messages.push(ChatMessage::user(format!("Tool validation error:\n{err}")));
                return Ok(None);
            }

            if crate::mcp::is_mcp_tool(&tool) {
                let tool_result = if let Some(session) = mcp_session {
                    match crate::mcp::call_tool_async(session, &tool, &arguments).await {
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
                Ok(ctx) => execute_tracked_tool_async(ctx, change_tracker, &tool, &arguments).await,
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

            if should_force_escalation(
                run_config,
                activity_log,
                &tool,
                &tool_result,
                &task_spec.objective,
            ) {
                let reason = if tool_result.output.contains("empty directory") {
                    "empty workspace — creating files"
                } else if should_escalate_after_exploration_loop(
                    run_config,
                    activity_log,
                    &task_spec.objective,
                ) {
                    "exploration complete — implementing"
                } else {
                    "repeated read failures — switching to editor"
                };
                try_escalate_to_editor(
                    settings,
                    run_config,
                    counters,
                    task_spec,
                    activity_log,
                    messages,
                    reason,
                )?;
                emit_progress(
                    progress,
                    task_spec,
                    activity_log,
                    "running",
                    &phase_running_summary(run_config, run_config.phase),
                );
                messages.push(ChatMessage::assistant(raw));
                messages.push(ChatMessage::user(format!(
                    "Tool result:\n{}",
                    tool_observation(&tool, &tool_result)
                )));
                return Ok(None);
            }

            if tool_result.ok
                && run_config.verify_enabled
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
            if let Some(nudge) = loop_guard_nudge(
                loop_guard,
                run_config,
                &tool,
                &arguments,
                activity_log,
                &task_spec.objective,
            ) {
                messages.push(ChatMessage::user(nudge));
            }
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

    ensure_executor_summary(&mut result);

    Ok(result)
}

fn record_tool_activity(activity_log: &mut Vec<ActivityStep>, tool: &str, tool_result: &ToolResult) {
    let detail = if tool_result.ok {
        truncate_for_log(&tool_result.output, 4_096)
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

async fn execute_tracked_tool_async(
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

    let result = execute_tool_async(ctx.clone(), tool.to_string(), args.clone()).await;

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

const MAX_EXECUTOR_MESSAGES: usize = 80;

fn cap_message_history(messages: &mut Vec<ChatMessage>) {
    if messages.len() <= MAX_EXECUTOR_MESSAGES {
        return;
    }
    let remove = messages.len() - MAX_EXECUTOR_MESSAGES;
    messages.drain(1..1 + remove);
}

pub fn ensure_executor_summary(result: &mut ExecutorResult) {
    if !result.summary.trim().is_empty() || !result.content.trim().is_empty() {
        return;
    }

    if !result.file_changes.is_empty() {
        let paths: Vec<String> = result
            .file_changes
            .iter()
            .map(|change| format!("- {} ({})", change.path, change.change_type))
            .collect();
        result.summary = format!("Updated {} file(s)", result.file_changes.len());
        result.content = format!("Applied changes:\n{}", paths.join("\n"));
        if matches!(result.status, ExecutorStatus::Error) {
            result.status = ExecutorStatus::Success;
        }
        return;
    }

    if let Some(last) = result.activity_log.last() {
        result.summary = if last.step.starts_with("tool:") {
            format!("Finished: {}", last.step.strip_prefix("tool:").unwrap_or(&last.step))
        } else {
            "Task finished".into()
        };
        result.content = last.detail.clone();
    } else {
        result.summary = "Done".into();
        result.content = "Completed the requested task.".into();
    }
}

pub fn template_format(result: &ExecutorResult) -> String {
    let mut normalized = result.clone();
    ensure_executor_summary(&mut normalized);

    if normalized.content.trim().is_empty() {
        normalized.summary.clone()
    } else if normalized.summary.trim().is_empty() {
        normalized.content.clone()
    } else {
        format!(
            "{}\n\n{}",
            normalized.summary.trim(),
            normalized.content.trim()
        )
    }
}

fn build_step_limit_result(
    task_spec: &TaskSpec,
    activity_log: &[ActivityStep],
    change_tracker: &ChangeTracker,
    tool_ctx: &Result<ToolContext, crate::tools::SandboxError>,
) -> ExecutorResult {
    let file_changes = if let Ok(ctx) = tool_ctx {
        change_tracker.finalize(&ctx.sandbox)
    } else {
        Vec::new()
    };

    if !file_changes.is_empty() {
        let paths: Vec<String> = file_changes
            .iter()
            .map(|change| format!("- {} ({})", change.path, change.change_type))
            .collect();
        return ExecutorResult {
            status: ExecutorStatus::Success,
            summary: format!("Updated {} file(s)", file_changes.len()),
            content: format!(
                "Applied changes to your project before hitting the step limit:\n{}\n\nIf anything is missing, send a narrower follow-up.",
                paths.join("\n")
            ),
            activity_log: activity_log.to_vec(),
            file_changes,
        };
    }

    let recent = activity_log
        .iter()
        .rev()
        .take(5)
        .map(|step| format!("{} — {}", step.step, step.detail))
        .collect::<Vec<_>>()
        .join("\n");

    ExecutorResult {
        status: ExecutorStatus::Error,
        summary: "Could not finish within the step limit".into(),
        content: format!(
            "I worked on \"{}\" but hit the step limit before completing.\n\nRecent activity:\n{}\n\nTry a narrower request or switch to the Deep tier.",
            task_spec.objective,
            recent
        ),
        activity_log: activity_log.to_vec(),
        file_changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::task::TaskSpec;

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
    fn flexible_parse_accepts_plain_text() {
        let action =
            parse_executor_action_flexible("Here is the answer you asked for.", true).unwrap();
        assert!(matches!(
            action,
            ExecutorAction::FinalAnswer {
                status,
                content,
                ..
            } if status == "success" && content == "Here is the answer you asked for."
        ));
    }

    #[test]
    fn flexible_parse_rejects_plain_text_when_disabled() {
        let err = parse_executor_action_flexible("Here is the answer you asked for.", false).unwrap_err();
        assert!(matches!(err, ExecutorError::Parse(ref msg) if msg == PLAIN_TEXT_RESPONSE));
    }

    #[test]
    fn flexible_parse_accepts_legacy_envelope() {
        let raw = r#"{"status":"success","summary":"Done","content":"All good","activity_log":[]}"#;
        let action = parse_executor_action_flexible(raw, false).unwrap();
        assert!(matches!(
            action,
            ExecutorAction::FinalAnswer {
                status,
                summary,
                content,
                ..
            } if status == "success" && summary == "Done" && content == "All good"
        ));
    }

    use crate::agents::profile::AgentTier;

    #[test]
    fn rejects_invalid_task_spec() {
        let spec = TaskSpec {
            objective: "".into(),
            context: "ctx".into(),
            constraints: vec![],
            expected_output: "out".into(),
        };
        let mut run_config =
            RunConfig::from_settings(&AiSettings::default(), AgentTier::Auto, "fix bug");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let err = rt
            .block_on(execute(
                &AiSettings::default(),
                &mut run_config,
                &spec,
                &[],
                None,
                None,
            ))
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
}
