//! The launcher's own settings, kept as JSON in the data folder.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// The offline name the game plays as (`--name`).
    pub nickname: String,
    /// The installed release Play starts.
    pub selected_version: Option<String>,
    /// The official launcher's game folder, when not the default.
    pub minecraft_dir: Option<String>,
    /// A Java 25 home to hand to `horizon-prepare`, when not on the PATH.
    pub java_home: Option<String>,
    /// `ru`, `en`, or `system`.
    pub language: String,
    /// Copy the pinned version's sounds during preparation.
    pub include_sounds: bool,
    /// Extra words appended to the game's command line.
    pub extra_args: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            nickname: default_nickname(),
            selected_version: None,
            minecraft_dir: None,
            java_home: None,
            language: "ru".to_string(),
            include_sounds: true,
            extra_args: String::new(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let Ok(text) = std::fs::read_to_string(crate::paths::settings_path()) else {
            return Self::default();
        };
        serde_json::from_str::<Self>(&text)
            .map(Self::normalized)
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let normalized = self.clone().normalized();
        let text = serde_json::to_string_pretty(&normalized).map_err(|e| e.to_string())?;
        std::fs::write(crate::paths::settings_path(), text).map_err(|e| e.to_string())
    }

    /// A name the game accepts: 3 to 16 letters, digits or `_`.
    pub fn normalized(mut self) -> Self {
        self.nickname = sanitize_nickname(&self.nickname);
        if !matches!(self.language.as_str(), "ru" | "en" | "system") {
            self.language = "system".to_string();
        }
        self
    }
}

pub fn sanitize_nickname(raw: &str) -> String {
    let mut name: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(16)
        .collect();
    if name.len() < 3 {
        name = default_nickname();
    }
    name
}

fn default_nickname() -> String {
    let raw = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default();
    let name: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(12)
        .collect();
    if name.len() >= 3 {
        name
    } else {
        "Player".to_string()
    }
}
