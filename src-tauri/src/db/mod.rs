mod executor_runs;
mod memories;
mod migrations;
mod models;
mod settings;
mod task_queue;

pub use executor_runs::{complete_executor_run, get_executor_run, insert_executor_run};
pub use memories::{create_memory, delete_memory, list_memories, top_memories_for_context, update_memory, Memory};
pub use models::{Conversation, Message};
pub use settings::{get_setting, set_setting};
pub use task_queue::{
    clear_completed, complete_queued_task, enqueue_task, fail_running_tasks, list_pending_tasks,
    pop_next_pending, QueuedTask,
};

use migrations::{MIGRATION_V1, MIGRATION_V2, MIGRATION_V3, SCHEMA_VERSION};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

pub const LUNA_GREETING_EN: &str = "Hey — I'm Luna. I'm glad you're here.\n\nTell me what's on your mind, or ask for help with something you're working toward. I'm listening.";
pub const LUNA_GREETING_FA: &str = "سلام — من لونا هستم. خوشحالم که اینجایی.\n\nهر چیزی که ذهنتو مشغول کرده بگو، من اینجام یا اگه میخوای توی پروژه هات کمکت کنم.";
pub const DEFAULT_CONVERSATION_TITLE: &str = "Chat with Luna";

pub fn greeting_for_settings(settings: &crate::settings::AiSettings) -> &'static str {
    crate::personalities::greeting_for(&settings.personality_id, &settings.ui_locale)
}

fn is_seed_greeting(content: &str) -> bool {
    crate::personalities::PERSONALITIES
        .iter()
        .any(|p| content == p.greeting_en || content == p.greeting_fa)
}

/// If the conversation only contains Luna's seed greeting, update it to match the current UI locale.
pub fn refresh_seed_greeting_if_pristine(conn: &Connection) -> SqlResult<()> {
    let settings = crate::settings::load(conn)?;
    let greeting = greeting_for_settings(&settings);

    let Some(conversation) = list_conversations(conn)?.into_iter().next() else {
        return Ok(());
    };

    let mut stmt = conn.prepare(
        "SELECT id, role, content FROM messages
         WHERE conversation_id = ?1
         ORDER BY created_at ASC",
    )?;

    let rows: Vec<(String, String, String)> = stmt
        .query_map(params![conversation.id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<SqlResult<Vec<_>>>()?;

    if rows.len() != 1 {
        return Ok(());
    }

    let (message_id, role, content) = &rows[0];
    if role != "companion" || !is_seed_greeting(content) || content == greeting {
        return Ok(());
    }

    conn.execute(
        "UPDATE messages SET content = ?1 WHERE id = ?2",
        params![greeting, message_id],
    )?;

    Ok(())
}

pub struct DbState {
    pub conn: Mutex<Connection>,
}

pub fn open(path: &Path) -> SqlResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;
    }

    let conn = Connection::open(path)?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(MIGRATION_V1)?;

    let version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0);

    if version == 0 {
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            params![1],
        )?;
    }

    if version < 2 {
        conn.execute_batch(MIGRATION_V2)?;
        conn.execute("UPDATE schema_version SET version = ?1", params![2])?;
    }

    if version < 3 {
        conn.execute_batch(MIGRATION_V3)?;
        conn.execute("UPDATE schema_version SET version = ?1", params![SCHEMA_VERSION])?;
    }

    Ok(())
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn list_conversations(conn: &Connection) -> SqlResult<Vec<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at
         FROM conversations
         ORDER BY updated_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;

    rows.collect()
}

pub fn get_or_create_active_conversation(conn: &Connection) -> SqlResult<Conversation> {
    if let Some(conversation) = list_conversations(conn)?.into_iter().next() {
        return Ok(conversation);
    }

    create_conversation(conn, Some(DEFAULT_CONVERSATION_TITLE))
}

pub fn create_conversation(conn: &Connection, title: Option<&str>) -> SqlResult<Conversation> {
    let id = new_id();
    let timestamp = now_rfc3339();

    conn.execute(
        "INSERT INTO conversations (id, title, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, title, timestamp, timestamp],
    )?;

    Ok(Conversation {
        id,
        title: title.map(str::to_string),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    })
}

