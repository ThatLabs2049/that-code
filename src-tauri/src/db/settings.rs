use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};

pub fn get_setting(conn: &Connection, key: &str) -> SqlResult<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod settings_tests {
    use super::super::{migrate_in_memory, get_setting, set_setting};

    #[test]
    fn setting_roundtrip() {
        let conn = migrate_in_memory();
        assert!(get_setting(&conn, "ai_settings").unwrap().is_none());

        set_setting(&conn, "ai_settings", r#"{"base_url":"https://example.com"}"#).unwrap();
        let value = get_setting(&conn, "ai_settings").unwrap().unwrap();
        assert!(value.contains("example.com"));
    }
}
