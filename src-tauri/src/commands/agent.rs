use crate::agent_plan::{PendingPlanView, PlanState};
use crate::commands::chat::{non_empty_assistant_message, persist_agent_result_best_effort, send_result, SendMessageResult};
use crate::db::{self, DbState};
use crate::orchestrator::{self, AgentProgressPayload};
use crate::run_state::RunState;
use crate::settings;
use tauri::{AppHandle, Emitter, State};

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

fn emit_progress(app: &AppHandle, payload: AgentProgressPayload) {
    let _ = app.emit("executor-progress", &payload);
}

#[tauri::command]
pub fn get_pending_plan(
    plan_state: State<PlanState>,
    conversation_id: String,
) -> Option<PendingPlanView> {
    plan_state
        .get_briefing(&conversation_id)
        .map(|briefing| PendingPlanView { briefing })
}

#[tauri::command]
pub async fn respond_to_agent_plan(
    app: AppHandle,
    state: State<'_, DbState>,
    run_state: State<'_, RunState>,
    plan_state: State<'_, PlanState>,
    conversation_id: String,
    approved: bool,
) -> Result<SendMessageResult, String> {
    let pending = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        plan_state
            .take_persisted(&conn, &conversation_id)
            .ok_or_else(|| "No pending plan for this conversation".to_string())?
    };

    if !approved {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let _ = db::insert_message(
            &conn,
            &conversation_id,
            "companion",
            "Plan rejected. Adjust your request or disable plan review in Settings → Agent.",
        )
        .map_err(db_error)?;
        let _ = app.emit("messages-updated", &conversation_id);
        return Ok(send_result(
            None,
            false,
            Vec::new(),
            false,
            None,
            Some(non_empty_assistant_message(
                "Plan rejected. Adjust your request or disable plan review in Settings → Agent.",
            )),
        ));
    }

    let cancel_flag = run_state.try_register(&conversation_id)?;

    let result = respond_to_agent_plan_inner(
        &app,
        &state,
        &conversation_id,
        &pending,
        Some(&cancel_flag),
    )
    .await;

    if let Ok(conn) = state.conn.lock() {
        let _ = db::fail_running_tasks(&conn, &conversation_id);
    }

    run_state.clear(&conversation_id);
    result
}

async fn respond_to_agent_plan_inner(
    app: &AppHandle,
    state: &State<'_, DbState>,
    conversation_id: &str,
    pending: &crate::agent_plan::PendingPlan,
    cancel: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<SendMessageResult, String> {
    let (history, ai_settings) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let history = db::get_messages(&conn, conversation_id).map_err(db_error)?;
        let ai_settings = settings::load(&conn).map_err(db_error)?;
        (history, ai_settings)
    };

    let app_for_progress = app.clone();
    let progress_callback = move |payload: AgentProgressPayload| {
        emit_progress(&app_for_progress, payload);
    };

    let record = orchestrator::run_agent_from_plan(
        &ai_settings,
        &history,
        conversation_id,
        pending,
        Some(&progress_callback),
        cancel.map(|flag| flag.as_ref()),
    )
    .await?;

    let has_file_changes = !record.result.file_changes.is_empty();
    let turn = orchestrator::TurnResult {
        final_message: crate::agents::executor::template_format(&record.result),
        agent_record: Some(record),
        retrieved_context: Vec::new(),
        awaiting_plan_approval: false,
        plan_content: None,
        pending_plan: None,
    };

    let (_, run_id) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let persisted = persist_agent_result_best_effort(&conn, conversation_id, &turn);
        let _ = app.emit("messages-updated", conversation_id);
        persisted
    };

    Ok(send_result(
        run_id,
        has_file_changes,
        Vec::new(),
        false,
        None,
        Some(non_empty_assistant_message(&turn.final_message)),
    ))
}
