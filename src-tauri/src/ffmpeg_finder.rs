use crate::error::Result;
use std::path::{Path, PathBuf};

/// Search for ffmpeg and ffprobe executables in common locations.
///
/// Returns a tuple of `(ffmpeg_path, ffprobe_path)` where each may be `None`
/// if the corresponding executable could not be found.
pub fn detect_ffmpeg() -> Result<(Option<String>, Option<String>)> {
    let ffmpeg = find_executable("ffmpeg");
    let ffprobe = find_executable("ffprobe");
    Ok((ffmpeg.map(|p| p.to_string_lossy().to_string()), ffprobe.map(|p| p.to_string_lossy().to_string())))
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };

    // 1. Search PATH.
    if let Some(path) = find_in_path(&exe_name) {
        return Some(path);
    }

    // 2. Search common installation directories.
    let candidates: Vec<PathBuf> = if cfg!(windows) {
        [
            r"C:\ffmpeg\bin",
            r"C:\Program Files\ffmpeg\bin",
            r"C:\Program Files (x86)\ffmpeg\bin",
            r"C:\Users\%USERPROFILE%\ffmpeg\bin",
        ]
        .iter()
        .map(PathBuf::from)
        .collect()
    } else if cfg!(target_os = "macos") {
        [
            "/usr/local/bin",
            "/opt/homebrew/bin",
            "/opt/local/bin",
            "/usr/bin",
            "/opt/ffmpeg/bin",
            "/Applications/ffmpeg",
        ]
        .iter()
        .map(PathBuf::from)
        .collect()
    } else {
        [
            "/usr/bin",
            "/usr/local/bin",
            "/opt/ffmpeg/bin",
            "/opt/local/bin",
            "~/.local/bin",
        ]
        .iter()
        .map(PathBuf::from)
        .collect()
    };

    for dir in candidates {
        let candidate = expand_tilde(&dir).join(&exe_name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn find_in_path(exe_name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    let separator = if cfg!(windows) { ';' } else { ':' };

    for dir in path_var.split(separator) {
        let candidate = Path::new(dir).join(exe_name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(windows)]
    {
        true
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
}

fn expand_tilde(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&s[2..]);
        }
    }
    path.to_path_buf()
}
