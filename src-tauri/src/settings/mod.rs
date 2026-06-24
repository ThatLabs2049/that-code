use serde::{Deserialize, Serialize};

pub const AI_SETTINGS_KEY: &str = "ai_settings";

pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_AGENT_MODEL: &str = "gpt-5-mini";
pub const DEFAULT_STRONG_MODEL: &str = "gpt-5";
pub const DEFAULT_AGENT_TEMPERATURE: f32 = 0.3;
pub const DEFAULT_AUTO_ESCALATE: bool = true;
pub const DEFAULT_AGENT_TIER: &str = "auto";
pub const DEFAULT_EXECUTOR_VISIBILITY: bool = false;
pub const DEFAULT_UI_LOCALE: &str = "en";
pub const DEFAULT_ALLOW_FILE_OVERWRITES: bool = true;
const DEFAULT_PROJECT_RULES_ENABLED: bool = true;
const DEFAULT_PLAN_BEFORE_EDIT: bool = false;
const DEFAULT_EDITOR_OPEN_URL: &str = "vscode://file/{path}";

pub const DEFAULT_RAG_ENABLED: bool = false;
pub const DEFAULT_EMBEDDING_BASE_URL: &str = "http://localhost:11434/v1";
pub const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text";
pub const DEFAULT_RAG_TOP_K: u32 = 8;
pub const DEFAULT_VERIFY_ENABLED: bool = true;
pub const DEFAULT_CONTEXT_PACK_ENABLED: bool = true;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiSettings {
    pub base_url: String,
    pub api_key: String,
    #[serde(
        alias = "companion_model",
        alias = "executor_model",
        default = "default_agent_model"
    )]
    pub agent_model: String,
    #[serde(
        alias = "companion_temperature",
        alias = "executor_temperature",
        default = "default_agent_temperature"
    )]
    pub agent_temperature: f32,
    #[serde(default = "default_fast_model")]
    pub fast_model: String,
    #[serde(default = "default_strong_model")]
    pub strong_model: String,
    #[serde(default = "default_auto_escalate")]
    pub auto_escalate: bool,
    #[serde(default = "default_agent_tier")]
    pub default_agent_tier: String,
    #[serde(default = "default_executor_visibility")]
    pub executor_visibility: bool,
    #[serde(default = "default_ui_locale")]
    pub ui_locale: String,
    #[serde(default)]
    pub workspace_path: Option<String>,
    #[serde(default = "default_allow_file_overwrites")]
    pub allow_file_overwrites: bool,
    #[serde(default = "default_rag_enabled")]
    pub rag_enabled: bool,
    #[serde(default = "default_embedding_base_url")]
    pub embedding_base_url: String,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
    #[serde(default)]
    pub embedding_api_key: String,
    #[serde(default = "default_rag_top_k")]
    pub rag_top_k: u32,
    #[serde(default = "default_verify_enabled")]
    pub verify_enabled: bool,
    #[serde(default)]
    pub verify_command: Option<String>,
    #[serde(default = "default_context_pack_enabled")]
    pub context_pack_enabled: bool,
    #[serde(default)]
    pub command_allowlist_extra: Vec<String>,
    #[serde(default = "default_project_rules_enabled")]
    pub project_rules_enabled: bool,
    #[serde(default = "default_plan_before_edit")]
    pub plan_before_edit: bool,
    #[serde(default = "default_editor_open_url")]
    pub editor_open_url: String,
    #[serde(default)]
    pub rag_auto_index: bool,
    #[serde(default = "default_task_queue_enabled")]
    pub task_queue_enabled: bool,
    #[serde(default)]
    pub mcp_enabled: bool,
    #[serde(default)]
    pub mcp_server_command: Option<String>,
    #[serde(default)]
    pub onboarding_completed: bool,
    #[serde(default = "default_theme_preference")]
    pub theme_preference: String,
}

fn default_fast_model() -> String {
    DEFAULT_AGENT_MODEL.into()
}

fn default_strong_model() -> String {
    DEFAULT_STRONG_MODEL.into()
}

fn default_auto_escalate() -> bool {
    DEFAULT_AUTO_ESCALATE
}

fn default_agent_tier() -> String {
    DEFAULT_AGENT_TIER.into()
}

fn default_theme_preference() -> String {
    "system".into()
}

fn default_agent_model() -> String {
    DEFAULT_AGENT_MODEL.into()
}

