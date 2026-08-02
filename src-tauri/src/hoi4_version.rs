use regex::Regex;
use std::path::Path;

/// Read the HOI4 version string from `launcher-settings.json`.
///
/// Reads the `rawVersion` field (e.g. `"1.19.1.0"`), keeps only the first
/// three dot-separated components, and prefixes it with `"v"` for
/// storage/display, returning `"v1.19.1"`. Use [`extract_supported_version`]
/// when writing it to a `.mod` descriptor.
pub fn detect_game_version(hoi4_dir: &Path) -> Option<String> {
    let settings_path = hoi4_dir.join("launcher-settings.json");
    let content = std::fs::read_to_string(settings_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("rawVersion")?.as_str().and_then(format_raw_version)
}

fn format_raw_version(raw: &str) -> Option<String> {
    let truncated: String = raw
        .split('.')
        .take(3)
        .collect::<Vec<_>>()
        .join(".");
    if truncated.is_empty() {
        return None;
    }
    Some(format!("v{}", truncated))
}

/// Extract a HOI4-compatible `supported_version` value from a raw version
/// string.
///
/// Examples:
/// - `"Operation Postern v1.19.1.0.b49d (31fb)"` → `"v1.19.1"`
/// - `"v1.19.*"` → `"v1.19.*"`
/// - `"1.14.7"` → `"v1.14.7"`
/// - `"1.14"` → `"v1.14"`
/// - `"*"` → `"*"`
/// - anything without a match is returned trimmed.
pub fn extract_supported_version(raw: &str) -> String {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"v?(\d+\.\d+(?:\.\d+|\.\*)?)").unwrap());
    re.captures(raw)
        .and_then(|caps| caps.get(1).map(|m| format!("v{}", m.as_str())))
        .unwrap_or_else(|| raw.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_version_from_launcher_settings() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = std::fs::File::create(dir.path().join("launcher-settings.json")).unwrap();
        file.write_all(br#"{"rawVersion": "1.14.7"}"#).unwrap();
        assert_eq!(detect_game_version(dir.path()), Some("v1.14.7".to_string()));
    }

    #[test]
    fn detects_full_launcher_version_string() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = std::fs::File::create(dir.path().join("launcher-settings.json")).unwrap();
        file.write_all(br#"{"version": "Operation Postern v1.19.1.0.b49d (31fb)", "rawVersion": "1.19.1.0"}"#)
            .unwrap();
        assert_eq!(
            detect_game_version(dir.path()),
            Some("v1.19.1".to_string())
        );
    }

    #[test]
    fn truncates_raw_version_to_three_components() {
        use super::format_raw_version;
        assert_eq!(
            format_raw_version("1.19.1.0.b49d"),
            Some("v1.19.1".to_string())
        );
        assert_eq!(
            format_raw_version("1.19.1.0"),
            Some("v1.19.1".to_string())
        );
        assert_eq!(
            format_raw_version("1.14.7"),
            Some("v1.14.7".to_string())
        );
        assert_eq!(
            format_raw_version("1.19"),
            Some("v1.19".to_string())
        );
        assert_eq!(format_raw_version(""), None);
    }

    #[test]
    fn returns_none_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(detect_game_version(dir.path()), None);
    }

    #[test]
    fn returns_none_when_raw_version_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut file = std::fs::File::create(dir.path().join("launcher-settings.json")).unwrap();
        file.write_all(br#"{"version": "Operation Postern v1.19.1.0.b49d (31fb)"}"#)
            .unwrap();
        assert_eq!(detect_game_version(dir.path()), None);
    }

    #[test]
    fn extracts_supported_version_from_full_string() {
        assert_eq!(
            extract_supported_version("Operation Postern v1.19.1.0.b49d (31fb)"),
            "v1.19.1"
        );
    }

    #[test]
    fn extracts_supported_version_from_plain_version() {
        assert_eq!(extract_supported_version("1.14.7"), "v1.14.7");
        assert_eq!(extract_supported_version("v1.14.7"), "v1.14.7");
    }

    #[test]
    fn extracts_supported_version_with_wildcard() {
        assert_eq!(extract_supported_version("1.19.*"), "v1.19.*");
        assert_eq!(extract_supported_version("v1.19.*"), "v1.19.*");
    }

    #[test]
    fn extracts_supported_version_with_two_components() {
        assert_eq!(extract_supported_version("1.19"), "v1.19");
        assert_eq!(extract_supported_version("v1.19"), "v1.19");
    }

    #[test]
    fn falls_back_to_raw_supported_version() {
        assert_eq!(extract_supported_version("*"), "*");
        assert_eq!(extract_supported_version("  latest  "), "latest");
    }
}
