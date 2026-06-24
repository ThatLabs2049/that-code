use crate::agents::profile::AgentTier;
use crate::agents::task::{build_task_from_message, last_user_message};
use crate::db::{self, DbState};
use crate::agent_plan::PlanState;
use crate::orchestrator::{self, AgentProgressPayload, AgentRunResult, AssistantStreamPayload, TurnResult};
use crate::rag;
use crate::tools::truncate_at_byte_boundary;
use crate::rag_ann_state::RagAnnState;
use crate::run_state::RunState;
use crate::settings;
use crate::workspace;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executor_run_id: Option<String>,
    pub has_file_changes: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retrieved_context: Vec<rag::RetrievedChunk>,
    #[serde(default)]
    pub awaiting_plan_approval: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_message: Option<String>,
}

pub(crate) fn send_result(
    executor_run_id: Option<String>,
    has_file_changes: bool,
    retrieved_context: Vec<rag::RetrievedChunk>,
    awaiting_plan_approval: bool,
    plan_content: Option<String>,
    assistant_message: Option<String>,
) -> SendMessageResult {
    SendMessageResult {
        executor_run_id,
        has_file_changes,
        retrieved_context,
        awaiting_plan_approval,
        plan_content,
        assistant_message,
    }
}

pub(crate) fn non_empty_assistant_message(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        "Completed the requested task.".into()
    } else {
        trimmed.to_string()
    }
}

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

fn slim_executor_result_for_storage(
    result: &crate::agents::executor::ExecutorResult,
) -> crate::agents::executor::ExecutorResult {
    let mut slim = result.clone();
    const MAX_CONTENT: usize = 8_192;
    const MAX_DIFF: usize = 8_192;

    if slim.content.len() > MAX_CONTENT {
        slim.content = format!("{}… [truncated]", truncate_at_byte_boundary(&slim.content, MAX_CONTENT));
    }

    slim.file_changes = slim
        .file_changes
        .into_iter()
        .map(|mut change| {
            if change.diff.len() > MAX_DIFF {
                change.diff = format!("{}… [truncated]", truncate_at_byte_boundary(&change.diff, MAX_DIFF));
            }
            if let Some(ref mut before) = change.before_content {
                if before.len() > MAX_CONTENT {
                    *before = format!("{}… [truncated]", truncate_at_byte_boundary(before, MAX_CONTENT));
                }
            }
            if let Some(ref mut after) = change.after_content {
                if after.len() > MAX_CONTENT {
                    *after = format!("{}… [truncated]", truncate_at_byte_boundary(after, MAX_CONTENT));
                }
            }
            change
        })
        .collect();

    slim
}

fn persist_agent_result(
    conn: &rusqlite::Connection,
    conversation_id: &str,
    turn: &TurnResult,
) -> Result<(db::Message, Option<String>), rusqlite::Error> {
    let assistant_message = db::insert_message(
        conn,
        conversation_id,
        "companion",
        &non_empty_assistant_message(&turn.final_message),
    )?;

    let Some(record) = turn.agent_record.as_ref() else {
        return Ok((assistant_message, None));
    };

    let storage_result = slim_executor_result_for_storage(&record.result);
    let task_spec_json = serde_json::to_string(&record.task_spec).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(err.to_string())))
    })?;
    let result_json = serde_json::to_string(&storage_result).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(err.to_string())))
    })?;

    let run = db::insert_executor_run(
        conn,
        conversation_id,
        Some(&assistant_message.id),
        &task_spec_json,
        "running",
    )?;

    db::complete_executor_run(
        conn,
        &run.id,
        &result_json,
        record.db_status,
    )?;

    Ok((assistant_message, Some(run.id)))
}

pub(crate) fn persist_agent_result_best_effort(
    conn: &rusqlite::Connection,
    conversation_id: &str,
    turn: &TurnResult,
) -> (Option<db::Message>, Option<String>) {
    match persist_agent_result(conn, conversation_id, turn) {
        Ok((message, run_id)) => (Some(message), run_id),
        Err(err) => {
            eprintln!("agent persist failed: {err}");
            match db::insert_message(
                conn,
                conversation_id,
                "companion",
                &non_empty_assistant_message(&turn.final_message),
            ) {
                Ok(message) => (Some(message), None),
                Err(insert_err) => {
                    eprintln!("assistant message fallback failed: {insert_err}");
                    (None, None)
                }
            }
        }
    }
}

fn emit_progress(app: &AppHandle, payload: AgentProgressPayload) {
    let _ = app.emit("executor-progress", &payload);
}

fn emit_stream(app: &AppHandle, payload: AssistantStreamPayload) {
    let _ = app.emit("assistant-stream", &payload);
}

#[tauri::command]
pub fn list_conversations(state: State<'_, DbState>) -> Result<Vec<db::Conversation>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_conversations(&conn).map_err(db_error)
}

