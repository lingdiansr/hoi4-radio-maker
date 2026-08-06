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

/// Import/processing status of an audio file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    Pending,
    Processing,
    Ready,
    Error,
    Cancelled,
}

impl ImportStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportStatus::Pending => "pending",
            ImportStatus::Processing => "processing",
            ImportStatus::Ready => "ready",
            ImportStatus::Error => "error",
            ImportStatus::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for ImportStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "pending" => Ok(ImportStatus::Pending),
            "processing" => Ok(ImportStatus::Processing),
            "ready" => Ok(ImportStatus::Ready),
            "error" => Ok(ImportStatus::Error),
            "cancelled" => Ok(ImportStatus::Cancelled),
            _ => Err(format!("unknown import_status: {s}")),
        }
    }
}

/// An audio file in the global audio library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub source_hash: String,
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
    pub import_status: ImportStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Metadata extracted from an audio file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u32,
    /// Title from the file's embedded tags (ID3 etc.), if present.
    pub title: Option<String>,
    /// Artist from the file's embedded tags (ID3 etc.), if present.
    pub artist: Option<String>,
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
    #[serde(default)]
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

/// Request payload for updating an existing project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
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

/// Request payload for updating a single audio file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAudioFileRequest {
    pub title: Option<String>,
    pub artist: Option<Option<String>>,
    pub volume: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<Option<String>>,
}

/// Request payload for batch updating multiple audio files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateAudioFileRequest {
    pub artist: Option<Option<String>>,
    pub volume: Option<f64>,
    pub tags: Option<Vec<String>>,
}

/// A single failed file within a batch import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchImportFailedFile {
    pub path: String,
    pub message: String,
}
