use crate::agents::companion::TaskSpec;
use crate::db::{self, DbState, QueuedTask};
use tauri::State;

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

#[tauri::command]
pub fn list_queued_tasks(
    state: State<'_, DbState>,
    conversation_id: String,
) -> Result<Vec<QueuedTask>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_pending_tasks(&conn, &conversation_id).map_err(db_error)
}

#[tauri::command]
pub fn queue_task(
    state: State<'_, DbState>,
    conversation_id: String,
    task_spec: TaskSpec,
) -> Result<QueuedTask, String> {
    task_spec.validate().map_err(|err| err.to_string())?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::enqueue_task(&conn, &conversation_id, &task_spec).map_err(db_error)
}

#[tauri::command]
pub fn clear_completed_tasks(
    state: State<'_, DbState>,
    conversation_id: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::clear_completed(&conn, &conversation_id).map_err(db_error)
}
