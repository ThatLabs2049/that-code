use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub id: String,
    pub source_path: String,
    pub chunk_index: i32,
    pub content: String,
    pub embedding: Vec<f32>,
    pub updated_at: String,
}

pub fn clear_chunks(conn: &Connection) -> SqlResult<()> {
    conn.execute("DELETE FROM rag_chunks", [])?;
    Ok(())
}

pub fn insert_chunk(conn: &Connection, chunk: &RagChunk) -> SqlResult<()> {
    let embedding_json = serde_json::to_string(&chunk.embedding).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(err))
    })?;

    conn.execute(
        "INSERT INTO rag_chunks (id, source_path, chunk_index, content, embedding, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            chunk.id,
            chunk.source_path,
            chunk.chunk_index,
            chunk.content,
            embedding_json,
            chunk.updated_at,
        ],
    )?;
    Ok(())
}

pub fn list_embedding_records(conn: &Connection) -> SqlResult<Vec<(String, Vec<f32>)>> {
    let mut stmt = conn.prepare("SELECT id, embedding FROM rag_chunks")?;
    let rows = stmt.query_map([], |row| {
        let embedding_json: String = row.get(1)?;
        let embedding: Vec<f32> = serde_json::from_str(&embedding_json).map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;
        Ok((row.get::<_, String>(0)?, embedding))
    })?;
    rows.collect()
}

pub fn get_chunks_by_ids(conn: &Connection, ids: &[String]) -> SqlResult<Vec<RagChunk>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT id, source_path, chunk_index, content, embedding, updated_at
         FROM rag_chunks
         WHERE id IN ({placeholders})"
    );

    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = ids
        .iter()
        .map(|id| id as &dyn rusqlite::ToSql)
        .collect();
    let rows = stmt.query_map(params.as_slice(), |row| {
        let embedding_json: String = row.get(4)?;
        let embedding: Vec<f32> = serde_json::from_str(&embedding_json).map_err(|err| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(err))
        })?;
        Ok(RagChunk {
            id: row.get(0)?,
            source_path: row.get(1)?,
            chunk_index: row.get(2)?,
            content: row.get(3)?,
            embedding,
            updated_at: row.get(5)?,
        })
    })?;

    let mut by_id: std::collections::HashMap<String, RagChunk> = rows
        .filter_map(Result::ok)
        .map(|chunk| (chunk.id.clone(), chunk))
        .collect();

    Ok(ids
        .iter()
        .filter_map(|id| by_id.remove(id))
        .collect())
}

pub fn chunk_count(conn: &Connection) -> SqlResult<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM rag_chunks", [], |row| row.get(0))?;
    Ok(count as usize)
}

pub fn delete_chunks_for_path(conn: &Connection, source_path: &str) -> SqlResult<()> {
    conn.execute("DELETE FROM rag_chunks WHERE source_path = ?1", params![source_path])?;
    Ok(())
}

pub fn list_source_paths(conn: &Connection) -> SqlResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT source_path FROM rag_chunks")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

pub fn latest_path_index_time(conn: &Connection, source_path: &str) -> SqlResult<Option<String>> {
    conn.query_row(
        "SELECT MAX(updated_at) FROM rag_chunks WHERE source_path = ?1",
        params![source_path],
        |row| row.get(0),
    )
    .optional()
    .map(|opt| opt.flatten())
}

pub fn latest_index_time(conn: &Connection) -> SqlResult<Option<String>> {
    conn.query_row("SELECT MAX(updated_at) FROM rag_chunks", [], |row| row.get(0))
        .optional()
        .map(|opt| opt.flatten())
}
