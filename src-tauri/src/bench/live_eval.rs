//! Optional live LLM eval against the fixture repo. Not run in CI by default.
//!
//! ```bash
//! set MUSE_EVAL_LIVE=1
//! set MUSE_API_BASE=http://localhost:11434/v1
//! set MUSE_MODEL=llama3.2
//! cargo test live_eval --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture
//! ```

use std::path::Path;

use crate::agents::companion::TaskSpec;
use crate::agents::executor::{self, ExecutorStatus};
use crate::bench::fixture_root;
use crate::settings::AiSettings;

pub struct LiveScenario {
    pub id: &'static str,
    pub objective: &'static str,
    pub context: &'static str,
    pub expected_output: &'static str,
    /// Substring that should appear in executor summary or content on success.
    pub success_hint: &'static str,
}

pub fn live_scenarios() -> &'static [LiveScenario] {
    &[
        LiveScenario {
            id: "read_lib",
            objective: "Read src/lib.rs and confirm the add function exists.",
            context: "Rust calculator fixture crate.",
            expected_output: "Brief confirmation mentioning add.",
            success_hint: "add",
        },
        LiveScenario {
            id: "grep_subtract",
            objective: "Use grep to find where subtract is defined in src/.",
            context: "Rust calculator fixture crate.",
            expected_output: "File path and line mentioning subtract.",
            success_hint: "subtract",
        },
        LiveScenario {
            id: "list_src",
            objective: "List the files in the src/ directory.",
            context: "Rust calculator fixture crate.",
            expected_output: "Directory listing including lib.rs.",
            success_hint: "lib.rs",
        },
        LiveScenario {
            id: "read_cargo",
            objective: "Read Cargo.toml and state the package name.",
            context: "Rust calculator fixture crate.",
            expected_output: "Package name from the manifest.",
            success_hint: "rust-calc",
        },
        LiveScenario {
            id: "file_info",
            objective: "Use file_info on src/lib.rs and report whether it is a file.",
            context: "Rust calculator fixture crate.",
            expected_output: "Confirmation that lib.rs is a file.",
            success_hint: "lib.rs",
        },
    ]
}

pub fn settings_from_env(workspace: &Path) -> Option<AiSettings> {
    if std::env::var("MUSE_EVAL_LIVE").ok().as_deref() != Some("1") {
        return None;
    }

    let base_url = std::env::var("MUSE_API_BASE")
        .unwrap_or_else(|_| "http://localhost:11434/v1".into());
    let model = std::env::var("MUSE_MODEL").unwrap_or_else(|_| "llama3.2".into());

    Some(AiSettings {
        base_url,
        api_key: std::env::var("MUSE_API_KEY").unwrap_or_default(),
        companion_model: model.clone(),
        executor_model: model,
        workspace_path: Some(workspace.to_string_lossy().into()),
        verify_enabled: false,
        context_pack_enabled: true,
        mcp_enabled: false,
        task_queue_enabled: false,
        rag_enabled: false,
        ..AiSettings::default()
    })
}

#[tokio::test]
#[ignore = "requires MUSE_EVAL_LIVE=1 and a reachable LLM API"]
async fn live_eval_scenarios() {
    let workspace = fixture_root();
    let Some(settings) = settings_from_env(&workspace) else {
        eprintln!("Skipping live eval (set MUSE_EVAL_LIVE=1 to run)");
        return;
    };

    let mut failures = Vec::new();

    for scenario in live_scenarios() {
        eprintln!("live eval: {}", scenario.id);
        let task_spec = TaskSpec {
            objective: scenario.objective.into(),
            context: scenario.context.into(),
            constraints: vec![],
            expected_output: scenario.expected_output.into(),
        };

        match executor::execute(&settings, &task_spec, &[], None, None).await {
            Ok(result) if result.status == ExecutorStatus::Success => {
                let combined = format!("{} {}", result.summary, result.content).to_lowercase();
                if !combined.contains(&scenario.success_hint.to_lowercase()) {
                    failures.push(format!(
                        "{}: success but missing hint {:?}",
                        scenario.id, scenario.success_hint
                    ));
                }
            }
            Ok(result) => failures.push(format!(
                "{}: status {:?} — {}",
                scenario.id, result.status, result.summary
            )),
            Err(err) => failures.push(format!("{}: {err}", scenario.id)),
        }
    }

    assert!(
        failures.is_empty(),
        "live eval failures:\n{}",
        failures.join("\n")
    );
}
