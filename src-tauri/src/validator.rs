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
/// consistent, that every song has a localization key, that station/song
/// identifiers only contain ASCII letters, digits, and underscores, and that
/// the triggers written into `.txt` exist in the loaded game/mod vocabulary.
pub async fn validate_mod_output(
    output_dir: &Path,
    ffprobe_path: Option<&str>,
    vocabulary: &crate::scripts::ScriptVocabulary,
) -> Result<ValidationReport> {
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
    let assignment_re =
        Regex::new(r#"(\w+)\s*=\s*([A-Za-z0-9_]+)"#).expect("assignment regex is valid");

    let mut asset_names: HashSet<String> = HashSet::new();
    let mut referenced_ogg_files: HashSet<PathBuf> = HashSet::new();
    let mut songs: HashSet<String> = HashSet::new();
    let mut station_ids: HashSet<String> = HashSet::new();
    // `name = value` pairs found inside `.txt` files, checked against the
    // loaded game/mod trigger vocabulary. Keys like `music`/`song`/`factor`
    // are filtered out before validation.
    let mut txt_assignments: Vec<(String, String, PathBuf)> = Vec::new();

    if music_dir.is_dir() {
        // Collect music/ plus every nested subdirectory (stations may be
        // organised as music/<subdir>/, as in most Workshop radio mods).
        let mut dirs: Vec<PathBuf> = vec![music_dir.clone()];
        let mut i = 0;
        while i < dirs.len() {
            let current = dirs[i].clone();
            i += 1;
            let mut entries = tokio::fs::read_dir(&current).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if tokio::fs::metadata(&path).await?.is_dir() {
                    dirs.push(path);
                }
            }
        }

        // Parse .asset files. `file` paths are relative to the directory
        // holding the .asset, so flat and subdirectory layouts both resolve.
        for dir in &dirs {
            let mut entries = tokio::fs::read_dir(dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("asset")
                    && tokio::fs::metadata(&path).await?.is_file()
                {
                    let content = tokio::fs::read_to_string(&path).await?;
                    let base = path.parent().unwrap_or(dir).to_path_buf();
                    for cap in name_re.captures_iter(&content) {
                        asset_names.insert(cap[1].to_string());
                    }
                    for cap in file_re.captures_iter(&content) {
                        let file = cap[1].to_string();
                        referenced_ogg_files.insert(base.join(&file));
                    }
                }
            }
        }

        // Parse .txt files.
        for dir in &dirs {
            let mut entries = tokio::fs::read_dir(dir).await?;
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
                    for cap in assignment_re.captures_iter(&content) {
                        txt_assignments.push((
                            cap[1].to_string(),
                            cap[2].to_string(),
                            path.clone(),
                        ));
                    }
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

    // 6. Trigger validation. Only runs when the project loaded a vocabulary;
    // without one (no game directory configured) the check is skipped rather
    // than reported as a failure.
    if !vocabulary.is_empty() {
        for (key, value, path) in &txt_assignments {
            if STRUCTURAL_KEYS.contains(&key.as_str()) {
                continue;
            }
            if !vocabulary.triggers.contains_key(key) {
                warnings.push(format!(
                    "Unknown trigger '{}' in {}: not found in the loaded game/mod vocabulary",
                    key,
                    path.display()
                ));
                continue;
            }
            // Values that take a definition from the game data are checked so a
            // mistyped tag/ideology is caught instead of silently doing nothing.
            if matches!(key.as_str(), "tag" | "is_in_faction_with")
                && !vocabulary.country_tags.is_empty()
                && !vocabulary.country_tags.contains(value)
            {
                warnings.push(format!(
                    "Unknown country tag '{}' for '{}' in {}",
                    value,
                    key,
                    path.display()
                ));
            }
            if key == "has_government"
                && !vocabulary.ideologies.is_empty()
                && !vocabulary.ideologies.contains(value)
            {
                warnings.push(format!(
                    "Unknown ideology '{}' for 'has_government' in {}",
                    value,
                    path.display()
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
            match check_ogg_decodable(&ogg_path, ffprobe_path).await {
                Ok(true) => {}
                Ok(false) => {
                    errors.push(format!("OGG file is not decodable: {}", ogg_path.display()));
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
                errors.push(format!("Song name '{}' is missing localization key", name));
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

/// Assignment keys in `.txt` files that are structure, not triggers: the
/// surrounding `music = { … }` blocks and their chance fields.
const STRUCTURAL_KEYS: &[&str] = &[
    "music",
    "music_station",
    "song",
    "chance",
    "factor",
    "add",
    "base",
    "modifier",
];

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

async fn check_ogg_decodable(path: &Path, ffprobe_path: Option<&str>) -> Result<bool> {
    let binary = ffprobe_path.unwrap_or("ffprobe");
    let output = tokio::process::Command::new(binary)
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
            message: format!("{binary} failed for {}: {}", path.display(), e),
        })?;

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn validator_uses_custom_ffprobe_path() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let music_dir = tmp.path().join("music");
        let loc_dir = tmp.path().join("localisation").join("simp_chinese");
        std::fs::create_dir_all(&music_dir).unwrap();
        std::fs::create_dir_all(&loc_dir).unwrap();

        // Minimal OGG placeholder — we only need the validator to call our
        // fake binary and report success.
        let fake_ogg = music_dir.join("audio_001.ogg");
        std::fs::write(&fake_ogg, b"OggS").unwrap();

        std::fs::write(
            music_dir.join("station.asset"),
            r#"music = { name = "audio_001" file = "audio_001.ogg" volume = 0.75 }"#,
        )
        .unwrap();

        std::fs::write(
            music_dir.join("station.txt"),
            r#"music_station = "station" music = { song = "audio_001" }"#,
        )
        .unwrap();

        std::fs::write(
            loc_dir.join("proj_music_l_simp_chinese.yml"),
            "\u{FEFF}l_simp_chinese:\n audio_001:0 \"Song\"\n",
        )
        .unwrap();

        let fake_ffprobe = tmp.path().join("fake_ffprobe");
        std::fs::write(
            &fake_ffprobe,
            "#!/bin/sh\necho '{\"streams\":[{\"codec_type\":\"audio\"}]}'\n",
        )
        .unwrap();
        let mut perm = std::fs::metadata(&fake_ffprobe).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&fake_ffprobe, perm).unwrap();

        let report = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(validate_mod_output(
                tmp.path(),
                Some(fake_ffprobe.to_str().unwrap()),
                &Default::default(),
            ))
            .unwrap();

        assert!(report.passed, "report.errors = {:?}", report.errors);
        assert_eq!(report.ogg_files_checked, 1);
    }
}
