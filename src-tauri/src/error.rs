use serde::Serialize;
use thiserror::Error;

/// Application-wide error type.
///
/// All Tauri commands return `Result<T, Hoi4RadioError>`. The enum derives
/// `Serialize` so Tauri can forward structured errors to the frontend.
#[derive(Error, Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Hoi4RadioError {
    #[error("IO error: {message}")]
    Io { message: String },

    #[error("Database error: {message}")]
    Db { message: String },

    #[error("JSON error: {message}")]
    Json { message: String },

    #[error("Audio analysis failed: {message}")]
    AudioAnalysis { message: String },

    #[error("Transcoding failed: {message}")]
    Transcoding { message: String },

    #[error("Project not found: {id}")]
    ProjectNotFound { id: String },

    #[error("Validation failed: {message}")]
    Validation { message: String },

    #[error("ffmpeg / ffprobe not found. Please install ffmpeg or specify paths manually in settings.")]
    FfmpegNotFound,

    #[error("{message}")]
    Other { message: String },
}

impl From<std::io::Error> for Hoi4RadioError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
        }
    }
}

impl From<rusqlite::Error> for Hoi4RadioError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Db {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for Hoi4RadioError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json {
            message: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Hoi4RadioError>;
