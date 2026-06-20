use crate::agents::companion::TaskSpec;
use crate::db::Message;
use crate::settings::AiSettings;

/// User message implies file/code/project work (needs workspace tools).
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
        "کد",
        "فایل",
        "پوشه",
        "پروژه",
        "بنویس",
        "پیاده",
        "اصلاح",
        "بساز",
        "برنامه",
        "درست کن",
        "رفع",
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
        "feel sad",
        "feel happy",
        "love you",
        "miss you",
        "nice to meet",
        "سلام",
        "چطوری",
        "ممنون",
        "مرسی",
        "حالم",
        "احساس",
    ];
    CASUAL.iter().any(|k| lower.contains(k)) && !is_file_or_code_task(user_message)
}

/// User message should go to the executor (planning, research, or file/code work).
pub fn is_executor_task(user_message: &str) -> bool {
    if is_file_or_code_task(user_message) {
        return true;
    }

    if is_casual_chat_only(user_message) {
        return false;
    }

    let lower = user_message.to_lowercase();
    const KEYWORDS: &[&str] = &[
        "plan",
        "schedule",
        "analyze",
        "analysis",
        "research",
        "summarize",
        "break down",
        "actionable",
        "برنامه",
        "تحلیل",
        "خلاصه",
    ];
    KEYWORDS.iter().any(|kw| lower.contains(kw))
}

pub fn response_contains_implementation(message: &str) -> bool {
    if message.contains("```") {
        return true;
    }

    let lower = message.to_lowercase();
    if lower.contains("fn main")
        || lower.contains("function ")
        || lower.contains("import ")
        || lower.contains("export ")
        || lower.contains("const ")
        || lower.contains("class ")
    {
        return message.lines().count() >= 3;
    }

    false
}

pub fn should_force_delegate(
    user_message: &str,
    direct_reply: Option<&str>,
    settings: &AiSettings,
) -> bool {
    if is_casual_chat_only(user_message) {
        return false;
    }

    if settings.workspace_configured()
        && (is_file_or_code_task(user_message) || user_message.trim().len() > 12)
    {
        return true;
    }

    if is_file_or_code_task(user_message) {
        return settings.workspace_configured();
    }

    if !is_executor_task(user_message) {
        return false;
    }

    if let Some(reply) = direct_reply {
        response_contains_implementation(reply)
    } else {
        true
    }
}

pub fn auto_holding_message(ui_locale: &str) -> String {
    match ui_locale {
        "fa" => "دارم روش کار می‌کنم — چند لحظه.".into(),
        _ => "On it — working in your project now.".into(),
    }
}

pub fn auto_task_spec(user_message: &str, history: &[Message]) -> TaskSpec {
    let context = recent_context(history, 8);
    let file_task = is_file_or_code_task(user_message) || !user_message.trim().is_empty();

    TaskSpec {
        objective: user_message.trim().to_string(),
        context,
        constraints: vec![
            "Work autonomously — explore with tools, do not ask the user questions".into(),
            "Use edit_file for changes when possible; run_command to verify".into(),
            if file_task {
                "Use list_dir, grep, read_file, edit_file, write_file, run_command as needed".into()
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

pub fn last_user_message(history: &[Message]) -> Option<&str> {
    history
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_code_tasks() {
        assert!(is_file_or_code_task("Write a function to parse JSON"));
        assert!(is_file_or_code_task("What's in the src folder?"));
    }

    #[test]
    fn casual_chat_stays_direct() {
        let settings = AiSettings {
            workspace_path: Some("/tmp/project".into()),
            ..AiSettings::default()
        };
        assert!(!should_force_delegate("how are you today?", None, &settings));
    }

    #[test]
    fn forces_delegate_when_workspace_set() {
        let settings = AiSettings {
            workspace_path: Some("/tmp/project".into()),
            ..AiSettings::default()
        };
        assert!(should_force_delegate("implement login form", None, &settings));
        assert!(should_force_delegate("go ahead and fix it", None, &settings));
    }

    #[test]
    fn detects_code_in_direct_reply() {
        assert!(response_contains_implementation(
            "Here:\n```js\nfunction hi(){}\n```"
        ));
    }
}
