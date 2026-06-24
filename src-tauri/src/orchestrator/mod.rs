use crate::agents::executor::{
    self, ActivityStep, ExecutorProgress, ExecutorResult, ExecutorStatus,
};
use crate::agents::profile::{build_scout_briefing, AgentPhase, AgentTier, RunConfig};
use crate::agents::task::{
    build_task_from_message, is_casual_chat_only, is_file_or_code_task, last_user_message,
    TaskSpec,
};
use crate::ai::{chat_completion, chat_completion_stream, prompts, ChatCompletionRequest, ChatMessage};
use crate::db::{DbState, Message};
use crate::rag::{self, RetrievedChunk};
use crate::workspace;
use crate::agent_plan::PendingPlan;
use crate::run_state::RunState;
use crate::settings::AiSettings;
use serde::Serialize;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentActivityView {
    pub task_spec: TaskSpec,
    pub status: String,
    pub summary: String,
    pub activity_log: Vec<ActivityStep>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProgressPayload {
    pub conversation_id: String,
    pub phase: String,
    pub activity: Option<AgentActivityView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantStreamPayload {
    pub conversation_id: String,
    pub stream_id: String,
    pub delta: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

pub type ProgressFn = dyn Fn(AgentProgressPayload) + Send + Sync;
pub type StreamFn = dyn Fn(AssistantStreamPayload) + Send + Sync;
pub type AckPersistFn = dyn Fn(&str) + Send + Sync;

#[derive(Debug, Clone)]
pub struct AgentRecord {
    pub task_spec: TaskSpec,
    pub result: ExecutorResult,
    pub db_status: &'static str,
}

#[derive(Debug, Clone)]
pub struct TurnResult {
    pub final_message: String,
    pub agent_record: Option<AgentRecord>,
    pub retrieved_context: Vec<RetrievedChunk>,
    pub awaiting_plan_approval: bool,
    pub plan_content: Option<String>,
    pub pending_plan: Option<PendingPlan>,
}

pub enum AgentRunResult {
    Completed(AgentRecord),
    AwaitingPlan(PendingPlan),
}

pub fn not_configured_message() -> &'static str {
    "Add an API key and provider in Settings, then try again."
}

pub fn error_message(err: &str) -> String {
    crate::ai::client::user_facing_error_text(err)
}

pub fn cancelled_message() -> String {
    "Run stopped — you can ask again anytime.".into()
}

pub fn enrich_task_with_memories(task_spec: &mut TaskSpec, memories: &[crate::db::Memory]) {
    if memories.is_empty() {
        return;
    }

    let block = memories
        .iter()
        .map(|memory| format!("- {}", memory.content))
        .collect::<Vec<_>>()
        .join("\n");

    task_spec.context = format!(
        "{}\n\n--- Project notes ---\n{}",
        task_spec.context, block
    );
}

pub fn enrich_task_with_context(settings: &AiSettings, task_spec: &mut TaskSpec) {
    if !settings.context_pack_enabled || !settings.workspace_configured() {
        return;
    }

    let Some(workspace) = settings.workspace_path.as_ref() else {
        return;
    };

    let workspace_path = std::path::Path::new(workspace);
    let pack = crate::context::build_context_pack(workspace_path);
    if pack.trim().is_empty() {
        return;
    }

    task_spec.context = format!(
        "{}\n\n--- Project context ---\n{}",
        task_spec.context, pack
    );

    if crate::context::is_workspace_empty(workspace_path) {
        task_spec.context = format!(
            "{}\n\n--- Workspace state ---\nThe folder is empty. Call list_dir to confirm, \
             then create files with write_file. Do not read_file paths that do not exist yet.",
            task_spec.context
        );
    }
}

pub fn enrich_task_with_rules(settings: &AiSettings, task_spec: &mut TaskSpec) {
    if !settings.project_rules_enabled || !settings.workspace_configured() {
        return;
    }

    let Some(workspace) = settings.workspace_path.as_ref() else {
        return;
    };

    let Some(rules) = crate::context::load_project_rules(std::path::Path::new(workspace)) else {
        return;
    };

    task_spec.context = format!(
        "{}\n\n--- Project rules ---\n{}",
        task_spec.context, rules
    );
}

pub async fn enrich_task_with_rag(
    settings: &AiSettings,
    task_spec: &mut TaskSpec,
    ann_state: &crate::rag_ann_state::RagAnnState,
    db_state: &DbState,
) -> Vec<RetrievedChunk> {
    let objective = task_spec.objective.clone();
    let retrieved = match rag::retrieve_chunks_for_query(ann_state, db_state, settings, &objective)
        .await
    {
        Ok(chunks) => chunks,
        Err(err) => {
            eprintln!("RAG retrieval failed: {err}");
            return Vec::new();
        }
    };

    if retrieved.is_empty() {
        return Vec::new();
    }

    let context = retrieved
        .iter()
        .map(|chunk| {
            format!(
                "[{}] (score {:.2})\n{}",
                chunk.source_path, chunk.score, chunk.snippet
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    task_spec.context = format!(
        "{}\n\n--- Retrieved project context (local RAG) ---\n{}",
        task_spec.context, context
    );

    retrieved
}

pub fn enrich_task_with_symbols(settings: &AiSettings, task_spec: &mut TaskSpec) {
    if !settings.workspace_configured() {
        return;
    }

    let Some(workspace) = settings.workspace_path.as_ref() else {
        return;
    };

    let tokens = symbol_tokens_from_objective(&task_spec.objective);
    if tokens.is_empty() {
        return;
    }

    let workspace_path = std::path::Path::new(workspace);
    let mut hits = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for token in tokens.iter().take(4) {
        let Ok(found) = crate::workspace::search_workspace_symbols(workspace_path, token) else {
            continue;
        };
        for hit in found.into_iter().take(3) {
            let key = format!("{}:{}", hit.path, hit.name);
            if seen.insert(key) {
                hits.push(hit);
            }
        }
    }

    if hits.is_empty() {
        return;
    }

    let block = hits
        .iter()
        .take(8)
        .map(|hit| format!("{} {} — {}:{}", hit.kind, hit.name, hit.path, hit.line))
        .collect::<Vec<_>>()
        .join("\n");

    task_spec.context = format!(
        "{}\n\n--- Matching symbols ---\n{}\nUse read_file on these paths before editing.",
        task_spec.context, block
    );
}

fn symbol_tokens_from_objective(objective: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in objective.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else if !current.is_empty() {
            if current.len() >= 3 && !is_common_word(&current) {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }
    if current.len() >= 3 && !is_common_word(&current) {
        tokens.push(current);
    }

    tokens.sort_by_key(|token| std::cmp::Reverse(token.len()));
    tokens.dedup();
    tokens
}

fn is_common_word(word: &str) -> bool {
    matches!(
        word.to_ascii_lowercase().as_str(),
        "the" | "and" | "for" | "with" | "from" | "this" | "that" | "file" | "code" | "fix"
            | "add" | "use" | "make" | "change" | "update" | "please" | "help" | "test"
    )
}

pub fn enrich_task_with_attachments(
    settings: &AiSettings,
    task_spec: &mut TaskSpec,
    attachments: &[workspace::MessageAttachment],
) -> Result<(), String> {
    if attachments.is_empty() {
        return Ok(());
    }

    let block = workspace::build_attachment_context(settings, attachments)?;
    if block.trim().is_empty() {
        return Ok(());
    }

    task_spec.context = format!(
        "{}\n\n--- Attached context ---\n{}",
        task_spec.context, block
    );
    Ok(())
}

fn should_run_agent(settings: &AiSettings, user_message: &str) -> bool {
    if is_casual_chat_only(user_message) {
        return false;
    }
    if settings.workspace_configured() {
        return true;
    }
    is_file_or_code_task(user_message)
}

pub async fn stream_task_acknowledgment(
    settings: &AiSettings,
    user_message: &str,
    task_spec: &TaskSpec,
    conversation_id: &str,
    stream_id: &str,
    stream: &StreamFn,
) -> Result<String, String> {
    let request = ChatCompletionRequest {
        model: settings.effective_fast_model().to_string(),
        messages: vec![
            prompts::agent_chat_message_for_ack(),
            ChatMessage::user(format!(
                "User request:\n{user_message}\n\nTask objective:\n{}",
                task_spec.objective
            )),
        ],
        temperature: settings.agent_temperature.min(0.5),
        max_tokens: Some(220),
        tools: None,
        json_object_mode: false,
    };

    let content = match chat_completion_stream(settings, request.clone(), |delta| {
        stream(AssistantStreamPayload {
            conversation_id: conversation_id.to_string(),
            stream_id: stream_id.to_string(),
            delta: delta.to_string(),
            done: false,
            content: None,
        });
    })
    .await
    {
        Ok(content) if !content.trim().is_empty() => content,
        _ => chat_completion(settings, request)
            .await
            .map_err(|err| err.user_message())?,
    };

    stream(AssistantStreamPayload {
        conversation_id: conversation_id.to_string(),
        stream_id: stream_id.to_string(),
        delta: String::new(),
        done: true,
        content: Some(content.clone()),
    });

    Ok(content)
}

pub async fn try_stream_task_acknowledgment(
    settings: &AiSettings,
    user_message: &str,
    task_spec: &TaskSpec,
    conversation_id: &str,
    stream_id: &str,
    stream: &StreamFn,
) -> Option<String> {
    match tokio::time::timeout(
        std::time::Duration::from_secs(45),
        stream_task_acknowledgment(
            settings,
            user_message,
            task_spec,
            conversation_id,
            stream_id,
            stream,
        ),
    )
    .await
    {
        Ok(Ok(content)) if content.trim().is_empty() => None,
        Ok(Ok(content)) => Some(content),
        Ok(Err(err)) => {
            eprintln!("task acknowledgment failed: {err}");
            None
        }
        Err(_) => {
            eprintln!("task acknowledgment timed out after 45 seconds");
            None
        }
    }
}

async fn run_direct_chat(
    settings: &AiSettings,
    history: &[Message],
    memories: &[crate::db::Memory],
    conversation_id: &str,
    stream_id: &str,
    stream: Option<&StreamFn>,
) -> Result<String, String> {
    let mut messages = vec![prompts::agent_chat_message()];

    if !memories.is_empty() {
        let mut notes = String::from("\n\n## Project notes\n");
        for memory in memories {
            notes.push_str("- ");
            notes.push_str(&memory.content);
            notes.push('\n');
        }
        if let Some(system) = messages.first_mut() {
            system.content.push_str(&notes);
        }
    }

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

    let request = ChatCompletionRequest {
        model: settings.effective_fast_model().to_string(),
        messages,
        temperature: settings.agent_temperature,
        max_tokens: None,
        tools: None,
        json_object_mode: false,
    };

    if let Some(on_stream) = stream {
        let content = chat_completion_stream(settings, request, |delta| {
            on_stream(AssistantStreamPayload {
                conversation_id: conversation_id.to_string(),
                stream_id: stream_id.to_string(),
                delta: delta.to_string(),
                done: false,
                content: None,
            });
        })
        .await
        .map_err(|err| err.user_message())?;

        on_stream(AssistantStreamPayload {
            conversation_id: conversation_id.to_string(),
            stream_id: stream_id.to_string(),
            delta: String::new(),
            done: true,
            content: Some(content.clone()),
        });
        Ok(content)
    } else {
        chat_completion(settings, request)
            .await
            .map_err(|err| err.user_message())
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run_agent_task(
    settings: &AiSettings,
    history: &[Message],
    conversation_id: &str,
    task_spec: TaskSpec,
    agent_tier: AgentTier,
    progress: Option<&ProgressFn>,
    cancel: Option<&AtomicBool>,
) -> Result<AgentRunResult, String> {
    if cancel.is_some_and(RunState::is_cancelled) {
        return Err(cancelled_message());
    }

    task_spec.validate().map_err(|err| {
        error_message(&format!("Could not frame that task: {err}"))
    })?;

    let mut run_config = RunConfig::from_settings(settings, agent_tier, &task_spec.objective);

    emit_phase(
        progress,
        conversation_id,
        "running",
        Some(partial_activity(
            &task_spec,
            "running",
            &format!("{} — Working…", run_config.tier.as_str()),
            vec![ActivityStep {
                step: "start".into(),
                detail: task_spec.objective.clone(),
            }],
        )),
    );

    let executor_callback = |update: ExecutorProgress| {
        emit_phase(
            progress,
            conversation_id,
            "running",
            Some(AgentActivityView {
                task_spec: update.task_spec,
                status: update.status,
                summary: update.summary,
                activity_log: update.activity_log,
            }),
        );
    };

    let agent_result = match executor::execute(
        settings,
        &mut run_config,
        &task_spec,
        history,
        Some(&executor_callback),
        cancel,
    )
    .await
    {
        Ok(result) => result,
        Err(executor::ExecutorError::PlanApprovalRequired(plan)) => {
            emit_phase(
                progress,
                conversation_id,
                "plan_review",
                Some(partial_activity(
                    &plan.task_spec,
                    "plan_review",
                    "Waiting for plan approval",
                    plan.activity_log.clone(),
                )),
            );
            return Ok(AgentRunResult::AwaitingPlan(PendingPlan {
                conversation_id: conversation_id.to_string(),
                task_spec: plan.task_spec.clone(),
                tier: agent_tier,
                activity_log: plan.activity_log.clone(),
                briefing: plan.briefing.clone(),
            }));
        }
        Err(executor::ExecutorError::Cancelled) => return Err(cancelled_message()),
        Err(err) => return Err(err.user_message()),
    };

    let db_status = agent_db_status(&agent_result.status);
    let record = AgentRecord {
        task_spec: task_spec.clone(),
        result: agent_result,
        db_status,
    };

    emit_phase(
        progress,
        conversation_id,
        "complete",
        Some(to_activity_view(&record)),
    );

    Ok(AgentRunResult::Completed(record))
}

pub async fn run_agent_from_plan(
    settings: &AiSettings,
    history: &[Message],
    conversation_id: &str,
    pending: &PendingPlan,
    progress: Option<&ProgressFn>,
    cancel: Option<&AtomicBool>,
) -> Result<AgentRecord, String> {
    if cancel.is_some_and(RunState::is_cancelled) {
        return Err(cancelled_message());
    }

    let mut run_config =
        RunConfig::from_settings(settings, pending.tier, &pending.task_spec.objective);
    run_config.phase = crate::agents::profile::AgentPhase::Editor;
    run_config.max_scout_steps = 0;

    let mut task_spec = pending.task_spec.clone();
    task_spec.context = format!(
        "{}\n\n--- Approved plan ---\n{}",
        task_spec.context, pending.briefing
    );

    let executor_callback = |update: ExecutorProgress| {
        emit_phase(
            progress,
            conversation_id,
            "running",
            Some(AgentActivityView {
                task_spec: update.task_spec,
                status: update.status,
                summary: update.summary,
                activity_log: update.activity_log,
            }),
        );
    };

    let agent_result = executor::execute(
        settings,
        &mut run_config,
        &task_spec,
        history,
        Some(&executor_callback),
        cancel,
    )
    .await
    .map_err(|err| match err {
        executor::ExecutorError::Cancelled => cancelled_message(),
        executor::ExecutorError::PlanApprovalRequired(_) => {
            "Plan approval is only required once per run".into()
        }
        err => err.user_message(),
    })?;

    let db_status = agent_db_status(&agent_result.status);
    let record = AgentRecord {
        task_spec,
        result: agent_result,
        db_status,
    };

    emit_phase(
        progress,
        conversation_id,
        "complete",
        Some(to_activity_view(&record)),
    );

    Ok(record)
}

pub async fn run_explore_then_implement(
    settings: &AiSettings,
    history: &[Message],
    conversation_id: &str,
    mut task_spec: TaskSpec,
    progress: Option<&ProgressFn>,
    cancel: Option<&AtomicBool>,
) -> Result<AgentRecord, String> {
    if cancel.is_some_and(RunState::is_cancelled) {
        return Err(cancelled_message());
    }

    emit_phase(
        progress,
        conversation_id,
        "running",
        Some(partial_activity(
            &task_spec,
            "running",
            "Explore — read-only scout pass",
            vec![ActivityStep {
                step: "explore".into(),
                detail: task_spec.objective.clone(),
            }],
        )),
    );

    let explore_outcome = run_agent_task(
        settings,
        history,
        conversation_id,
        task_spec.clone(),
        AgentTier::Explain,
        progress,
        cancel,
    )
    .await?;

    let explore_record = match explore_outcome {
        AgentRunResult::Completed(record) => record,
        AgentRunResult::AwaitingPlan(_) => {
            return Err("Explore pass should not require plan approval".into());
        }
    };

    let briefing = build_scout_briefing(&explore_record.result.activity_log, &task_spec);
    task_spec.context = format!(
        "{}\n\n--- Explore findings ---\n{}\n\n---\n{}",
        task_spec.context, explore_record.result.summary, briefing
    );

    emit_phase(
        progress,
        conversation_id,
        "running",
        Some(partial_activity(
            &task_spec,
            "running",
            "Implement — applying changes from explore pass",
            vec![ActivityStep {
                step: "implement".into(),
                detail: "Editor phase".into(),
            }],
        )),
    );

    let mut run_config =
        RunConfig::from_settings(settings, AgentTier::Auto, &task_spec.objective);
    run_config.phase = AgentPhase::Editor;
    run_config.max_scout_steps = 0;

    let executor_callback = |update: ExecutorProgress| {
        emit_phase(
            progress,
            conversation_id,
            "running",
            Some(AgentActivityView {
                task_spec: update.task_spec,
                status: update.status,
                summary: update.summary,
                activity_log: update.activity_log,
            }),
        );
    };

    let agent_result = executor::execute(
        settings,
        &mut run_config,
        &task_spec,
        history,
        Some(&executor_callback),
        cancel,
    )
    .await
    .map_err(|err| match err {
        executor::ExecutorError::Cancelled => cancelled_message(),
        executor::ExecutorError::PlanApprovalRequired(_) => {
            "Plan approval is not used during explore implement pass".into()
        }
        err => err.user_message(),
    })?;

    let db_status = agent_db_status(&agent_result.status);
    let record = AgentRecord {
        task_spec,
        result: agent_result,
        db_status,
    };

    emit_phase(
        progress,
        conversation_id,
        "complete",
        Some(to_activity_view(&record)),
    );

    Ok(record)
}

#[allow(clippy::too_many_arguments)]
pub async fn run_turn(
    settings: &AiSettings,
    history: &[Message],
    conversation_id: &str,
    memories: &[crate::db::Memory],
    db_state: &DbState,
    ann_state: &crate::rag_ann_state::RagAnnState,
    agent_tier: AgentTier,
    attachments: &[workspace::MessageAttachment],
    explore_then_implement: bool,
    stream_id: &str,
    stream: Option<&StreamFn>,
    progress: Option<&ProgressFn>,
    cancel: Option<&AtomicBool>,
    on_ack_persist: Option<&AckPersistFn>,
) -> Result<TurnResult, String> {
    if settings.api_key.trim().is_empty()
        && !crate::ai::client::is_local_provider(&settings.base_url)
    {
        return Ok(TurnResult {
            final_message: not_configured_message().into(),
            agent_record: None,
            retrieved_context: Vec::new(),
            awaiting_plan_approval: false,
            plan_content: None,
            pending_plan: None,
        });
    }

    let user_message = last_user_message(history).unwrap_or("");

    if !should_run_agent(settings, user_message) {
        let message = run_direct_chat(
            settings,
            history,
            memories,
            conversation_id,
            stream_id,
            stream,
        )
        .await?;
        return Ok(TurnResult {
            final_message: message,
            agent_record: None,
            retrieved_context: Vec::new(),
            awaiting_plan_approval: false,
            plan_content: None,
            pending_plan: None,
        });
    }

    if !settings.workspace_configured() && is_file_or_code_task(user_message) {
        return Ok(TurnResult {
            final_message: "Pick a project folder in Settings first — then I can read and edit files in your repo.".into(),
            agent_record: None,
            retrieved_context: Vec::new(),
            awaiting_plan_approval: false,
            plan_content: None,
            pending_plan: None,
        });
    }

    let mut task_spec = build_task_from_message(user_message, history);
    let run_config = RunConfig::from_settings(settings, agent_tier, user_message);

    if run_config.context_pack_enabled {
        enrich_task_with_context(settings, &mut task_spec);
    }
    enrich_task_with_memories(&mut task_spec, memories);
    enrich_task_with_rules(settings, &mut task_spec);
    enrich_task_with_symbols(settings, &mut task_spec);
    enrich_task_with_attachments(settings, &mut task_spec, attachments)?;
    let retrieved_context = if run_config.rag_enabled {
        enrich_task_with_rag(settings, &mut task_spec, ann_state, db_state).await
    } else {
        Vec::new()
    };

    let ack_message = if let Some(stream_fn) = stream {
        try_stream_task_acknowledgment(
            settings,
            user_message,
            &task_spec,
            conversation_id,
            stream_id,
            stream_fn,
        )
        .await
    } else {
        None
    };

    if let Some(ack) = ack_message.as_deref() {
        if let Some(persist) = on_ack_persist {
            persist(ack);
        }
    }

    if explore_then_implement {
        let record = run_explore_then_implement(
            settings,
            history,
            conversation_id,
            task_spec,
            progress,
            cancel,
        )
        .await?;
        let final_message = executor::template_format(&record.result);
        return Ok(TurnResult {
            final_message,
            agent_record: Some(record),
            retrieved_context,
            awaiting_plan_approval: false,
            plan_content: None,
            pending_plan: None,
        });
    }

    let run_outcome = run_agent_task(
        settings,
        history,
        conversation_id,
        task_spec,
        agent_tier,
        progress,
        cancel,
    )
    .await?;

    let record = match run_outcome {
        AgentRunResult::Completed(record) => record,
        AgentRunResult::AwaitingPlan(pending) => {
            return Ok(TurnResult {
                final_message: String::new(),
                agent_record: None,
                retrieved_context,
                awaiting_plan_approval: true,
                plan_content: Some(pending.briefing.clone()),
                pending_plan: Some(pending),
            });
        }
    };

    let final_message = executor::template_format(&record.result);

    Ok(TurnResult {
        final_message,
        agent_record: Some(record),
        retrieved_context,
        awaiting_plan_approval: false,
        plan_content: None,
        pending_plan: None,
    })
}

fn emit_phase(
    progress: Option<&ProgressFn>,
    conversation_id: &str,
    phase: &str,
    activity: Option<AgentActivityView>,
) {
    if let Some(callback) = progress {
        callback(AgentProgressPayload {
            conversation_id: conversation_id.to_string(),
            phase: phase.into(),
            activity,
        });
    }
}

fn partial_activity(
    task_spec: &TaskSpec,
    status: &str,
    summary: &str,
    activity_log: Vec<ActivityStep>,
) -> AgentActivityView {
    AgentActivityView {
        task_spec: task_spec.clone(),
        status: status.into(),
        summary: summary.into(),
        activity_log,
    }
}

fn agent_db_status(status: &ExecutorStatus) -> &'static str {
    match status {
        ExecutorStatus::Success | ExecutorStatus::NeedsClarification => "done",
        ExecutorStatus::Error => "error",
    }
}

pub fn to_activity_view(record: &AgentRecord) -> AgentActivityView {
    AgentActivityView {
        task_spec: TaskSpec {
            objective: record.task_spec.objective.clone(),
            context: String::new(),
            constraints: record.task_spec.constraints.clone(),
            expected_output: record.task_spec.expected_output.clone(),
        },
        status: record.result.status.as_str().to_string(),
        summary: record.result.summary.clone(),
        activity_log: record.result.activity_log.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_db_status_maps_error() {
        assert_eq!(agent_db_status(&ExecutorStatus::Error), "error");
        assert_eq!(agent_db_status(&ExecutorStatus::Success), "done");
    }

    #[test]
    fn runs_agent_for_code_tasks_when_workspace_set() {
        let settings = AiSettings {
            workspace_path: Some("/tmp/project".into()),
            ..AiSettings::default()
        };
        assert!(should_run_agent(&settings, "fix the bug in main.rs"));
        assert!(!should_run_agent(&settings, "hello"));
    }

    #[test]
    fn direct_chat_for_casual_without_workspace() {
        let settings = AiSettings::default();
        assert!(!should_run_agent(&settings, "how are you?"));
    }
}
