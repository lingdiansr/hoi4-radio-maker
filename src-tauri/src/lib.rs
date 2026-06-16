pub mod audio;
pub mod audio_repo;
pub mod db;
pub mod error;
pub mod generator;
pub mod models;
pub mod station;

use std::path::PathBuf;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize logging to both stdout and a rolling log file.
///
/// Logs are written to the application data directory under `hoi4-radio-maker/logs/`.
/// The default filter is `info`, overridable via the `RUST_LOG` environment variable.
pub fn init_logging() {
    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hoi4-radio-maker");
    let log_dir = app_dir.join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "hoi4-radio-maker.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    // Keep the guard alive for the lifetime of the app so logs are flushed.
    std::mem::forget(_guard);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
