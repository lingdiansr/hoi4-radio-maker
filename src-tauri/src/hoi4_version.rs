use std::path::Path;

pub fn detect_game_version(hoi4_dir: &Path) -> Option<String> {
    let settings_path = hoi4_dir.join("launcher-settings.json");
    let content = std::fs::read_to_string(settings_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("version")?.as_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_version_from_launcher_settings() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = std::fs::File::create(dir.path().join("launcher-settings.json")).unwrap();
        file.write_all(br#"{"version": "1.14.7"}"#).unwrap();
        assert_eq!(detect_game_version(dir.path()), Some("1.14.7".to_string()));
    }

    #[test]
    fn returns_none_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(detect_game_version(dir.path()), None);
    }

    #[test]
    fn returns_none_when_version_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = std::fs::File::create(dir.path().join("launcher-settings.json")).unwrap();
        file.write_all(br#"{}"#).unwrap();
        assert_eq!(detect_game_version(dir.path()), None);
    }
}
