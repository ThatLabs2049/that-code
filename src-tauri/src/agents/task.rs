use crate::db::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskSpec {
    pub objective: String,
    pub context: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub expected_output: String,
}

impl TaskSpec {
    pub fn validate(&self) -> Result<(), String> {
        if self.objective.trim().is_empty() {
            return Err("objective cannot be empty".into());
        }
        if self.context.trim().is_empty() {
            return Err("context cannot be empty".into());
        }
        if self.expected_output.trim().is_empty() {
            return Err("expected_output cannot be empty".into());
        }
        Ok(())
    }
}

pub fn last_user_message(history: &[Message]) -> Option<&str> {
    history
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
}

pub fn is_file_or_code_task(user_message: &str) -> bool {
    let lower = user_message.to_lowercase();
    const KEYWORDS: &[&str] = &[
        "code",
        "implement",
        "write a ",
        "write the ",
        "write me ",
        "create ",
        "create a ",
        "create file",
        "add a file",
        "new file",
        "refactor",
        "fix",
        "bug",
        "function",
        "class ",
        "script",
        "component",
        "module",
        "readme",
        "in src",
        "in the src",
        "src folder",
        "src/",
        "in my project",
        "my project",
        "codebase",
        "repository",
        "repo",
        "workspace",
        "scaffold",
        "build",
        "make ",
        "edit ",
        "update ",
        "change ",
        "patch",
        "snippet",
        "typescript",
        "javascript",
        "python",
        "rust",
        "react",
        "test",
        "cargo",
        "npm",
        "go ahead",
        "just do",
        "can you",
        "please",
        "help me with",
        "look at",
        "check ",
    ];
    KEYWORDS.iter().any(|kw| lower.contains(kw))
}

pub fn is_casual_chat_only(user_message: &str) -> bool {
    let lower = user_message.to_lowercase();
    const CASUAL: &[&str] = &[
        "how are you",
        "hello",
        "hi ",
        "hey ",
        "thanks",
        "thank you",
        "good morning",
        "good night",
        "bye",
    ];
    CASUAL.iter().any(|k| lower.contains(k)) && !is_file_or_code_task(user_message)
}

pub fn build_task_from_message(user_message: &str, history: &[Message]) -> TaskSpec {
    let context = recent_context(history, 8);
    let file_task = is_file_or_code_task(user_message) || !user_message.trim().is_empty();

    TaskSpec {
        objective: user_message.trim().to_string(),
        context,
        constraints: vec![
            "Work autonomously — explore briefly (≤3 tools), then edit or write".into(),
            "Never read_file or grep the same path/pattern twice".into(),
            "Use edit_file for changes when possible; run_command to verify".into(),
            if file_task {
                "Use list_dir, grep, read_file, edit_file, write_file, run_command as needed"
                    .into()
            } else {
                "Produce structured actionable output".into()
            },
            "Stay inside the workspace sandbox".into(),
        ],
        expected_output:
            "Complete the request using tools; summarize files changed and results in final_answer"
                .into(),
    }
}

fn recent_context(history: &[Message], limit: usize) -> String {
    history
        .iter()
        .rev()
        .take(limit)
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_task_spec() {
        let spec = TaskSpec {
            objective: "".into(),
            context: "ctx".into(),
            constraints: vec![],
            expected_output: "out".into(),
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn detects_code_tasks() {
        assert!(is_file_or_code_task("Write a function to parse JSON"));
        assert!(is_file_or_code_task("What's in the src folder?"));
    }

    #[test]
    fn casual_chat_detected() {
        assert!(is_casual_chat_only("how are you today?"));
        assert!(!is_casual_chat_only("fix the bug in src/main.rs"));
    }
}
