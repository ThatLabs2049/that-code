mod command;
mod handlers;
mod sandbox;
mod schema;
mod verify;

pub(crate) use command::parse_argv_line;
pub use handlers::{execute_tool, tool_context_from_settings, ToolContext, ToolResult};
pub use sandbox::{truncate_at_byte_boundary, SandboxError, WorkspaceSandbox};
pub use schema::{
    is_workspace_tool, openai_tool_definitions_filtered, tools_prompt_section,
    validate_tool_call,
};
pub use verify::{resolve_verify_command, run_verify, MAX_VERIFY_RETRIES};

pub async fn execute_tool_async(
    ctx: ToolContext,
    tool: String,
    args: serde_json::Value,
) -> ToolResult {
    match tokio::task::spawn_blocking(move || execute_tool(&ctx, &tool, &args)).await {
        Ok(result) => result,
        Err(err) => ToolResult {
            ok: false,
            output: String::new(),
            error: Some(format!("tool task failed: {err}")),
        },
    }
}

#[cfg(test)]
pub use verify::infer_verify_command;