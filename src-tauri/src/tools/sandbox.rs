use std::path::{Component, Path, PathBuf};

pub const MAX_READ_BYTES: usize = 512 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("workspace is not configured")]
    NotConfigured,
    #[error("path must be relative to the workspace")]
    AbsolutePath,
    #[error("path escapes the workspace")]
    OutsideWorkspace,
    #[error("invalid path")]
    InvalidPath,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct WorkspaceSandbox {
    root: PathBuf,
}

impl WorkspaceSandbox {
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, SandboxError> {
        let root = root.as_ref();
        if !root.exists() {
            return Err(SandboxError::InvalidPath);
        }

        let canonical = root.canonicalize()?;
        Ok(Self { root: canonical })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve(&self, relative: &str) -> Result<PathBuf, SandboxError> {
        let trimmed = relative.trim();
        if trimmed.is_empty() {
            return self.resolve_components(Path::new("."));
        }

        let path = Path::new(trimmed);
        if path.is_absolute() {
            return Err(SandboxError::AbsolutePath);
        }

        self.resolve_components(path)
    }

    fn resolve_components(&self, path: &Path) -> Result<PathBuf, SandboxError> {
        let mut joined = self.root.clone();

        for component in path.components() {
            match component {
                Component::CurDir => {}
                Component::Normal(part) => joined.push(part),
                Component::ParentDir => return Err(SandboxError::OutsideWorkspace),
                Component::RootDir | Component::Prefix(_) => {
                    return Err(SandboxError::AbsolutePath);
                }
            }
        }

        self.ensure_inside(&joined)
    }

    fn ensure_inside(&self, path: &Path) -> Result<PathBuf, SandboxError> {
        if path.exists() {
            let canonical = path.canonicalize()?;
            if !canonical.starts_with(&self.root) {
                return Err(SandboxError::OutsideWorkspace);
            }
            return Ok(canonical);
        }

        // New paths: walk up to nearest existing ancestor, canonicalize, re-append tail.
        let mut tail: Vec<PathBuf> = Vec::new();
        let mut current = path.to_path_buf();

        loop {
            if current.exists() {
                let canonical = current.canonicalize()?;
                if !canonical.starts_with(&self.root) {
                    return Err(SandboxError::OutsideWorkspace);
                }
                let mut resolved = canonical;
                for segment in tail.into_iter().rev() {
                    resolved.push(segment);
                }
                return Ok(resolved);
            }

            match current.file_name().map(|n| n.to_owned()) {
                Some(name) => {
                    tail.push(PathBuf::from(name));
                    current.pop();
                }
                None => break,
            }
        }

        if path.starts_with(&self.root) {
            return Ok(path.to_path_buf());
        }

        let candidate = self.root.join(path);
        if candidate.starts_with(&self.root) {
            Ok(candidate)
        } else {
            Err(SandboxError::OutsideWorkspace)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_workspace() -> (PathBuf, WorkspaceSandbox) {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("muse-sandbox-test-{n}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("readme.txt"), "hello").unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        let sandbox = WorkspaceSandbox::from_root(&root).unwrap();
        (root, sandbox)
    }

    #[test]
    fn resolves_relative_paths() {
        let (_root, sandbox) = temp_workspace();
        let resolved = sandbox.resolve("src/main.rs").unwrap();
        assert!(resolved.ends_with("main.rs"));
        assert!(resolved.starts_with(sandbox.root()));
    }

    #[test]
    fn blocks_parent_traversal() {
        let (_root, sandbox) = temp_workspace();
        assert!(matches!(
            sandbox.resolve("../outside"),
            Err(SandboxError::OutsideWorkspace)
        ));
    }

    #[test]
    fn blocks_absolute_paths() {
        let (_root, sandbox) = temp_workspace();
        #[cfg(windows)]
        let absolute = "C:\\Windows\\System32";
        #[cfg(not(windows))]
        let absolute = "/etc/passwd";
        assert!(matches!(sandbox.resolve(absolute), Err(SandboxError::AbsolutePath)));
    }

    #[test]
    fn resolves_new_file_under_workspace() {
        let (_root, sandbox) = temp_workspace();
        let resolved = sandbox.resolve("src/new.txt").unwrap();
        assert!(resolved.starts_with(sandbox.root()));
        assert!(resolved.ends_with("new.txt"));
    }
}
