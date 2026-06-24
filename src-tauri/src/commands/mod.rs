mod agent;
mod cancel;
mod changes;
mod chat;
mod memory;
mod workspace;
mod rag;
mod settings;
mod task_queue;

use serde::Serialize;

pub use agent::{get_pending_plan, respond_to_agent_plan};
pub use changes::{
    get_executor_run_changes, get_executor_run_diff_hunks, get_executor_run_file_diff,
    open_in_editor, reject_executor_hunks, revert_executor_run,
};
pub use cancel::cancel_run;
pub use chat::{clear_history, get_active_conversation, get_messages, list_conversations, send_message};
pub use memory::{create_memory, delete_memory, list_memories, update_memory};
pub use rag::{
    cancel_rag_index, get_rag_status, index_workspace_changes, index_workspace_rag, search_codebase,
    test_embedding_connection,
};
pub use workspace::{get_workspace_git_status, search_workspace_paths, search_workspace_symbols};
pub use settings::{get_settings, test_ai_connection, update_settings};
pub use task_queue::{clear_completed_tasks, list_queued_tasks, queue_task};
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub app: String,
    pub version: String,
}

#[tauri::command]
pub fn health_check() -> HealthStatus {
    HealthStatus {
        status: "ok".into(),
        app: "ThatCode".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    }
}
