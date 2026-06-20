use crate::db::{self, DbState, Memory};
use tauri::State;

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

#[tauri::command]
pub fn list_memories(state: State<'_, DbState>) -> Result<Vec<Memory>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_memories(&conn).map_err(db_error)
}

#[tauri::command]
pub fn create_memory(state: State<'_, DbState>, content: String) -> Result<Memory, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::create_memory(&conn, &content).map_err(db_error)
}

#[tauri::command]
pub fn update_memory(
    state: State<'_, DbState>,
    id: String,
    content: String,
) -> Result<Memory, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::update_memory(&conn, &id, &content).map_err(db_error)
}

#[tauri::command]
pub fn delete_memory(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_memory(&conn, &id).map_err(db_error)
}
