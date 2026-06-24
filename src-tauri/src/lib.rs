mod agents;
mod agent_plan;
mod ai;
#[cfg(test)]
mod bench;
mod changes;
mod commands;
mod context;
mod db;
mod mcp;
mod orchestrator;
mod rag;
mod rag_ann_state;
mod rag_index_state;
mod run_state;
mod settings;
mod tools;
mod watcher;
mod workspace;

use agent_plan::PlanState;
use commands::{
    cancel_rag_index, cancel_run, clear_completed_tasks, clear_history, create_memory, delete_memory,
    get_active_conversation, get_executor_run_changes, get_executor_run_diff_hunks,
    get_executor_run_file_diff, get_messages, get_rag_status, get_settings, get_workspace_git_status,
    health_check,
    index_workspace_changes, index_workspace_rag, list_conversations, list_memories,
    list_queued_tasks, open_in_editor, queue_task, reject_executor_hunks, respond_to_agent_plan,
    get_pending_plan, revert_executor_run, search_codebase, search_workspace_paths, search_workspace_symbols,
    send_message, test_ai_connection,
    test_embedding_connection, update_memory, update_settings,
};
use db::DbState;
use rag_ann_state::RagAnnState;
use rag_index_state::RagIndexState;
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
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(tauri::include_image!("icons/32x32.png"));
            }

            let db_path = app
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?
                .join("muse.db");

            let conn = db::open(&db_path).map_err(|e| e.to_string())?;
            let plan_state = PlanState::new();
            plan_state.hydrate_from_db(&conn);
            let ann_state = RagAnnState::new();
            let _ = ann_state.rebuild_from_conn(&conn);
            app.manage(DbState {
                conn: Mutex::new(conn),
            });
            app.manage(RunState::new());
            app.manage(RagIndexState::new());
            app.manage(ann_state);
            app.manage(plan_state);
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
            cancel_rag_index,
            index_workspace_changes,
            test_embedding_connection,
            search_codebase,
            search_workspace_paths,
            search_workspace_symbols,
            get_workspace_git_status,
            get_executor_run_changes,
            get_executor_run_file_diff,
            revert_executor_run,
            list_memories,
            create_memory,
            update_memory,
            delete_memory,
            list_queued_tasks,
            queue_task,
            clear_completed_tasks,
            respond_to_agent_plan,
            get_pending_plan,
            get_executor_run_diff_hunks,
            reject_executor_hunks,
            open_in_editor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
