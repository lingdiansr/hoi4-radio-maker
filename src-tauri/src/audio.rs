use crate::error::{Hoi4RadioError, Result};
use crate::models::AudioMetadata;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncReadExt;
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
pub async fn analyze_audio<P: AsRef<Path>>(
    path: P,
    ffprobe_path: Option<&str>,
) -> Result<AudioMetadata> {
    let path = path.as_ref();
    let binary = ffprobe_path.unwrap_or("ffprobe");

    let output = Command::new(binary)
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
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Hoi4RadioError::AudioAnalysis {
                message: format!("{binary} not found. Please install ffmpeg or specify the path in settings."),
            },
            _ => Hoi4RadioError::Io {
                message: e.to_string(),
            },
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Hoi4RadioError::AudioAnalysis {
            message: format!("{binary} exited with status {:?}: {stderr}", output.status),
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
/// If `cancel_rx` is provided and becomes true, the ffmpeg child process is
/// killed and a cancellation error is returned.
pub async fn transcode_to_ogg<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    output: Q,
    ffmpeg_path: Option<&str>,
    mut cancel_rx: Option<tokio::sync::watch::Receiver<bool>>,
) -> Result<PathBuf> {
    let input = input.as_ref();
    let output = output.as_ref();
    let binary = ffmpeg_path.unwrap_or("ffmpeg");

    let mut child = Command::new(binary)
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
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Hoi4RadioError::Transcoding {
                message: format!("{binary} not found. Please install ffmpeg or specify the path in settings."),
            },
            _ => Hoi4RadioError::Io {
                message: e.to_string(),
            },
        })?;

    // If cancellation was already requested before we started, kill immediately.
    if let Some(ref mut rx) = cancel_rx {
        if *rx.borrow_and_update() {
            let _ = child.kill().await;
            return Err(Hoi4RadioError::Transcoding {
                message: "transcoding cancelled".to_string(),
            });
        }
    }

    let stderr_handle = child.stderr.take();
    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        if let Some(mut h) = stderr_handle {
            let _ = h.read_to_end(&mut buf).await;
        }
        buf
    });

    let status = match cancel_rx {
        Some(mut rx) => {
            tokio::select! {
                status = child.wait() => status,
                _ = rx.changed() => {
                    if *rx.borrow() {
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        return Err(Hoi4RadioError::Transcoding {
                            message: "transcoding cancelled".to_string(),
                        });
                    }
                    child.wait().await
                }
            }
        }
        None => child.wait().await,
    }
    .map_err(|e| Hoi4RadioError::Io {
        message: format!("failed to wait for ffmpeg: {e}"),
    })?;

    let stderr_bytes = stderr_task.await.unwrap_or_default();

    if !status.success() {
        let stderr = String::from_utf8_lossy(&stderr_bytes);
        return Err(Hoi4RadioError::Transcoding {
            message: format!("{binary} exited with status {:?}: {stderr}", status),
        });
    }

    Ok(output.to_path_buf())
}

/// Compute a BLAKE3 hash of a file's contents.
pub async fn compute_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    let mut file = tokio::fs::File::open(path).await.map_err(|e| Hoi4RadioError::Io {
        message: format!("failed to open {}: {e}", path.display()),
    })?;

    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buffer).await.map_err(|e| Hoi4RadioError::Io {
            message: format!("failed to read {}: {e}", path.display()),
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}
