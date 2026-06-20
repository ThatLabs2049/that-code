use crate::db::DbState;
use crate::rag;
use crate::settings;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RagIndexResponse {
    pub files_indexed: usize,
    pub files_skipped: usize,
    pub chunks_stored: usize,
}

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

fn load_settings_with_probe(
    state: &State<'_, DbState>,
    probe: Option<settings::UpdateAiSettings>,
) -> Result<settings::AiSettings, String> {
    let mut settings = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        settings::load(&conn).map_err(db_error)?
    };

    if let Some(probe) = probe {
        settings
            .apply_embedding_probe(probe)
            .map_err(|e| e.to_string())?;
    }

    Ok(settings)
}

#[tauri::command]
pub fn get_rag_status(state: State<'_, DbState>) -> Result<rag::RagStatus, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let settings = settings::load(&conn).map_err(db_error)?;
    rag::status(&conn, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn index_workspace_rag(
    state: State<'_, DbState>,
    probe: Option<settings::UpdateAiSettings>,
) -> Result<RagIndexResponse, String> {
    let settings = load_settings_with_probe(&state, probe)?;

    let (chunks, summary) = rag::build_workspace_index(&settings).await?;

    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        rag::persist_workspace_index(&conn, &chunks)?;
    }

    Ok(RagIndexResponse {
        files_indexed: summary.files_indexed,
        files_skipped: summary.files_skipped,
        chunks_stored: summary.chunks_stored,
    })
}

#[tauri::command]
pub async fn index_workspace_changes(
    state: State<'_, DbState>,
    probe: Option<settings::UpdateAiSettings>,
) -> Result<RagIndexResponse, String> {
    let settings = load_settings_with_probe(&state, probe)?;

    let indexed_times = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        rag::indexed_path_times(&conn)?
    };

    let (paths_to_replace, chunks, summary) =
        rag::build_incremental_index(&settings, &indexed_times).await?;

    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        rag::persist_incremental_index(&conn, &paths_to_replace, &chunks)?;
    }

    Ok(RagIndexResponse {
        files_indexed: summary.files_indexed,
        files_skipped: summary.files_skipped,
        chunks_stored: summary.chunks_stored,
    })
}

#[tauri::command]
pub async fn test_embedding_connection(
    state: State<'_, DbState>,
    probe: Option<settings::UpdateAiSettings>,
) -> Result<crate::ai::EmbeddingTestResult, String> {
    let settings = load_settings_with_probe(&state, probe)?;

    rag::test_embedding(&settings)
        .await
        .map_err(|e| e.to_string())
}
