use crate::agents::task::is_file_or_code_task;
use crate::settings::AiSettings;
use serde::{Deserialize, Serialize};

/// User-selected cost/quality preset. `Auto` resolves from the message heuristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentTier {
    #[default]
    Auto,
    Quick,
    Standard,
    Deep,
    Explain,
}

impl AgentTier {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "quick" => Self::Quick,
            "standard" => Self::Standard,
            "deep" => Self::Deep,
            "explain" => Self::Explain,
            _ => Self::Auto,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Quick => "quick",
            Self::Standard => "standard",
            Self::Deep => "deep",
            Self::Explain => "explain",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectiveMode {
    Quick,
    Standard,
    Deep,
    Explain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPhase {
    Scout,
    Editor,
}

pub const READ_ONLY_TOOLS: &[&str] = &[
    "list_dir",
    "read_file",
    "grep",
    "search_files",
    "file_info",
];

pub const MUTATING_TOOLS: &[&str] = &[
    "write_file",
    "edit_file",
    "delete_file",
    "create_dir",
    "run_command",
    "git_add",
    "git_commit",
    "git_checkout_branch",
];

/// Resolved runtime configuration for a single agent run.
#[derive(Debug, Clone)]
pub struct RunConfig {
    pub tier: AgentTier,
    pub(crate) mode: EffectiveMode,
    pub phase: AgentPhase,
    pub fast_model: String,
    pub strong_model: String,
    pub temperature: f32,
    pub auto_escalate: bool,
    pub verify_enabled: bool,
    pub context_pack_enabled: bool,
    pub rag_enabled: bool,
    pub max_scout_steps: usize,
    pub max_editor_steps: usize,
    pub scout_read_only: bool,
}

pub fn is_implementation_task(user_message: &str) -> bool {
    is_file_or_code_task(user_message) && !is_explain_task(user_message)
}

impl RunConfig {
    pub fn from_settings(settings: &AiSettings, tier: AgentTier, user_message: &str) -> Self {
        let mode = resolve_mode(tier, user_message);
        let fast_model = settings.effective_fast_model().to_string();
        let strong_model = settings.effective_strong_model().to_string();
        let implement = is_implementation_task(user_message);

        match mode {
            EffectiveMode::Quick => Self {
                tier,
                mode,
                phase: AgentPhase::Editor,
                fast_model,
                strong_model,
                temperature: settings.agent_temperature,
                auto_escalate: false,
                verify_enabled: settings.verify_enabled,
                context_pack_enabled: settings.context_pack_enabled,
                rag_enabled: false,
                max_scout_steps: 0,
                max_editor_steps: 18,
                scout_read_only: false,
            },
            EffectiveMode::Standard => Self {
                tier,
                mode,
                phase: if implement {
                    AgentPhase::Editor
                } else {
                    AgentPhase::Scout
                },
                fast_model,
                strong_model,
                temperature: settings.agent_temperature,
                auto_escalate: settings.auto_escalate,
                verify_enabled: settings.verify_enabled,
                context_pack_enabled: settings.context_pack_enabled,
                rag_enabled: settings.rag_enabled,
                max_scout_steps: if implement { 0 } else { 8 },
                max_editor_steps: 22,
                scout_read_only: !implement,
            },
            EffectiveMode::Deep => Self {
                tier,
                mode,
                phase: AgentPhase::Editor,
                fast_model,
                strong_model,
                temperature: settings.agent_temperature,
                auto_escalate: false,
                verify_enabled: true,
                context_pack_enabled: true,
                rag_enabled: settings.rag_enabled,
                max_scout_steps: 0,
                max_editor_steps: 35,
                scout_read_only: false,
            },
            EffectiveMode::Explain => Self {
                tier,
                mode,
                phase: AgentPhase::Scout,
                fast_model,
                strong_model,
                temperature: settings.agent_temperature.min(0.4),
                auto_escalate: false,
                verify_enabled: false,
                context_pack_enabled: settings.context_pack_enabled,
                rag_enabled: settings.rag_enabled,
                max_scout_steps: 8,
                max_editor_steps: 0,
                scout_read_only: true,
            },
        }
    }

    pub fn phase_label(&self, phase: AgentPhase) -> &'static str {
        match phase {
            AgentPhase::Scout => "scout",
            AgentPhase::Editor => "editor",
        }
    }

    pub fn model_for_phase(&self, phase: AgentPhase) -> &str {
        match self.mode {
            EffectiveMode::Quick | EffectiveMode::Explain => &self.fast_model,
            EffectiveMode::Deep => &self.strong_model,
            EffectiveMode::Standard => match phase {
                AgentPhase::Scout => &self.fast_model,
                AgentPhase::Editor => &self.strong_model,
            },
        }
    }

    pub fn max_steps_for_phase(&self, phase: AgentPhase) -> usize {
        match phase {
            AgentPhase::Scout => self.max_scout_steps,
            AgentPhase::Editor => self.max_editor_steps,
        }
    }

    pub fn is_tool_allowed(&self, phase: AgentPhase, tool: &str) -> bool {
        if tool == "final_answer" {
            return true;
        }

        if self.mode == EffectiveMode::Explain {
            return READ_ONLY_TOOLS.contains(&tool);
        }

        if phase == AgentPhase::Scout && self.scout_read_only {
            return READ_ONLY_TOOLS.contains(&tool);
        }

        if crate::mcp::is_mcp_tool(tool) {
            return self.mode != EffectiveMode::Explain
                && !(phase == AgentPhase::Scout && self.scout_read_only);
        }

        crate::tools::is_workspace_tool(tool)
    }

    pub fn should_escalate_on_tool(&self, phase: AgentPhase, tool: &str) -> bool {
        phase == AgentPhase::Scout
            && self.scout_read_only
            && self.auto_escalate
            && self.mode == EffectiveMode::Standard
            && (MUTATING_TOOLS.contains(&tool) || tool == "run_command")
    }

    pub fn should_escalate_on_scout_exhausted(&self, phase: AgentPhase, scout_steps: usize) -> bool {
        phase == AgentPhase::Scout
            && self.mode == EffectiveMode::Standard
            && self.auto_escalate
            && scout_steps >= self.max_scout_steps
            && self.max_editor_steps > 0
    }
}

/// Skip read-only scout when the workspace is empty but the user wants files created.
pub fn maybe_promote_to_editor_for_greenfield(
    settings: &AiSettings,
    run_config: &mut RunConfig,
    objective: &str,
) {
    if run_config.phase != AgentPhase::Scout || !run_config.scout_read_only {
        return;
    }
    if run_config.mode != EffectiveMode::Standard {
        return;
    }
    if !crate::agents::task::is_file_or_code_task(objective) {
        return;
    }
    let Some(workspace) = settings.workspace_path.as_ref() else {
        return;
    };
    if !crate::context::is_workspace_empty(std::path::Path::new(workspace)) {
        return;
    }

    run_config.phase = AgentPhase::Editor;
    run_config.max_scout_steps = 0;
}

pub fn should_escalate_after_empty_listing(
    run_config: &RunConfig,
    tool_name: &str,
    tool_result: &crate::tools::ToolResult,
    objective: &str,
) -> bool {
    run_config.phase == AgentPhase::Scout
        && run_config.mode == EffectiveMode::Standard
        && run_config.auto_escalate
        && tool_name == "list_dir"
        && tool_result.ok
        && tool_result.output.contains("empty directory")
        && crate::agents::task::is_file_or_code_task(objective)
}

pub fn should_escalate_after_read_failures(
    run_config: &RunConfig,
    activity_log: &[crate::agents::executor::ActivityStep],
    objective: &str,
) -> bool {
    if run_config.phase != AgentPhase::Scout
        || run_config.mode != EffectiveMode::Standard
        || !run_config.auto_escalate
        || !crate::agents::task::is_file_or_code_task(objective)
    {
        return false;
    }

    let failures = activity_log
        .iter()
        .rev()
        .take(10)
        .filter(|step| {
            step.step.starts_with("tool:read_file")
                && step.detail.contains("path is not a file")
        })
        .count();

    failures >= 3
}

pub fn should_escalate_after_exploration_loop(
    run_config: &RunConfig,
    activity_log: &[crate::agents::executor::ActivityStep],
    objective: &str,
) -> bool {
    if run_config.phase != AgentPhase::Scout
        || !run_config.scout_read_only
        || !run_config.auto_escalate
        || run_config.mode != EffectiveMode::Standard
        || !is_implementation_task(objective)
    {
        return false;
    }

    let tool_steps = activity_log
        .iter()
        .filter(|step| step.step.starts_with("tool:"))
        .count();

    tool_steps >= 5
}

fn resolve_mode(tier: AgentTier, user_message: &str) -> EffectiveMode {
    match tier {
        AgentTier::Quick => EffectiveMode::Quick,
        AgentTier::Standard => EffectiveMode::Standard,
        AgentTier::Deep => EffectiveMode::Deep,
        AgentTier::Explain => EffectiveMode::Explain,
        AgentTier::Auto => {
            if is_explain_task(user_message) {
                EffectiveMode::Explain
            } else if is_deep_task(user_message) {
                EffectiveMode::Deep
            } else {
                EffectiveMode::Standard
            }
        }
    }
}

pub fn is_explain_task(user_message: &str) -> bool {
    let lower = user_message.to_lowercase();
    const EXPLAIN: &[&str] = &[
        "explain",
        "what does",
        "what is",
        "how does",
        "how do",
        "describe",
        "walk me through",
        "review this",
        "code review",
        "understand",
        "why does",
        "tell me about",
        "چیه",
        "چطور",
        "توضیح",
        "بررسی",
    ];
    const MUTATE: &[&str] = &[
        "fix",
        "implement",
        "write",
        "create",
        "add ",
        "refactor",
        "delete",
        "update",
        "change ",
        "patch",
        "commit",
        "درست کن",
        "بساز",
        "اضافه کن",
    ];

    EXPLAIN.iter().any(|kw| lower.contains(kw)) && !MUTATE.iter().any(|kw| lower.contains(kw))
}

pub fn is_deep_task(user_message: &str) -> bool {
    let lower = user_message.to_lowercase();
    const DEEP: &[&str] = &[
        "refactor",
        "restructure",
        "migrate",
        "migration",
        "architecture",
        "redesign",
        "across the",
        "multiple files",
        "entire codebase",
        "whole project",
        "large refactor",
        "ریفکتور",
        "بازنویسی",
        "مهاجرت",
    ];
    DEEP.iter().any(|kw| lower.contains(kw))
}

pub fn build_scout_briefing(activity_log: &[crate::agents::executor::ActivityStep], task_spec: &crate::agents::task::TaskSpec) -> String {
    let reads = activity_log
        .iter()
        .filter(|step| step.step.starts_with("tool:read_file") || step.step.starts_with("tool:grep"))
        .map(|step| format!("- {}", step.detail))
        .collect::<Vec<_>>();

    let mut briefing = String::from("Scout phase complete. Continue with edits and verification.\n\n");
    briefing.push_str(&format!("## Original objective\n{}\n\n", task_spec.objective));

    if !reads.is_empty() {
        briefing.push_str("## Scout findings\n");
        for line in reads.iter().take(12) {
            briefing.push_str(line);
            briefing.push('\n');
        }
        briefing.push('\n');
    }

    briefing.push_str("Implement the objective. Use edit_file for surgical changes. Run verify commands when appropriate.");
    briefing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_resolves_explain() {
        let settings = AiSettings::default();
        let config = RunConfig::from_settings(&settings, AgentTier::Auto, "explain how auth works");
        assert_eq!(config.mode, EffectiveMode::Explain);
        assert!(!config.is_tool_allowed(AgentPhase::Scout, "write_file"));
        assert!(config.is_tool_allowed(AgentPhase::Scout, "read_file"));
    }

    #[test]
    fn auto_resolves_deep() {
        let settings = AiSettings::default();
        let config = RunConfig::from_settings(
            &settings,
            AgentTier::Auto,
            "refactor the entire module structure",
        );
        assert_eq!(config.mode, EffectiveMode::Deep);
        assert_eq!(config.phase, AgentPhase::Editor);
    }

    #[test]
    fn auto_implementation_starts_in_editor() {
        let settings = AiSettings::default();
        let config = RunConfig::from_settings(&settings, AgentTier::Auto, "fix the bug");
        assert_eq!(config.phase, AgentPhase::Editor);
        assert!(!config.scout_read_only);
        assert!(config.is_tool_allowed(AgentPhase::Editor, "edit_file"));
    }

    #[test]
    fn auto_explore_escalates_on_mutate() {
        let settings = AiSettings::default();
        let config = RunConfig::from_settings(
            &settings,
            AgentTier::Auto,
            "summarize the design patterns used",
        );
        assert_eq!(config.phase, AgentPhase::Scout);
        assert!(config.should_escalate_on_tool(AgentPhase::Scout, "edit_file"));
    }

    #[test]
    fn quick_uses_fast_model_only() {
        let settings = AiSettings::default();
        let config = RunConfig::from_settings(&settings, AgentTier::Quick, "fix typo");
        assert_eq!(config.phase, AgentPhase::Editor);
        assert_eq!(config.model_for_phase(AgentPhase::Editor), settings.effective_fast_model());
    }

    #[test]
    fn greenfield_empty_workspace_starts_in_editor() {
        let root = std::env::temp_dir().join(format!(
            "thatcode-greenfield-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let settings = AiSettings {
            workspace_path: Some(root.to_string_lossy().into()),
            ..AiSettings::default()
        };
        let config =
            RunConfig::from_settings(&settings, AgentTier::Auto, "make a login page");
        assert_eq!(config.phase, AgentPhase::Editor);
        assert!(!config.scout_read_only);
        let _ = std::fs::remove_dir_all(root);
    }
}
