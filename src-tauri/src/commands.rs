use crate::audio::transcode_to_ogg;
use crate::audio_repo::AudioRepository;
use crate::db::Db;
use crate::error::{Hoi4RadioError, Result};
use crate::generator::generate_mod;
use crate::models::{AudioFile, CreateProjectRequest, Project};
use crate::settings::Settings;
use crate::station::StationRepository;
use crate::validator::validate_mod_output;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

/// Shared application state exposed to Tauri commands.
pub struct AppState {
    pub db: Mutex<Db>,
}

fn lock_db(state: &AppState) -> Result<std::sync::MutexGuard<'_, Db>> {
    state.db.lock().map_err(|e| Hoi4RadioError::Other {
        message: format!("database lock poisoned: {e}"),
    })
}

#[tauri::command]
pub fn create_project(state: State<'_, AppState>, req: CreateProjectRequest) -> Result<Project> {
    let db = lock_db(&state)?;
    db.create_project(&req)
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>> {
    let db = lock_db(&state)?;
    db.list_projects()
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>, id: String) -> Result<Option<Project>> {
    let db = lock_db(&state)?;
    db.get_project(&id)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: String) -> Result<()> {
    let db = lock_db(&state)?;
    db.delete_project(&id)
}

#[tauri::command]
pub fn list_audio_files(state: State<'_, AppState>, project_id: String) -> Result<Vec<AudioFile>> {
    let db = lock_db(&state)?;
    AudioRepository::new(&db).list(&project_id)
}

#[tauri::command]
pub fn delete_audio_file(state: State<'_, AppState>, id: String) -> Result<()> {
    let db = lock_db(&state)?;
    AudioRepository::new(&db).delete(&id)
}

#[tauri::command]
pub async fn import_audio(
    state: State<'_, AppState>,
    project_id: String,
    path: String,
) -> Result<AudioFile> {
    let source_path = PathBuf::from(&path);

    // Resolve the project's output directory before any await point.
    let output_dir = {
        let db = lock_db(&state)?;
        match db.get_project(&project_id)? {
            Some(project) => project.output_dir,
            None => return Err(Hoi4RadioError::ProjectNotFound { id: project_id }),
        }
    };

    let metadata = crate::audio::analyze_audio(&source_path).await?;

    let id = format!("audio_{}", Uuid::new_v4().to_string().replace('-', ""));
    let ogg_filename = format!("{}.ogg", id);
    let music_dir = output_dir.join("music");
    std::fs::create_dir_all(&music_dir)?;
    let ogg_path = music_dir.join(&ogg_filename);

    transcode_to_ogg(&source_path, &ogg_path).await?;

    let title = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| id.clone());

    let audio = AudioFile {
        id: id.clone(),
        title,
        artist: None,
        source_path: source_path.clone(),
        ogg_filename,
        duration_secs: metadata.duration_secs,
        sample_rate: metadata.sample_rate,
        channels: metadata.channels,
        volume: 0.75,
        tags: vec![],
        notes: None,
    };

    let db = lock_db(&state)?;
    AudioRepository::new(&db).create(&project_id, &audio)
}

#[tauri::command]
pub fn list_stations(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<crate::models::Station>> {
    let db = lock_db(&state)?;
    StationRepository::new(&db).list_by_project(&project_id)
}

#[tauri::command]
pub fn create_station(
    state: State<'_, AppState>,
    project_id: String,
    name: String,
) -> Result<crate::models::Station> {
    let db = lock_db(&state)?;
    StationRepository::new(&db).create(&project_id, &name)
}

#[tauri::command]
pub fn add_station_entry(
    state: State<'_, AppState>,
    station_id: String,
    audio_file_id: String,
    chance: crate::models::ChanceConfig,
) -> Result<()> {
    let db = lock_db(&state)?;
    StationRepository::new(&db).add_entry(&station_id, &audio_file_id, chance)
}

#[tauri::command]
pub fn remove_station_entry(
    state: State<'_, AppState>,
    station_id: String,
    audio_file_id: String,
) -> Result<()> {
    let db = lock_db(&state)?;
    StationRepository::new(&db).remove_entry(&station_id, &audio_file_id)
}

#[tauri::command]
pub fn generate_project_mod(state: State<'_, AppState>, project_id: String) -> Result<String> {
    let db = lock_db(&state)?;

    let project = db
        .get_project(&project_id)?
        .ok_or_else(|| Hoi4RadioError::ProjectNotFound {
            id: project_id.clone(),
        })?;

    let stations = StationRepository::new(&db).list_by_project(&project_id)?;
    let audio_files = AudioRepository::new(&db).list(&project_id)?;

    generate_mod(&project, &stations, &audio_files, &project.output_dir)?;

    Ok(project.output_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn validate_project_mod(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<crate::validator::ValidationReport> {
    let output_dir = {
        let db = lock_db(&state)?;
        match db.get_project(&project_id)? {
            Some(project) => project.output_dir,
            None => return Err(Hoi4RadioError::ProjectNotFound { id: project_id }),
        }
    };

    validate_mod_output(&output_dir).await
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings> {
    let db = lock_db(&state)?;
    Settings::get(&db)
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<()> {
    let db = lock_db(&state)?;
    settings.save(&db)
}
