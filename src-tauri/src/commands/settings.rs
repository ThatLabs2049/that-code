use crate::db::{self, DbState};
use crate::settings::{self, AiSettingsView, UpdateAiSettings};
use crate::ai::{self, AiTestResult};
use tauri::State;

fn db_error(err: rusqlite::Error) -> String {
    err.to_string()
}

#[tauri::command]
pub fn get_settings(state: State<'_, DbState>) -> Result<AiSettingsView, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let settings = settings::load(&conn).map_err(db_error)?;
    Ok(settings.to_view())
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, DbState>,
    update: UpdateAiSettings,
) -> Result<AiSettingsView, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut settings = settings::load(&conn).map_err(db_error)?;
    let locale_changed = update.ui_locale.is_some();
    settings.apply_update(update)?;
    settings::save(&conn, &settings).map_err(db_error)?;

    if locale_changed {
        db::refresh_seed_greeting_if_pristine(&conn).map_err(db_error)?;
    }

    Ok(settings.to_view())
}

#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, DbState>,
    probe: Option<UpdateAiSettings>,
) -> Result<AiTestResult, String> {
    let mut settings = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        settings::load(&conn).map_err(db_error)?
    };

    if let Some(probe) = probe {
        settings
            .apply_connection_probe(probe)
            .map_err(|e| e.to_string())?;
    }

    ai::test_connection(&settings)
        .await
        .map_err(|err| err.user_message())
}
