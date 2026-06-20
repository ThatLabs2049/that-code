use serde::{Deserialize, Serialize};

pub const AI_SETTINGS_KEY: &str = "ai_settings";

pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_COMPANION_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_EXECUTOR_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_COMPANION_TEMPERATURE: f32 = 0.9;
pub const DEFAULT_EXECUTOR_TEMPERATURE: f32 = 0.3;
pub const DEFAULT_EXECUTOR_VISIBILITY: bool = false;
pub const DEFAULT_UI_LOCALE: &str = "en";
pub const DEFAULT_ALLOW_FILE_OVERWRITES: bool = true;
pub const EXECUTOR_MAX_STEPS: usize = 25;
pub const DEFAULT_RAG_ENABLED: bool = false;
pub const DEFAULT_EMBEDDING_BASE_URL: &str = "http://localhost:11434/v1";
pub const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text";
pub const DEFAULT_RAG_TOP_K: u32 = 5;
pub const DEFAULT_VERIFY_ENABLED: bool = true;
pub const DEFAULT_CONTEXT_PACK_ENABLED: bool = true;

pub const DEFAULT_PERSONALITY_ID: &str = "luna";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiSettings {
    pub base_url: String,
    pub api_key: String,
    pub companion_model: String,
    pub executor_model: String,
    pub companion_temperature: f32,
    pub executor_temperature: f32,
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
    #[serde(default = "default_personality_id")]
    pub personality_id: String,
    #[serde(default)]
    pub command_allowlist_extra: Vec<String>,
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

fn default_theme_preference() -> String {
    "system".into()
}

fn default_personality_id() -> String {
    DEFAULT_PERSONALITY_ID.into()
}

fn default_task_queue_enabled() -> bool {
    true
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
            companion_model: DEFAULT_COMPANION_MODEL.into(),
            executor_model: DEFAULT_EXECUTOR_MODEL.into(),
            companion_temperature: DEFAULT_COMPANION_TEMPERATURE,
            executor_temperature: DEFAULT_EXECUTOR_TEMPERATURE,
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
            personality_id: DEFAULT_PERSONALITY_ID.into(),
            command_allowlist_extra: Vec::new(),
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
    pub companion_model: String,
    pub executor_model: String,
    pub companion_temperature: f32,
    pub executor_temperature: f32,
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
    pub personality_id: String,
    pub command_allowlist_extra: Vec<String>,
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
    pub companion_model: Option<String>,
    pub executor_model: Option<String>,
    pub companion_temperature: Option<f32>,
    pub executor_temperature: Option<f32>,
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
    pub personality_id: Option<String>,
    pub command_allowlist_extra: Option<Vec<String>>,
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

        if !(0.0..=2.0).contains(&self.companion_temperature) {
            return Err("companion temperature must be between 0 and 2".into());
        }

        if !(0.0..=2.0).contains(&self.executor_temperature) {
            return Err("executor temperature must be between 0 and 2".into());
        }

        if self.companion_model.trim().is_empty() || self.executor_model.trim().is_empty() {
            return Err("model names cannot be empty".into());
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

        if !crate::personalities::is_valid_id(&self.personality_id) {
            return Err("personality id is invalid".into());
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
            companion_model: self.companion_model.clone(),
            executor_model: self.executor_model.clone(),
            companion_temperature: self.companion_temperature,
            executor_temperature: self.executor_temperature,
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
            personality_id: self.personality_id.clone(),
            command_allowlist_extra: self.command_allowlist_extra.clone(),
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

        if let Some(companion_model) = update.companion_model {
            self.companion_model = companion_model.trim().to_string();
        }

        if let Some(executor_model) = update.executor_model {
            self.executor_model = executor_model.trim().to_string();
        }

        if let Some(companion_temperature) = update.companion_temperature {
            self.companion_temperature = companion_temperature;
        }

        if let Some(executor_temperature) = update.executor_temperature {
            self.executor_temperature = executor_temperature;
        }

        if let Some(executor_visibility) = update.executor_visibility {
            self.executor_visibility = executor_visibility;
        }

        if let Some(ui_locale) = update.ui_locale {
            self.ui_locale = ui_locale.trim().to_string();
        }

        if let Some(workspace_path) = update.workspace_path {
            self.workspace_path = workspace_path.map(|p| p.trim().to_string()).filter(|p| !p.is_empty());
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
            self.verify_command = verify_command.map(|c| c.trim().to_string()).filter(|c| !c.is_empty());
        }

        if let Some(context_pack_enabled) = update.context_pack_enabled {
            self.context_pack_enabled = context_pack_enabled;
        }

        if let Some(personality_id) = update.personality_id {
            self.personality_id = personality_id.trim().to_string();
        }

        if let Some(command_allowlist_extra) = update.command_allowlist_extra {
            self.command_allowlist_extra = command_allowlist_extra
                .into_iter()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
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

        if let Some(companion_model) = update.companion_model {
            self.companion_model = companion_model.trim().to_string();
        }

        if let Some(executor_model) = update.executor_model {
            self.executor_model = executor_model.trim().to_string();
        }

        if let Some(companion_temperature) = update.companion_temperature {
            self.companion_temperature = companion_temperature;
        }

        if let Some(executor_temperature) = update.executor_temperature {
            self.executor_temperature = executor_temperature;
        }

        if self.base_url.trim().is_empty() {
            return Err("base URL cannot be empty".into());
        }

        if self.companion_model.trim().is_empty() {
            return Err("companion model cannot be empty".into());
        }

        if !(0.0..=2.0).contains(&self.companion_temperature) {
            return Err("companion temperature must be between 0 and 2".into());
        }

        if !(0.0..=2.0).contains(&self.executor_temperature) {
            return Err("executor temperature must be between 0 and 2".into());
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

pub fn load(conn: &rusqlite::Connection) -> rusqlite::Result<AiSettings> {
    let stored = crate::db::get_setting(conn, AI_SETTINGS_KEY)?;

    match stored {
        Some(json) => {
            let settings: AiSettings = serde_json::from_str(&json).map_err(|err| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(err))
            })?;
            Ok(settings)
        }
        None => Ok(AiSettings::default()),
    }
}

pub fn save(conn: &rusqlite::Connection, settings: &AiSettings) -> rusqlite::Result<()> {
    settings
        .validate()
        .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            err,
        ))))?;

    let json = serde_json::to_string(settings).map_err(|err| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(err))
    })?;

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
                companion_model: Some("llama3.2".into()),
                ..Default::default()
            })
            .unwrap();
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
}
