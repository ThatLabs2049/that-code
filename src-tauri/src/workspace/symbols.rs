use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::rag::{load_ignore_patterns, should_ignore_relative_path};
use crate::tools::WorkspaceSandbox;

const MAX_SYMBOL_RESULTS: usize = 12;
const MAX_WALK_DEPTH: usize = 8;
const MAX_FILE_BYTES: usize = 256_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSymbolHit {
    pub name: String,
    pub path: String,
    pub line: u32,
    pub kind: String,
}

const TEXT_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "kt", "cs", "cpp", "c", "h", "hpp",
    "rb", "php", "swift", "vue", "svelte", "md", "toml", "yaml", "yml", "json",
];

pub fn search_workspace_symbols(workspace: &Path, query: &str) -> Result<Vec<WorkspaceSymbolHit>, String> {
    let trimmed = query.trim();
    if trimmed.len() < 2 {
        return Ok(Vec::new());
    }

    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;
    let ignore_patterns = load_ignore_patterns(sandbox.root());
    let needle = trimmed.to_lowercase();

    let mut hits: Vec<(i32, WorkspaceSymbolHit)> = Vec::new();
    collect_symbol_hits(
        sandbox.root(),
        sandbox.root(),
        &ignore_patterns,
        &needle,
        0,
        &mut hits,
    );

    hits.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.path.cmp(&b.1.path)));
    hits.truncate(MAX_SYMBOL_RESULTS);

    Ok(hits.into_iter().map(|(_, hit)| hit).collect())
}

#[allow(clippy::only_used_in_recursion)]
fn collect_symbol_hits(
    root: &Path,
    dir: &Path,
    ignore_patterns: &[String],
    needle: &str,
    depth: usize,
    hits: &mut Vec<(i32, WorkspaceSymbolHit)>,
) {
    if depth > MAX_WALK_DEPTH {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return,
    };

    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        if path.is_dir() {
            if should_ignore_relative_path(&relative, ignore_patterns) {
                continue;
            }
            collect_symbol_hits(root, &path, ignore_patterns, needle, depth + 1, hits);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if should_ignore_relative_path(&relative, ignore_patterns) {
            continue;
        }

        if !is_text_file(&relative) {
            continue;
        }

        scan_file_for_symbols(&relative, &path, needle, hits);
    }
}

fn is_text_file(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map(|ext| TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn scan_file_for_symbols(
    relative: &str,
    path: &Path,
    needle: &str,
    hits: &mut Vec<(i32, WorkspaceSymbolHit)>,
) {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };
    if metadata.len() as usize > MAX_FILE_BYTES {
        return;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        if let Some((name, kind, score)) = match_symbol_line(trimmed, needle) {
            hits.push((
                score,
                WorkspaceSymbolHit {
                    name,
                    path: relative.to_string(),
                    line: (line_idx + 1) as u32,
                    kind,
                },
            ));
        }
    }
}

fn match_symbol_line(line: &str, needle: &str) -> Option<(String, String, i32)> {
    let patterns: &[(&str, &str)] = &[
        ("fn ", "function"),
        ("pub fn ", "function"),
        ("async fn ", "function"),
        ("pub async fn ", "function"),
        ("function ", "function"),
        ("export function ", "function"),
        ("export async function ", "function"),
        ("const ", "const"),
        ("export const ", "const"),
        ("let ", "variable"),
        ("class ", "class"),
        ("export class ", "class"),
        ("interface ", "interface"),
        ("export interface ", "interface"),
        ("type ", "type"),
        ("export type ", "type"),
        ("struct ", "struct"),
        ("pub struct ", "struct"),
        ("enum ", "enum"),
        ("pub enum ", "enum"),
        ("trait ", "trait"),
        ("pub trait ", "trait"),
        ("impl ", "impl"),
        ("def ", "function"),
        ("async def ", "function"),
    ];

    let lower_line = line.to_lowercase();

    for (prefix, kind) in patterns {
        if !lower_line.contains(prefix) {
            continue;
        }
        if let Some(name) = extract_name_after_prefix(line, prefix) {
            let name_lower = name.to_lowercase();
            let score = if name_lower == needle {
                100
            } else if name_lower.starts_with(needle) {
                80
            } else if name_lower.contains(needle) {
                50
            } else {
                continue;
            };
            return Some((name, (*kind).into(), score));
        }
    }

    None
}

fn extract_name_after_prefix(line: &str, prefix: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let idx = lower.find(prefix)?;
    let rest = line[idx + prefix.len()..].trim_start();
    let mut name = String::new();
    for ch in rest.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            break;
        }
    }
    if name.len() >= 2 {
        Some(name)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_name_from_rust_fn() {
        assert_eq!(
            extract_name_after_prefix("pub fn search_workspace_symbols(", "pub fn "),
            Some("search_workspace_symbols".into())
        );
    }

    #[test]
    fn match_symbol_line_exact() {
        let hit = match_symbol_line("export function useChat() {", "usechat");
        assert!(hit.is_some());
        let (name, kind, score) = hit.unwrap();
        assert_eq!(name, "useChat");
        assert_eq!(kind, "function");
        assert_eq!(score, 100);
    }
}
