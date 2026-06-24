use crate::db::DbState;

use crate::settings;

use crate::workspace;

use std::path::Path;

use tauri::State;



#[tauri::command]

pub fn search_workspace_paths(

    state: State<'_, DbState>,

    query: String,

) -> Result<Vec<workspace::WorkspacePathHit>, String> {

    let settings = {

        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        settings::load(&conn).map_err(|e| e.to_string())?

    };



    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {

        return Err("Pick a workspace folder first".into());

    };



    workspace::search_workspace_paths(Path::new(workspace), &query)

}



#[tauri::command]

pub fn search_workspace_symbols(

    state: State<'_, DbState>,

    query: String,

) -> Result<Vec<workspace::WorkspaceSymbolHit>, String> {

    let settings = {

        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        settings::load(&conn).map_err(|e| e.to_string())?

    };



    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {

        return Err("Pick a workspace folder first".into());

    };



    workspace::search_workspace_symbols(Path::new(workspace), &query)

}



#[tauri::command]

pub fn get_workspace_git_status(

    state: State<'_, DbState>,

) -> Result<workspace::WorkspaceGitStatus, String> {

    let settings = {

        let conn = state.conn.lock().map_err(|e| e.to_string())?;

        settings::load(&conn).map_err(|e| e.to_string())?

    };



    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {

        return Ok(workspace::WorkspaceGitStatus::default());

    };



    Ok(workspace::get_workspace_git_status(Path::new(workspace)))

}