pub fn get_messages(conn: &Connection, conversation_id: &str) -> SqlResult<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, created_at
         FROM messages
         WHERE conversation_id = ?1
         ORDER BY created_at ASC",
    )?;

    let messages = stmt
        .query_map(params![conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

    if messages.is_empty() {
        let settings = crate::settings::load(conn)?;
        let greeting = greeting_for_settings(&settings);
        let message = insert_message(conn, conversation_id, "companion", greeting)?;
        return Ok(vec![message]);
    }

    Ok(messages)
}

pub fn insert_message(
    conn: &Connection,
    conversation_id: &str,
    role: &str,
    content: &str,
) -> SqlResult<Message> {
    let id = new_id();
    let created_at = now_rfc3339();

    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, conversation_id, role, content, created_at],
    )?;

    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![created_at, conversation_id],
    )?;

    Ok(Message {
        id,
        conversation_id: conversation_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        created_at,
    })
}

pub fn send_user_message(
    conn: &Connection,
    conversation_id: &str,
    content: &str,
) -> SqlResult<Message> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName(
            "message content cannot be empty".into(),
        ));
    }

    conversation_exists(conn, conversation_id)?;
    insert_message(conn, conversation_id, "user", trimmed)
}

fn conversation_exists(conn: &Connection, conversation_id: &str) -> SqlResult<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversations WHERE id = ?1",
        params![conversation_id],
        |row| row.get(0),
    )?;

    if exists == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

pub fn clear_conversation_messages(conn: &Connection, conversation_id: &str) -> SqlResult<Message> {
    conversation_exists(conn, conversation_id)?;

    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
    )?;

    let timestamp = now_rfc3339();
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![timestamp, conversation_id],
    )?;

    let settings = crate::settings::load(conn)?;
    let greeting = greeting_for_settings(&settings);
    insert_message(conn, conversation_id, "companion", greeting)
}

#[cfg(test)]
pub(crate) fn migrate_in_memory() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
    migrate(&conn).unwrap();
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_db() -> Connection {
        migrate_in_memory()
    }

    #[test]
    fn creates_default_conversation_and_seeds_greeting() {
        let conn = in_memory_db();
        let conversation = get_or_create_active_conversation(&conn).unwrap();
        let messages = get_messages(&conn, &conversation.id).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "companion");
        assert_eq!(messages[0].content, LUNA_GREETING_EN);
    }

    #[test]
    fn seeds_persian_greeting_when_locale_is_fa() {
        let conn = in_memory_db();
        let mut settings = crate::settings::load(&conn).unwrap();
        settings.ui_locale = "fa".into();
        crate::settings::save(&conn, &settings).unwrap();

        let conversation = get_or_create_active_conversation(&conn).unwrap();
        let messages = get_messages(&conn, &conversation.id).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, LUNA_GREETING_FA);
    }

    #[test]
    fn refreshes_pristine_greeting_on_locale_change() {
        let conn = in_memory_db();
        let conversation = get_or_create_active_conversation(&conn).unwrap();
        get_messages(&conn, &conversation.id).unwrap();

        let mut settings = crate::settings::load(&conn).unwrap();
        settings.ui_locale = "fa".into();
        crate::settings::save(&conn, &settings).unwrap();
        refresh_seed_greeting_if_pristine(&conn).unwrap();

        let messages = get_messages(&conn, &conversation.id).unwrap();
        assert_eq!(messages[0].content, LUNA_GREETING_FA);
    }

    #[test]
    fn send_user_message_persists() {
        let conn = in_memory_db();
        let conversation = get_or_create_active_conversation(&conn).unwrap();
        get_messages(&conn, &conversation.id).unwrap();

        let message = send_user_message(&conn, &conversation.id, "Hello Luna").unwrap();
        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello Luna");

        let messages = get_messages(&conn, &conversation.id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].content, "Hello Luna");
    }
}