fn default_agent_temperature() -> f32 {
    DEFAULT_AGENT_TEMPERATURE
}

fn default_task_queue_enabled() -> bool {
    true
}

fn default_plan_before_edit() -> bool {
    DEFAULT_PLAN_BEFORE_EDIT
}

fn default_editor_open_url() -> String {
    DEFAULT_EDITOR_OPEN_URL.into()
}

fn default_project_rules_enabled() -> bool {
    DEFAULT_PROJECT_RULES_ENABLED
}

fn default_rag_enabled() -> bool {
    DEFAULT_RAG_ENABLED
}

fn default_embedding_base_url() -> String {
    DEFAULT_EMBEDDING_BASE_URL.into()
}

fn default_embedding_model() -> String {
    DEFAULT_EMBEDDING_MODEL.into()
}

fn default_context_pack_enabled() -> bool {
    DEFAULT_CONTEXT_PACK_ENABLED
}

fn default_verify_enabled() -> bool {
    DEFAULT_VERIFY_ENABLED
}

fn default_rag_top_k() -> u32 {
    DEFAULT_RAG_TOP_K
}

fn default_allow_file_overwrites() -> bool {
    DEFAULT_ALLOW_FILE_OVERWRITES
}

fn default_ui_locale() -> String {
    DEFAULT_UI_LOCALE.into()
}

