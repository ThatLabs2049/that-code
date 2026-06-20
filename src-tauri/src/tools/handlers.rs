use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use super::command;
use super::sandbox::{SandboxError, WorkspaceSandbox, MAX_READ_BYTES};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub ok: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub sandbox: WorkspaceSandbox,
    pub allow_overwrites: bool,
    pub command_allowlist_extra: Vec<String>,
}

pub fn list_dir(ctx: &ToolContext, path: &str) -> ToolResult {
    match ctx.sandbox.resolve(path) {
        Ok(resolved) => match fs::read_dir(&resolved) {
            Ok(entries) => {
                let mut names: Vec<String> = entries
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
                    .collect();
                names.sort();
                ToolResult {
                    ok: true,
                    output: names.join("\n"),
                    error: None,
                }
            }
            Err(err) => tool_error(format!("could not list directory: {err}")),
        },
        Err(err) => tool_error(err.to_string()),
    }
}

pub fn read_file(ctx: &ToolContext, path: &str) -> ToolResult {
    let resolved = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    if !resolved.is_file() {
        return tool_error("path is not a file");
    }

    let metadata = match fs::metadata(&resolved) {
        Ok(meta) => meta,
        Err(err) => return tool_error(format!("could not read metadata: {err}")),
    };

    if metadata.len() as usize > MAX_READ_BYTES {
        return tool_error(format!(
            "file exceeds size limit of {} bytes",
            MAX_READ_BYTES
        ));
    }

    match fs::read_to_string(&resolved) {
        Ok(content) => ToolResult {
            ok: true,
            output: content,
            error: None,
        },
        Err(err) if is_binary_read_error(&err) => {
            tool_error("binary files are not supported; only text files can be read")
        }
        Err(err) => tool_error(format!("could not read file: {err}")),
    }
}

pub fn write_file(ctx: &ToolContext, path: &str, content: &str) -> ToolResult {
    let resolved = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    if resolved.exists() && !ctx.allow_overwrites {
        return tool_error(
            "file already exists and overwriting is disabled in Settings — enable \
             \"Allow modifying files\" or choose a new path",
        );
    }

    write_bytes(ctx, &resolved, content.as_bytes(), path)
}

pub fn edit_file(ctx: &ToolContext, path: &str, old_string: &str, new_string: &str) -> ToolResult {
    if !ctx.allow_overwrites {
        return tool_error("file edits require \"Allow modifying files\" in Settings");
    }

    if old_string.is_empty() {
        return tool_error("old_string cannot be empty");
    }

    let resolved = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    if !resolved.is_file() {
        return tool_error("path is not a file");
    }

    let content = match fs::read_to_string(&resolved) {
        Ok(text) => text,
        Err(err) => return tool_error(format!("could not read file: {err}")),
    };

    let matches = content.match_indices(old_string).count();
    if matches == 0 {
        return tool_error("old_string not found in file");
    }
    if matches > 1 {
        return tool_error(format!(
            "old_string matches {matches} times — include more surrounding context to make it unique"
        ));
    }

    let updated = content.replacen(old_string, new_string, 1);
    write_bytes(ctx, &resolved, updated.as_bytes(), path)
}

pub fn grep(ctx: &ToolContext, pattern: &str, path: &str) -> ToolResult {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return tool_error("pattern cannot be empty");
    }

    let root = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    let pattern_lower = pattern.to_lowercase();
    let mut hits = Vec::new();
    grep_path(&root, ctx, &pattern_lower, &mut hits, 0, 6);

    ToolResult {
        ok: true,
        output: if hits.is_empty() {
            "No matches.".into()
        } else {
            hits.join("\n")
        },
        error: None,
    }
}

pub fn file_info(ctx: &ToolContext, path: &str) -> ToolResult {
    let resolved = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    if !resolved.exists() {
        return ToolResult {
            ok: true,
            output: format!("{path}: not found"),
            error: None,
        };
    }

    let meta = match fs::metadata(&resolved) {
        Ok(meta) => meta,
        Err(err) => return tool_error(format!("could not stat: {err}")),
    };

    let kind = if meta.is_dir() {
        "directory"
    } else if meta.is_file() {
        "file"
    } else {
        "other"
    };

    ToolResult {
        ok: true,
        output: format!("{path}: {kind}, {} bytes", meta.len()),
        error: None,
    }
}

pub fn create_dir(ctx: &ToolContext, path: &str) -> ToolResult {
    let resolved = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    match fs::create_dir_all(&resolved) {
        Ok(()) => ToolResult {
            ok: true,
            output: format!("created {path}"),
            error: None,
        },
        Err(err) => tool_error(format!("could not create directory: {err}")),
    }
}

