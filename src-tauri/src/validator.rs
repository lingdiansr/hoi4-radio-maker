use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::{Hoi4RadioError, Result};

/// Report produced by [`validate_mod_output`].
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub ogg_files_checked: usize,
}

/// Validates a generated HOI4 radio mod directory.
///
/// Checks that required directories exist, that every OGG file referenced in
/// `.asset` files exists and is decodable, that `.asset` and `.txt` files are
/// consistent, that every song has a localization key, and that station/song
/// identifiers only contain ASCII letters, digits, and underscores.
pub async fn validate_mod_output(output_dir: &Path) -> Result<ValidationReport> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut ogg_files_checked = 0usize;

    let music_dir = output_dir.join("music");
    let loc_dir = output_dir.join("localisation").join("simp_chinese");

    // 1. Directory structure.
    if !music_dir.is_dir() {
        errors.push(format!("Missing music/ directory: {}", music_dir.display()));
    }
    if !loc_dir.is_dir() {
        errors.push(format!(
            "Missing localisation/simp_chinese/ directory: {}",
            loc_dir.display()
        ));
    }

    let name_re = Regex::new(r#"name\s*=\s*"([^"]+)""#).expect("name regex is valid");
    let song_re = Regex::new(r#"song\s*=\s*"([^"]+)""#).expect("song regex is valid");
    let file_re = Regex::new(r#"file\s*=\s*"([^"]+)""#).expect("file regex is valid");
    let station_re =
        Regex::new(r#"music_station\s*=\s*"([^"]+)""#).expect("station regex is valid");

    let mut asset_names: HashSet<String> = HashSet::new();
    let mut referenced_ogg_files: HashSet<PathBuf> = HashSet::new();
    let mut songs: HashSet<String> = HashSet::new();
    let mut station_ids: HashSet<String> = HashSet::new();

    if music_dir.is_dir() {
        // Parse .asset files.
        let mut entries = tokio::fs::read_dir(&music_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("asset")
                && tokio::fs::metadata(&path).await?.is_file()
            {
                let content = tokio::fs::read_to_string(&path).await?;
                for cap in name_re.captures_iter(&content) {
                    asset_names.insert(cap[1].to_string());
                }
                for cap in file_re.captures_iter(&content) {
                    let file = cap[1].to_string();
                    referenced_ogg_files.insert(music_dir.join(&file));
                }
            }
        }

        // Parse .txt files.
        let mut entries = tokio::fs::read_dir(&music_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("txt")
                && tokio::fs::metadata(&path).await?.is_file()
            {
                let content = tokio::fs::read_to_string(&path).await?;
                for cap in station_re.captures_iter(&content) {
                    station_ids.insert(cap[1].to_string());
                }
                for cap in song_re.captures_iter(&content) {
                    songs.insert(cap[1].to_string());
                }
            }
        }

        // 3. Asset / TXT consistency.
        for song in &songs {
            if !asset_names.contains(song) {
                errors.push(format!(
                    "Song '{}' referenced in .txt has no matching name in .asset",
                    song
                ));
            }
        }
        for name in &asset_names {
            if !songs.contains(name) {
                warnings.push(format!(
                    "Name '{}' in .asset has no matching song in .txt",
                    name
                ));
            }
        }

        // 5. ID naming validation.
        for name in &asset_names {
            if !is_valid_id(name) {
                errors.push(format!(
                    "Invalid song name '{}': must contain only ASCII letters, digits, and underscores",
                    name
                ));
            }
        }
        for station_id in &station_ids {
            if !is_valid_id(station_id) {
                errors.push(format!(
                    "Invalid station ID '{}': must contain only ASCII letters, digits, and underscores",
                    station_id
                ));
            }
        }
    }

    // 2. OGG file validation.
    for ogg_path in referenced_ogg_files {
        if !ogg_path.exists() {
            errors.push(format!(
                "Referenced OGG file does not exist: {}",
                ogg_path.display()
            ));
        } else {
            ogg_files_checked += 1;
            match check_ogg_decodable(&ogg_path).await {
                Ok(true) => {}
                Ok(false) => {
                    errors.push(format!(
                        "OGG file is not decodable: {}",
                        ogg_path.display()
                    ));
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to check OGG file {}: {}",
                        ogg_path.display(),
                        e
                    ));
                }
            }
        }
    }

    // 4. Localization validation.
    if loc_dir.is_dir() {
        let mut loc_keys: HashSet<String> = HashSet::new();
        let mut entries = tokio::fs::read_dir(&loc_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yml")
                && tokio::fs::metadata(&path).await?.is_file()
            {
                let content = tokio::fs::read_to_string(&path).await?;
                for line in content.lines() {
                    if let Some(idx) = line.find(":0") {
                        let key = line[..idx].trim().to_string();
                        if !key.is_empty() {
                            loc_keys.insert(key);
                        }
                    }
                }
            }
        }

        for name in &asset_names {
            if !loc_keys.contains(name) {
                errors.push(format!(
                    "Song name '{}' is missing localization key",
                    name
                ));
            }
        }
    }

    let passed = errors.is_empty();
    Ok(ValidationReport {
        passed,
        errors,
        warnings,
        ogg_files_checked,
    })
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

async fn check_ogg_decodable(path: &Path) -> Result<bool> {
    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_streams",
            &path.to_string_lossy(),
        ])
        .output()
        .await
        .map_err(|e| Hoi4RadioError::Other {
            message: format!("ffprobe failed for {}: {}", path.display(), e),
        })?;

    Ok(output.status.success())
}
