use std::fs;
use std::path::Path;
use std::process::Command;

const MAX_TREE_LINES: usize = 80;
const MAX_TREE_DEPTH: usize = 3;

pub fn build_context_pack(workspace: &Path) -> String {
    let tree = project_tree(workspace, workspace, 0, MAX_TREE_DEPTH);
    let tree_section = if tree.trim().is_empty() {
        "(empty workspace — no project files yet. Use write_file to create new files.)".to_string()
    } else {
        format!("--- Project tree (depth {MAX_TREE_DEPTH}) ---\n{tree}")
    };

    let mut sections = vec![tree_section];

    if let Some(git) = git_status_short(workspace) {
        sections.push(format!("--- Git status ---\n{git}"));
    }

    sections.join("\n\n")
}

const MAX_RULES_BYTES: usize = 16 * 1024;

pub fn detect_project_rules_file(workspace: &Path) -> Option<String> {
    for relative in [
        ".thatcode/rules.md",
        ".cursorrules",
        "AGENTS.md",
    ] {
        if workspace.join(relative).is_file() {
            return Some(relative.into());
        }
    }
    None
}

pub fn is_workspace_empty(workspace: &Path) -> bool {
    let Ok(entries) = fs::read_dir(workspace) else {
        return true;
    };

    !entries.filter_map(|e| e.ok()).any(|entry| {
        let name = entry.file_name().to_string_lossy().into_owned();
        !name.starts_with('.') || name == ".git"
    })
}

pub fn load_project_rules(workspace: &Path) -> Option<String> {
    for relative in [
        ".thatcode/rules.md",
        ".cursorrules",
        "AGENTS.md",
    ] {
        let path = workspace.join(relative);
        if !path.is_file() {
            continue;
        }
        let content = fs::read_to_string(&path).ok()?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            continue;
        }
        let capped = if trimmed.len() > MAX_RULES_BYTES {
            format!("{}… [truncated]", &trimmed[..MAX_RULES_BYTES])
        } else {
            trimmed.to_string()
        };
        return Some(format!("(from {relative})\n{capped}"));
    }
    None
}

#[allow(clippy::only_used_in_recursion)]
fn project_tree(root: &Path, dir: &Path, depth: usize, max_depth: usize) -> String {
    if depth > max_depth {
        return String::new();
    }

    let mut lines = Vec::new();
    let indent = "  ".repeat(depth);

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return String::new(),
    };

    let mut entries = entries;
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        if lines.len() >= MAX_TREE_LINES {
            lines.push(format!("{indent}…"));
            break;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') && name != ".git" {
            continue;
        }
        if name == "node_modules" || name == "target" || name == "dist" {
            continue;
        }

        let path = entry.path();
        let kind = if path.is_dir() { "dir" } else { "file" };
        lines.push(format!("{indent}{name} ({kind})"));

        if path.is_dir() && depth < max_depth {
            let nested = project_tree(root, &path, depth + 1, max_depth);
            if !nested.is_empty() {
                lines.push(nested);
            }
        }
    }

    lines.join("\n")
}

fn git_status_short(workspace: &Path) -> Option<String> {
    if !workspace.join(".git").exists() {
        return None;
    }

    let output = Command::new("git")
        .args(["-C", &workspace.to_string_lossy(), "status", "--short"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        Some("(clean working tree)".into())
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn builds_tree_for_workspace() {
        let cwd = env::current_dir().unwrap();
        let tree = project_tree(&cwd, &cwd, 0, 1);
        assert!(!tree.is_empty());
    }
}
