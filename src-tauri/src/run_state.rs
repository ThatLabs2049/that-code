use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct RunState {
    cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl RunState {
    pub fn new() -> Self {
        Self {
            cancellations: Mutex::new(HashMap::new()),
        }
    }

    pub fn try_register(&self, conversation_id: &str) -> Result<Arc<AtomicBool>, String> {
        let mut guard = self
            .cancellations
            .lock()
            .map_err(|e| e.to_string())?;
        if guard.contains_key(conversation_id) {
            return Err(
                "A run is already in progress for this conversation. Wait for it to finish or cancel it."
                    .into(),
            );
        }
        let flag = Arc::new(AtomicBool::new(false));
        guard.insert(conversation_id.to_string(), flag.clone());
        Ok(flag)
    }

    pub fn cancel(&self, conversation_id: &str) -> bool {
        let Ok(guard) = self.cancellations.lock() else {
            return false;
        };
        if let Some(flag) = guard.get(conversation_id) {
            flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn clear(&self, conversation_id: &str) {
        if let Ok(mut guard) = self.cancellations.lock() {
            guard.remove(conversation_id);
        }
    }

    pub fn is_cancelled(flag: &AtomicBool) -> bool {
        flag.load(Ordering::SeqCst)
    }
}

impl Default for RunState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_concurrent_registration_for_same_conversation() {
        let state = RunState::new();
        let first = state.try_register("conv-1").expect("first register");
        assert!(state.try_register("conv-1").is_err());
        assert!(!RunState::is_cancelled(&first));
        state.clear("conv-1");
        assert!(state.try_register("conv-1").is_ok());
    }
}
