use crate::db::Db;
use crate::error::Result;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// Settings enriched with transient, derived values for the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct SettingsResponse {
    #[serde(flatten)]
    pub settings: Settings,
    pub detected_supported_version: Option<String>,
    pub ffmpeg_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub hoi4_game_dir: Option<String>,
    pub theme: String,
    pub import_concurrency: u32,
    pub default_project_dir: Option<String>,
    pub default_author: Option<String>,
    pub default_version: Option<String>,
    pub default_supported_version: Option<String>,
    pub default_tags: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ffmpeg_path: None,
            ffprobe_path: None,
            hoi4_game_dir: None,
            theme: "dark".to_string(),
            import_concurrency: 8,
            default_project_dir: None,
            default_author: None,
            default_version: Some("0.1.0".to_string()),
            default_supported_version: None,
            default_tags: vec!["Sound".to_string()],
        }
    }
}

impl Settings {
    pub fn get(db: &Db) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let value: Option<String> = stmt.query_row(["settings"], |row| row.get(0)).optional()?;
        match value {
            Some(v) => Ok(serde_json::from_str(&v).unwrap_or_default()),
            None => Ok(Self::default()),
        }
    }

    pub fn save(&self, db: &Db) -> Result<()> {
        let value = serde_json::to_string(self)?;
        db.conn().execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            ["settings", &value],
        )?;
        Ok(())
    }
}
