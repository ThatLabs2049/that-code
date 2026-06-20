mod cancel;
mod changes;
mod chat;
mod memory;
mod rag;
mod settings;
mod task_queue;

use serde::Serialize;

pub use changes::{get_executor_run_changes, revert_executor_run};
pub use cancel::cancel_run;
pub use chat::{clear_history, get_active_conversation, get_messages, list_conversations, send_message};
pub use memory::{create_memory, delete_memory, list_memories, update_memory};
pub use rag::{
    get_rag_status, index_workspace_changes, index_workspace_rag, test_embedding_connection,
};
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
        app: "Muse".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    }
}
