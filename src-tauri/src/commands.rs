use crate::audio::{analyze_audio, compute_file_hash, transcode_to_ogg};
use crate::audio_repo::AudioRepository;
use crate::db::{BatchImportResult, Db};
use crate::error::{Hoi4RadioError, Result};
use crate::generator::generate_mod;
use crate::models::{AudioFile, CreateProjectRequest, Project, UpdateProjectRequest};
use crate::settings::Settings;
use crate::station::StationRepository;
use crate::validator::validate_mod_output;
use chrono::Utc;
use futures::stream::{StreamExt, TryStreamExt};
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
pub fn update_project(
    state: State<'_, AppState>,
    id: String,
    req: UpdateProjectRequest,
) -> Result<Project> {
    let db = lock_db(&state)?;
    db.update_project(&id, &req)
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
pub fn list_all_audio_files(state: State<'_, AppState>) -> Result<Vec<AudioFile>> {
    let db = lock_db(&state)?;
    let files = AudioRepository::new(&db).list_all()?;
    tracing::info!(count = files.len(), "listed all audio files");
    Ok(files)
}

#[tauri::command]
pub fn add_audio_to_project(
    state: State<'_, AppState>,
    project_id: String,
    audio_ids: Vec<String>,
) -> Result<()> {
    let db = lock_db(&state)?;
    let repo = AudioRepository::new(&db);
    for id in audio_ids {
        repo.add_to_project(&project_id, &id)?;
    }
    Ok(())
}

#[tauri::command]
pub fn remove_audio_from_project(
    state: State<'_, AppState>,
    project_id: String,
    audio_id: String,
) -> Result<()> {
    let db = lock_db(&state)?;
    AudioRepository::new(&db).remove_from_project(&project_id, &audio_id)
}

#[tauri::command]
pub fn delete_audio_file(state: State<'_, AppState>, id: String) -> Result<()> {
    let db = lock_db(&state)?;
    AudioRepository::new(&db).delete(&id)
}

/// Batch import audio files into the global library.
///
/// If `project_id` is provided, the imported/duplicate audio files are also
/// added to that project's reference list.
#[tauri::command]
pub async fn import_audio_batch(
    state: State<'_, AppState>,
    paths: Vec<String>,
    project_id: Option<String>,
) -> Result<BatchImportResult> {
    if paths.is_empty() {
        return Ok(BatchImportResult {
            created: vec![],
            existing: vec![],
        });
    }

    tracing::info!(
        project_id = ?project_id,
        path_count = paths.len(),
        "starting audio import batch"
    );

    // Resolve settings and validate project (if given) before any await point.
    let (settings, audio_store_dir) = {
        let db = lock_db(&state).map_err(|e| {
            tracing::error!(error = %e, "failed to lock database");
            e
        })?;
        if let Some(ref pid) = project_id {
            if db.get_project(pid)?.is_none() {
                tracing::error!(project_id = %pid, "project not found for audio import");
                return Err(Hoi4RadioError::ProjectNotFound { id: pid.clone() });
            }
        }
        let settings = Settings::get(&db).map_err(|e| {
            tracing::error!(error = %e, "failed to load settings");
            e
        })?;
        tracing::debug!(settings = ?settings, "loaded settings");
        let app_dir = dirs::data_dir()
            .ok_or_else(|| {
                tracing::error!("could not determine application data directory");
                Hoi4RadioError::Other {
                    message: "could not determine application data directory".to_string(),
                }
            })?
            .join("hoi4-radio-maker")
            .join("audio");
        tracing::debug!(audio_store_dir = %app_dir.display(), "resolved audio store directory");
        (settings, app_dir)
    };

    let ffmpeg_path = settings.ffmpeg_path.as_deref();
    let ffprobe_path = settings.ffprobe_path.as_deref();
    let hash_concurrency = settings.import_concurrency.max(1) as usize;
    let transcode_concurrency = (hash_concurrency / 2).max(1);

    // Ensure the global audio store exists.
    tokio::fs::create_dir_all(&audio_store_dir).await?;

    // 1. Compute hashes concurrently.
    let hashed = futures::stream::iter(paths)
        .map(|path| {
            let path = PathBuf::from(path);
            async move {
                let hash = compute_file_hash(&path).await?;
                Ok::<_, Hoi4RadioError>((path, hash))
            }
        })
        .buffer_unordered(hash_concurrency)
        .try_collect::<Vec<_>>()
        .await?;

    // 2. Check database for existing files and split into new/existing.
    let mut existing: Vec<AudioFile> = Vec::new();
    let mut to_transcode: Vec<(PathBuf, String)> = Vec::new();
    {
        let db = lock_db(&state)?;
        let repo = AudioRepository::new(&db);
        for (path, hash) in hashed {
            let existing_audio = repo.get_by_hash(&hash)?;
            match existing_audio {
                Some(audio) => existing.push(audio),
                None => to_transcode.push((path, hash)),
            }
        }
    }

    // 3. Transcode new files concurrently with limited ffmpeg processes.
    let transcoded = futures::stream::iter(to_transcode)
        .map(|(source_path, source_hash)| {
            let audio_store_dir = audio_store_dir.clone();
            async move {
                let metadata = analyze_audio(&source_path, ffprobe_path).await?;
                let id = format!("audio_{}", Uuid::new_v4().to_string().replace('-', ""));
                let ogg_filename = format!("{}.ogg", id);
                let ogg_path = audio_store_dir.join(&ogg_filename);

                transcode_to_ogg(&source_path, &ogg_path, ffmpeg_path).await?;

                let title = source_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| id.clone());

                let now = Utc::now();
                let audio = AudioFile {
                    id,
                    source_hash,
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
                    created_at: now,
                    updated_at: now,
                };

                Ok::<_, Hoi4RadioError>(audio)
            }
        })
        .buffer_unordered(transcode_concurrency)
        .try_collect::<Vec<_>>()
        .await?;

    // 4. Persist everything and create project references if requested.
    let mut created = Vec::new();
    {
        let db = lock_db(&state)?;
        let repo = AudioRepository::new(&db);
        for audio in transcoded {
            repo.create(&audio)?;
            created.push(audio);
        }
        if let Some(ref pid) = project_id {
            for audio in existing.iter().chain(created.iter()) {
                repo.add_to_project(pid, &audio.id)?;
            }
        }
    }

    tracing::info!(
        created_count = created.len(),
        existing_count = existing.len(),
        project_id = ?project_id,
        "audio import batch completed"
    );

    Ok(BatchImportResult { created, existing })
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
    tracing::info!(
        station_id = %station_id,
        audio_file_id = %audio_file_id,
        ?chance,
        "adding station entry"
    );
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

    let audio_store_dir = dirs::data_dir()
        .ok_or_else(|| Hoi4RadioError::Other {
            message: "could not determine application data directory".to_string(),
        })?
        .join("hoi4-radio-maker")
        .join("audio");

    generate_mod(
        &project,
        &stations,
        &audio_files,
        &project.output_dir,
        &audio_store_dir,
    )?;

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
    use crate::ffmpeg_finder::detect_ffmpeg;

    let db = lock_db(&state)?;
    let mut settings = Settings::get(&db)?;

    // Auto-detect ffmpeg/ffprobe on first access if the user has not
    // manually specified the paths.
    if settings.ffmpeg_path.is_none() || settings.ffprobe_path.is_none() {
        let (ffmpeg, ffprobe) = detect_ffmpeg()?;
        settings.ffmpeg_path = settings.ffmpeg_path.or(ffmpeg);
        settings.ffprobe_path = settings.ffprobe_path.or(ffprobe);
        settings.save(&db)?;
    }

    if settings.ffmpeg_path.is_none() || settings.ffprobe_path.is_none() {
        return Err(Hoi4RadioError::FfmpegNotFound);
    }

    Ok(settings)
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<()> {
    let db = lock_db(&state)?;
    settings.save(&db)
}