fn default_executor_visibility() -> bool {
    DEFAULT_EXECUTOR_VISIBILITY
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.into(),
            api_key: String::new(),
            agent_model: DEFAULT_AGENT_MODEL.into(),
            agent_temperature: DEFAULT_AGENT_TEMPERATURE,
            fast_model: DEFAULT_AGENT_MODEL.into(),
            strong_model: DEFAULT_STRONG_MODEL.into(),
            auto_escalate: DEFAULT_AUTO_ESCALATE,
            default_agent_tier: DEFAULT_AGENT_TIER.into(),
            executor_visibility: DEFAULT_EXECUTOR_VISIBILITY,
            ui_locale: DEFAULT_UI_LOCALE.into(),
            workspace_path: None,
            allow_file_overwrites: DEFAULT_ALLOW_FILE_OVERWRITES,
            rag_enabled: DEFAULT_RAG_ENABLED,
            embedding_base_url: DEFAULT_EMBEDDING_BASE_URL.into(),
            embedding_model: DEFAULT_EMBEDDING_MODEL.into(),
            embedding_api_key: String::new(),
            rag_top_k: DEFAULT_RAG_TOP_K,
            verify_enabled: DEFAULT_VERIFY_ENABLED,
            verify_command: None,
            context_pack_enabled: DEFAULT_CONTEXT_PACK_ENABLED,
            command_allowlist_extra: Vec::new(),
            project_rules_enabled: DEFAULT_PROJECT_RULES_ENABLED,
            plan_before_edit: DEFAULT_PLAN_BEFORE_EDIT,
            editor_open_url: DEFAULT_EDITOR_OPEN_URL.into(),
            rag_auto_index: false,
            task_queue_enabled: true,
            mcp_enabled: false,
            mcp_server_command: None,
            onboarding_completed: false,
            theme_preference: "system".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsView {
    pub base_url: String,
    pub api_key_masked: String,
    pub api_key_configured: bool,
    pub agent_model: String,
    pub agent_temperature: f32,
    pub fast_model: String,
    pub strong_model: String,
    pub auto_escalate: bool,
    pub default_agent_tier: String,
    pub executor_visibility: bool,
    pub ui_locale: String,
    pub workspace_path: Option<String>,
    pub allow_file_overwrites: bool,
    pub rag_enabled: bool,
    pub embedding_base_url: String,
    pub embedding_model: String,
    pub embedding_api_key_masked: String,
    pub embedding_api_key_configured: bool,
    pub rag_top_k: u32,
    pub verify_enabled: bool,
    pub verify_command: Option<String>,
    pub context_pack_enabled: bool,
    pub command_allowlist_extra: Vec<String>,
    pub project_rules_enabled: bool,
    pub plan_before_edit: bool,
    pub editor_open_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_rules_file: Option<String>,
    pub rag_auto_index: bool,
    pub task_queue_enabled: bool,
    pub mcp_enabled: bool,
    pub mcp_server_command: Option<String>,
    pub onboarding_completed: bool,
    pub theme_preference: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateAiSettings {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    #[serde(alias = "companionModel", alias = "executorModel")]
    pub agent_model: Option<String>,
    #[serde(alias = "companionTemperature", alias = "executorTemperature")]
    pub agent_temperature: Option<f32>,
    pub fast_model: Option<String>,
    pub strong_model: Option<String>,
    pub auto_escalate: Option<bool>,
    pub default_agent_tier: Option<String>,
    pub executor_visibility: Option<bool>,
    pub ui_locale: Option<String>,
    pub workspace_path: Option<Option<String>>,
    pub allow_file_overwrites: Option<bool>,
    pub rag_enabled: Option<bool>,
    pub embedding_base_url: Option<String>,
    pub embedding_model: Option<String>,
    pub embedding_api_key: Option<String>,
    pub rag_top_k: Option<u32>,
    pub verify_enabled: Option<bool>,
    pub verify_command: Option<Option<String>>,
    pub context_pack_enabled: Option<bool>,
    pub command_allowlist_extra: Option<Vec<String>>,
    pub project_rules_enabled: Option<bool>,
    pub plan_before_edit: Option<bool>,
    pub editor_open_url: Option<String>,
    pub rag_auto_index: Option<bool>,
    pub task_queue_enabled: Option<bool>,
    pub mcp_enabled: Option<bool>,
    pub mcp_server_command: Option<Option<String>>,
    pub onboarding_completed: Option<bool>,
    pub theme_preference: Option<String>,
}

impl AiSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.base_url.trim().is_empty() {
            return Err("base URL cannot be empty".into());
        }

        if !(0.0..=2.0).contains(&self.agent_temperature) {
            return Err("agent temperature must be between 0 and 2".into());
        }

        if self.agent_model.trim().is_empty() {
            return Err("model name cannot be empty".into());
        }

        if self.fast_model.trim().is_empty() {
            return Err("fast model name cannot be empty".into());
        }

        if self.effective_strong_model().trim().is_empty() {
            return Err("strong model name cannot be empty".into());
        }

        const VALID_TIERS: &[&str] = &["auto", "quick", "deep", "explain"];
        let tier = normalize_agent_tier(&self.default_agent_tier);
        if !VALID_TIERS.contains(&tier.as_str()) {
            return Err(
                "default agent tier must be auto, quick, deep, or explain".into(),
            );
        }

        if self.ui_locale != "en" && self.ui_locale != "fa" {
            return Err("ui locale must be \"en\" or \"fa\"".into());
        }

        if self.embedding_model.trim().is_empty() && self.rag_enabled {
            return Err("embedding model cannot be empty when RAG is enabled".into());
        }

        if self.rag_top_k == 0 || self.rag_top_k > 20 {
            return Err("rag top K must be between 1 and 20".into());
        }

        if !["dark", "light", "system"].contains(&self.theme_preference.as_str()) {
            return Err("theme preference must be \"dark\", \"light\", or \"system\"".into());
        }

        Ok(())
    }

    pub fn to_view(&self) -> AiSettingsView {
        AiSettingsView {
            base_url: self.base_url.clone(),
            api_key_masked: mask_api_key(&self.api_key),
            api_key_configured: !self.api_key.trim().is_empty(),
            agent_model: self.agent_model.clone(),
            agent_temperature: self.agent_temperature,
            fast_model: self.fast_model.clone(),
            strong_model: self.effective_strong_model().to_string(),
            auto_escalate: self.auto_escalate,
            default_agent_tier: self.default_agent_tier.clone(),
            executor_visibility: self.executor_visibility,
            ui_locale: self.ui_locale.clone(),
            workspace_path: self.workspace_path.clone(),
            allow_file_overwrites: self.allow_file_overwrites,
            rag_enabled: self.rag_enabled,
            embedding_base_url: self.embedding_base_url.clone(),
            embedding_model: self.embedding_model.clone(),
            embedding_api_key_masked: mask_api_key(&self.embedding_api_key),
            embedding_api_key_configured: !self.embedding_api_key.trim().is_empty(),
            rag_top_k: self.rag_top_k,
            verify_enabled: self.verify_enabled,
            verify_command: self.verify_command.clone(),
            context_pack_enabled: self.context_pack_enabled,
            command_allowlist_extra: self.command_allowlist_extra.clone(),
            project_rules_enabled: self.project_rules_enabled,
            plan_before_edit: self.plan_before_edit,
            editor_open_url: self.editor_open_url.clone(),
            project_rules_file: self.workspace_path.as_ref().and_then(|path| {
                crate::context::detect_project_rules_file(std::path::Path::new(path))
            }),
            rag_auto_index: self.rag_auto_index,
            task_queue_enabled: self.task_queue_enabled,
            mcp_enabled: self.mcp_enabled,
            mcp_server_command: self.mcp_server_command.clone(),
            onboarding_completed: self.onboarding_completed,
            theme_preference: self.theme_preference.clone(),
        }
    }

    pub fn apply_update(&mut self, update: UpdateAiSettings) -> Result<(), String> {
        if let Some(base_url) = update.base_url {
            self.base_url = base_url.trim().to_string();
        }

        if let Some(api_key) = update.api_key {
            self.api_key = api_key;
        }

        if let Some(agent_model) = update.agent_model {
            self.agent_model = agent_model.trim().to_string();
        }

        if let Some(agent_temperature) = update.agent_temperature {
            self.agent_temperature = agent_temperature;
        }

        if let Some(fast_model) = update.fast_model {
            self.fast_model = fast_model.trim().to_string();
        }

        if let Some(strong_model) = update.strong_model {
            self.strong_model = strong_model.trim().to_string();
        }

        if let Some(auto_escalate) = update.auto_escalate {
            self.auto_escalate = auto_escalate;
        }

        if let Some(default_agent_tier) = update.default_agent_tier {
            self.default_agent_tier = normalize_agent_tier(&default_agent_tier);
        }

        if let Some(executor_visibility) = update.executor_visibility {
            self.executor_visibility = executor_visibility;
        }

        if let Some(ui_locale) = update.ui_locale {
            self.ui_locale = ui_locale.trim().to_string();
        }

        if let Some(workspace_path) = update.workspace_path {
            self.workspace_path = workspace_path
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty());
        }

        if let Some(allow_file_overwrites) = update.allow_file_overwrites {
            self.allow_file_overwrites = allow_file_overwrites;
        }

        if let Some(rag_enabled) = update.rag_enabled {
            self.rag_enabled = rag_enabled;
        }

        if let Some(embedding_base_url) = update.embedding_base_url {
            self.embedding_base_url = embedding_base_url.trim().to_string();
        }

        if let Some(embedding_model) = update.embedding_model {
            self.embedding_model = embedding_model.trim().to_string();
        }

        if let Some(embedding_api_key) = update.embedding_api_key {
            self.embedding_api_key = embedding_api_key;
        }

        if let Some(rag_top_k) = update.rag_top_k {
            self.rag_top_k = rag_top_k;
        }

        if let Some(verify_enabled) = update.verify_enabled {
            self.verify_enabled = verify_enabled;
        }

        if let Some(verify_command) = update.verify_command {
            self.verify_command = verify_command
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty());
        }

        if let Some(context_pack_enabled) = update.context_pack_enabled {
            self.context_pack_enabled = context_pack_enabled;
        }

        if let Some(command_allowlist_extra) = update.command_allowlist_extra {
            self.command_allowlist_extra = command_allowlist_extra
                .into_iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            for prefix in &self.command_allowlist_extra {
                if prefix.len() < 4 {
                    return Err("Command allowlist entries must be at least 4 characters".into());
                }
                if !prefix.contains(' ') && !prefix.ends_with('/') {
                    return Err(
                        "Command allowlist entries must include a space (e.g. \"npm run \") or end with /"
                            .into(),
                    );
                }
            }
        }

        if let Some(project_rules_enabled) = update.project_rules_enabled {
            self.project_rules_enabled = project_rules_enabled;
        }

        if let Some(plan_before_edit) = update.plan_before_edit {
            self.plan_before_edit = plan_before_edit;
        }

        if let Some(editor_open_url) = update.editor_open_url {
            let trimmed = editor_open_url.trim();
            if !trimmed.is_empty() {
                self.editor_open_url = trimmed.to_string();
            }
        }

        if let Some(rag_auto_index) = update.rag_auto_index {
            self.rag_auto_index = rag_auto_index;
        }

        if let Some(task_queue_enabled) = update.task_queue_enabled {
            self.task_queue_enabled = task_queue_enabled;
        }

        if let Some(mcp_enabled) = update.mcp_enabled {
            self.mcp_enabled = mcp_enabled;
        }

        if let Some(mcp_server_command) = update.mcp_server_command {
            self.mcp_server_command = mcp_server_command
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty());
        }

        if let Some(onboarding_completed) = update.onboarding_completed {
            self.onboarding_completed = onboarding_completed;
        }

        if let Some(theme_preference) = update.theme_preference {
            self.theme_preference = theme_preference.trim().to_string();
        }

        self.validate()
    }

    pub fn apply_connection_probe(&mut self, update: UpdateAiSettings) -> Result<(), String> {
        if let Some(base_url) = update.base_url {
            self.base_url = base_url.trim().to_string();
        }

        if let Some(api_key) = update.api_key {
            self.api_key = api_key;
        }

        if let Some(agent_model) = update.agent_model {
            self.agent_model = agent_model.trim().to_string();
        }

        if let Some(agent_temperature) = update.agent_temperature {
            self.agent_temperature = agent_temperature;
        }

        if let Some(fast_model) = update.fast_model {
            self.fast_model = fast_model.trim().to_string();
        }

        if let Some(strong_model) = update.strong_model {
            self.strong_model = strong_model.trim().to_string();
        }

        if self.base_url.trim().is_empty() {
            return Err("base URL cannot be empty".into());
        }

        if self.agent_model.trim().is_empty() {
            return Err("agent model cannot be empty".into());
        }

        if !(0.0..=2.0).contains(&self.agent_temperature) {
            return Err("agent temperature must be between 0 and 2".into());
        }

        Ok(())
    }

    pub fn apply_embedding_probe(&mut self, update: UpdateAiSettings) -> Result<(), String> {
        if let Some(api_key) = update.api_key {
            self.api_key = api_key;
        }

        if let Some(embedding_base_url) = update.embedding_base_url {
            self.embedding_base_url = embedding_base_url.trim().to_string();
        }

        if let Some(embedding_model) = update.embedding_model {
            self.embedding_model = embedding_model.trim().to_string();
        }

        if let Some(embedding_api_key) = update.embedding_api_key {
            self.embedding_api_key = embedding_api_key;
        }

        if self.embedding_base_url.trim().is_empty() {
            return Err("embedding base URL cannot be empty".into());
        }

        if self.embedding_model.trim().is_empty() {
            return Err("embedding model cannot be empty".into());
        }

        Ok(())
    }

    pub fn workspace_configured(&self) -> bool {
        self.workspace_path
            .as_ref()
            .is_some_and(|p| !p.trim().is_empty())
    }

    pub fn effective_fast_model(&self) -> &str {
        let trimmed = self.fast_model.trim();
        if trimmed.is_empty() {
            self.agent_model.trim()
        } else {
            trimmed
        }
    }

    pub fn effective_strong_model(&self) -> &str {
        let trimmed = self.strong_model.trim();
        if trimmed.is_empty() {
            self.agent_model.trim()
        } else {
            trimmed
        }
    }
}