pub fn delete_file(ctx: &ToolContext, path: &str) -> ToolResult {
    if !ctx.allow_overwrites {
        return tool_error("deleting files requires \"Allow modifying files\" in Settings");
    }

    let resolved = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    if resolved.is_dir() {
        return tool_error("delete_file only removes files, not directories");
    }

    if !resolved.exists() {
        return tool_error("file does not exist");
    }

    match fs::remove_file(&resolved) {
        Ok(()) => ToolResult {
            ok: true,
            output: format!("deleted {path}"),
            error: None,
        },
        Err(err) => tool_error(format!("could not delete file: {err}")),
    }
}

fn write_bytes(_ctx: &ToolContext, resolved: &Path, bytes: &[u8], display_path: &str) -> ToolResult {
    if let Some(parent) = resolved.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            return tool_error(format!("could not create parent directories: {err}"));
        }
    }

    match fs::write(resolved, bytes) {
        Ok(()) => ToolResult {
            ok: true,
            output: format!("wrote {} bytes to {display_path}", bytes.len()),
            error: None,
        },
        Err(err) => tool_error(format!("could not write file: {err}")),
    }
}

fn grep_path(
    path: &Path,
    ctx: &ToolContext,
    pattern: &str,
    hits: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth || hits.len() >= 80 {
        return;
    }

    if path.is_file() {
        if let Ok(meta) = fs::metadata(path) {
            if meta.len() as usize > MAX_READ_BYTES {
                return;
            }
        }
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(relative) = path.strip_prefix(ctx.sandbox.root()) {
                for (idx, line) in content.lines().enumerate() {
                    if hits.len() >= 80 {
                        break;
                    }
                    if line.to_lowercase().contains(pattern) {
                        hits.push(format!("{}:{}: {}", relative.display(), idx + 1, line.trim()));
                    }
                }
            }
        }
        return;
    }

    if !path.is_dir() {
        return;
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        if hits.len() >= 80 {
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        grep_path(&entry.path(), ctx, pattern, hits, depth + 1, max_depth);
    }
}

pub fn search_files(ctx: &ToolContext, query: &str, path: &str) -> ToolResult {
    let query = query.trim();
    if query.is_empty() {
        return tool_error("search query cannot be empty");
    }

    let root = match ctx.sandbox.resolve(path) {
        Ok(path) => path,
        Err(err) => return tool_error(err.to_string()),
    };

    if !root.is_dir() {
        return tool_error("search path must be a directory");
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    search_dir(&root, ctx, &query_lower, &mut matches, 0, 4);

    ToolResult {
        ok: true,
        output: if matches.is_empty() {
            "No matches found.".into()
        } else {
            matches.join("\n")
        },
        error: None,
    }
}

fn search_dir(
    dir: &Path,
    ctx: &ToolContext,
    query: &str,
    matches: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) {
    if depth > max_depth || matches.len() >= 50 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        if matches.len() >= 50 {
            break;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if name == "node_modules" || name == "target" {
                continue;
            }
            search_dir(&path, ctx, query, matches, depth + 1, max_depth);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if name.to_lowercase().contains(query) {
            if let Ok(relative) = path.strip_prefix(ctx.sandbox.root()) {
                matches.push(format!("{}/ (filename match)", relative.display()));
            }
            continue;
        }

        if let Ok(meta) = fs::metadata(&path) {
            if meta.len() as usize > MAX_READ_BYTES {
                continue;
            }
        }

        if let Ok(content) = fs::read_to_string(&path) {
            if content.to_lowercase().contains(query) {
                if let Ok(relative) = path.strip_prefix(ctx.sandbox.root()) {
                    matches.push(relative.display().to_string());
                }
            }
        }
    }
}

pub fn execute_tool(ctx: &ToolContext, tool: &str, args: &serde_json::Value) -> ToolResult {
    match tool {
        "list_dir" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            list_dir(ctx, path)
        }
        "read_file" => {
            let Some(path) = args.get("path").and_then(|v| v.as_str()) else {
                return tool_error("read_file requires path");
            };
            read_file(ctx, path)
        }
        "write_file" => {
            let Some(path) = args.get("path").and_then(|v| v.as_str()) else {
                return tool_error("write_file requires path");
            };
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            write_file(ctx, path, content)
        }
        "edit_file" => {
            let Some(path) = args.get("path").and_then(|v| v.as_str()) else {
                return tool_error("edit_file requires path");
            };
            let Some(old_string) = args.get("old_string").and_then(|v| v.as_str()) else {
                return tool_error("edit_file requires old_string");
            };
            let new_string = args.get("new_string").and_then(|v| v.as_str()).unwrap_or("");
            edit_file(ctx, path, old_string, new_string)
        }
        "grep" => {
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            grep(ctx, pattern, path)
        }
        "file_info" => {
            let Some(path) = args.get("path").and_then(|v| v.as_str()) else {
                return tool_error("file_info requires path");
            };
            file_info(ctx, path)
        }
        "create_dir" => {
            let Some(path) = args.get("path").and_then(|v| v.as_str()) else {
                return tool_error("create_dir requires path");
            };
            create_dir(ctx, path)
        }
        "delete_file" => {
            let Some(path) = args.get("path").and_then(|v| v.as_str()) else {
                return tool_error("delete_file requires path");
            };
            delete_file(ctx, path)
        }
        "search_files" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            search_files(ctx, query, path)
        }
        "run_command" => {
            let Some(command) = args.get("command").and_then(|v| v.as_str()) else {
                return tool_error("run_command requires command");
            };
            command::run_command(
                &ctx.sandbox,
                command,
                &ctx.command_allowlist_extra,
            )
        }
        "git_add" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
            command::git_add(&ctx.sandbox, path, &ctx.command_allowlist_extra)
        }
        "git_commit" => {
            let Some(message) = args.get("message").and_then(|v| v.as_str()) else {
                return tool_error("git_commit requires message");
            };
            command::git_commit(&ctx.sandbox, message, &ctx.command_allowlist_extra)
        }
        "git_checkout_branch" => {
            let Some(branch) = args.get("branch").and_then(|v| v.as_str()) else {
                return tool_error("git_checkout_branch requires branch");
            };
            command::git_checkout_branch(&ctx.sandbox, branch, &ctx.command_allowlist_extra)
        }
        other => tool_error(format!("unknown tool: {other}")),
    }
}

