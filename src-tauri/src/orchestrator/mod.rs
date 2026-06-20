use crate::agents::companion::{
    self, CompanionError, CompanionReply, TaskSpec, not_configured_message,
};
use crate::agents::delegation::{
    auto_holding_message, auto_task_spec, last_user_message, should_force_delegate,
};
use crate::agents::executor::{
    self, ActivityStep, ExecutorProgress, ExecutorResult, ExecutorStatus,
};
use crate::ai::ClientError;
use crate::db::Message;
use crate::rag;
use crate::run_state::RunState;
use crate::settings::AiSettings;
use serde::Serialize;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorActivityView {
    pub task_spec: TaskSpec,
    pub status: String,
    pub summary: String,
    pub activity_log: Vec<ActivityStep>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorProgressPayload {
    pub conversation_id: String,
    pub phase: String,
    pub activity: Option<ExecutorActivityView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionStreamPayload {
    pub conversation_id: String,
    pub stream_id: String,
    pub phase: String,
    pub delta: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

pub type ProgressFn = dyn Fn(ExecutorProgressPayload) + Send + Sync;
pub type CompanionStreamFn = dyn Fn(CompanionStreamPayload) + Send + Sync;

#[derive(Debug, Clone)]
pub enum CompanionIntent {
    Direct {
        message: String,
    },
    Delegate {
        holding_message: String,
        task_spec: TaskSpec,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct DelegatedResult {
    pub final_message: String,
    pub executor_record: ExecutorRecord,
}

#[derive(Debug, Clone)]
pub struct ExecutorRecord {
    pub task_spec: TaskSpec,
    pub result: ExecutorResult,
    pub db_status: &'static str,
}

pub async fn companion_intent(
    settings: &AiSettings,
    history: &[Message],
    memories: &[crate::db::Memory],
    stream: Option<&CompanionStreamFn>,
    stream_id: &str,
    conversation_id: &str,
) -> CompanionIntent {
    if settings.api_key.trim().is_empty() && !crate::ai::client::is_local_provider(&settings.base_url) {
        return CompanionIntent::Error {
            message: not_configured_message().into(),
        };
    }

    let user_message = last_user_message(history).unwrap_or("");

    if let Some(intent) = try_auto_delegate_intent(
        settings,
        history,
        user_message,
        stream,
        stream_id,
        conversation_id,
    )
    .await
    {
        return intent;
    }

    match companion::respond(settings, history, memories).await {
        Ok(CompanionReply::Direct { message }) => {
            if should_force_delegate(user_message, Some(&message), settings) {
                let task_spec = auto_task_spec(user_message, history);
                let holding_message = resolve_delegate_plan(
                    settings,
                    history,
                    user_message,
                    None,
                    stream,
                    stream_id,
                    conversation_id,
                )
                .await;
                CompanionIntent::Delegate {
                    holding_message,
                    task_spec,
                }
            } else {
                CompanionIntent::Direct { message }
            }
        }
        Ok(CompanionReply::Delegate {
            message: holding_message,
            task_spec,
        }) => {
            let plan = resolve_delegate_plan(
                settings,
                history,
                user_message,
                Some(holding_message),
                stream,
                stream_id,
                conversation_id,
            )
            .await;
            CompanionIntent::Delegate {
                holding_message: plan,
                task_spec,
            }
        }
        Err(CompanionError::Client(ClientError::NotConfigured)) => CompanionIntent::Error {
            message: not_configured_message().into(),
        },
        Err(err) => {
            if let Some(intent) = try_auto_delegate_intent(
                settings,
                history,
                user_message,
                stream,
                stream_id,
                conversation_id,
            )
            .await
            {
                eprintln!("companion intent failed ({err}); auto-delegating");
                intent
            } else {
                CompanionIntent::Error {
                    message: companion::error_message(&err.to_string()),
                }
            }
        }
    }
}

async fn try_auto_delegate_intent(
    settings: &AiSettings,
    history: &[Message],
    user_message: &str,
    stream: Option<&CompanionStreamFn>,
    stream_id: &str,
    conversation_id: &str,
) -> Option<CompanionIntent> {
    if !should_force_delegate(user_message, None, settings) {
        return None;
    }

    let task_spec = auto_task_spec(user_message, history);
    if task_spec.validate().is_err() {
        return None;
    }

    let holding_message = resolve_delegate_plan(
        settings,
        history,
        user_message,
        None,
        stream,
        stream_id,
        conversation_id,
    )
    .await;

    Some(CompanionIntent::Delegate {
        holding_message,
        task_spec,
    })
}

async fn resolve_delegate_plan(
    settings: &AiSettings,
    history: &[Message],
    objective: &str,
    candidate: Option<String>,
    stream: Option<&CompanionStreamFn>,
    stream_id: &str,
    conversation_id: &str,
) -> String {
    if let Some(message) = candidate {
        if is_plan_message(&message) {
            if let Some(emit) = stream {
                emit(CompanionStreamPayload {
                    conversation_id: conversation_id.to_string(),
                    stream_id: stream_id.to_string(),
                    phase: "plan".into(),
                    delta: String::new(),
                    done: true,
                    content: Some(message.clone()),
                });
            }
            return message;
        }
    }

    let result = if let Some(emit) = stream {
        companion::delegate_plan_message(
            settings,
            history,
            objective,
            Some(&|delta: &str| {
                emit(CompanionStreamPayload {
                    conversation_id: conversation_id.to_string(),
                    stream_id: stream_id.to_string(),
                    phase: "plan".into(),
                    delta: delta.to_string(),
                    done: false,
                    content: None,
                });
            }),
        )
        .await
    } else {
        companion::delegate_plan_message(settings, history, objective, None).await
    };

    match result {
        Ok(message) => {
            if let Some(emit) = stream {
                emit(CompanionStreamPayload {
                    conversation_id: conversation_id.to_string(),
                    stream_id: stream_id.to_string(),
                    phase: "plan".into(),
                    delta: String::new(),
                    done: true,
                    content: Some(message.clone()),
                });
            }
            message
        }
        Err(_) => auto_holding_message(&settings.ui_locale),
    }
}

fn is_plan_message(message: &str) -> bool {
    let trimmed = message.trim();
    trimmed.len() >= 40
        && !trimmed.contains("```")
        && (trimmed.contains('.') || trimmed.contains('؟') || trimmed.contains('!'))
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
        "{}\n\n--- User memories ---\n{}",
        task_spec.context, block
    );
}

pub fn enrich_task_with_context(
    settings: &AiSettings,
    task_spec: &mut TaskSpec,
) {
    if !settings.context_pack_enabled || !settings.workspace_configured() {
        return;
    }

    let Some(workspace) = settings.workspace_path.as_ref() else {
        return;
    };

    let pack = crate::context::build_context_pack(std::path::Path::new(workspace));
    if pack.trim().is_empty() {
        return;
    }

    task_spec.context = format!(
        "{}\n\n--- Project context ---\n{}",
        task_spec.context, pack
    );
}

pub async fn enrich_task_with_rag(
    settings: &AiSettings,
    task_spec: &mut TaskSpec,
    rag_chunks: Vec<rag::RagChunk>,
) {
    if !settings.rag_enabled || rag_chunks.is_empty() {
        return;
    }

    match rag::retrieve_context(settings, &rag_chunks, &task_spec.objective).await {
        Ok(context) if !context.trim().is_empty() => {
            task_spec.context = format!(
                "{}\n\n--- Retrieved project context (local RAG) ---\n{}",
                task_spec.context, context
            );
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run_delegated_task(
    settings: &AiSettings,
    history: &[Message],
    conversation_id: &str,
    task_spec: TaskSpec,
    progress: Option<&ProgressFn>,
    stream: Option<&CompanionStreamFn>,
    stream_id: &str,
    cancel: Option<&AtomicBool>,
) -> Result<DelegatedResult, String> {
    if cancel.is_some_and(RunState::is_cancelled) {
        return Err(cancelled_message());
    }
    if let Err(err) = task_spec.validate() {
        return Err(companion::error_message(&format!(
            "I had trouble framing that task: {err}"
        )));
    }

    emit_phase(
        progress,
        conversation_id,
        "executing",
        Some(partial_activity(
            &task_spec,
            "running",
            "Working on it…",
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
            "executing",
            Some(ExecutorActivityView {
                task_spec: update.task_spec,
                status: update.status,
                summary: update.summary,
                activity_log: update.activity_log,
            }),
        );
    };

    let executor_result = executor::execute(
        settings,
        &task_spec,
        history,
        Some(&executor_callback),
        cancel,
    )
    .await
    .map_err(|err| match err {
        crate::agents::executor::ExecutorError::Cancelled => cancelled_message(),
        err => companion::error_message(&err.to_string()),
    })?;

    let record = ExecutorRecord {
        task_spec: task_spec.clone(),
        result: executor_result.clone(),
        db_status: executor_db_status(&executor_result.status),
    };

    emit_phase(
        progress,
        conversation_id,
        "formatting",
        Some(to_activity_view(&record)),
    );

    if cancel.is_some_and(RunState::is_cancelled) {
        return Err(cancelled_message());
    }

    let final_message = if executor::should_skip_format_pass(&executor_result) {
        executor::template_format(&executor_result)
    } else if let Some(emit) = stream {
        companion::format_executor_result(
            settings,
            history,
            &task_spec,
            &executor_result,
            Some(&|delta: &str| {
                emit(CompanionStreamPayload {
                    conversation_id: conversation_id.to_string(),
                    stream_id: stream_id.to_string(),
                    phase: "final".into(),
                    delta: delta.to_string(),
                    done: false,
                    content: None,
                });
            }),
        )
        .await
        .inspect(|message| {
            emit(CompanionStreamPayload {
                conversation_id: conversation_id.to_string(),
                stream_id: stream_id.to_string(),
                phase: "final".into(),
                delta: String::new(),
                done: true,
                content: Some(message.clone()),
            });
        })
        .map_err(|err| companion::error_message(&err.to_string()))?
    } else {
        companion::format_executor_result(settings, history, &task_spec, &executor_result, None)
            .await
            .map_err(|err| companion::error_message(&err.to_string()))?
    };

    Ok(DelegatedResult {
        final_message,
        executor_record: record,
    })
}

fn cancelled_message() -> String {
    "I stopped the agent run here — you can ask again anytime, and we'll pick up fresh.".into()
}

fn emit_phase(
    progress: Option<&ProgressFn>,
    conversation_id: &str,
    phase: &str,
    activity: Option<ExecutorActivityView>,
) {
    if let Some(callback) = progress {
        callback(ExecutorProgressPayload {
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
) -> ExecutorActivityView {
    ExecutorActivityView {
        task_spec: task_spec.clone(),
        status: status.into(),
        summary: summary.into(),
        activity_log,
    }
}

fn executor_db_status(status: &ExecutorStatus) -> &'static str {
    match status {
        ExecutorStatus::Success | ExecutorStatus::NeedsClarification => "done",
        ExecutorStatus::Error => "error",
    }
}

pub fn to_activity_view(record: &ExecutorRecord) -> ExecutorActivityView {
    ExecutorActivityView {
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
    fn executor_db_status_maps_error() {
        assert_eq!(executor_db_status(&ExecutorStatus::Error), "error");
        assert_eq!(executor_db_status(&ExecutorStatus::Success), "done");
    }

    #[test]
    fn accepts_plan_message() {
        assert!(is_plan_message(
            "I understand you want to fix auth. I'll scan src, patch the handler, and run tests."
        ));
    }
}
