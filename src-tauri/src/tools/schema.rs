use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
}

pub const TOOL_SPECS: &[ToolSpec] = &[
    ToolSpec {
        name: "list_dir",
        description: "List entries under a relative directory path",
        required: &[],
        optional: &["path"],
    },
    ToolSpec {
        name: "read_file",
        description: "Read a text file (path required)",
        required: &["path"],
        optional: &[],
    },
    ToolSpec {
        name: "write_file",
        description: "Create or overwrite a file (path, content required)",
        required: &["path", "content"],
        optional: &[],
    },
    ToolSpec {
        name: "edit_file",
        description: "Replace a unique old_string with new_string in a file",
        required: &["path", "old_string"],
        optional: &["new_string"],
    },
    ToolSpec {
        name: "grep",
        description: "Search for pattern in files under path",
        required: &["pattern"],
        optional: &["path"],
    },
    ToolSpec {
        name: "search_files",
        description: "Search filenames and content for query",
        required: &["query"],
        optional: &[],
    },
    ToolSpec {
        name: "file_info",
        description: "Metadata for a file or directory",
        required: &["path"],
        optional: &[],
    },
    ToolSpec {
        name: "create_dir",
        description: "Create a directory under workspace",
        required: &["path"],
        optional: &[],
    },
    ToolSpec {
        name: "delete_file",
        description: "Delete a file under workspace",
        required: &["path"],
        optional: &[],
    },
    ToolSpec {
        name: "run_command",
        description: "Run an allowlisted build/test command in the workspace root (e.g. npm test, cargo test). No shell pipes or chaining.",
        required: &["command"],
        optional: &[],
    },
    ToolSpec {
        name: "git_add",
        description: "Stage files for commit (path optional, default .)",
        required: &[],
        optional: &["path"],
    },
    ToolSpec {
        name: "git_commit",
        description: "Create a git commit with message",
        required: &["message"],
        optional: &[],
    },
    ToolSpec {
        name: "git_checkout_branch",
        description: "Create and switch to a new git branch",
        required: &["branch"],
        optional: &[],
    },
];

const FINAL_ANSWER_SPEC: ToolSpec = ToolSpec {
    name: "final_answer",
    description: "Finish the task with status, summary, and user-facing content",
    required: &["status", "summary", "content"],
    optional: &[],
};

pub fn is_workspace_tool(tool: &str) -> bool {
    TOOL_SPECS.iter().any(|spec| spec.name == tool)
}

pub fn validate_tool_call(tool: &str, args: &Value) -> Result<(), String> {
    if tool == FINAL_ANSWER_SPEC.name {
        return validate_args(&FINAL_ANSWER_SPEC, args);
    }

    let Some(spec) = TOOL_SPECS.iter().find(|s| s.name == tool) else {
        return Err(format!("unknown tool: {tool}"));
    };

    validate_args(spec, args)
}

fn validate_args(spec: &ToolSpec, args: &Value) -> Result<(), String> {
    let obj = args
        .as_object()
        .ok_or_else(|| "arguments must be a JSON object".to_string())?;

    for key in spec.required {
        if !obj.contains_key(*key) {
            return Err(format!("{} requires argument '{key}'", spec.name));
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn openai_tool_definitions() -> Vec<Value> {
    openai_tool_definitions_filtered(|_| true)
}

pub fn openai_tool_definitions_filtered(mut allow: impl FnMut(&str) -> bool) -> Vec<Value> {
    TOOL_SPECS
        .iter()
        .filter(|spec| allow(spec.name))
        .map(openai_tool_definition)
        .chain(std::iter::once(openai_tool_definition(&FINAL_ANSWER_SPEC)))
        .collect()
}

fn openai_tool_definition(spec: &ToolSpec) -> Value {
    let mut properties = serde_json::Map::new();
    for key in spec.required.iter().chain(spec.optional.iter()) {
        properties.insert(
            (*key).to_string(),
            json!({ "type": "string", "description": key }),
        );
    }

    json!({
        "type": "function",
        "function": {
            "name": spec.name,
            "description": spec.description,
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": spec.required,
            }
        }
    })
}

pub fn tools_prompt_section() -> String {
    let mut lines = vec!["## Tool schemas".to_string()];
    for spec in TOOL_SPECS {
        let required = if spec.required.is_empty() {
            "(optional: path)".to_string()
        } else {
            format!("required: {}", spec.required.join(", "))
        };
        lines.push(format!("- `{}`: {} — {}", spec.name, spec.description, required));
    }
    lines.push("- `final_answer`: finish with status, summary, content".into());
    lines.push(
        "Prefer native function calls when the API supports them. Fallback JSON: {\"action\":\"tool_call\",\"tool\":\"name\",\"arguments\":{...}} or {\"action\":\"final_answer\",\"status\":\"success|error|needs_clarification\",\"summary\":\"...\",\"content\":\"...\"}.".into(),
    );
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rejects_missing_required_arg() {
        let err = validate_tool_call("read_file", &json!({})).unwrap_err();
        assert!(err.contains("path"));
    }

    #[test]
    fn accepts_valid_tool_call() {
        validate_tool_call("read_file", &json!({"path":"src/main.rs"})).unwrap();
    }

    #[test]
    fn builds_openai_tool_definitions() {
        let tools = openai_tool_definitions();
        assert!(tools.len() > TOOL_SPECS.len());
        assert!(tools
            .iter()
            .any(|tool| tool["function"]["name"] == "final_answer"));
    }
}
