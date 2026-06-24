use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::rag::{load_ignore_patterns, should_ignore_name, should_ignore_relative_path};
use crate::tools::WorkspaceSandbox;

const MAX_SEARCH_RESULTS: usize = 12;
const MAX_WALK_DEPTH: usize = 8;
const ATTACHMENT_READ_BYTES: usize = 8_192;
const FOLDER_PREVIEW_FILES: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePathHit {
    pub path: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageAttachment {
    pub path: String,
    pub kind: String,
    #[serde(default)]
    pub line: Option<u32>,
    #[serde(default)]
    pub symbol: Option<String>,
}

pub fn search_workspace_paths(workspace: &Path, query: &str) -> Result<Vec<WorkspacePathHit>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;
    let ignore_patterns = load_ignore_patterns(sandbox.root());
    let needle = trimmed.to_lowercase();

    let mut hits: Vec<(i32, WorkspacePathHit)> = Vec::new();
    collect_search_hits(
        sandbox.root(),
        sandbox.root(),
        &ignore_patterns,
        &needle,
        0,
        &mut hits,
    );

    hits.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.path.cmp(&b.1.path)));
    hits.truncate(MAX_SEARCH_RESULTS);

    Ok(hits.into_iter().map(|(_, hit)| hit).collect())
}

#[allow(clippy::only_used_in_recursion)]
fn collect_search_hits(
    root: &Path,
    dir: &Path,
    ignore_patterns: &[String],
    needle: &str,
    depth: usize,
    hits: &mut Vec<(i32, WorkspacePathHit)>,
) {
    if depth > MAX_WALK_DEPTH {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return,
    };

    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        if path.is_dir() {
            if should_ignore_name(&name, ignore_patterns)
                || should_ignore_relative_path(&relative, ignore_patterns)
            {
                continue;
            }
            if let Some(score) = score_path(&relative, needle) {
                hits.push((
                    score,
                    WorkspacePathHit {
                        path: relative.clone(),
                        kind: "folder".into(),
                    },
                ));
            }
            collect_search_hits(root, &path, ignore_patterns, needle, depth + 1, hits);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if should_ignore_relative_path(&relative, ignore_patterns) {
            continue;
        }

        if let Some(score) = score_path(&relative, needle) {
            hits.push((
                score,
                WorkspacePathHit {
                    path: relative,
                    kind: "file".into(),
                },
            ));
        }
    }
}

fn score_path(path: &str, needle: &str) -> Option<i32> {
    let lower = path.to_lowercase();
    let file_name = lower.rsplit('/').next().unwrap_or(&lower);
    if file_name == needle {
        return Some(100);
    }
    if file_name.starts_with(needle) {
        return Some(80);
    }
    if lower.starts_with(needle) {
        return Some(70);
    }
    if lower.contains(needle) {
        return Some(50);
    }
    None
}

