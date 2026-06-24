use crate::db::DbState;
use crate::rag::{self, RagAnnIndex};
use rusqlite::Connection;
use std::sync::Mutex;

pub struct RagAnnState {
    index: Mutex<Option<RagAnnIndex>>,
}

impl RagAnnState {
    pub fn new() -> Self {
        Self {
            index: Mutex::new(None),
        }
    }

    pub fn rebuild_from_conn(&self, conn: &Connection) -> Result<usize, String> {
        let records = rag::list_embedding_records(conn).map_err(|err| err.to_string())?;
        if records.is_empty() {
            if let Ok(mut guard) = self.index.lock() {
                *guard = None;
            }
            return Ok(0);
        }

        let index = RagAnnIndex::build(records)?;
        let len = index.len();
        if let Ok(mut guard) = self.index.lock() {
            *guard = Some(index);
        }
        Ok(len)
    }

    pub fn rebuild_from_db(&self, db_state: &DbState) -> Result<usize, String> {
        let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
        self.rebuild_from_conn(&conn)
    }

    pub fn search(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        let Ok(guard) = self.index.lock() else {
            return Vec::new();
        };
        guard
            .as_ref()
            .map(|index| index.search(query, k))
            .unwrap_or_default()
    }

    pub fn is_ready(&self) -> bool {
        self.index
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(RagAnnIndex::len))
            .unwrap_or(0)
            > 0
    }
}

impl Default for RagAnnState {
    fn default() -> Self {
        Self::new()
    }
}
