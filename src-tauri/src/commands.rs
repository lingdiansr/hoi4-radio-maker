use crate::audio::{analyze_audio, compute_file_hash, transcode_to_ogg};
use crate::audio_repo::AudioRepository;
use crate::db::{BatchImportResult, Db};
use crate::error::{Hoi4RadioError, Result};
use crate::generator::generate_mod;
use crate::models::{
    AudioFile, BatchUpdateAudioFileRequest, CreateProjectRequest, Project, UpdateAudioFileRequest,
    UpdateProjectRequest,
};
use crate::settings::{Settings, SettingsResponse};
use crate::station::StationRepository;
use crate::validator::validate_mod_output;
use chrono::Utc;
use futures::stream::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// Shared application state exposed to Tauri commands.
pub struct AppState {
    pub db: Mutex<Db>,
    pub cancel_import: Arc<AtomicBool>,
}

fn lock_db(state: &AppState) -> Result<std::sync::MutexGuard<'_, Db>> {
    state.db.lock().map_err(|e| Hoi4RadioError::Other {
        message: format!("database lock poisoned: {e}"),
    })
}

fn is_import_cancelled(state: &AppState) -> bool {
    state.cancel_import.load(Ordering::SeqCst)
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    req: CreateProjectRequest,
) -> Result<Project> {
    let output_dir = {
        let db = lock_db(&state)?;
        resolve_output_dir(&db, &req)?
    };

    // Ensure the output directory exists.
    tokio::fs::create_dir_all(&output_dir).await?;

    let mod_descriptor = write_mod_descriptor(&req, &output_dir).await?;

    let author = req
        .author
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(detect_system_user);

    let db_req = CreateProjectRequest {
        name: req.name.clone(),
        version: req.version.clone(),
        supported_version: req.supported_version.clone(),
        tags: req.tags.clone(),
        author,
        output_dir,
    };

    let db = lock_db(&state)?;
    let project = db.create_project(&db_req)?;

    tracing::info!(
        project_id = %project.id,
        output_dir = %project.output_dir.display(),
        mod_descriptor = %mod_descriptor.display(),
        "created new project"
    );

    Ok(project)
}

/// Resolve the project output directory from the request or settings.
fn resolve_output_dir(db: &Db, req: &CreateProjectRequest) -> Result<PathBuf> {
    if let Some(dir) = req.output_dir.to_str().filter(|s| !s.is_empty()) {
        let path = PathBuf::from(dir);
        if path.components().count() > 1 {
            return Ok(path);
        }
    }

    let settings = Settings::get(db)?;
    let base = settings
        .default_project_dir
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::document_dir)
        .ok_or_else(|| Hoi4RadioError::Other {
            message: "could not determine default project directory".to_string(),
        })?;

    let folder_name = sanitize_folder_name(&req.name);
    Ok(base.join("mod").join(&folder_name))
}

fn detect_system_user() -> Option<String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok()
        .filter(|s| !s.is_empty())
}

