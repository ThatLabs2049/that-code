use crate::agents::executor::ActivityStep;
use crate::db::{self, DbState};
use crate::orchestrator::{
    self, CompanionIntent, CompanionStreamPayload, ExecutorProgressPayload,
};
use crate::rag;
use crate::run_state::RunState;
use crate::settings;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executor_run_id: Option<String>,
    pub has_file_changes: bool,
}

fn send_result(executor_run_id: Option<String>, has_file_changes: bool) -> SendMessageResult {
    SendMessageResult {
        executor_run_id,
        has_file_changes,
    }
}

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

fn slim_executor_result_for_storage(result: &crate::agents::executor::ExecutorResult) -> crate::agents::executor::ExecutorResult {
    let mut slim = result.clone();
    const MAX_CONTENT: usize = 8_192;
    const MAX_DIFF: usize = 8_192;

    if slim.content.len() > MAX_CONTENT {
        slim.content = format!("{}… [truncated]", &slim.content[..MAX_CONTENT]);
    }

    slim.file_changes = slim
        .file_changes
        .into_iter()
        .map(|mut change| {
            change.before_content = None;
            change.after_content = None;
            if change.diff.len() > MAX_DIFF {
                change.diff = format!("{}… [truncated]", &change.diff[..MAX_DIFF]);
            }
            change
        })
        .collect();

    slim
}

fn persist_delegated_result(
    conn: &rusqlite::Connection,
    conversation_id: &str,
    holding_id: &str,
    delegated: &orchestrator::DelegatedResult,
) -> Result<(db::Message, String), rusqlite::Error> {
    let storage_result = slim_executor_result_for_storage(&delegated.executor_record.result);
    let task_spec_json = serde_json::to_string(&delegated.executor_record.task_spec).map_err(
        |err| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(err.to_string()))),
    )?;
    let result_json = serde_json::to_string(&storage_result).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(err.to_string())))
    })?;

    let run = db::insert_executor_run(
        conn,
        conversation_id,
        Some(holding_id),
        &task_spec_json,
        "running",
    )?;

    let companion_message = db::insert_message(
        conn,
        conversation_id,
        "companion",
        &delegated.final_message,
    )?;

    db::complete_executor_run(
        conn,
        &run.id,
        &result_json,
        delegated.executor_record.db_status,
    )?;

    Ok((companion_message, run.id))
}

fn persist_delegated_result_best_effort(
    conn: &rusqlite::Connection,
    conversation_id: &str,
    holding_id: &str,
    delegated: &orchestrator::DelegatedResult,
) -> (Option<db::Message>, Option<String>) {
    match persist_delegated_result(conn, conversation_id, holding_id, delegated) {
        Ok((message, run_id)) => (Some(message), Some(run_id)),
        Err(err) => {
            eprintln!("executor persist failed: {err}");
            match db::insert_message(
                conn,
                conversation_id,
                "companion",
                &delegated.final_message,
            ) {
                Ok(message) => (Some(message), None),
                Err(insert_err) => {
                    eprintln!("companion message fallback failed: {insert_err}");
                    (None, None)
                }
            }
        }
    }
}

fn emit_progress(app: &AppHandle, payload: ExecutorProgressPayload) {
    let _ = app.emit("executor-progress", &payload);
}

