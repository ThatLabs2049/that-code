use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub index: usize,
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
}

pub fn parse_diff_hunks(diff: &str) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();
    let mut current_old = Vec::new();
    let mut current_new = Vec::new();
    let mut in_hunk = false;

    for line in diff.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        }

        if let Some(stripped) = line.strip_prefix('-') {
            if in_hunk && current_new.is_empty() && current_old.is_empty() {
                // continue same hunk
            } else if in_hunk && !current_old.is_empty() && !current_new.is_empty() {
                push_hunk(&mut hunks, &mut current_old, &mut current_new);
            }
            in_hunk = true;
            current_old.push(stripped.to_string());
            continue;
        }

        if let Some(stripped) = line.strip_prefix('+') {
            in_hunk = true;
            current_new.push(stripped.to_string());
            continue;
        }

        if in_hunk && (!current_old.is_empty() || !current_new.is_empty()) {
            push_hunk(&mut hunks, &mut current_old, &mut current_new);
            in_hunk = false;
        }
    }

    if !current_old.is_empty() || !current_new.is_empty() {
        push_hunk(&mut hunks, &mut current_old, &mut current_new);
    }

    hunks
}

fn push_hunk(hunks: &mut Vec<DiffHunk>, old_lines: &mut Vec<String>, new_lines: &mut Vec<String>) {
    if old_lines.is_empty() && new_lines.is_empty() {
        return;
    }
    hunks.push(DiffHunk {
        index: hunks.len(),
        old_lines: std::mem::take(old_lines),
        new_lines: std::mem::take(new_lines),
    });
}

pub fn apply_hunk_selection(
    before: &str,
    after: &str,
    diff: &str,
    rejected_hunk_indices: &[usize],
) -> Result<String, String> {
    let hunks = parse_diff_hunks(diff);
    if hunks.is_empty() || rejected_hunk_indices.is_empty() {
        return Ok(after.to_string());
    }

    let reject: std::collections::HashSet<usize> = rejected_hunk_indices.iter().copied().collect();
    let mut content = after.to_string();

    for hunk in hunks.iter().rev() {
        if !reject.contains(&hunk.index) {
            continue;
        }
        if let Some(next) = replace_once(&content, &hunk.new_lines, &hunk.old_lines) {
            content = next;
        } else if hunk.new_lines.is_empty() && !hunk.old_lines.is_empty() {
            content = format!("{}\n{}", content, hunk.old_lines.join("\n"));
        } else {
            return Err(format!("could not reject hunk {}", hunk.index));
        }
    }

    if content.trim().is_empty() && !before.trim().is_empty() {
        return Ok(before.to_string());
    }

    Ok(content)
}

fn replace_once(content: &str, from: &[String], to: &[String]) -> Option<String> {
    if from.is_empty() {
        return None;
    }

    let lines: Vec<&str> = content.lines().collect();
    let from_refs: Vec<&str> = from.iter().map(String::as_str).collect();

    for start in 0..=lines.len().saturating_sub(from.len()) {
        if lines[start..start + from.len()] == from_refs[..] {
            let mut result: Vec<&str> = lines[..start].to_vec();
            result.extend(to.iter().map(String::as_str));
            result.extend(&lines[start + from.len()..]);
            return Some(result.join("\n"));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_hunks() {
        let diff = "--- a/foo.rs\n+++ b/foo.rs\n-old\n+new\n";
        let hunks = parse_diff_hunks(diff);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].old_lines, vec!["old".to_string()]);
        assert_eq!(hunks[0].new_lines, vec!["new".to_string()]);
    }
}
