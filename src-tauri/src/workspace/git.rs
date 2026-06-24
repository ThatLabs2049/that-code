use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceGitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}

pub fn get_workspace_git_status(workspace: &Path) -> WorkspaceGitStatus {
    if !workspace.join(".git").exists() {
        return WorkspaceGitStatus::default();
    }

    let branch = git_output(workspace, &["rev-parse", "--abbrev-ref", "HEAD"])
        .filter(|s| !s.is_empty() && s != "HEAD");

    let porcelain = git_output(workspace, &["status", "--porcelain"]);
    let files_changed = porcelain
        .as_ref()
        .map(|text| text.lines().filter(|l| !l.trim().is_empty()).count() as u32)
        .unwrap_or(0);

    let (insertions, deletions) = parse_diff_stat(&git_output(workspace, &["diff", "--shortstat"]));

    WorkspaceGitStatus {
        is_repo: true,
        branch,
        files_changed,
        insertions,
        deletions,
    }
}

fn git_output(workspace: &Path, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(workspace).args(args);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(text)
}

fn parse_diff_stat(text: &Option<String>) -> (u32, u32) {
    let Some(text) = text else {
        return (0, 0);
    };
    let mut insertions = 0u32;
    let mut deletions = 0u32;
    for part in text.split(',') {
        let part = part.trim();
        if part.contains("insertion") {
            insertions = part
                .split_whitespace()
                .next()
                .and_then(|t| t.parse().ok())
                .unwrap_or(0);
        } else if part.contains("deletion") {
            deletions = part
                .split_whitespace()
                .next()
                .and_then(|t| t.parse().ok())
                .unwrap_or(0);
        }
    }
    (insertions, deletions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shortstat() {
        let text = Some(" 2 files changed, 15 insertions(+), 3 deletions(-)".into());
        assert_eq!(parse_diff_stat(&text), (15, 3));
    }
}
