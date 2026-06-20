mod agents;
mod ai;
#[cfg(test)]
mod bench;
mod changes;
mod commands;
mod context;
mod db;
mod mcp;
mod orchestrator;
mod personalities;
mod rag;
mod run_state;
mod settings;
mod tools;
mod watcher;

use commands::{
    cancel_run, clear_completed_tasks, clear_history, create_memory, delete_memory,
    get_active_conversation, get_messages, get_rag_status, get_settings, health_check,
    index_workspace_changes, index_workspace_rag, list_conversations, list_memories,
    list_queued_tasks, queue_task, get_executor_run_changes, revert_executor_run, send_message, test_ai_connection,
    test_embedding_connection, update_memory, update_settings,
};
use db::DbState;
use run_state::RunState;
use std::sync::Mutex;
use tauri::Manager;
use watcher::WatcherState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?
                .join("muse.db");

            let conn = db::open(&db_path).map_err(|e| e.to_string())?;
            app.manage(DbState {
                conn: Mutex::new(conn),
            });
            app.manage(RunState::new());
            app.manage(WatcherState::new());
            watcher::start_workspace_watcher(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health_check,
            list_conversations,
            get_active_conversation,
            get_messages,
            send_message,
            cancel_run,
            clear_history,
            get_settings,
            update_settings,
            test_ai_connection,
            get_rag_status,
            index_workspace_rag,
            index_workspace_changes,
            test_embedding_connection,
            get_executor_run_changes,
            revert_executor_run,
            list_memories,
            create_memory,
            update_memory,
            delete_memory,
            list_queued_tasks,
            queue_task,
            clear_completed_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
