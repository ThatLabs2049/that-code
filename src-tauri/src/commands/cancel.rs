use crate::run_state::RunState;
use tauri::State;

#[tauri::command]
pub fn cancel_run(state: State<'_, RunState>, conversation_id: String) -> Result<bool, String> {
    Ok(state.cancel(&conversation_id))
}