#[tauri::command]
pub fn get_active_conversation(state: State<'_, DbState>) -> Result<db::Conversation, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_or_create_active_conversation(&conn).map_err(db_error)
}

#[tauri::command]
pub fn get_messages(
    state: State<'_, DbState>,
    conversation_id: String,
) -> Result<Vec<db::Message>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_messages(&conn, &conversation_id).map_err(db_error)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, DbState>,
    run_state: State<'_, RunState>,
    plan_state: State<'_, PlanState>,
    ann_state: State<'_, RagAnnState>,
    conversation_id: String,
    content: String,
    agent_tier: Option<String>,
    attachments: Option<Vec<workspace::MessageAttachment>>,
    explore_then_implement: Option<bool>,
) -> Result<SendMessageResult, String> {
    if plan_state.has(&conversation_id) {
        return Err(
            "Approve or reject the pending plan before sending a new message.".into(),
        );
    }

    let cancel_flag = run_state.try_register(&conversation_id)?;
    let attachment_list = attachments.unwrap_or_default();
    let explore = explore_then_implement.unwrap_or(false);

    let result = send_message_inner(
        &app,
        &state,
        ann_state.inner(),
        &plan_state,
        &conversation_id,
        &content,
        agent_tier,
        &attachment_list,
        explore,
        Some(&cancel_flag),
    )
    .await;

    if let Ok(conn) = state.conn.lock() {
        let _ = db::fail_running_tasks(&conn, &conversation_id);
    }

    run_state.clear(&conversation_id);
    result
}

