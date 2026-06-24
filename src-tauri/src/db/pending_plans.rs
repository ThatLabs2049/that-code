use rusqlite::{params, Connection, Result as SqlResult};

use crate::agent_plan::PendingPlan;
use crate::agents::profile::AgentTier;
use crate::agents::task::TaskSpec;

pub fn upsert_pending_plan(conn: &Connection, plan: &PendingPlan) -> SqlResult<()> {
    let task_spec_json = serde_json::to_string(&plan.task_spec).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(err.to_string())))
    })?;
    let activity_json = serde_json::to_string(&plan.activity_log).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(err.to_string())))
    })?;
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO pending_plans (conversation_id, task_spec, tier, briefing, activity_log, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(conversation_id) DO UPDATE SET
           task_spec = excluded.task_spec,
           tier = excluded.tier,
           briefing = excluded.briefing,
           activity_log = excluded.activity_log,
           created_at = excluded.created_at",
        params![
            plan.conversation_id,
            task_spec_json,
            plan.tier.as_str(),
            plan.briefing,
            activity_json,
            created_at,
        ],
    )?;
    Ok(())
}

pub fn delete_pending_plan(conn: &Connection, conversation_id: &str) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM pending_plans WHERE conversation_id = ?1",
        params![conversation_id],
    )?;
    Ok(())
}

pub fn list_pending_plans(conn: &Connection) -> SqlResult<Vec<PendingPlan>> {
    let mut stmt = conn.prepare(
        "SELECT conversation_id, task_spec, tier, briefing, activity_log
         FROM pending_plans",
    )?;
    let rows = stmt.query_map([], |row| {
        let task_spec_json: String = row.get(1)?;
        let task_spec: TaskSpec = serde_json::from_str(&task_spec_json).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(err))
        })?;
        let tier_raw: String = row.get(2)?;
        let activity_json: String = row.get(4)?;
        let activity_log = serde_json::from_str(&activity_json).unwrap_or_default();
        Ok(PendingPlan {
            conversation_id: row.get(0)?,
            task_spec,
            tier: AgentTier::parse(&tier_raw),
            briefing: row.get(3)?,
            activity_log,
        })
    })?;
    rows.collect()
}
