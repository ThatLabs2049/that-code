use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::chunk::chunk_text;
use super::embeddings::{create_embedding, cosine_similarity, EmbeddingError};
use super::store::{
    chunk_count, delete_chunks_for_path, insert_chunk, latest_index_time, list_source_paths,
    latest_path_index_time, clear_chunks, RagChunk,
};
use crate::ai::EmbeddingTestResult;
use crate::settings::AiSettings;
use crate::tools::WorkspaceSandbox;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const MAX_INDEX_FILES: usize = 400;
const MAX_FILE_BYTES: usize = 512 * 1024;

const TEXT_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "md", "json", "toml", "yaml", "yml", "css", "html", "txt",
    "py", "go", "sql", "vue", "svelte",
];

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RagIndexResult {
    pub files_indexed: usize,
    pub files_skipped: usize,
    pub chunks_stored: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RagStatus {
    pub enabled: bool,
    pub chunk_count: usize,
    pub last_indexed_at: Option<String>,
}

pub async fn build_workspace_index(
    settings: &AiSettings,
) -> Result<(Vec<RagChunk>, RagIndexResult), String> {
    if !settings.rag_enabled {
        return Err("RAG is disabled in settings".into());
    }

    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Err("Pick a workspace folder before indexing".into());
    };

    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;
    let files = collect_text_files(sandbox.root());
    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut files_indexed = 0;
    let mut chunks = Vec::new();

    for file in files.into_iter().take(MAX_INDEX_FILES) {
        let relative = file
            .strip_prefix(sandbox.root())
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");

        let content = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(_) => continue,
        };

        files_indexed += 1;

        for (index, chunk) in chunk_text(&content).into_iter().enumerate() {
            let embedding = create_embedding(settings, &chunk)
                .await
                .map_err(|e| e.to_string())?;

            chunks.push(RagChunk {
                id: uuid::Uuid::new_v4().to_string(),
                source_path: relative.clone(),
                chunk_index: index as i32,
                content: chunk,
                embedding,
                updated_at: timestamp.clone(),
            });
        }
    }

    let chunks_stored = chunks.len();

    Ok((
        chunks,
        RagIndexResult {
            files_indexed,
            files_skipped: 0,
            chunks_stored,
        },
    ))
}

