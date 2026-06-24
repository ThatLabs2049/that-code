pub const SCHEMA_VERSION: i32 = 4;

pub const MIGRATION_V1: &str = r"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS conversations (
    id          TEXT PRIMARY KEY,
    title       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id               TEXT PRIMARY KEY,
    conversation_id  TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role             TEXT NOT NULL CHECK (role IN ('user', 'companion')),
    content          TEXT NOT NULL,
    created_at       TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation_id
    ON messages (conversation_id, created_at);

CREATE TABLE IF NOT EXISTS executor_runs (
    id               TEXT PRIMARY KEY,
    conversation_id  TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    message_id       TEXT REFERENCES messages(id) ON DELETE SET NULL,
    task_spec        TEXT NOT NULL,
    result           TEXT,
    status           TEXT NOT NULL CHECK (status IN ('pending', 'running', 'done', 'error')),
    created_at       TEXT NOT NULL,
    completed_at     TEXT
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

pub const MIGRATION_V2: &str = r"
CREATE TABLE IF NOT EXISTS rag_chunks (
    id           TEXT PRIMARY KEY,
    source_path  TEXT NOT NULL,
    chunk_index  INTEGER NOT NULL,
    content      TEXT NOT NULL,
    embedding    TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rag_source ON rag_chunks(source_path);
";

pub const MIGRATION_V3: &str = r"
CREATE TABLE IF NOT EXISTS memories (
    id          TEXT PRIMARY KEY,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_queue (
    id               TEXT PRIMARY KEY,
    conversation_id  TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    task_spec        TEXT NOT NULL,
    status           TEXT NOT NULL CHECK (status IN ('pending', 'running', 'done', 'error')),
    position         INTEGER NOT NULL,
    created_at       TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_task_queue_conversation
    ON task_queue (conversation_id, status, position);
";

pub const MIGRATION_V4: &str = r"
CREATE TABLE IF NOT EXISTS pending_plans (
    conversation_id  TEXT PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    task_spec        TEXT NOT NULL,
    tier             TEXT NOT NULL,
    briefing         TEXT NOT NULL,
    activity_log     TEXT NOT NULL DEFAULT '[]',
    created_at       TEXT NOT NULL
);
";
