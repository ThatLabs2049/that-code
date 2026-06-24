mod ann;
mod chunk;
mod embeddings;
mod ignore;
mod index;
mod store;

pub use ann::RagAnnIndex;
pub use ignore::{load_ignore_patterns, should_ignore_name, should_ignore_relative_path};
pub use index::{
    build_incremental_index, indexed_path_times, persist_incremental_index,
    persist_workspace_index, retrieve_chunks_for_query, status, test_embedding,
    build_workspace_index_with_progress, IndexProgress, RagStatus, RetrievedChunk,
};
pub use store::list_embedding_records;
