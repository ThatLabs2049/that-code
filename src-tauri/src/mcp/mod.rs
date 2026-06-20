use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde_json::{json, Value};

static MCP_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct McpTool {
    pub full_name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct McpSession {
    child: Mutex<std::process::Child>,
}

impl McpSession {
    pub fn spawn(command_line: &str) -> Result<Self, String> {
        let command_line = command_line.trim();
        if command_line.is_empty() {
            return Err("MCP server command is empty".into());
        }

        let child = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", command_line])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
        } else {
            Command::new("sh")
                .args(["-lc", command_line])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
        }
        .map_err(|err| format!("failed to spawn MCP server: {err}"))?;

        let session = Self {
            child: Mutex::new(child),
        };

        session.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "muse", "version": env!("CARGO_PKG_VERSION") }
            }),
        )?;
        session.notify("notifications/initialized", json!({}))?;
        Ok(session)
    }

    pub fn list_tools(&self) -> Result<Vec<McpTool>, String> {
        let response = self.request("tools/list", json!({}))?;
        let tools = response
            .get("tools")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(tools
            .into_iter()
            .filter_map(|tool| {
                let name = tool.get("name")?.as_str()?.to_string();
                Some(McpTool {
                    full_name: format!("mcp_{name}"),
                    description: tool
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MCP tool")
                        .to_string(),
                    input_schema: tool
                        .get("inputSchema")
                        .cloned()
                        .unwrap_or_else(|| json!({"type":"object","properties":{}})),
                })
            })
            .collect())
    }

    pub fn call_tool(&self, name: &str, arguments: &Value) -> Result<String, String> {
        let mcp_name = name.strip_prefix("mcp_").unwrap_or(name);
        let response = self.request(
            "tools/call",
            json!({ "name": mcp_name, "arguments": arguments }),
        )?;

        if let Some(content) = response.get("content").and_then(|v| v.as_array()) {
            let text = content
                .iter()
                .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n");
            if !text.is_empty() {
                return Ok(text);
            }
        }

        Ok(response.to_string())
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), String> {
        let mut child = self.child.lock().map_err(|_| "MCP lock poisoned".to_string())?;
        let stdin = child.stdin.as_mut().ok_or("MCP stdin unavailable")?;
        let payload = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        writeln!(stdin, "{payload}").map_err(|err| err.to_string())?;
        stdin.flush().map_err(|err| err.to_string())
    }

    fn request(&self, method: &str, params: Value) -> Result<Value, String> {
        let id = MCP_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        {
            let mut child = self.child.lock().map_err(|_| "MCP lock poisoned".to_string())?;
            let stdin = child.stdin.as_mut().ok_or("MCP stdin unavailable")?;
            writeln!(stdin, "{payload}").map_err(|err| err.to_string())?;
            stdin.flush().map_err(|err| err.to_string())?;
        }

        let mut child = self.child.lock().map_err(|_| "MCP lock poisoned".to_string())?;
        let stdout = child.stdout.as_mut().ok_or("MCP stdout unavailable")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        for _ in 0..50 {
            line.clear();
            reader.read_line(&mut line).map_err(|err| err.to_string())?;
            if line.trim().is_empty() {
                continue;
            }
            let parsed: Value = serde_json::from_str(line.trim())
                .map_err(|err| format!("invalid MCP response: {err}"))?;
            if parsed.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(error) = parsed.get("error") {
                    return Err(error.to_string());
                }
                return parsed
                    .get("result")
                    .cloned()
                    .ok_or_else(|| "MCP response missing result".into());
            }
        }
        Err("timed out waiting for MCP response".into())
    }
}

pub fn openai_tools_from_mcp(tools: &[McpTool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.full_name,
                    "description": tool.description,
                    "parameters": tool.input_schema
                }
            })
        })
        .collect()
}

pub fn is_mcp_tool(name: &str) -> bool {
    name.starts_with("mcp_")
}
