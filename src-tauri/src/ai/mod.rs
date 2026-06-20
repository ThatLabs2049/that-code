pub mod client;
pub mod prompts;
pub mod types;

pub use client::{
    chat_completion, chat_completion_executor, chat_completion_stream, test_connection,
    tools_api_unsupported, ClientError,
};
pub use types::{
    AiTestResult, ChatCompletionRequest, ChatMessage, EmbeddingTestResult, ToolCall,
};
