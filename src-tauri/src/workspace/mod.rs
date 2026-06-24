mod git;
mod paths;
mod symbols;

pub use git::{get_workspace_git_status, WorkspaceGitStatus};
pub use paths::{
    build_attachment_context, search_workspace_paths, sort_files_for_index, MessageAttachment,
    WorkspacePathHit,
};
pub use symbols::{search_workspace_symbols, WorkspaceSymbolHit};