pub fn mask_api_key(api_key: &str) -> String {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.len() <= 4 {
        return "••••".to_string();
    }

    let suffix = &trimmed[trimmed.len() - 4..];
    format!("••••••••{suffix}")
}

fn parse_stored_settings(json: &str) -> Result<AiSettings, serde_json::Error> {
    if let Ok(mut settings) = serde_json::from_str::<AiSettings>(json) {
        settings.default_agent_tier = normalize_agent_tier(&settings.default_agent_tier);
        return Ok(settings);
    }

    let mut value: serde_json::Value = serde_json::from_str(json)?;
    let Some(obj) = value.as_object_mut() else {
        let mut settings: AiSettings = serde_json::from_value(value)?;
        settings.default_agent_tier = normalize_agent_tier(&settings.default_agent_tier);
        return Ok(settings);
    };

    // Muse stored both companion_* and executor_*; serde aliases error when both exist.
    if !obj.contains_key("agent_model") {
        if let Some(model) = obj
            .get("executor_model")
            .or_else(|| obj.get("companion_model"))
            .and_then(|v| v.as_str())
        {
            obj.insert("agent_model".into(), model.into());
        }
    }

    if !obj.contains_key("agent_temperature") {
        if let Some(temp) = obj
            .get("executor_temperature")
            .or_else(|| obj.get("companion_temperature"))
            .and_then(|v| v.as_f64())
        {
            obj.insert("agent_temperature".into(), temp.into());
        }
    }

    obj.remove("companion_model");
    obj.remove("executor_model");
    obj.remove("companion_temperature");
    obj.remove("executor_temperature");
    obj.remove("personality_id");

    let mut settings: AiSettings = serde_json::from_value(value)?;
    settings.default_agent_tier = normalize_agent_tier(&settings.default_agent_tier);
    Ok(settings)
}

