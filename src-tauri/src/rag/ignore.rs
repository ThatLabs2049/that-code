use std::fs;
use std::path::Path;

const BUILTIN_IGNORE_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    "__pycache__",
    ".next",
    "coverage",
    ".turbo",
    ".cache",
    "out",
];

pub fn load_ignore_patterns(workspace: &Path) -> Vec<String> {
    let mut patterns: Vec<String> = BUILTIN_IGNORE_DIRS
        .iter()
        .map(|name| name.to_string())
        .collect();

    for file_name in [".thatcodeignore", ".gitignore"] {
        let path = workspace.join(file_name);
        if let Ok(text) = fs::read_to_string(&path) {
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                patterns.push(trimmed.to_string());
            }
        }
    }

    patterns
}

pub fn should_ignore_name(name: &str, patterns: &[String]) -> bool {
    for pattern in patterns {
        if pattern_matches_name(name, pattern) {
            return true;
        }
    }
    false
}

pub fn should_ignore_relative_path(relative: &str, patterns: &[String]) -> bool {
    let normalized = relative.replace('\\', "/");
    for pattern in patterns {
        if pattern_matches_path(&normalized, pattern) {
            return true;
        }
    }
    false
}

fn pattern_matches_name(name: &str, pattern: &str) -> bool {
    if pattern.contains('/') {
        return false;
    }
    pattern_matches_segment(name, pattern)
}

fn pattern_matches_path(path: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        return glob_match(path, pattern);
    }

    if path == pattern {
        return true;
    }

    if let Some(suffix) = pattern.strip_prefix('/') {
        return path.ends_with(suffix.trim_start_matches('/'));
    }

    path.split('/').any(|segment| segment == pattern)
        || path.starts_with(&format!("{pattern}/"))
        || path.contains(&format!("/{pattern}/"))
}

fn pattern_matches_segment(segment: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        return glob_match(segment, pattern);
    }
    segment == pattern
}

fn glob_match(text: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }
    if let Some(middle) = pattern.strip_prefix('*').and_then(|s| s.strip_suffix('*')) {
        return text.contains(middle);
    }
    text == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_node_modules_by_name() {
        assert!(should_ignore_name("node_modules", &["node_modules".into()]));
    }

    #[test]
    fn glob_suffix_matches() {
        assert!(should_ignore_relative_path("logs/app.log", &["*.log".into()]));
    }
}