fn emit_companion_stream(app: &AppHandle, payload: CompanionStreamPayload) {
    let _ = app.emit("companion-stream", &payload);
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
pub async fn send_message(
    app: AppHandle,
    state: State<'_, DbState>,
    run_state: State<'_, RunState>,
    conversation_id: String,
    content: String,
) -> Result<SendMessageResult, String> {
    let cancel_flag = run_state.try_register(&conversation_id)?;
    let stream_id = uuid::Uuid::new_v4().to_string();

    let result = send_message_inner(
        &app,
        &state,
        &conversation_id,
        &content,
        &stream_id,
        Some(&cancel_flag),
    )
    .await;

    if let Ok(conn) = state.conn.lock() {
        let _ = db::fail_running_tasks(&conn, &conversation_id);
    }

    run_state.clear(&conversation_id);
    result
}

async fn send_message_inner(
    app: &AppHandle,
    state: &State<'_, DbState>,
    conversation_id: &str,
    content: &str,
    stream_id: &str,
    cancel: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<SendMessageResult, String> {
    let (_user_message, history, ai_settings, memories) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let user_message =
            db::send_user_message(&conn, conversation_id, content).map_err(db_error)?;
        let history = db::get_messages(&conn, conversation_id).map_err(db_error)?;
        let ai_settings = settings::load(&conn).map_err(db_error)?;
        let memories = db::top_memories_for_context(&conn, 8).unwrap_or_default();
        (user_message, history, ai_settings, memories)
    };

    let app_for_stream = app.clone();
    let stream_callback = move |payload: CompanionStreamPayload| {
        emit_companion_stream(&app_for_stream, payload);
    };

    let intent = orchestrator::companion_intent(
        &ai_settings,
        &history,
        &memories,
        Some(&stream_callback),
        stream_id,
        conversation_id,
    )
    .await;

    match intent {
        CompanionIntent::Direct { message } | CompanionIntent::Error { message } => {
            let _companion_message = {
                let conn = state.conn.lock().map_err(|e| e.to_string())?;
                db::insert_message(&conn, conversation_id, "companion", &message)
                    .map_err(db_error)?
            };

            Ok(send_result(None, false))
        }
        CompanionIntent::Delegate {
            holding_message,
            task_spec,
        } => {
            task_spec.validate().map_err(|e| e.to_string())?;

            let holding_db = {
                let conn = state.conn.lock().map_err(|e| e.to_string())?;
                db::insert_message(&conn, conversation_id, "companion", &holding_message)
                    .map_err(db_error)?
            };

            emit_progress(
                app,
                ExecutorProgressPayload {
                    conversation_id: conversation_id.to_string(),
                    phase: "holding".into(),
                    activity: ai_settings.executor_visibility.then(|| {
                        orchestrator::ExecutorActivityView {
                            task_spec: task_spec.clone(),
                            status: "running".into(),
                            summary: holding_message.clone(),
                            activity_log: vec![ActivityStep {
                                step: "intent".into(),
                                detail: task_spec.objective.clone(),
                            }],
                        }
                    }),
                },
            );

            let app_for_progress = app.clone();
            let progress_callback = move |payload: ExecutorProgressPayload| {
                emit_progress(&app_for_progress, payload);
            };

            let (rag_chunks, task_memories) = {
                let conn = state.conn.lock().map_err(|e| e.to_string())?;
                let rag_chunks = rag::list_chunks(&conn).unwrap_or_default();
                (rag_chunks, memories.clone())
            };

            let mut delegated = None;
            let mut last_executor_run_id = None;

            if ai_settings.task_queue_enabled {
                {
                    let conn = state.conn.lock().map_err(|e| e.to_string())?;
                    db::enqueue_task(&conn, conversation_id, &task_spec).map_err(db_error)?;
                }

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
                        let _ = db::complete_queued_task(&conn, &queued.id, "error");
                        let _companion_message = db::insert_message(
                            &conn,
                            conversation_id,
                            "companion",
                            "I stopped the agent run here — you can ask again anytime, and we'll pick up fresh.",
                        )
                        .map_err(db_error)?;

                        return Ok(send_result(None, false));
                    }

                    let mut spec = queued.task_spec;
                    orchestrator::enrich_task_with_context(&ai_settings, &mut spec);
                    orchestrator::enrich_task_with_memories(&mut spec, &task_memories);
                    orchestrator::enrich_task_with_rag(&ai_settings, &mut spec, rag_chunks.clone())
                        .await;

                    match orchestrator::run_delegated_task(
                        &ai_settings,
                        &history,
                        conversation_id,
                        spec,
                        Some(&progress_callback),
                        Some(&stream_callback),
                        stream_id,
                        cancel.map(|flag| flag.as_ref()),
                    )
                    .await
                    {
                        Ok(result) => {
                            let conn = state.conn.lock().map_err(|e| e.to_string())?;
                            db::complete_queued_task(
                                &conn,
                                &queued.id,
                                result.executor_record.db_status,
                            )
                            .map_err(db_error)?;
                            let (_, run_id) = persist_delegated_result_best_effort(
                                &conn,
                                conversation_id,
                                &holding_db.id,
                                &result,
                            );
                            if let Some(run_id) = run_id {
                                last_executor_run_id = Some(run_id);
                            }
                            delegated = Some(result);
                        }
                        Err(message) => {
                            let conn = state.conn.lock().map_err(|e| e.to_string())?;
                            let _ = db::complete_queued_task(&conn, &queued.id, "error");

                            let _companion_message = {
                                db::insert_message(&conn, conversation_id, "companion", &message)
                                    .map_err(db_error)?
                            };

                            return Ok(send_result(None, false));
                        }
                    }
                }
            } else {
                let mut spec = task_spec;
                orchestrator::enrich_task_with_context(&ai_settings, &mut spec);
                orchestrator::enrich_task_with_memories(&mut spec, &task_memories);
                orchestrator::enrich_task_with_rag(&ai_settings, &mut spec, rag_chunks.clone())
                    .await;

                match orchestrator::run_delegated_task(
                    &ai_settings,
                    &history,
                    conversation_id,
                    spec,
                    Some(&progress_callback),
                    Some(&stream_callback),
                    stream_id,
                    cancel.map(|flag| flag.as_ref()),
                )
                .await
                {
                    Ok(result) => {
                        let conn = state.conn.lock().map_err(|e| e.to_string())?;
                        let (_, run_id) = persist_delegated_result_best_effort(
                            &conn,
                            conversation_id,
                            &holding_db.id,
                            &result,
                        );
                        if let Some(run_id) = run_id {
                            last_executor_run_id = Some(run_id);
                        }
                        delegated = Some(result);
                    }
                    Err(message) => {
                        let _companion_message = {
                            let conn = state.conn.lock().map_err(|e| e.to_string())?;
                            db::insert_message(&conn, conversation_id, "companion", &message)
                                .map_err(db_error)?
                        };

                        return Ok(send_result(None, false));
                    }
                }
            }

            let delegated = delegated.ok_or_else(|| "No delegated task ran".to_string())?;

            let has_file_changes = !delegated.executor_record.result.file_changes.is_empty();

            Ok(send_result(last_executor_run_id, has_file_changes))
        }
    }
}

#[tauri::command]
pub fn clear_history(
    state: State<'_, DbState>,
    conversation_id: String,
) -> Result<Vec<db::Message>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::clear_conversation_messages(&conn, &conversation_id).map_err(db_error)?;
    db::get_messages(&conn, &conversation_id).map_err(db_error)
}