pub(crate) fn normalize_agent_tier(raw: &str) -> String {
    let tier = raw.trim().to_lowercase();
    match tier.as_str() {
        "auto" | "quick" | "standard" | "deep" | "explain" => tier,
        _ => "auto".into(),
    }
}

pub fn load(conn: &rusqlite::Connection) -> rusqlite::Result<AiSettings> {
    let stored = crate::db::get_setting(conn, AI_SETTINGS_KEY)?;

    match stored {
        Some(json) => {
            let settings = parse_stored_settings(&json)
                .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
            Ok(settings)
        }
        None => Ok(AiSettings::default()),
    }
}

pub fn save(conn: &rusqlite::Connection, settings: &AiSettings) -> rusqlite::Result<()> {
    settings.validate().map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            err,
        )))
    })?;

    let json = serde_json::to_string(settings)
        .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

    crate::db::set_setting(conn, AI_SETTINGS_KEY, &json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_api_key_suffix() {
        assert_eq!(mask_api_key(""), "");
        assert_eq!(mask_api_key("abc"), "••••");
        assert_eq!(mask_api_key("sk-test-key-1234"), "••••••••1234");
    }

    #[test]
    fn apply_update_keeps_existing_api_key() {
        let mut settings = AiSettings {
            api_key: "secret".into(),
            ..AiSettings::default()
        };

        settings
            .apply_update(UpdateAiSettings {
                base_url: Some("https://proxy.example/v1".into()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(settings.api_key, "secret");
        assert_eq!(settings.base_url, "https://proxy.example/v1");
    }

    #[test]
    fn apply_connection_probe_skips_rag_validation() {
        let mut settings = AiSettings {
            rag_enabled: true,
            embedding_model: String::new(),
            ..AiSettings::default()
        };

        settings
            .apply_connection_probe(UpdateAiSettings {
                base_url: Some("http://localhost:11434/v1".into()),
                agent_model: Some("llama3.2".into()),
                fast_model: Some("fast-mini".into()),
                strong_model: Some("strong-pro".into()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(settings.fast_model, "fast-mini");
        assert_eq!(settings.strong_model, "strong-pro");
    }

    #[test]
    fn apply_embedding_probe_uses_unsaved_values() {
        let mut settings = AiSettings::default();

        settings
            .apply_embedding_probe(UpdateAiSettings {
                embedding_base_url: Some("http://localhost:11434/v1".into()),
                embedding_model: Some("nomic-embed-text".into()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(settings.embedding_model, "nomic-embed-text");
    }

    #[test]
    fn deserialize_legacy_muse_settings_with_dual_models() {
        let json = r#"{
            "base_url": "https://api.openai.com/v1",
            "api_key": "sk-test",
            "companion_model": "gpt-4",
            "executor_model": "gpt-4o-mini",
            "companion_temperature": 0.9,
            "executor_temperature": 0.3,
            "personality_id": "luna"
        }"#;

        let settings = parse_stored_settings(json).unwrap();
        assert_eq!(settings.agent_model, "gpt-4o-mini");
        assert_eq!(settings.agent_temperature, 0.3);
        assert_eq!(settings.strong_model, DEFAULT_STRONG_MODEL);
    }

    #[test]
    fn deserialize_legacy_settings_with_executor_model_only() {
        let json = r#"{
            "base_url": "https://api.openai.com/v1",
            "api_key": "sk-test",
            "executor_model": "llama3.2",
            "executor_temperature": 0.2
        }"#;

        let settings = parse_stored_settings(json).unwrap();
        assert_eq!(settings.agent_model, "llama3.2");
        assert_eq!(settings.agent_temperature, 0.2);
    }

    #[test]
    fn preserves_standard_agent_tier() {
        assert_eq!(normalize_agent_tier("standard"), "standard");
        assert_eq!(normalize_agent_tier("  STANDARD "), "standard");
    }
}