fn sanitize_folder_name(name: &str) -> String {
    name.trim()
        .replace(|c: char| c.is_ascii_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'), "_")
        .replace(' ', "_")
}

/// Write the HOI4 `.mod` descriptor next to the output directory.
async fn write_mod_descriptor(req: &CreateProjectRequest, output_dir: &Path) -> Result<PathBuf> {
    let folder_name = sanitize_folder_name(&req.name);
    let mod_file = output_dir
        .parent()
        .ok_or_else(|| Hoi4RadioError::Other {
            message: "output directory has no parent".to_string(),
        })?
        .join(format!("{}.mod", folder_name));

    let tags_line = req
        .tags
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(" ");
    let tags = if tags_line.is_empty() {
        "\"Sound\"".to_string()
    } else {
        format!("{{ {} }}", tags_line)
    };

    let mut lines = vec![
        format!("name=\"{}\"", req.name.replace('"', "\\\"")),
        format!("path=\"mod/{}\"", folder_name),
        format!("tags={}", tags),
        format!("version=\"{}\"", req.version.replace('"', "\\\"")),
        format!(
            "supported_version=\"{}\"",
            req.supported_version.replace('"', "\\\"")
        ),
    ];
    if let Some(author) = &req.author {
        lines.push(format!("author=\"{}\"", author.replace('"', "\\\"")));
    }
    lines.push(String::new());

    tokio::fs::write(&mod_file, lines.join("\n")).await?;
    Ok(mod_file)
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

    // output_dir is immutable after project creation; preserve the existing value.
    let existing = db
        .get_project(&id)?
        .ok_or_else(|| Hoi4RadioError::ProjectNotFound { id: id.clone() })?;

    let req = UpdateProjectRequest {
        name: req.name,
        version: req.version,
        supported_version: req.supported_version,
        tags: req.tags,
        author: req.author,
        output_dir: existing.output_dir,
    };

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

#[tauri::command]
pub fn update_audio_file(
    state: State<'_, AppState>,
    id: String,
    req: UpdateAudioFileRequest,
) -> Result<AudioFile> {
    let db = lock_db(&state)?;
    AudioRepository::new(&db).update(&id, &req)
}

#[tauri::command]
pub fn batch_update_audio_files(
    state: State<'_, AppState>,
    ids: Vec<String>,
    req: BatchUpdateAudioFileRequest,
) -> Result<Vec<AudioFile>> {
    let db = lock_db(&state)?;
    AudioRepository::new(&db).batch_update(&ids, &req)
}

#[derive(serde::Serialize, Clone)]
struct ImportFileEvent {
    session_id: String,
    path: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(serde::Serialize, Clone)]
struct ImportPhaseEvent {
    session_id: String,
    phase: String,
    current: usize,
    total: usize,
}

#[derive(serde::Serialize, Clone)]
struct ImportStartedEvent {
    session_id: String,
    total: usize,
    files: Vec<ImportFileEvent>,
}

#[derive(serde::Serialize, Clone)]
struct ImportResultEvent {
    session_id: String,
    created: Vec<AudioFile>,
    existing: Vec<AudioFile>,
    failed: Vec<ImportFailedFile>,
}

#[derive(serde::Serialize, Clone)]
struct ImportFailedFile {
    path: String,
    message: String,
}

fn emit_import_event(app: &AppHandle, event: &str, payload: impl serde::Serialize + Clone) {
    if let Err(e) = app.emit(event, payload) {
        tracing::warn!(error = %e, event = %event, "failed to emit import event");
    }
}

fn emit_import_file(
    app: &AppHandle,
    session_id: &str,
    path: impl Into<String>,
    status: &str,
    message: Option<String>,
) {
    emit_import_event(
        app,
        "import:file",
        ImportFileEvent {
            session_id: session_id.to_string(),
            path: path.into(),
            status: status.to_string(),
            message,
        },
    );
}

fn emit_import_phase(
    app: &AppHandle,
    session_id: &str,
    phase: &str,
    current: usize,
    total: usize,
) {
    emit_import_event(
        app,
        "import:phase",
        ImportPhaseEvent {
            session_id: session_id.to_string(),
            phase: phase.to_string(),
            current,
            total,
        },
    );
}

#[tauri::command]
pub fn cancel_import(state: State<'_, AppState>) -> Result<()> {
    tracing::info!("import cancellation requested");
    state.cancel_import.store(true, Ordering::SeqCst);
    Ok(())
}

async fn transcode_one(
    source_path: PathBuf,
    source_hash: String,
    audio_store_dir: &Path,
    ffmpeg_path: Option<&str>,
    ffprobe_path: Option<&str>,
) -> Result<AudioFile> {
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
    Ok(AudioFile {
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
    })
}

/// Batch import audio files into the global library.
///
/// If `project_id` is provided, the imported/duplicate audio files are also
/// added to that project's reference list.
#[tauri::command]
pub async fn import_audio_batch(
    state: State<'_, AppState>,
    app: AppHandle,
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

    state.cancel_import.store(false, Ordering::SeqCst);
    let session_id = format!("import_{}", Uuid::new_v4().to_string().replace('-', ""));

    emit_import_event(
        &app,
        "import:started",
        ImportStartedEvent {
            session_id: session_id.clone(),
            total: paths.len(),
            files: paths
                .iter()
                .map(|p| ImportFileEvent {
                    session_id: session_id.clone(),
                    path: p.clone(),
                    status: "pending".to_string(),
                    message: None,
                })
                .collect(),
        },
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
    emit_import_phase(&app, &session_id, "hashing", 0, paths.len());
    let mut hashed: Vec<(PathBuf, String)> = Vec::new();
    let mut failed: Vec<ImportFailedFile> = Vec::new();
    {
        let mut stream = futures::stream::iter(paths.clone())
            .map(|path| {
                let session_id = session_id.clone();
                let app = app.clone();
                async move {
                    let path = PathBuf::from(path);
                    let path_str = path.to_string_lossy().to_string();
                    emit_import_file(&app, &session_id, &path_str, "hashing", None);
                    let result = compute_file_hash(&path).await;
                    (path, result)
                }
            })
            .buffer_unordered(hash_concurrency);

        let mut processed = 0usize;
        while let Some((path, result)) = stream.next().await {
            if is_import_cancelled(&state) {
                tracing::info!("import cancelled during hashing");
                break;
            }
            processed += 1;
            let path_str = path.to_string_lossy().to_string();
            match result {
                Ok(hash) => {
                    hashed.push((path, hash));
                    emit_import_file(&app, &session_id, &path_str, "pending", None);
                }
                Err(e) => {
                    let msg = e.to_string();
                    failed.push(ImportFailedFile {
                        path: path_str.clone(),
                        message: msg.clone(),
                    });
                    emit_import_file(&app, &session_id, &path_str, "error", Some(msg));
                }
            }
            emit_import_phase(&app, &session_id, "hashing", processed, paths.len());
        }
    }

    // 2. Check database for existing files and split into new/existing.
    let hashed_len = hashed.len();
    emit_import_phase(&app, &session_id, "checking", 0, hashed_len);
    let mut existing: Vec<AudioFile> = Vec::new();
    let mut to_transcode: Vec<(PathBuf, String)> = Vec::new();
    {
        let db = lock_db(&state)?;
        let repo = AudioRepository::new(&db);
        for (idx, (path, hash)) in hashed.into_iter().enumerate() {
            let path_str = path.to_string_lossy().to_string();
            match repo.get_by_hash(&hash)? {
                Some(audio) => {
                    existing.push(audio);
                    emit_import_file(&app, &session_id, &path_str, "existing", None);
                }
                None => {
                    to_transcode.push((path, hash));
                    emit_import_file(&app, &session_id, &path_str, "pending", None);
                }
            }
            emit_import_phase(&app, &session_id, "checking", idx + 1, hashed_len);
        }
    }

    // 3. Transcode new files concurrently with limited ffmpeg processes.
    let to_transcode_len = to_transcode.len();
    emit_import_phase(&app, &session_id, "transcoding", 0, to_transcode_len);
    let mut created: Vec<AudioFile> = Vec::new();
    {
        let mut stream = futures::stream::iter(to_transcode)
            .map(|(source_path, source_hash)| {
                let audio_store_dir = audio_store_dir.clone();
                let session_id = session_id.clone();
                let app = app.clone();
                let ffmpeg_path = ffmpeg_path.map(|s| s.to_string());
                let ffprobe_path = ffprobe_path.map(|s| s.to_string());
                async move {
                    let path_str = source_path.to_string_lossy().to_string();
                    emit_import_file(&app, &session_id, &path_str, "transcoding", None);
                    let result = transcode_one(
                        source_path,
                        source_hash,
                        &audio_store_dir,
                        ffmpeg_path.as_deref(),
                        ffprobe_path.as_deref(),
                    )
                    .await;
                    (path_str, result)
                }
            })
            .buffer_unordered(transcode_concurrency);

        let mut processed = 0usize;
        while let Some((path_str, result)) = stream.next().await {
            if is_import_cancelled(&state) {
                tracing::info!("import cancelled during transcoding");
                break;
            }
            processed += 1;
            match result {
                Ok(audio) => {
                    created.push(audio);
                    emit_import_file(&app, &session_id, &path_str, "created", None);
                }
                Err(e) => {
                    let msg = e.to_string();
                    failed.push(ImportFailedFile {
                        path: path_str.clone(),
                        message: msg.clone(),
                    });
                    emit_import_file(&app, &session_id, &path_str, "error", Some(msg));
                }
            }
            emit_import_phase(&app, &session_id, "transcoding", processed, to_transcode_len);
        }
    }

    // 4. Persist everything and create project references if requested.
    emit_import_phase(&app, &session_id, "persisting", 0, created.len() + existing.len());
    {
        let db = lock_db(&state)?;
        let repo = AudioRepository::new(&db);
        for audio in &created {
            repo.create(audio)?;
        }
        if let Some(ref pid) = project_id {
            for audio in existing.iter().chain(created.iter()) {
                repo.add_to_project(pid, &audio.id)?;
            }
        }
    }
    emit_import_phase(
        &app,
        &session_id,
        "persisting",
        created.len() + existing.len(),
        created.len() + existing.len(),
    );

    let cancelled = is_import_cancelled(&state);
    let result = BatchImportResult {
        created: created.clone(),
        existing: existing.clone(),
    };
    let event = ImportResultEvent {
        session_id: session_id.clone(),
        created,
        existing,
        failed,
    };

    if cancelled {
        tracing::info!(
            created_count = result.created.len(),
            existing_count = result.existing.len(),
            project_id = ?project_id,
            "audio import batch cancelled"
        );
        emit_import_event(&app, "import:cancelled", event);
    } else {
        tracing::info!(
            created_count = result.created.len(),
            existing_count = result.existing.len(),
            project_id = ?project_id,
            "audio import batch completed"
        );
        emit_import_event(&app, "import:completed", event);
    }

    Ok(result)
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
    let repo = StationRepository::new(&db);
    if repo.find_by_name(&project_id, &name)?.is_some() {
        return Err(Hoi4RadioError::StationNameExists { name });
    }
    repo.create(&project_id, &name)
}

#[tauri::command]
pub fn delete_station(state: State<'_, AppState>, station_id: String) -> Result<()> {
    let db = lock_db(&state)?;
    StationRepository::new(&db).delete(&station_id)
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
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsResponse> {
    use crate::ffmpeg_finder::detect_ffmpeg;
    use crate::hoi4_version::detect_game_version;

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

    let ffmpeg_available = settings.ffmpeg_path.is_some() && settings.ffprobe_path.is_some();

    let detected_supported_version = settings
        .hoi4_game_dir
        .as_deref()
        .filter(|_| settings.default_supported_version.is_none())
        .and_then(|dir| detect_game_version(std::path::Path::new(dir)));

    Ok(SettingsResponse {
        settings,
        detected_supported_version,
        ffmpeg_available,
    })
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<()> {
    let db = lock_db(&state)?;
    settings.save(&db)
}

#[tauri::command]
pub fn get_default_project_dir(state: State<'_, AppState>) -> Result<String> {
    let db = lock_db(&state)?;
    let settings = Settings::get(&db)?;
    let base = settings
        .default_project_dir
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(dirs::document_dir)
        .ok_or_else(|| Hoi4RadioError::Other {
            message: "could not determine default project directory".to_string(),
        })?;
    Ok(base.join("mod").to_string_lossy().to_string())
}