#[allow(clippy::too_many_arguments)]
async fn send_message_inner(
    app: &AppHandle,
    state: &State<'_, DbState>,
    ann_state: &RagAnnState,
    plan_state: &PlanState,
    conversation_id: &str,
    content: &str,
    agent_tier: Option<String>,
    attachments: &[workspace::MessageAttachment],
    explore_then_implement: bool,
    cancel: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<SendMessageResult, String> {
    let (history, ai_settings, memories) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let _user_message =
            db::send_user_message(&conn, conversation_id, content).map_err(db_error)?;
        let history = db::get_messages(&conn, conversation_id).map_err(db_error)?;
        let ai_settings = settings::load(&conn).map_err(db_error)?;
        let memories = db::top_memories_for_context(&conn, 8).unwrap_or_default();
        (history, ai_settings, memories)
    };

    let tier = AgentTier::parse(
        agent_tier
            .as_deref()
            .unwrap_or(ai_settings.default_agent_tier.as_str()),
    );

    let app_for_progress = app.clone();
    let progress_callback = move |payload: AgentProgressPayload| {
        emit_progress(&app_for_progress, payload);
    };

    let stream_id = uuid::Uuid::new_v4().to_string();
    let app_for_stream = app.clone();
    let stream_callback = move |payload: AssistantStreamPayload| {
        emit_stream(&app_for_stream, payload);
    };

    let mut last_executor_run_id = None;

    if ai_settings.task_queue_enabled && ai_settings.workspace_configured() {
        let user_objective = last_user_message(&history).unwrap_or(content);
        let mut queued_spec = build_task_from_message(user_objective, &history);
        orchestrator::enrich_task_with_attachments(&ai_settings, &mut queued_spec, attachments)?;

        let _ack = orchestrator::try_stream_task_acknowledgment(
            &ai_settings,
            user_objective,
            &queued_spec,
            conversation_id,
            &stream_id,
            &stream_callback,
        )
        .await;

        {
            let conn = state.conn.lock().map_err(|e| e.to_string())?;
            db::enqueue_task(&conn, conversation_id, &queued_spec).map_err(db_error)?;
        }

        let mut has_file_changes = false;
        let mut last_retrieved_context = Vec::new();
        let mut last_assistant_message: Option<String> = None;

        loop {
            let queued = {
                let conn = state.conn.lock().map_err(|e| e.to_string())?;
                db::pop_next_pending(&conn, conversation_id).map_err(db_error)?
            };
            let Some(queued) = queued else {
                break;
            };

            if cancel.is_some_and(|flag| RunState::is_cancelled(flag)) {
                let conn = state.conn.lock().map_err(|e| e.to_string())?;
                let _ = db::fail_all_active_tasks(&conn, conversation_id);
                let _ = db::insert_message(
                    &conn,
                    conversation_id,
                    "companion",
                    &orchestrator::cancelled_message(),
                )
                .map_err(db_error)?;
                return Ok(send_result(
                    None,
                    false,
                    Vec::new(),
                    false,
                    None,
                    Some(non_empty_assistant_message(&orchestrator::cancelled_message())),
                ));
            }

            queued_spec = queued.task_spec;
            let run_config = crate::agents::profile::RunConfig::from_settings(
                &ai_settings,
                tier,
                &queued_spec.objective,
            );
            if run_config.context_pack_enabled {
                orchestrator::enrich_task_with_context(&ai_settings, &mut queued_spec);
            }
            orchestrator::enrich_task_with_memories(&mut queued_spec, &memories);
            orchestrator::enrich_task_with_rules(&ai_settings, &mut queued_spec);
            orchestrator::enrich_task_with_symbols(&ai_settings, &mut queued_spec);
            last_retrieved_context = if run_config.rag_enabled {
                orchestrator::enrich_task_with_rag(&ai_settings, &mut queued_spec, ann_state, state)
                    .await
            } else {
                Vec::new()
            };

            match orchestrator::run_agent_task(
                &ai_settings,
                &history,
                conversation_id,
                queued_spec,
                tier,
                Some(&progress_callback),
                cancel.map(|flag| flag.as_ref()),
            )
            .await
            {
                Ok(AgentRunResult::Completed(record)) => {
                    has_file_changes = has_file_changes || !record.result.file_changes.is_empty();
                    let final_message = crate::agents::executor::template_format(&record.result);
                    last_assistant_message = Some(non_empty_assistant_message(&final_message));
                    let turn = TurnResult {
                        final_message,
                        agent_record: Some(record),
                        retrieved_context: last_retrieved_context.clone(),
                        awaiting_plan_approval: false,
                        plan_content: None,
                        pending_plan: None,
                    };
                    let conn = state.conn.lock().map_err(|e| e.to_string())?;
                    db::complete_queued_task(
                        &conn,
                        &queued.id,
                        turn.agent_record.as_ref().map(|r| r.db_status).unwrap_or("error"),
                    )
                    .map_err(db_error)?;
                    let (_, run_id) =
                        persist_agent_result_best_effort(&conn, conversation_id, &turn);
                    let _ = app.emit("messages-updated", conversation_id);
                    if let Some(run_id) = run_id {
                        last_executor_run_id = Some(run_id);
                    }
                }
                Ok(AgentRunResult::AwaitingPlan(pending)) => {
                    let conn = state.conn.lock().map_err(|e| e.to_string())?;
                    plan_state
                        .store_persisted(&conn, pending.clone())
                        .map_err(db_error)?;
                    let _ = db::complete_queued_task(&conn, &queued.id, "done");
                    return Ok(send_result(
                        None,
                        false,
                        last_retrieved_context,
                        true,
                        Some(pending.briefing.clone()),
                        None,
                    ));
                }
                Err(message) => {
                    let conn = state.conn.lock().map_err(|e| e.to_string())?;
                    let _ = db::complete_queued_task(&conn, &queued.id, "error");
                    let _ = db::insert_message(&conn, conversation_id, "companion", &message)
                        .map_err(db_error)?;
                    let _ = app.emit("messages-updated", conversation_id);
                    return Ok(send_result(
                        None,
                        false,
                        Vec::new(),
                        false,
                        None,
                        Some(non_empty_assistant_message(&message)),
                    ));
                }
            }
        }

        let _ = app.emit("messages-updated", conversation_id);
        return Ok(send_result(
            last_executor_run_id,
            has_file_changes,
            last_retrieved_context,
            false,
            None,
            last_assistant_message,
        ));
    }

    let turn = orchestrator::run_turn(
        &ai_settings,
        &history,
        conversation_id,
        &memories,
        state.inner(),
        ann_state,
        tier,
        attachments,
        explore_then_implement,
        &stream_id,
        Some(&stream_callback),
        Some(&progress_callback),
        cancel.map(|flag| flag.as_ref()),
        None,
    )
    .await?;

    let has_file_changes = turn
        .agent_record
        .as_ref()
        .is_some_and(|r| !r.result.file_changes.is_empty());

    let (_, run_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        if let Some(plan) = turn.pending_plan.clone() {
            plan_state
                .store_persisted(&conn, plan)
                .map_err(db_error)?;
        }
        let persisted = if turn.awaiting_plan_approval {
            (None, None)
        } else {
            persist_agent_result_best_effort(&conn, conversation_id, &turn)
        };
        let _ = app.emit("messages-updated", conversation_id);
        persisted
    };

    let assistant_message = if turn.awaiting_plan_approval {
        None
    } else {
        Some(non_empty_assistant_message(&turn.final_message))
    };

    Ok(send_result(
        run_id,
        has_file_changes,
        turn.retrieved_context,
        turn.awaiting_plan_approval,
        turn.plan_content,
        assistant_message,
    ))
}

#[tauri::command]
pub fn clear_history(
    state: State<'_, DbState>,
    plan_state: State<'_, PlanState>,
    conversation_id: String,
) -> Result<Vec<db::Message>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    plan_state.discard_persisted(&conn, &conversation_id);
    db::clear_conversation_messages(&conn, &conversation_id).map_err(db_error)?;
    db::get_messages(&conn, &conversation_id).map_err(db_error)
}
