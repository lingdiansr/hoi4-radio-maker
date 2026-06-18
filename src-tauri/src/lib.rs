pub mod audio;
pub mod audio_repo;
pub mod commands;
pub mod db;
pub mod error;
pub mod ffmpeg_finder;
pub mod generator;
pub mod models;
pub mod settings;
pub mod station;
pub mod validator;

use crate::db::Db;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri_plugin_log::{Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("hoi4_radio_maker_lib::commands", log::LevelFilter::Debug)
                .level_for("hoi4_radio_maker_lib::audio", log::LevelFilter::Debug)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("app".into()),
                    }),
                    #[cfg(debug_assertions)]
                    Target::new(TargetKind::Webview),
                ])
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(feature = "e2e-testing")]
    {
        builder = builder.plugin(tauri_plugin_playwright::init());
    }

    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hoi4-radio-maker");
    std::fs::create_dir_all(&app_dir).ok();
    let db_path = app_dir.join("app.db");
    let db = Db::open(&db_path).expect("failed to open database");

    builder
        .manage(commands::AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::update_project,
            commands::delete_project,
            commands::list_audio_files,
            commands::list_all_audio_files,
            commands::add_audio_to_project,
            commands::remove_audio_from_project,
            commands::delete_audio_file,
            commands::import_audio_batch,
            commands::list_stations,
            commands::create_station,
            commands::add_station_entry,
            commands::remove_station_entry,
            commands::generate_project_mod,
            commands::validate_project_mod,
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