pub fn tool_context_from_settings(
    workspace_path: &Option<String>,
    allow_overwrites: bool,
    command_allowlist_extra: &[String],
) -> Result<ToolContext, SandboxError> {
    let Some(path) = workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Err(SandboxError::NotConfigured);
    };

    Ok(ToolContext {
        sandbox: WorkspaceSandbox::from_root(path)?,
        allow_overwrites,
        command_allowlist_extra: command_allowlist_extra.to_vec(),
    })
}

fn tool_error(message: impl Into<String>) -> ToolResult {
    ToolResult {
        ok: false,
        output: String::new(),
        error: Some(message.into()),
    }
}

fn is_binary_read_error(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::InvalidData
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::sandbox::WorkspaceSandbox;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_context() -> (PathBuf, ToolContext) {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("muse-tools-test-{n}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("readme.txt"), "hello world").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();

        let ctx = ToolContext {
            sandbox: WorkspaceSandbox::from_root(&root).unwrap(),
            allow_overwrites: false,
            command_allowlist_extra: vec![],
        };
        (root, ctx)
    }

    #[test]
    fn lists_directory_entries() {
        let (_root, ctx) = test_context();
        let result = list_dir(&ctx, "src");
        assert!(result.ok);
        assert!(result.output.contains("main.rs"));
    }

    #[test]
    fn reads_text_file() {
        let (_root, ctx) = test_context();
        let result = read_file(&ctx, "readme.txt");
        assert!(result.ok);
        assert_eq!(result.output, "hello world");
    }

    #[test]
    fn blocks_overwrite_without_permission() {
        let (_root, ctx) = test_context();
        let result = write_file(&ctx, "readme.txt", "new");
        assert!(!result.ok);
        assert!(result.error.unwrap().contains("overwriting"));
    }

    #[test]
    fn writes_new_file() {
        let (_root, ctx) = test_context();
        let result = write_file(&ctx, "notes.txt", "draft");
        assert!(result.ok);
        assert_eq!(read_file(&ctx, "notes.txt").output, "draft");
    }

    #[test]
    fn edit_file_replaces_unique_match() {
        let (_root, ctx) = test_context();
        let mut ctx = ctx;
        ctx.allow_overwrites = true;
        let result = edit_file(&ctx, "readme.txt", "hello", "hi");
        assert!(result.ok);
        assert_eq!(read_file(&ctx, "readme.txt").output, "hi world");
    }

    #[test]
    fn grep_finds_line() {
        let (_root, ctx) = test_context();
        let result = grep(&ctx, "hello", ".");
        assert!(result.ok);
        assert!(result.output.contains("readme.txt"));
    }
}
