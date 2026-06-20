use std::collections::{HashMap, HashSet};
use std::fs;

use serde::{Deserialize, Serialize};

use crate::tools::WorkspaceSandbox;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub path: String,
    pub change_type: String,
    pub diff: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_content: Option<String>,
}

#[derive(Debug, Default)]
pub struct ChangeTracker {
    before: HashMap<String, Option<String>>,
    touched: Vec<String>,
}

impl ChangeTracker {
    pub fn capture_before(&mut self, sandbox: &WorkspaceSandbox, relative_path: &str) {
        let key = normalize_path(relative_path);
        if self.before.contains_key(&key) {
            return;
        }

        let content = match sandbox.resolve(&key) {
            Ok(resolved) if resolved.is_file() => fs::read_to_string(&resolved).ok(),
            Ok(_) => None,
            Err(_) => None,
        };
        self.before.insert(key, content);
    }

    pub fn note_touched(&mut self, relative_path: &str) {
        let key = normalize_path(relative_path);
        if !self.touched.contains(&key) {
            self.touched.push(key);
        }
    }

    pub fn finalize(&self, sandbox: &WorkspaceSandbox) -> Vec<FileChange> {
        let mut changes = Vec::new();

        for path in &self.touched {
            let before = self.before.get(path).cloned().flatten();
            let after = match sandbox.resolve(path) {
                Ok(resolved) if resolved.is_file() => fs::read_to_string(&resolved).ok(),
                Ok(_) => None,
                Err(_) => None,
            };

            let change_type = match (&before, &after) {
                (None, Some(_)) => "created",
                (Some(_), None) => "deleted",
                (Some(b), Some(a)) if b == a => continue,
                (Some(_), Some(_)) => "modified",
                (None, None) => continue,
            };

            let diff = unified_diff(path, before.as_deref().unwrap_or(""), after.as_deref().unwrap_or(""));

            changes.push(FileChange {
                path: path.clone(),
                change_type: change_type.into(),
                diff,
                before_content: before,
                after_content: after,
            });
        }

        changes
    }
}

pub fn revert_file_changes(
    sandbox: &WorkspaceSandbox,
    changes: &[FileChange],
    paths: Option<&[String]>,
) -> Result<(), String> {
    let selected = paths.map(|items| items.iter().map(String::as_str).collect::<HashSet<_>>());

    for change in changes {
        if let Some(set) = &selected {
            if !set.contains(change.path.as_str()) {
                continue;
            }
        }
        revert_change(sandbox, change)?;
    }

    Ok(())
}

fn revert_change(sandbox: &WorkspaceSandbox, change: &FileChange) -> Result<(), String> {
    let resolved = sandbox.resolve(&change.path).map_err(|e| e.to_string())?;

    match change.change_type.as_str() {
        "created" => {
            if resolved.exists() {
                fs::remove_file(&resolved).map_err(|e| e.to_string())?;
            }
        }
        "deleted" => {
            if let Some(parent) = resolved.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = change.before_content.clone().unwrap_or_default();
            fs::write(&resolved, content).map_err(|e| e.to_string())?;
        }
        "modified" => {
            let content = change
                .before_content
                .clone()
                .ok_or_else(|| format!("missing snapshot for {}", change.path))?;
            fs::write(&resolved, content).map_err(|e| e.to_string())?;
        }
        other => return Err(format!("unknown change type: {other}")),
    }

    Ok(())
}

fn normalize_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

fn unified_diff(path: &str, before: &str, after: &str) -> String {
    if before == after {
        return String::new();
    }

    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();

    let mut out = format!("--- a/{path}\n+++ b/{path}\n");
    let max_len = before_lines.len().max(after_lines.len());

    for index in 0..max_len {
        let b = before_lines.get(index).copied().unwrap_or("");
        let a = after_lines.get(index).copied().unwrap_or("");
        if b == a {
            continue;
        }
        if !b.is_empty() {
            out.push_str(&format!("-{b}\n"));
        }
        if !a.is_empty() {
            out.push_str(&format!("+{a}\n"));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_unified_diff() {
        let diff = unified_diff("src/main.rs", "old\n", "new\n");
        assert!(diff.contains("-old"));
        assert!(diff.contains("+new"));
    }
}
