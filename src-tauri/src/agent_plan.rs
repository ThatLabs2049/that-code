use crate::agents::executor::ActivityStep;
use crate::agents::profile::AgentTier;
use crate::agents::task::TaskSpec;
use crate::db::{self, delete_pending_plan, upsert_pending_plan};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPlanView {
    pub briefing: String,
}

#[derive(Debug, Clone)]
pub struct PendingPlan {
    pub conversation_id: String,
    pub task_spec: TaskSpec,
    pub tier: AgentTier,
    pub activity_log: Vec<ActivityStep>,
    pub briefing: String,
}

pub struct PlanState {
    pending: Mutex<HashMap<String, PendingPlan>>,
}

impl PlanState {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

    pub fn hydrate_from_db(&self, conn: &Connection) {
        let Ok(plans) = db::list_pending_plans(conn) else {
            return;
        };
        let Ok(mut guard) = self.pending.lock() else {
            return;
        };
        for plan in plans {
            guard.insert(plan.conversation_id.clone(), plan);
        }
    }

    pub fn store_persisted(
        &self,
        conn: &Connection,
        plan: PendingPlan,
    ) -> Result<(), rusqlite::Error> {
        upsert_pending_plan(conn, &plan)?;
        if let Ok(mut guard) = self.pending.lock() {
            guard.insert(plan.conversation_id.clone(), plan);
        }
        Ok(())
    }

    pub fn take_persisted(
        &self,
        conn: &Connection,
        conversation_id: &str,
    ) -> Option<PendingPlan> {
        let plan = self.take(conversation_id);
        if plan.is_some() {
            let _ = delete_pending_plan(conn, conversation_id);
        }
        plan
    }

    pub fn discard_persisted(&self, conn: &Connection, conversation_id: &str) {
        self.discard(conversation_id);
        let _ = delete_pending_plan(conn, conversation_id);
    }

    pub fn take(&self, conversation_id: &str) -> Option<PendingPlan> {
        self.pending
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(conversation_id))
    }

    pub fn discard(&self, conversation_id: &str) {
        if let Ok(mut guard) = self.pending.lock() {
            guard.remove(conversation_id);
        }
    }

    pub fn has(&self, conversation_id: &str) -> bool {
        self.pending
            .lock()
            .ok()
            .is_some_and(|guard| guard.contains_key(conversation_id))
    }

    pub fn get_briefing(&self, conversation_id: &str) -> Option<String> {
        self.pending
            .lock()
            .ok()
            .and_then(|guard| guard.get(conversation_id).map(|plan| plan.briefing.clone()))
    }
}

impl Default for PlanState {
    fn default() -> Self {
        Self::new()
    }
}
