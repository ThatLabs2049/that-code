use rusqlite::{params, Connection, Result as SqlResult};
use uuid::Uuid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExecutorRun {
    pub id: String,
    pub conversation_id: String,
    pub message_id: Option<String>,
    pub task_spec: String,
    pub result: Option<String>,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn insert_executor_run(
    conn: &Connection,
    conversation_id: &str,
    message_id: Option<&str>,
    task_spec: &str,
    status: &str,
) -> SqlResult<ExecutorRun> {
    let id = new_id();
    let created_at = now_rfc3339();

    conn.execute(
        "INSERT INTO executor_runs (id, conversation_id, message_id, task_spec, result, status, created_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, NULL)",
        params![id, conversation_id, message_id, task_spec, status, created_at],
    )?;

    Ok(ExecutorRun {
        id,
        conversation_id: conversation_id.to_string(),
        message_id: message_id.map(str::to_string),
        task_spec: task_spec.to_string(),
        result: None,
        status: status.to_string(),
        created_at,
        completed_at: None,
    })
}

pub fn complete_executor_run(
    conn: &Connection,
    run_id: &str,
    result: &str,
    status: &str,
) -> SqlResult<()> {
    let completed_at = now_rfc3339();
    conn.execute(
        "UPDATE executor_runs SET result = ?1, status = ?2, completed_at = ?3 WHERE id = ?4",
        params![result, status, completed_at, run_id],
    )?;
    Ok(())
}

pub fn update_executor_run_result(conn: &Connection, run_id: &str, result: &str) -> SqlResult<()> {
    conn.execute(
        "UPDATE executor_runs SET result = ?1 WHERE id = ?2",
        params![result, run_id],
    )?;
    Ok(())
}

pub fn get_executor_run(conn: &Connection, run_id: &str) -> SqlResult<ExecutorRun> {
    conn.query_row(
        "SELECT id, conversation_id, message_id, task_spec, result, status, created_at, completed_at
         FROM executor_runs WHERE id = ?1",
        params![run_id],
        |row| {
            Ok(ExecutorRun {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                message_id: row.get(2)?,
                task_spec: row.get(3)?,
                result: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                completed_at: row.get(7)?,
            })
        },
    )
}
