use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agents::companion::TaskSpec;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueuedTask {
    pub id: String,
    pub conversation_id: String,
    pub task_spec: TaskSpec,
    pub status: String,
    pub position: i32,
    pub created_at: String,
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn enqueue_task(
    conn: &Connection,
    conversation_id: &str,
    task_spec: &TaskSpec,
) -> SqlResult<QueuedTask> {
    let position: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM task_queue WHERE conversation_id = ?1 AND status = 'pending'",
        params![conversation_id],
        |row| row.get(0),
    )?;
    let task_json = serde_json::to_string(task_spec).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(err.to_string())))
    })?;
    let queued = QueuedTask {
        id: new_id(),
        conversation_id: conversation_id.to_string(),
        task_spec: task_spec.clone(),
        status: "pending".into(),
        position,
        created_at: now_rfc3339(),
    };
    conn.execute(
        "INSERT INTO task_queue (id, conversation_id, task_spec, status, position, created_at)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
        params![
            queued.id,
            queued.conversation_id,
            task_json,
            queued.position,
            queued.created_at
        ],
    )?;
    Ok(queued)
}

pub fn list_pending_tasks(conn: &Connection, conversation_id: &str) -> SqlResult<Vec<QueuedTask>> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, task_spec, status, position, created_at
         FROM task_queue
         WHERE conversation_id = ?1 AND status = 'pending'
         ORDER BY position ASC",
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| {
        let task_json: String = row.get(2)?;
        let task_spec: TaskSpec = serde_json::from_str(&task_json).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(err))
        })?;
        Ok(QueuedTask {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            task_spec,
            status: row.get(3)?,
            position: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn pop_next_pending(conn: &Connection, conversation_id: &str) -> SqlResult<Option<QueuedTask>> {
    let mut tasks = list_pending_tasks(conn, conversation_id)?;
    let Some(next) = tasks.drain(..1).next() else {
        return Ok(None);
    };
    conn.execute(
        "UPDATE task_queue SET status = 'running' WHERE id = ?1",
        params![next.id],
    )?;
    Ok(Some(next))
}

pub fn complete_queued_task(conn: &Connection, id: &str, status: &str) -> SqlResult<()> {
    conn.execute(
        "UPDATE task_queue SET status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

pub fn fail_running_tasks(conn: &Connection, conversation_id: &str) -> SqlResult<()> {
    conn.execute(
        "UPDATE task_queue SET status = 'error' WHERE conversation_id = ?1 AND status = 'running'",
        params![conversation_id],
    )?;
    Ok(())
}

pub fn clear_completed(conn: &Connection, conversation_id: &str) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM task_queue WHERE conversation_id = ?1 AND status IN ('done', 'error')",
        params![conversation_id],
    )?;
    Ok(())
}
