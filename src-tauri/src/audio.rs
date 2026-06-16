use crate::error::{Hoi4RadioError, Result};
use crate::models::AudioMetadata;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Output produced by `ffprobe -show_streams -show_format -print_format json`.
#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

/// A single stream reported by ffprobe.
#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
}

/// The format section reported by ffprobe.
#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

/// Analyze an audio file with ffprobe and return its metadata.
///
/// Returns an `AudioAnalysis` error if ffprobe fails, the output cannot be
/// parsed, or no audio stream is found.
pub async fn analyze_audio<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
    let path = path.as_ref();

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
            path.to_string_lossy().as_ref(),
        ])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Hoi4RadioError::AudioAnalysis {
            message: format!("ffprobe exited with status {:?}: {stderr}", output.status),
        });
    }

    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)?;

    let audio_stream = parsed
        .streams
        .into_iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"))
        .ok_or_else(|| Hoi4RadioError::AudioAnalysis {
            message: "no audio stream found".to_string(),
        })?;

    let sample_rate = audio_stream
        .sample_rate
        .as_deref()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Hoi4RadioError::AudioAnalysis {
            message: "missing or invalid sample rate".to_string(),
        })?;

    let channels = audio_stream.channels.ok_or_else(|| Hoi4RadioError::AudioAnalysis {
        message: "missing channel count".to_string(),
    })?;

    let duration_secs = parsed
        .format
        .as_ref()
        .and_then(|f| f.duration.as_deref())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Hoi4RadioError::AudioAnalysis {
            message: "missing or invalid duration".to_string(),
        })?;

    Ok(AudioMetadata {
        duration_secs,
        sample_rate,
        channels,
    })
}

/// Transcode an audio file to Ogg Vorbis with ffmpeg.
///
/// The output is encoded at 44.1 kHz using libvorbis quality 4.
/// Returns the output path, or a `Transcoding` error if ffmpeg fails.
pub async fn transcode_to_ogg<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<PathBuf> {
    let input = input.as_ref();
    let output = output.as_ref();

    let result = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input.to_string_lossy().as_ref(),
            "-ar",
            "44100",
            "-c:a",
            "libvorbis",
            "-q:a",
            "4",
            output.to_string_lossy().as_ref(),
        ])
        .output()
        .await?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(Hoi4RadioError::Transcoding {
            message: format!("ffmpeg exited with status {:?}: {stderr}", result.status),
        });
    }

    Ok(output.to_path_buf())
}