pub fn persist_workspace_index(conn: &Connection, chunks: &[RagChunk]) -> Result<(), String> {
    clear_chunks(conn).map_err(|e| e.to_string())?;
    for chunk in chunks {
        insert_chunk(conn, chunk).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn indexed_path_times(conn: &Connection) -> Result<HashMap<String, Option<String>>, String> {
    let paths = list_source_paths(conn).map_err(|e| e.to_string())?;
    let mut times = HashMap::new();
    for path in paths {
        let latest = latest_path_index_time(conn, &path).map_err(|e| e.to_string())?;
        times.insert(path, latest);
    }
    Ok(times)
}

pub async fn build_incremental_index(
    settings: &AiSettings,
    indexed_times: &HashMap<String, Option<String>>,
) -> Result<(Vec<String>, Vec<RagChunk>, RagIndexResult), String> {
    if !settings.rag_enabled {
        return Err("RAG is disabled in settings".into());
    }

    let Some(workspace) = settings.workspace_path.as_ref().filter(|p| !p.trim().is_empty()) else {
        return Err("Pick a workspace folder before indexing".into());
    };

    let sandbox = WorkspaceSandbox::from_root(workspace).map_err(|e| e.to_string())?;
    let files = collect_text_files(sandbox.root());
    let on_disk: HashSet<String> = files
        .iter()
        .filter_map(|file| {
            file.strip_prefix(sandbox.root())
                .ok()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
        })
        .collect();

    let mut paths_to_replace = indexed_times
        .keys()
        .filter(|path| !on_disk.contains(*path))
        .cloned()
        .collect::<Vec<_>>();

    let timestamp = Utc::now().to_rfc3339();
    let mut files_indexed = 0;
    let mut chunks = Vec::new();

    for file in files.into_iter().take(MAX_INDEX_FILES) {
        let relative = file
            .strip_prefix(sandbox.root())
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/");

        if !file_needs_reindex(&file, indexed_times.get(&relative)) {
            continue;
        }

        paths_to_replace.push(relative.clone());

        let content = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(_) => continue,
        };

        files_indexed += 1;

        for (index, chunk) in chunk_text(&content).into_iter().enumerate() {
            let embedding = create_embedding(settings, &chunk)
                .await
                .map_err(|e| e.to_string())?;

            chunks.push(RagChunk {
                id: uuid::Uuid::new_v4().to_string(),
                source_path: relative.clone(),
                chunk_index: index as i32,
                content: chunk,
                embedding,
                updated_at: timestamp.clone(),
            });
        }
    }

    paths_to_replace.sort();
    paths_to_replace.dedup();

    let files_skipped = on_disk.len().saturating_sub(files_indexed);

    let chunks_stored = chunks.len();

    Ok((
        paths_to_replace,
        chunks,
        RagIndexResult {
            files_indexed,
            files_skipped,
            chunks_stored,
        },
    ))
}

pub fn persist_incremental_index(
    conn: &Connection,
    paths_to_replace: &[String],
    chunks: &[RagChunk],
) -> Result<(), String> {
    for path in paths_to_replace {
        delete_chunks_for_path(conn, path).map_err(|e| e.to_string())?;
    }
    for chunk in chunks {
        insert_chunk(conn, chunk).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn retrieve_context(
    settings: &AiSettings,
    chunks: &[RagChunk],
    query: &str,
) -> Result<String, String> {
    if !settings.rag_enabled || chunks.is_empty() {
        return Ok(String::new());
    }

    let query_embedding = create_embedding(settings, query)
        .await
        .map_err(|e| e.to_string())?;

    let mut scored: Vec<(f32, &RagChunk)> = chunks
        .iter()
        .map(|chunk| {
            let score = cosine_similarity(&query_embedding, &chunk.embedding);
            (score, chunk)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let top_k = settings.rag_top_k.max(1) as usize;
    let mut lines = Vec::new();

    for (score, chunk) in scored.into_iter().take(top_k) {
        if score < 0.2 {
            continue;
        }
        lines.push(format!(
            "[{}] (score {:.2})\n{}",
            chunk.source_path,
            score,
            truncate(&chunk.content, 600)
        ));
    }

    Ok(lines.join("\n\n"))
}

pub fn status(conn: &Connection, settings: &AiSettings) -> Result<RagStatus, String> {
    Ok(RagStatus {
        enabled: settings.rag_enabled,
        chunk_count: chunk_count(conn).map_err(|e| e.to_string())?,
        last_indexed_at: latest_index_time(conn).map_err(|e| e.to_string())?,
    })
}

pub async fn test_embedding(settings: &AiSettings) -> Result<EmbeddingTestResult, EmbeddingError> {
    let started = Instant::now();
    create_embedding(settings, "muse embedding test").await?;
    Ok(EmbeddingTestResult {
        ok: true,
        model: settings.embedding_model.clone(),
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

fn file_needs_reindex(file: &Path, indexed_at: Option<&Option<String>>) -> bool {
    let Some(indexed_at) = indexed_at.and_then(|value| value.as_ref()) else {
        return true;
    };

    let Some(file_modified) = file_modified_at(file) else {
        return true;
    };

    let Ok(indexed_time) = DateTime::parse_from_rfc3339(indexed_at) else {
        return true;
    };

    file_modified > indexed_time.with_timezone(&Utc)
}

fn file_modified_at(path: &Path) -> Option<DateTime<Utc>> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(DateTime::<Utc>::from(modified))
}

fn collect_text_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_files(root, root, &mut files, 0, 8);
    files.sort();
    files
}

#[allow(clippy::only_used_in_recursion)]
fn walk_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>, depth: usize, max_depth: usize) {
    if depth > max_depth || files.len() >= MAX_INDEX_FILES {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        if files.len() >= MAX_INDEX_FILES {
            break;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
            continue;
        }

        if path.is_dir() {
            walk_files(root, &path, files, depth + 1, max_depth);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };

        if !TEXT_EXTENSIONS
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
        {
            continue;
        }

        if let Ok(meta) = fs::metadata(&path) {
            if meta.len() as usize > MAX_FILE_BYTES {
                continue;
            }
        }

        files.push(path);
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    format!("{}…", text.chars().take(max).collect::<String>())
}
