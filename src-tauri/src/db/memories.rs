use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn list_memories(conn: &Connection) -> SqlResult<Vec<Memory>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, created_at, updated_at FROM memories ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Memory {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn create_memory(conn: &Connection, content: &str) -> SqlResult<Memory> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName(
            "content cannot be empty".into(),
        ));
    }
    let now = now_rfc3339();
    let memory = Memory {
        id: new_id(),
        content: trimmed.to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO memories (id, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![memory.id, memory.content, memory.created_at, memory.updated_at],
    )?;
    Ok(memory)
}

pub fn update_memory(conn: &Connection, id: &str, content: &str) -> SqlResult<Memory> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName(
            "content cannot be empty".into(),
        ));
    }
    let updated_at = now_rfc3339();
    let changed = conn.execute(
        "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
        params![trimmed, updated_at, id],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    conn.query_row(
        "SELECT id, content, created_at, updated_at FROM memories WHERE id = ?1",
        params![id],
        |row| {
            Ok(Memory {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    )
}

pub fn delete_memory(conn: &Connection, id: &str) -> SqlResult<()> {
    let changed = conn.execute("DELETE FROM memories WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

pub fn top_memories_for_context(conn: &Connection, limit: usize) -> SqlResult<Vec<Memory>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, created_at, updated_at FROM memories ORDER BY updated_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok(Memory {
            id: row.get(0)?,
            content: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;
    rows.collect()
}
