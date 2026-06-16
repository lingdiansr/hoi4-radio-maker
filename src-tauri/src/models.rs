use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single radio mod project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub version: String,
    pub supported_version: String,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub output_dir: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An audio file in the project's audio library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub source_path: PathBuf,
    pub ogg_filename: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u32,
    pub volume: f64,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

/// Metadata extracted from an audio file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u32,
}

/// A music station within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub entries: Vec<StationEntry>,
}

/// A song entry inside a station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationEntry {
    pub audio_file_id: String,
    pub chance: ChanceConfig,
}

/// Playback probability configuration for a station entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceConfig {
    pub factor: f64,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub factor: Option<f64>,
    pub add: Option<f64>,
    pub base: Option<f64>,
    pub triggers: Vec<Trigger>,
}

/// Clausewitz trigger conditions (simplified subset).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    HasWar { value: bool },
    Tag { value: String },
    HasGovernment { ideology: String },
    IsInFaction { tag: String },
}

/// Request payload for creating a new project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub version: String,
    pub supported_version: String,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub output_dir: PathBuf,
}

/// Request payload for creating a new station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStationRequest {
    pub project_id: String,
    pub name: String,
}

/// Request payload for adding a song to a station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddStationEntryRequest {
    pub station_id: String,
    pub audio_file_id: String,
    pub chance: ChanceConfig,
}
