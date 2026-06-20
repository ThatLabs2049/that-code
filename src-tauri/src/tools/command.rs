use std::process::Command;

use super::handlers::ToolResult;
use super::sandbox::WorkspaceSandbox;

const MAX_OUTPUT_BYTES: usize = 32 * 1024;

const DEFAULT_PREFIXES: &[&str] = &[
    "npm test",
    "npm run ",
    "pnpm test",
    "pnpm run ",
    "yarn test",
    "yarn run ",
    "cargo test",
    "cargo check",
    "cargo clippy",
    "cargo build",
    "git status",
    "git diff",
    "git log",
    "git add",
    "git commit",
    "git checkout -b",
    "git checkout ",
    "python -m pytest",
    "pytest",
    "go test",
];

pub fn run_command(
    sandbox: &WorkspaceSandbox,
    command: &str,
    extra_prefixes: &[String],
) -> ToolResult {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return tool_error("command cannot be empty");
    }

    if !is_allowed(trimmed, extra_prefixes) {
        return tool_error(format!(
            "command not allowed. Permitted prefixes include: {}",
            allowed_prefixes(extra_prefixes).join(", ")
        ));
    }

    let argv = match parse_command_argv(trimmed) {
        Ok(argv) => argv,
        Err(message) => return tool_error(message),
    };

    run_argv(sandbox, &argv)
}

fn validate_git_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path == "." {
        return Ok(());
    }
    if path
        .chars()
        .any(|c| matches!(c, ';' | '&' | '|' | '$' | '`' | '"' | '\'' | '\n' | '\r'))
    {
        return Err("path contains invalid characters".into());
    }
    Ok(())
}

pub fn git_add(sandbox: &WorkspaceSandbox, path: &str, extra_prefixes: &[String]) -> ToolResult {
    let path = path.trim();

    if let Err(message) = validate_git_path(path) {
        return tool_error(message);
    }

    if !(path.is_empty() || path == ".") {
        if let Err(err) = sandbox.resolve(path) {
            return tool_error(format!("invalid path: {err}"));
        }
    }

    if !is_allowed("git add", extra_prefixes) {
        return tool_error("git add is not allowed");
    }

    if path.is_empty() || path == "." {
        run_argv(sandbox, &["git".into(), "add".into(), ".".into()])
    } else {
        run_argv(
            sandbox,
            &["git".into(), "add".into(), path.to_string()],
        )
    }
}

pub fn git_commit(
    sandbox: &WorkspaceSandbox,
    message: &str,
    extra_prefixes: &[String],
) -> ToolResult {
    let message = message.trim();

    if message.is_empty() {
        return tool_error("commit message cannot be empty");
    }

    if !is_allowed("git commit", extra_prefixes) {
        return tool_error("git commit is not allowed");
    }

    run_argv(
        sandbox,
        &[
            "git".into(),
            "commit".into(),
            "-m".into(),
            message.to_string(),
        ],
    )
}

pub fn git_checkout_branch(
    sandbox: &WorkspaceSandbox,
    branch: &str,
    extra_prefixes: &[String],
) -> ToolResult {
    let branch = branch.trim();

    if branch.is_empty() {
        return tool_error("branch name cannot be empty");
    }

    if branch.contains(char::is_whitespace) {
        return tool_error("branch name cannot contain whitespace");
    }

    if branch.starts_with('-') {
        return tool_error("branch name cannot start with '-'");
    }

    let command = format!("git checkout -b {branch}");
    if !is_allowed(&command, extra_prefixes) {
        return tool_error("git checkout is not allowed");
    }

    run_argv(
        sandbox,
        &[
            "git".into(),
            "checkout".into(),
            "-b".into(),
            branch.to_string(),
        ],
    )
}

fn parse_command_argv(command: &str) -> Result<Vec<String>, String> {
    if contains_shell_metacharacters(command) {
        return Err("command contains unsupported shell characters".into());
    }

    if command.contains('"') || command.contains('\'') {
        return Err("command contains unsupported quote characters".into());
    }

    let tokens: Vec<String> = command.split_whitespace().map(String::from).collect();
    if tokens.is_empty() {
        return Err("command cannot be empty".into());
    }

    Ok(tokens)
}

fn contains_shell_metacharacters(command: &str) -> bool {
    command
        .chars()
        .any(|c| matches!(c, ';' | '|' | '&' | '$' | '`' | '\n' | '\r' | '<' | '>'))
}

fn run_argv(sandbox: &WorkspaceSandbox, argv: &[String]) -> ToolResult {
    if argv.is_empty() {
        return tool_error("command cannot be empty");
    }

    let program = &argv[0];
    let args: Vec<&str> = argv[1..].iter().map(String::as_str).collect();
    let root = sandbox.root();

    match Command::new(program).args(args).current_dir(root).output() {
        Ok(out) => {
            let stdout = truncate(String::from_utf8_lossy(&out.stdout).into_owned());
            let stderr = truncate(String::from_utf8_lossy(&out.stderr).into_owned());
            let code = out.status.code().unwrap_or(-1);

            ToolResult {
                ok: out.status.success(),
                output: format!(
                    "exit_code: {code}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
                ),
                error: if out.status.success() {
                    None
                } else {
                    Some(format!("command exited with code {code}"))
                },
            }
        }
        Err(err) => tool_error(format!("failed to run command: {err}")),
    }
}

fn allowed_prefixes(extra_prefixes: &[String]) -> Vec<String> {
    DEFAULT_PREFIXES
        .iter()
        .map(|s| (*s).to_string())
        .chain(extra_prefixes.iter().cloned())
        .collect()
}

fn is_allowed(command: &str, extra_prefixes: &[String]) -> bool {
    let lower = command.to_lowercase();
    allowed_prefixes(extra_prefixes)
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

fn truncate(text: String) -> String {
    if text.len() <= MAX_OUTPUT_BYTES {
        return text;
    }
    format!("{}… [truncated]", &text[..MAX_OUTPUT_BYTES])
}

fn tool_error(message: impl Into<String>) -> ToolResult {
    ToolResult {
        ok: false,
        output: String::new(),
        error: Some(message.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::sandbox::WorkspaceSandbox;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn sandbox() -> WorkspaceSandbox {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("muse-cmd-test-{n}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        WorkspaceSandbox::from_root(&root).unwrap()
    }

    #[test]
    fn blocks_disallowed_commands() {
        let sb = sandbox();
        let result = run_command(&sb, "rm -rf /", &[]);
        assert!(!result.ok);
    }

    #[test]
    fn blocks_shell_injection_after_allowed_prefix() {
        let sb = sandbox();
        let result = run_command(&sb, "git status; echo pwned", &[]);
        assert!(!result.ok);
        assert!(result.error.is_some());
    }

    #[test]
    fn allows_git_status() {
        let sb = sandbox();
        let result = run_command(&sb, "git status", &[]);
        assert!(result.output.contains("exit_code"));
    }

    #[test]
    fn allows_extra_prefix() {
        let sb = sandbox();
        let result = run_command(&sb, "make test", &["make ".into()]);
        assert!(
            !result
                .error
                .as_ref()
                .is_some_and(|message| message.contains("not allowed"))
        );
    }

    #[test]
    fn git_commit_uses_argv_not_shell() {
        let sb = sandbox();
        let result = git_commit(&sb, "test \"injection\"", &[]);
        assert!(result.output.contains("exit_code") || result.error.is_some());
    }
}
