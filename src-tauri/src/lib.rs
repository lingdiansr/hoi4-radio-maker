pub mod audio;
pub mod audio_repo;
pub mod commands;
pub mod db;
pub mod error;
pub mod generator;
pub mod models;
pub mod station;
pub mod validator;

use crate::db::Db;
use std::path::PathBuf;
use std::sync::Mutex;
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

    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hoi4-radio-maker");
    std::fs::create_dir_all(&app_dir).ok();
    let db_path = app_dir.join("app.db");
    let db = Db::open(&db_path).expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::delete_project,
            commands::list_audio_files,
            commands::delete_audio_file,
            commands::import_audio,
            commands::list_stations,
            commands::create_station,
            commands::add_station_entry,
            commands::remove_station_entry,
            commands::generate_project_mod,
            commands::validate_project_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