pub fn build_attachment_context(
    settings: &crate::settings::AiSettings,
    attachments: &[MessageAttachment],
) -> Result<String, String> {
    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Err("workspace not configured".into());
    };

    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;

    let mut sections = Vec::new();

    for attachment in attachments {
        let path = attachment.path.trim().replace('\\', "/");
        if path.is_empty() {
            continue;
        }

        match attachment.kind.as_str() {
            "symbol" => {
                let line = attachment.line.unwrap_or(1);
                let symbol = attachment.symbol.as_deref().unwrap_or("symbol");
                match sandbox.resolve(&path) {
                    Ok(resolved) if resolved.is_file() => match fs::read_to_string(&resolved) {
                        Ok(content) => {
                            let excerpt = excerpt_around_line(&content, line as usize, 12);
                            sections.push(format!(
                                "### Symbol: {symbol} ({path}:{line})\n{excerpt}"
                            ));
                        }
                        Err(err) => {
                            sections.push(format!(
                                "### Symbol: {symbol} ({path}:{line})\n(error: {err})"
                            ));
                        }
                    },
                    Ok(_) => sections.push(format!(
                        "### Symbol: {symbol} ({path}:{line})\n(not a file)"
                    )),
                    Err(err) => sections.push(format!(
                        "### Symbol: {symbol} ({path}:{line})\n(error: {err})"
                    )),
                }
            }
            "folder" => {
                let resolved = sandbox.resolve(&path).map_err(|e| e.to_string())?;
                if resolved.is_dir() {
                    let listing = list_directory(&resolved);
                    sections.push(format!("### Folder: {path}\n{listing}"));

                    let mut previewed = 0;
                    if let Ok(entries) = fs::read_dir(&resolved) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            if previewed >= FOLDER_PREVIEW_FILES {
                                break;
                            }
                            let child_path = entry.path();
                            if !child_path.is_file() {
                                continue;
                            }
                            let relative = child_path
                                .strip_prefix(sandbox.root())
                                .map(|p| p.to_string_lossy().replace('\\', "/"))
                                .unwrap_or_default();
                            if let Ok(content) = fs::read_to_string(&child_path) {
                                sections.push(format!(
                                    "#### {relative}\n{}",
                                    truncate_bytes(&content, ATTACHMENT_READ_BYTES)
                                ));
                                previewed += 1;
                            }
                        }
                    }
                } else {
                    sections.push(format!("### Folder: {path}\n(not a directory)"));
                }
            }
            _ => {
                match sandbox.resolve(&path) {
                    Ok(resolved) if resolved.is_file() => match fs::read_to_string(&resolved) {
                        Ok(content) => {
                            let cap = ATTACHMENT_READ_BYTES;
                            sections.push(format!(
                                "### File: {path}\n{}",
                                truncate_bytes(&content, cap)
                            ));
                        }
                        Err(err) => sections.push(format!("### File: {path}\n(error: {err})")),
                    },
                    Ok(_) => sections.push(format!("### File: {path}\n(not a file)")),
                    Err(err) => sections.push(format!("### File: {path}\n(error: {err})")),
                }
            }
        }
    }

    if sections.is_empty() {
        return Ok(String::new());
    }

    Ok(sections.join("\n\n"))
}

fn list_directory(dir: &Path) -> String {
    let mut names: Vec<String> = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let kind = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    "dir"
                } else {
                    "file"
                };
                format!("{name} ({kind})")
            })
            .collect(),
        Err(err) => return format!("(could not list: {err})"),
    };
    names.sort();
    names.join("\n")
}

fn truncate_bytes(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    format!("{}… [truncated]", &text[..max])
}

fn excerpt_around_line(content: &str, line: usize, context: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }
    let idx = line.saturating_sub(1).min(lines.len().saturating_sub(1));
    let start = idx.saturating_sub(context);
    let end = (idx + context + 1).min(lines.len());
    lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, l)| format!("{:>4}| {}", start + i + 1, l))
        .collect::<Vec<_>>()
        .join("\n")
}

fn prioritize_index_files(mut files: Vec<PathBuf>, root: &Path) -> Vec<PathBuf> {
    files.sort_by(|a, b| {
        let ra = a
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let rb = b
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        index_priority(&rb).cmp(&index_priority(&ra)).then_with(|| ra.cmp(&rb))
    });
    files
}

fn index_priority(relative: &str) -> u8 {
    let lower = relative.to_lowercase();
    if matches!(
        lower.as_str(),
        "cargo.toml"
            | "package.json"
            | "package-lock.json"
            | "pyproject.toml"
            | "go.mod"
            | "readme.md"
            | "tsconfig.json"
            | "makefile"
    ) {
        return 3;
    }
    if lower.starts_with("src/")
        || lower.starts_with("lib/")
        || lower.starts_with("app/")
        || lower.starts_with("packages/")
    {
        return 3;
    }
    if lower.starts_with("docs/") || lower.starts_with("test/") || lower.starts_with("tests/") {
        return 1;
    }
    2
}

pub fn sort_files_for_index(files: Vec<PathBuf>, root: &Path) -> Vec<PathBuf> {
    prioritize_index_files(files, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_filename_prefix_higher_than_contains() {
        assert!(score_path("src/main.rs", "main").unwrap() >= score_path("src/foo/main_extra.rs", "main").unwrap());
    }
}
