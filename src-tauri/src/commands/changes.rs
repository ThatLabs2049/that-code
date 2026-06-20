use crate::changes::{self, FileChange};
use crate::db::{self, DbState};
use crate::settings;
use crate::tools::WorkspaceSandbox;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
struct StoredExecutorResult {
    #[serde(default)]
    file_changes: Vec<FileChange>,
}

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

#[tauri::command]
pub fn revert_executor_run(
    state: State<'_, DbState>,
    run_id: String,
    paths: Option<Vec<String>>,
) -> Result<(), String> {
    let (result_json, settings) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let run = db::get_executor_run(&conn, &run_id).map_err(db_error)?;
        let settings = settings::load(&conn).map_err(db_error)?;
        (run.result.ok_or_else(|| "run has no stored result".to_string())?, settings)
    };

    let parsed: StoredExecutorResult = serde_json::from_str(&result_json)
        .map_err(|e| format!("could not parse run result: {e}"))?;

    if parsed.file_changes.is_empty() {
        return Ok(());
    }

    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Err("workspace not configured".into());
    };

    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;
    changes::revert_file_changes(
        &sandbox,
        &parsed.file_changes,
        paths.as_deref(),
    )
}

#[tauri::command]
pub fn get_executor_run_changes(
    state: State<'_, DbState>,
    run_id: String,
) -> Result<Vec<FileChange>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let run = db::get_executor_run(&conn, &run_id).map_err(db_error)?;
    let result_json = run
        .result
        .ok_or_else(|| "run has no stored result".to_string())?;
    let parsed: StoredExecutorResult = serde_json::from_str(&result_json)
        .map_err(|e| format!("could not parse run result: {e}"))?;

    Ok(parsed
        .file_changes
        .into_iter()
        .map(|mut change| {
            change.before_content = None;
            change.after_content = None;
            change
        })
        .collect())
}
