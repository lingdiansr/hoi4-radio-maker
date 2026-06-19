use std::path::Path;

pub fn detect_game_version(hoi4_dir: &Path) -> Option<String> {
    let settings_path = hoi4_dir.join("launcher-settings.json");
    let content = std::fs::read_to_string(settings_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("version")?.as_str().map(|s| s.to_string())
}
