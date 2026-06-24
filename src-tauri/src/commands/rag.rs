use crate::db::DbState;
use crate::rag;
use crate::rag_ann_state::RagAnnState;
use crate::rag_index_state::RagIndexState;
use crate::settings;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

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

fn rebuild_ann_index(ann_state: &RagAnnState, db_state: &DbState) {
    if let Err(err) = ann_state.rebuild_from_db(db_state) {
        eprintln!("RAG ANN rebuild failed: {err}");
    }
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
        if probe.rag_enabled == Some(true) {
            settings.rag_enabled = true;
        }
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
pub fn cancel_rag_index(index_state: State<'_, RagIndexState>) {
    index_state.request_cancel();
}

#[tauri::command]
pub async fn index_workspace_rag(
    app: AppHandle,
    state: State<'_, DbState>,
    index_state: State<'_, RagIndexState>,
    ann_state: State<'_, RagAnnState>,
    probe: Option<settings::UpdateAiSettings>,
) -> Result<RagIndexResponse, String> {
    let settings = load_settings_with_probe(&state, probe)?;
    index_state.begin();

    let app_for_progress = app.clone();
    let progress = Arc::new(move |payload: rag::IndexProgress| {
        let _ = app_for_progress.emit("rag-index-progress", &payload);
    });

    let (chunks, summary) = rag::build_workspace_index_with_progress(
        &settings,
        Some(index_state.cancel_flag()),
        Some(progress),
    )
    .await?;

    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        rag::persist_workspace_index(&conn, &chunks)?;
    }
    rebuild_ann_index(ann_state.inner(), state.inner());

    Ok(RagIndexResponse {
        files_indexed: summary.files_indexed,
        files_skipped: summary.files_skipped,
        chunks_stored: summary.chunks_stored,
    })
}

#[tauri::command]
pub async fn index_workspace_changes(
    state: State<'_, DbState>,
    ann_state: State<'_, RagAnnState>,
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
    rebuild_ann_index(ann_state.inner(), state.inner());

    Ok(RagIndexResponse {
        files_indexed: summary.files_indexed,
        files_skipped: summary.files_skipped,
        chunks_stored: summary.chunks_stored,
    })
}

#[tauri::command]
pub async fn search_codebase(
    state: State<'_, DbState>,
    ann_state: State<'_, RagAnnState>,
    query: String,
) -> Result<Vec<rag::RetrievedChunk>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err("Enter a search query".into());
    }

    let settings = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        settings::load(&conn).map_err(db_error)?
    };

    if !settings.rag_enabled {
        return Err("Enable local RAG in Settings first".into());
    }

    rag::retrieve_chunks_for_query(ann_state.inner(), state.inner(), &settings, trimmed).await
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
