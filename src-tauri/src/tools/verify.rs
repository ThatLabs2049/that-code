use std::path::Path;

use super::command;
use super::handlers::ToolResult;
use super::WorkspaceSandbox;

pub const MAX_VERIFY_RETRIES: usize = 3;

pub fn infer_verify_command(workspace: &Path) -> Option<String> {
    if workspace.join("Cargo.toml").exists() {
        return Some("cargo test".into());
    }
    if workspace.join("package.json").exists() {
        return Some("npm test".into());
    }
    if workspace.join("pyproject.toml").exists() || workspace.join("pytest.ini").exists() {
        return Some("pytest".into());
    }
    if workspace.join("go.mod").exists() {
        return Some("go test ./...".into());
    }
    None
}

pub fn resolve_verify_command(
    workspace: &Path,
    configured: Option<&str>,
) -> Option<String> {
    configured
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(str::to_string)
        .or_else(|| infer_verify_command(workspace))
}

pub fn run_verify(
    sandbox: &WorkspaceSandbox,
    command: &str,
    extra_prefixes: &[String],
) -> ToolResult {
    command::run_command(sandbox, command, extra_prefixes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn infers_cargo_for_rust_project() {
        let cwd = env::current_dir().unwrap();
        if cwd.join("Cargo.toml").exists() {
            assert_eq!(infer_verify_command(&cwd).as_deref(), Some("cargo test"));
        }
    }
}
