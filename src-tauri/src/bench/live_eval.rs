//! Optional live LLM eval against the fixture repo. Not run in CI by default.
//!
//! ```bash
//! set THATCODE_EVAL_LIVE=1
//! set THATCODE_API_BASE=http://localhost:11434/v1
//! set THATCODE_MODEL=llama3.2
//! cargo test live_eval --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture
//! ```
//! Legacy env names `MUSE_EVAL_LIVE`, `MUSE_API_BASE`, `MUSE_MODEL`, `MUSE_API_KEY` still work.

use std::path::Path;

use crate::agents::task::TaskSpec;
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

fn env_var(primary: &str, legacy: &str) -> Option<String> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(legacy).ok())
}

pub fn settings_from_env(workspace: &Path) -> Option<AiSettings> {
    let live = env_var("THATCODE_EVAL_LIVE", "MUSE_EVAL_LIVE");
    if live.as_deref() != Some("1") {
        return None;
    }

    let base_url = env_var("THATCODE_API_BASE", "MUSE_API_BASE")
        .unwrap_or_else(|| "http://localhost:11434/v1".into());
    let model = env_var("THATCODE_MODEL", "MUSE_MODEL").unwrap_or_else(|| "llama3.2".into());

    Some(AiSettings {
        base_url,
        api_key: env_var("THATCODE_API_KEY", "MUSE_API_KEY").unwrap_or_default(),
        agent_model: model.clone(),
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
#[ignore = "requires THATCODE_EVAL_LIVE=1 and a reachable LLM API"]
async fn live_eval_scenarios() {
    let workspace = fixture_root();
    let Some(settings) = settings_from_env(&workspace) else {
        eprintln!("Skipping live eval (set THATCODE_EVAL_LIVE=1 to run)");
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

        let mut run_config = crate::agents::profile::RunConfig::from_settings(
            &settings,
            crate::agents::profile::AgentTier::Auto,
            scenario.objective,
        );

        match executor::execute(&settings, &mut run_config, &task_spec, &[], None, None).await {
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
