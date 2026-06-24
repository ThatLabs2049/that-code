use crate::changes::{self, DiffHunk, FileChange};
use crate::db::{self, DbState};
use crate::settings;
use crate::tools::WorkspaceSandbox;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize, serde::Serialize)]
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
            change.diff = String::new();
            change
        })
        .collect())
}

#[tauri::command]
pub fn get_executor_run_file_diff(
    state: State<'_, DbState>,
    run_id: String,
    path: String,
) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let run = db::get_executor_run(&conn, &run_id).map_err(db_error)?;
    let result_json = run
        .result
        .ok_or_else(|| "run has no stored result".to_string())?;
    let parsed: StoredExecutorResult = serde_json::from_str(&result_json)
        .map_err(|e| format!("could not parse run result: {e}"))?;

    let normalized = path.trim().replace('\\', "/");
    parsed
        .file_changes
        .into_iter()
        .find(|change| change.path == normalized)
        .map(|change| change.diff)
        .ok_or_else(|| format!("no diff found for {path}"))
}

fn load_stored_change(
    state: &State<'_, DbState>,
    run_id: &str,
    path: &str,
) -> Result<(String, FileChange), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let run = db::get_executor_run(&conn, run_id).map_err(db_error)?;
    let result_json = run
        .result
        .ok_or_else(|| "run has no stored result".to_string())?;
    let parsed: StoredExecutorResult = serde_json::from_str(&result_json)
        .map_err(|e| format!("could not parse run result: {e}"))?;

    let normalized = path.trim().replace('\\', "/");
    parsed
        .file_changes
        .into_iter()
        .find(|change| change.path == normalized)
        .map(|change| (result_json, change))
        .ok_or_else(|| format!("no change found for {path}"))
}

#[tauri::command]
pub fn get_executor_run_diff_hunks(
    state: State<'_, DbState>,
    run_id: String,
    path: String,
) -> Result<Vec<DiffHunk>, String> {
    let (_, change) = load_stored_change(&state, &run_id, &path)?;
    Ok(changes::parse_diff_hunks(&change.diff))
}

#[tauri::command]
pub fn reject_executor_hunks(
    state: State<'_, DbState>,
    run_id: String,
    path: String,
    hunk_indices: Vec<usize>,
) -> Result<(), String> {
    let (result_json, change) = load_stored_change(&state, &run_id, &path)?;
    let before = change
        .before_content
        .as_deref()
        .ok_or_else(|| "missing before snapshot".to_string())?;
    let after = change
        .after_content
        .as_deref()
        .ok_or_else(|| "missing after snapshot".to_string())?;

    let next_content =
        changes::apply_hunk_selection(before, after, &change.diff, &hunk_indices)?;

    let settings = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        settings::load(&conn).map_err(db_error)?
    };
    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Err("workspace not configured".into());
    };
    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;
    let resolved = sandbox.resolve(&change.path).map_err(|e| e.to_string())?;
    std::fs::write(&resolved, &next_content).map_err(|e| e.to_string())?;

    let mut parsed: StoredExecutorResult = serde_json::from_str(&result_json)
        .map_err(|e| format!("could not parse run result: {e}"))?;
    for stored in &mut parsed.file_changes {
        if stored.path == change.path {
            stored.after_content = Some(next_content.clone());
            stored.diff = changes::unified_diff_for_strings(&stored.path, before, &next_content);
            break;
        }
    }

    let updated = serde_json::to_string(&parsed).map_err(|e| e.to_string())?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::update_executor_run_result(&conn, &run_id, &updated).map_err(db_error)
}

#[tauri::command]
pub fn open_in_editor(
    state: State<'_, DbState>,
    path: String,
    line: Option<u32>,
) -> Result<(), String> {
    let settings = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        settings::load(&conn).map_err(db_error)?
    };
    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Err("workspace not configured".into());
    };
    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;
    let resolved = sandbox.resolve(&path).map_err(|e| e.to_string())?;
    let path_str = resolved.to_string_lossy().replace('\\', "/");
    let line_num = line.unwrap_or(1).max(1);
    let url = settings
        .editor_open_url
        .replace("{path}", &path_str)
        .replace("{line}", &line_num.to_string());

    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}
