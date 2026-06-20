use std::fs;
use std::path::Path;
use std::process::Command;

const MAX_TREE_LINES: usize = 80;
const MAX_TREE_DEPTH: usize = 3;

pub fn build_context_pack(workspace: &Path) -> String {
    let mut sections = Vec::new();

    sections.push(format!(
        "--- Project tree (depth {MAX_TREE_DEPTH}) ---\n{}",
        project_tree(workspace, workspace, 0, MAX_TREE_DEPTH)
    ));

    if let Some(git) = git_status_short(workspace) {
        sections.push(format!("--- Git status ---\n{git}"));
    }

    sections.join("\n\n")
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
