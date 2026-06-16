# HOI4 Radio Maker Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform Tauri desktop app for creating Hearts of Iron IV radio/music mods, supporting project management, audio library, multi-station editing, and one-click mod generation.

**Architecture:** Rust backend owns all domain logic (project persistence via SQLite, audio analysis/transcoding via ffmpeg/ffprobe, HOI4 file generation, validation); Vue 3 frontend provides the GUI and communicates through Tauri commands. The project follows a modular Rust layout matching hoi4skill-cli's style.

**Tech Stack:** Tauri 2, Vue 3 + TypeScript, Vite, Bun, Rust. Planned additions: Vuetify 3 (Material Design 3), Pinia, Vue Router, rusqlite, serde, tempfile, tokio::process for ffmpeg.

---

## File Structure

```
/home/ldsr/code/Rust/hoi4-radio-maker/
├── src/                          # Vue 3 frontend
│   ├── main.ts                   # Frontend entry
│   ├── App.vue                   # Root component
│   ├── router.ts                 # Vue Router config
│   ├── vite-env.d.ts
│   ├── assets/
│   ├── stores/
│   │   └── project.ts            # Pinia store
│   ├── views/
│   │   ├── WelcomeView.vue
│   │   ├── ProjectView.vue
│   │   ├── AudioLibraryView.vue
│   │   ├── StationEditorView.vue
│   │   └── SettingsView.vue
│   └── components/
│       ├── ProjectList.vue
│       ├── AudioImporter.vue
│       ├── StationList.vue
│       └── SongEntryEditor.vue
├── src-tauri/                    # Rust backend (Tauri 2)
│   ├── src/
│   │   ├── main.rs               # Binary entry
│   │   ├── lib.rs                # Library entry: modules & Tauri commands
│   │   ├── error.rs              # Hoi4RadioError
│   │   ├── db.rs                 # SQLite setup & migrations
│   │   ├── models.rs             # Project, AudioFile, Station, etc.
│   │   ├── project.rs            # Project CRUD
│   │   ├── audio.rs              # Audio analysis and transcoding
│   │   ├── audio_repo.rs         # Audio file persistence helpers
│   │   ├── station.rs            # Station CRUD & entries
│   │   ├── generator.rs          # HOI4 mod file generation
│   │   ├── validator.rs          # Output validation
│   │   ├── settings.rs           # App settings persistence
│   │   └── commands.rs           # Tauri command handlers
│   ├── tests/                    # Rust integration tests
│   │   ├── models_test.rs
│   │   ├── db_test.rs
│   │   ├── audio_test.rs
│   │   ├── audio_repo_test.rs
│   │   ├── station_test.rs
│   │   ├── generator_test.rs
│   │   ├── validator_test.rs
│   │   └── integration_test.rs
│   ├── capabilities/
│   ├── icons/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── index.html                    # Vite entry HTML (project root)
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
└── docs/
    └── superpowers/
        └── specs/2026-06-12-hoi4radio-design.md
```

---

## Task 1: Project Already Bootstrapped

The project was initialized via `cargo create-tauri-app` using Tauri 2, Vue 3, TypeScript, Vite, and Bun. The following key files already exist and should not be recreated:

- `package.json` — Bun workspace with Vue/Vite scripts
- `vite.config.ts`
- `index.html`
- `tsconfig.json` / `tsconfig.node.json`
- `src/main.ts`
- `src/App.vue`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/build.rs`
- `src-tauri/src/main.rs` — binary entry
- `src-tauri/src/lib.rs` — library entry where modules are exposed and Tauri commands are registered

Key configuration values already in place:

`src-tauri/Cargo.toml`:

```toml
[package]
name = "hoi4-radio-maker"
version = "0.1.0"
description = "HOI4 Radio Maker"
authors = ["you"]
edition = "2021"

[lib]
name = "hoi4_radio_maker_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["process", "rt-multi-thread"] }
rusqlite = { version = "0.31.0", features = ["bundled", "chrono", "serde_json"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
tempfile = "3.10"

[features]
custom-protocol = ["tauri/custom-protocol"]
```

`src-tauri/tauri.conf.json`:

```json
{
  "productName": "hoi4-radio-maker",
  "version": "0.1.0",
  "identifier": "xyz.ldsr.hoi4-radio-maker",
  "build": {
    "beforeBuildCommand": "bun run build",
    "beforeDevCommand": "bun run dev",
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [
      {
        "title": "hoi4-radio-maker",
        "width": 1280,
        "height": 800,
        "resizable": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
```

`src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    hoi4_radio_maker_lib::run();
}
```

- [ ] **Verify dev build runs**

```bash
cd /home/ldsr/code/Rust/hoi4-radio-maker
bun install && bun run tauri dev
```

Expected: an empty window titled "hoi4-radio-maker" opens without errors.

---

## Task 2: Define Error Types and Core Models

**Files:**
- Create: `src-tauri/src/error.rs`, `src-tauri/src/models.rs`
- Modify: `src-tauri/src/lib.rs` to expose modules

- [ ] **Step 1: Write the failing model test**

`src-tauri/tests/models_test.rs`:

```rust
use hoi4_radio_maker_lib::models::Project;

#[test]
fn test_project_has_id_and_name() {
    let p = Project {
        id: "proj_1".into(),
        name: "My Radio".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec!["Sound".into()],
        author: Some("Alice".into()),
        output_dir: std::path::PathBuf::from("/tmp/out"),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert_eq!(p.name, "My Radio");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test models_test
```

Expected: FAIL because `models` module and `Project` struct do not exist.

- [ ] **Step 3: Implement error types and models**

`src-tauri/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Hoi4RadioError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Serde error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Audio analysis failed: {0}")]
    AudioAnalysis(String),

    #[error("Transcoding failed: {0}")]
    Transcoding(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Hoi4RadioError>;
```

`src-tauri/src/models.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub version: String,
    pub supported_version: String,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub output_dir: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub source_path: PathBuf,
    pub ogg_filename: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u32,
    pub volume: f64,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub entries: Vec<StationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationEntry {
    pub audio_file_id: String,
    pub chance: ChanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceConfig {
    pub factor: f64,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub factor: Option<f64>,
    pub add: Option<f64>,
    pub base: Option<f64>,
    pub triggers: Vec<Trigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    HasWar { value: bool },
    Tag { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub version: String,
    pub supported_version: String,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub output_dir: PathBuf,
}
```

- [ ] **Step 4: Update `src-tauri/src/lib.rs` to expose modules**

```rust
pub mod error;
pub mod models;

pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test --test models_test
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/models.rs src-tauri/src/lib.rs src-tauri/tests/models_test.rs
git commit -m "feat: add error types and core models"
```


---

## Task 3: SQLite Database Setup and Migrations

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing DB test**

`src-tauri/tests/db_test.rs`:

```rust
use hoi4_radio_maker_lib::db::Db;
use tempfile::TempDir;

#[test]
fn test_db_creates_tables() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.db");
    let db = Db::open(&path).unwrap();
    db.create_project(&hoi4_radio_maker_lib::models::CreateProjectRequest {
        name: "Test".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec![],
        author: None,
        output_dir: tmp.path().join("out"),
    }).unwrap();
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test db_test
```

Expected: FAIL because `Db` does not exist.

- [ ] **Step 3: Implement database layer**

`src-tauri/src/db.rs`:

```rust
use crate::error::{Hoi4RadioError, Result};
use crate::models::{CreateProjectRequest, Project};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                supported_version TEXT NOT NULL,
                tags TEXT NOT NULL,
                author TEXT,
                output_dir TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audio_files (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                artist TEXT,
                source_path TEXT NOT NULL,
                ogg_filename TEXT NOT NULL,
                duration_secs REAL,
                sample_rate INTEGER,
                channels INTEGER,
                volume REAL NOT NULL DEFAULT 0.75,
                tags TEXT NOT NULL DEFAULT '[]',
                notes TEXT,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS stations (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS station_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                station_id TEXT NOT NULL,
                audio_file_id TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                chance_config TEXT NOT NULL,
                FOREIGN KEY (station_id) REFERENCES stations(id) ON DELETE CASCADE,
                FOREIGN KEY (audio_file_id) REFERENCES audio_files(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            
            PRAGMA foreign_keys = ON;"
        )?;
        Ok(())
    }

    pub fn create_project(&self, req: &CreateProjectRequest) -> Result<Project> {
        let id = format!("proj_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        let now = Utc::now().to_rfc3339();
        let tags = serde_json::to_string(&req.tags)?;
        let output_dir = req.output_dir.to_string_lossy().to_string();

        self.conn.execute(
            "INSERT INTO projects (id, name, version, supported_version, tags, author, output_dir, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (&id, &req.name, &req.version, &req.supported_version, &tags, &req.author, &output_dir, &now, &now),
        )?;

        self.get_project(&id)?.ok_or_else(|| Hoi4RadioError::Other("Failed to read created project".into()))
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, supported_version, tags, author, output_dir, created_at, updated_at
             FROM projects WHERE id = ?1"
        )?;
        let row = stmt.query_row([id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                supported_version: row.get(3)?,
                tags: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                author: row.get(5)?,
                output_dir: row.get::<_, String>(6)?.into(),
                created_at: row.get::<_, String>(7)?.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: row.get::<_, String>(8)?.parse().unwrap_or_else(|_| Utc::now()),
            })
        }).optional()?;
        Ok(row)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, supported_version, tags, author, output_dir, created_at, updated_at
             FROM projects ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                supported_version: row.get(3)?,
                tags: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                author: row.get(5)?,
                output_dir: row.get::<_, String>(6)?.into(),
                created_at: row.get::<_, String>(7)?.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: row.get::<_, String>(8)?.parse().unwrap_or_else(|_| Utc::now()),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM projects WHERE id = ?1", [id])?;
        Ok(())
    }
}
```

- [ ] **Step 4: Add `uuid` dependency to `src-tauri/Cargo.toml`**

```toml
uuid = { version = "1.8", features = ["v4"] }
```

- [ ] **Step 5: Update `src-tauri/src/lib.rs` to include `db` module**

```rust
pub mod db;
pub mod error;
pub mod models;
```

- [ ] **Step 6: Run tests**

```bash
cd src-tauri && cargo test --test db_test
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/db.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/tests/db_test.rs
git commit -m "feat: add SQLite database layer and project CRUD"
```

---

## Task 4: Audio Import and Metadata Extraction

**Files:**
- Create: `src-tauri/src/audio.rs`

- [ ] **Step 1: Write failing audio metadata test**

`src-tauri/tests/audio_test.rs`:

```rust
use hoi4_radio_maker_lib::audio::analyze_audio;
use std::path::PathBuf;

#[tokio::test]
async fn test_analyze_missing_file_fails() {
    let result = analyze_audio(PathBuf::from("/nonexistent/file.mp3")).await;
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test audio_test
```

Expected: FAIL because `analyze_audio` is undefined.

- [ ] **Step 3: Implement audio module**

`src-tauri/src/audio.rs`:

```rust
use crate::error::{Hoi4RadioError, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioMetadata {
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u32,
}

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    sample_rate: Option<String>,
    channels: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
}

pub async fn analyze_audio<P: AsRef<Path>>(path: P) -> Result<AudioMetadata> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            "-show_format",
            path.as_ref().to_str().unwrap_or(""),
        ])
        .output()
        .await
        .map_err(|e| Hoi4RadioError::AudioAnalysis(format!("ffprobe failed: {e}")))?;

    if !output.status.success() {
        return Err(Hoi4RadioError::AudioAnalysis(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
    }

    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)?;
    let stream = parsed.streams.first().ok_or_else(|| {
        Hoi4RadioError::AudioAnalysis("No audio streams found".into())
    })?;

    let sample_rate = stream
        .sample_rate
        .as_ref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(44100);

    let channels = stream.channels.unwrap_or(2);

    let duration_secs = parsed
        .format
        .and_then(|f| f.duration)
        .and_then(|d| d.parse().ok())
        .unwrap_or(0.0);

    Ok(AudioMetadata {
        duration_secs,
        sample_rate,
        channels,
    })
}

pub async fn transcode_to_ogg<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    output: Q,
) -> Result<PathBuf> {
    let out = output.as_ref().to_path_buf();
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", input.as_ref().to_str().unwrap_or(""),
            "-ar", "44100",
            "-c:a", "libvorbis",
            "-q:a", "4",
            out.to_str().unwrap_or(""),
        ])
        .status()
        .await
        .map_err(|e| Hoi4RadioError::Transcoding(format!("ffmpeg failed: {e}")))?;

    if !status.success() {
        return Err(Hoi4RadioError::Transcoding("ffmpeg returned non-zero".into()));
    }

    Ok(out)
}
```

- [ ] **Step 4: Add `AudioFile` CRUD methods to `src-tauri/src/db.rs`**

Add `AudioFile` to the `use crate::models::{...}` import and append the following methods to `impl Db`:

```rust
use crate::models::{AudioFile, CreateProjectRequest, Project};

impl Db {
    // ... existing project methods ...

    pub fn create_audio_file(&self, project_id: &str, audio: &AudioFile) -> Result<AudioFile> {
        let tags = serde_json::to_string(&audio.tags)?;
        let source_path = audio.source_path.to_string_lossy().to_string();
        self.conn.execute(
            "INSERT INTO audio_files (id, project_id, title, artist, source_path, ogg_filename, duration_secs, sample_rate, channels, volume, tags, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            (
                &audio.id,
                project_id,
                &audio.title,
                &audio.artist,
                &source_path,
                &audio.ogg_filename,
                &audio.duration_secs,
                &audio.sample_rate,
                &audio.channels,
                &audio.volume,
                &tags,
                &audio.notes,
            ),
        )?;
        self.get_audio_file(&audio.id)?
            .ok_or_else(|| Hoi4RadioError::Other("Failed to read created audio file".into()))
    }

    pub fn list_audio_files(&self, project_id: &str) -> Result<Vec<AudioFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, artist, source_path, ogg_filename, duration_secs, sample_rate, channels, volume, tags, notes
             FROM audio_files WHERE project_id = ?1 ORDER BY title"
        )?;
        let rows = stmt.query_map([project_id], |row| {
            Ok(AudioFile {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                source_path: row.get::<_, String>(3)?.into(),
                ogg_filename: row.get(4)?,
                duration_secs: row.get(5)?,
                sample_rate: row.get(6)?,
                channels: row.get(7)?,
                volume: row.get(8)?,
                tags: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                notes: row.get(10)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_audio_file(&self, id: &str) -> Result<Option<AudioFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, artist, source_path, ogg_filename, duration_secs, sample_rate, channels, volume, tags, notes
             FROM audio_files WHERE id = ?1"
        )?;
        let row = stmt.query_row([id], |row| {
            Ok(AudioFile {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                source_path: row.get::<_, String>(3)?.into(),
                ogg_filename: row.get(4)?,
                duration_secs: row.get(5)?,
                sample_rate: row.get(6)?,
                channels: row.get(7)?,
                volume: row.get(8)?,
                tags: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
                notes: row.get(10)?,
            })
        }).optional()?;
        Ok(row)
    }

    pub fn delete_audio_file(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM audio_files WHERE id = ?1", [id])?;
        Ok(())
    }
}
```

- [ ] **Step 5: Create `src-tauri/src/audio_repo.rs` persistence helpers**

```rust
use crate::db::Db;
use crate::error::Result;
use crate::models::AudioFile;

pub struct AudioRepository<'a> {
    db: &'a Db,
}

impl<'a> AudioRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub fn create(&self, project_id: &str, audio: &AudioFile) -> Result<AudioFile> {
        self.db.create_audio_file(project_id, audio)
    }

    pub fn list(&self, project_id: &str) -> Result<Vec<AudioFile>> {
        self.db.list_audio_files(project_id)
    }

    pub fn get(&self, id: &str) -> Result<Option<AudioFile>> {
        self.db.get_audio_file(id)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.db.delete_audio_file(id)
    }
}
```

- [ ] **Step 6: Write the audio CRUD test**

`src-tauri/tests/audio_repo_test.rs`:

```rust
use hoi4_radio_maker_lib::db::Db;
use hoi4_radio_maker_lib::models::{AudioFile, CreateProjectRequest};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_audio_file_crud() {
    let tmp = TempDir::new().unwrap();
    let db = Db::open(tmp.path().join("test.db")).unwrap();
    let project = db.create_project(&CreateProjectRequest {
        name: "Test".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec![],
        author: None,
        output_dir: tmp.path().join("out"),
    }).unwrap();

    let audio = AudioFile {
        id: "audio_1".into(),
        title: "Test Song".into(),
        artist: None,
        source_path: PathBuf::from("/tmp/test.ogg"),
        ogg_filename: "audio_1.ogg".into(),
        duration_secs: 120.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.75,
        tags: vec![],
        notes: None,
    };

    db.create_audio_file(&project.id, &audio).unwrap();
    let list = db.list_audio_files(&project.id).unwrap();
    assert_eq!(list.len(), 1);

    let fetched = db.get_audio_file("audio_1").unwrap().unwrap();
    assert_eq!(fetched.title, "Test Song");

    db.delete_audio_file("audio_1").unwrap();
    assert!(db.get_audio_file("audio_1").unwrap().is_none());
}
```

- [ ] **Step 7: Run tests**

```bash
cd src-tauri && cargo test --test audio_test && cargo test --test audio_repo_test
```

Expected: both PASS.

- [ ] **Step 8: Update `src-tauri/src/lib.rs` to include `audio` and `audio_repo` modules**

```rust
pub mod audio;
pub mod audio_repo;
```

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/db.rs src-tauri/src/audio.rs src-tauri/src/audio_repo.rs src-tauri/src/lib.rs src-tauri/tests/audio_test.rs src-tauri/tests/audio_repo_test.rs
git commit -m "feat: add audio analysis, transcoding, persistence, and repository"
```


---

## Task 5: Station Model and CRUD

**Files:**
- Create: `src-tauri/src/station.rs`

- [ ] **Step 1: Write failing station test**

`src-tauri/tests/station_test.rs`:

```rust
use hoi4_radio_maker_lib::station::StationRepository;
use hoi4_radio_maker_lib::db::Db;
use tempfile::TempDir;

#[test]
fn test_create_station() {
    let tmp = TempDir::new().unwrap();
    let db = Db::open(tmp.path().join("test.db")).unwrap();
    let repo = StationRepository::new(&db);
    let proj = db.create_project(&hoi4_radio_maker_lib::models::CreateProjectRequest {
        name: "Test".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec![],
        author: None,
        output_dir: tmp.path().join("out"),
    }).unwrap();

    let station = repo.create(&proj.id, "Main Station").unwrap();
    assert_eq!(station.name, "Main Station");
    assert!(station.entries.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test station_test
```

Expected: FAIL.

- [ ] **Step 3: Implement station module**

`src-tauri/src/station.rs`:

```rust
use crate::db::Db;
use crate::error::{Hoi4RadioError, Result};
use crate::models::{ChanceConfig, Station, StationEntry};
use serde_json;
use uuid::Uuid;

pub struct StationRepository<'a> {
    db: &'a Db,
}

impl<'a> StationRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub fn create(&self, project_id: &str, name: &str) -> Result<Station> {
        let id = format!("station_{}", Uuid::new_v4().to_string().replace('-', ""));
        self.db.conn().execute(
            "INSERT INTO stations (id, project_id, name) VALUES (?1, ?2, ?3)",
            (&id, project_id, name),
        )?;
        self.get(&id)?.ok_or_else(|| Hoi4RadioError::Other("Failed to read created station".into()))
    }

    pub fn get(&self, id: &str) -> Result<Option<Station>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name FROM stations WHERE id = ?1"
        )?;
        let station = stmt.query_row([id], |row| {
            Ok(Station {
                id: row.get(0)?,
                name: row.get(1)?,
                entries: vec![],
            })
        }).optional()?;

        Ok(match station {
            Some(mut s) => {
                s.entries = self.list_entries(&s.id)?;
                Some(s)
            }
            None => None,
        })
    }

    pub fn list_by_project(&self, project_id: &str) -> Result<Vec<Station>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name FROM stations WHERE project_id = ?1 ORDER BY sort_order, id"
        )?;
        let rows = stmt.query_map([project_id], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((id, name))
        })?;

        let mut stations = Vec::new();
        for row in rows {
            let (id, name) = row?;
            let entries = self.list_entries(&id)?;
            stations.push(Station { id, name, entries });
        }
        Ok(stations)
    }

    pub fn add_entry(&self, station_id: &str, audio_file_id: &str, chance: ChanceConfig) -> Result<()> {
        let chance_json = serde_json::to_string(&chance)?;
        self.db.conn().execute(
            "INSERT INTO station_entries (station_id, audio_file_id, chance_config) VALUES (?1, ?2, ?3)",
            (station_id, audio_file_id, chance_json),
        )?;
        Ok(())
    }

    pub fn remove_entry(&self, station_id: &str, audio_file_id: &str) -> Result<()> {
        self.db.conn().execute(
            "DELETE FROM station_entries WHERE station_id = ?1 AND audio_file_id = ?2",
            (station_id, audio_file_id),
        )?;
        Ok(())
    }

    fn list_entries(&self, station_id: &str) -> Result<Vec<StationEntry>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT audio_file_id, chance_config FROM station_entries WHERE station_id = ?1 ORDER BY sort_order, id"
        )?;
        let rows = stmt.query_map([station_id], |row| {
            let audio_file_id: String = row.get(0)?;
            let chance_json: String = row.get(1)?;
            let chance: ChanceConfig = serde_json::from_str(&chance_json).unwrap_or(ChanceConfig { factor: 1.0, modifiers: vec![] });
            Ok(StationEntry { audio_file_id, chance })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.db.conn().execute("DELETE FROM stations WHERE id = ?1", [id])?;
        Ok(())
    }
}
```

- [ ] **Step 4: Update `src-tauri/src/lib.rs` to include `station` module**

```rust
pub mod station;
```

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test --test station_test
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/station.rs src-tauri/src/lib.rs src-tauri/tests/station_test.rs
git commit -m "feat: add station CRUD and entries"
```

---

## Task 6: HOI4 Mod File Generator

**Files:**
- Create: `src-tauri/src/generator.rs`

- [ ] **Step 1: Write failing generator test**

`src-tauri/tests/generator_test.rs`:

```rust
use hoi4_radio_maker_lib::generator::generate_mod;
use hoi4_radio_maker_lib::models::{AudioFile, ChanceConfig, Project, Station, StationEntry};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_generate_mod_creates_files() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("mod_out");
    let project = Project {
        id: "proj_test".into(),
        name: "Test Mod".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec!["Sound".into()],
        author: None,
        output_dir: out.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let audio = AudioFile {
        id: "song_001".into(),
        title: "Test Song".into(),
        artist: None,
        source_path: PathBuf::from("/tmp/test.mp3"),
        ogg_filename: "song_001.ogg".into(),
        duration_secs: 120.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.75,
        tags: vec![],
        notes: None,
    };
    let station = Station {
        id: "station_main".into(),
        name: "Main".into(),
        entries: vec![StationEntry {
            audio_file_id: "song_001".into(),
            chance: ChanceConfig { factor: 1.0, modifiers: vec![] },
        }],
    };

    generate_mod(&project, &[station], &[audio], &out).unwrap();

    assert!(out.join("descriptor.mod").exists());
    assert!(out.join("music/station_main.asset").exists());
    assert!(out.join("music/station_main.txt").exists());
    assert!(out.join("localisation/simp_chinese/proj_test_music_l_simp_chinese.yml").exists());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test generator_test
```

Expected: FAIL.

- [ ] **Step 3: Implement generator module**

`src-tauri/src/generator.rs`:

```rust
use crate::error::{Hoi4RadioError, Result};
use crate::models::{AudioFile, Project, Station, Trigger};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn generate_mod<P: AsRef<Path>>(
    project: &Project,
    stations: &[Station],
    audio_files: &[AudioFile],
    output_root: P,
) -> Result<()> {
    let root = output_root.as_ref();
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    fs::create_dir_all(root)?;
    fs::create_dir_all(root.join("music"))?;
    fs::create_dir_all(root.join("localisation/simp_chinese"))?;

    generate_descriptor_mod(project, root)?;
    generate_launcher_mod_file(project, root)?;

    let audio_map: HashMap<&str, &AudioFile> = audio_files.iter().map(|a| (a.id.as_str(), a)).collect();

    for station in stations {
        generate_station_asset(station, &audio_map, root)?;
        generate_station_txt(station, &audio_map, root)?;
    }

    generate_localisation(project, audio_files, root)?;
    Ok(())
}

fn generate_descriptor_mod(project: &Project, root: &Path) -> Result<()> {
    let tags = project.tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join("\n    ");
    let content = format!(
        "name=\"{}\"\nversion=\"{}\"\nsupported_version=\"{}\"\ntags={{\n    {}\n}}\n",
        escape_hoi4_string(&project.name),
        project.version,
        project.supported_version,
        tags
    );
    fs::write(root.join("descriptor.mod"), content)?;
    Ok(())
}

fn generate_launcher_mod_file(project: &Project, root: &Path) -> Result<()> {
    let dir_name = root.file_name().unwrap_or_else(|| root.as_os_str()).to_string_lossy();
    let content = format!(
        "name=\"{}\"\npath=\"mod/{}\"\nversion=\"{}\"\nsupported_version=\"{}\"\n",
        escape_hoi4_string(&project.name),
        dir_name,
        project.version,
        project.supported_version
    );
    fs::write(root.join("../").join(format!("{}.mod", dir_name)), content)?;
    Ok(())
}

fn generate_station_asset(
    station: &Station,
    audio_map: &HashMap<&str, &AudioFile>,
    root: &Path,
) -> Result<()> {
    let mut lines = vec![];
    for entry in &station.entries {
        let audio = audio_map.get(entry.audio_file_id.as_str()).ok_or_else(|| {
            Hoi4RadioError::Validation(format!("Audio file {} not found", entry.audio_file_id))
        })?;
        lines.push(format!(
            "music = {{\n    name = \"{}\"\n    file = \"{}\"\n    volume = {:.2}\n}}\n",
            audio.id, audio.ogg_filename, audio.volume
        ));
    }
    fs::write(root.join("music").join(format!("{}.asset", station.id)), lines.join("\n"))?;
    Ok(())
}

fn generate_station_txt(
    station: &Station,
    audio_map: &HashMap<&str, &AudioFile>,
    root: &Path,
) -> Result<()> {
    let mut lines = vec![format!("music_station = \"{}\"\n", station.id)];
    for entry in &station.entries {
        let _audio = audio_map.get(entry.audio_file_id.as_str()).ok_or_else(|| {
            Hoi4RadioError::Validation(format!("Audio file {} not found", entry.audio_file_id))
        })?;

        let mut modifier_lines = vec![];
        for m in &entry.chance.modifiers {
            if let Some(factor) = m.factor {
                modifier_lines.push(format!("            factor = {}", factor));
            }
            for trigger in &m.triggers {
                match trigger {
                    Trigger::HasWar { value } => {
                        modifier_lines.push(format!("            has_war = {}", if *value { "yes" } else { "no" }));
                    }
                    Trigger::Tag { value } => {
                        modifier_lines.push(format!("            tag = {}", value));
                    }
                }
            }
        }

        let chance_block = if modifier_lines.is_empty() {
            format!("        factor = {}", entry.chance.factor)
        } else {
            format!(
                "        factor = {}\n        modifier = {{\n{}\n        }}",
                entry.chance.factor,
                modifier_lines.join("\n")
            )
        };

        lines.push(format!(
            "music = {{\n    song = \"{}\"\n    chance = {{\n{}\n    }}\n}}\n",
            entry.audio_file_id, chance_block
        ));
    }

    fs::write(root.join("music").join(format!("{}.txt", station.id)), lines.join("\n"))?;
    Ok(())
}

fn generate_localisation(project: &Project, audio_files: &[AudioFile], root: &Path) -> Result<()> {
    let mut lines = vec!["l_simp_chinese:".into()];
    for audio in audio_files {
        lines.push(format!("  {}:0 \"{}\"", audio.id, escape_hoi4_string(&audio.title)));
    }
    let content = lines.join("\n") + "\n";
    let file_name = format!("{}_music_l_simp_chinese.yml", project.id);
    fs::write(root.join("localisation/simp_chinese").join(file_name), content)?;
    Ok(())
}

fn escape_hoi4_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
```

- [ ] **Step 4: Update `src-tauri/src/lib.rs` to include `generator` module**

```rust
pub mod generator;
```

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test --test generator_test
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/generator.rs src-tauri/src/lib.rs src-tauri/tests/generator_test.rs
git commit -m "feat: add HOI4 mod file generator"
```


---

## Task 6.5: Validation Module

**Files:**
- Create: `src-tauri/src/validator.rs`, `src-tauri/tests/validator_test.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing validator test**

`src-tauri/tests/validator_test.rs`:

```rust
use hoi4_radio_maker_lib::generator::generate_mod;
use hoi4_radio_maker_lib::models::{AudioFile, ChanceConfig, Project, Station, StationEntry};
use hoi4_radio_maker_lib::validator::validate_mod_output;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_validate_generated_mod_reports_missing_ogg() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("mod_out");
    let project = Project {
        id: "proj_test".into(),
        name: "Test Mod".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec!["Sound".into()],
        author: None,
        output_dir: out.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let audio = AudioFile {
        id: "song_001".into(),
        title: "Test Song".into(),
        artist: None,
        source_path: PathBuf::from("/tmp/test.mp3"),
        ogg_filename: "song_001.ogg".into(),
        duration_secs: 120.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.75,
        tags: vec![],
        notes: None,
    };
    let station = Station {
        id: "station_main".into(),
        name: "Main".into(),
        entries: vec![StationEntry {
            audio_file_id: "song_001".into(),
            chance: ChanceConfig { factor: 1.0, modifiers: vec![] },
        }],
    };

    generate_mod(&project, &[station], &[audio], &out).unwrap();

    let report = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(validate_mod_output(&out))
        .unwrap();
    assert!(!report.passed);
    assert!(report.errors.iter().any(|e| e.contains("song_001.ogg")));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test --test validator_test
```

Expected: FAIL because `validator` module does not exist.

- [ ] **Step 3: Implement validator module**

`src-tauri/src/validator.rs`:

```rust
use crate::error::{Hoi4RadioError, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub ogg_files_checked: usize,
}

pub async fn validate_mod_output(output_dir: &Path) -> Result<ValidationReport> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut ogg_files_checked = 0;

    let music_dir = output_dir.join("music");
    if !music_dir.exists() {
        errors.push(format!("Missing music directory: {}", music_dir.display()));
        return Ok(ValidationReport {
            passed: false,
            errors,
            warnings,
            ogg_files_checked: 0,
        });
    }

    let mut asset_songs: HashMap<String, Vec<String>> = HashMap::new();
    let mut txt_songs: HashMap<String, HashSet<String>> = HashMap::new();

    for entry in std::fs::read_dir(&music_dir)? {
        let entry = entry?;
        let path = entry.path();
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        if path.extension().and_then(|s| s.to_str()) == Some("asset") {
            let ids = parse_asset_songs(&path)?;
            for id in &ids {
                if !is_valid_id(id) {
                    errors.push(format!("Invalid audio id in {}.asset: {}", stem, id));
                }
            }
            asset_songs.insert(stem.clone(), ids);
        }

        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let ids = parse_txt_songs(&path)?;
            for id in &ids {
                if !is_valid_id(id) {
                    errors.push(format!("Invalid audio id in {}.txt: {}", stem, id));
                }
            }
            txt_songs.insert(stem, ids);
        }
    }

    for (station, songs) in &asset_songs {
        let txt_set = txt_songs.get(station).cloned().unwrap_or_default();
        for song in songs {
            if !txt_set.contains(song) {
                errors.push(format!(
                    "{}.asset references song {} missing from {}.txt",
                    station, song, station
                ));
            }
        }
    }

    for (station, songs) in &txt_songs {
        let asset_list = asset_songs.get(station).cloned().unwrap_or_default();
        let asset_set: HashSet<_> = asset_list.into_iter().collect();
        for song in songs {
            if !asset_set.contains(song) {
                errors.push(format!(
                    "{}.txt references song {} missing from {}.asset",
                    station, song, station
                ));
            }
        }
    }

    let mut localised: HashSet<String> = HashSet::new();
    let loc_dir = output_dir.join("localisation/simp_chinese");
    if loc_dir.exists() {
        for entry in std::fs::read_dir(&loc_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yml") {
                localised.extend(parse_localisation_keys(&entry.path())?);
            }
        }
    } else {
        warnings.push(format!("Missing localisation directory: {}", loc_dir.display()));
    }

    let all_songs: HashSet<String> = asset_songs
        .values()
        .flat_map(|v| v.iter().cloned())
        .collect();
    for song in &all_songs {
        if !localised.contains(song) {
            errors.push(format!("Missing localisation key for song: {}", song));
        }
    }

    for (station, _) in &asset_songs {
        let asset_path = music_dir.join(format!("{}.asset", station));
        let content = std::fs::read_to_string(&asset_path)?;
        let ogg_refs: Vec<String> = content
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                if l.starts_with("file") {
                    l.splitn(2, '=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"').to_string())
                } else {
                    None
                }
            })
            .collect();

        for ogg in ogg_refs {
            let ogg_path = music_dir.join(&ogg);
            if !ogg_path.exists() {
                errors.push(format!("Referenced OGG file does not exist: {}", ogg_path.display()));
            } else {
                ogg_files_checked += 1;
                if let Err(e) = check_ogg_decodable(&ogg_path).await {
                    errors.push(format!("OGG file not decodable ({}): {}", ogg_path.display(), e));
                }
            }
        }
    }

    let passed = errors.is_empty();
    Ok(ValidationReport {
        passed,
        errors,
        warnings,
        ogg_files_checked,
    })
}

fn parse_asset_songs(path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let mut names = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            if let Some(val) = line.splitn(2, '=').nth(1) {
                names.push(val.trim().trim_matches('"').to_string());
            }
        }
    }
    Ok(names)
}

fn parse_txt_songs(path: &Path) -> Result<HashSet<String>> {
    let content = std::fs::read_to_string(path)?;
    let mut songs = HashSet::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("song") {
            if let Some(val) = line.splitn(2, '=').nth(1) {
                songs.insert(val.trim().trim_matches('"').to_string());
            }
        }
    }
    Ok(songs)
}

fn parse_localisation_keys(path: &Path) -> Result<HashSet<String>> {
    let content = std::fs::read_to_string(path)?;
    let mut keys = HashSet::new();
    for line in content.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(key) = line.splitn(2, ':').next() {
            keys.insert(key.trim().to_string());
        }
    }
    Ok(keys)
}

async fn check_ogg_decodable(path: &Path) -> Result<()> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_format",
            "-show_streams",
            path.to_str().unwrap_or(""),
        ])
        .output()
        .await
        .map_err(|e| Hoi4RadioError::Validation(format!("ffprobe failed: {}", e)))?;

    if !output.status.success() {
        return Err(Hoi4RadioError::Validation(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(())
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
```

- [ ] **Step 4: Update `src-tauri/src/lib.rs` to include `validator` module**

```rust
pub mod validator;
```

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test --test validator_test
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/validator.rs src-tauri/src/lib.rs src-tauri/tests/validator_test.rs
git commit -m "feat: add mod output validator"
```

> The `validate_mod` Tauri command will be wired in Task 7.


---

## Task 7: Tauri Command Layer

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement command handlers**

`src-tauri/src/commands.rs`:

```rust
use crate::audio_repo::AudioRepository;
use crate::db::Db;
use crate::error::{Hoi4RadioError, Result};
use crate::generator::generate_mod;
use crate::models::{AudioFile, CreateProjectRequest, Project};
use crate::station::StationRepository;
use crate::validator::validate_mod_output;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

pub struct AppState {
    pub db: Mutex<Db>,
}

#[tauri::command]
pub fn create_project(state: State<AppState>, req: CreateProjectRequest) -> Result<Project> {
    state.db.lock().unwrap().create_project(&req)
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>> {
    state.db.lock().unwrap().list_projects()
}

#[tauri::command]
pub fn get_project(state: State<AppState>, id: String) -> Result<Option<Project>> {
    state.db.lock().unwrap().get_project(&id)
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<()> {
    state.db.lock().unwrap().delete_project(&id)
}

#[tauri::command]
pub fn list_stations(state: State<AppState>, project_id: String) -> Result<Vec<crate::models::Station>> {
    StationRepository::new(&state.db.lock().unwrap()).list_by_project(&project_id)
}

#[tauri::command]
pub fn create_station(state: State<AppState>, project_id: String, name: String) -> Result<crate::models::Station> {
    StationRepository::new(&state.db.lock().unwrap()).create(&project_id, &name)
}

#[tauri::command]
pub fn list_audio_files(state: State<AppState>, project_id: String) -> Result<Vec<AudioFile>> {
    state.db.lock().unwrap().list_audio_files(&project_id)
}

#[tauri::command]
pub fn delete_audio_file(state: State<AppState>, id: String) -> Result<()> {
    state.db.lock().unwrap().delete_audio_file(&id)
}

#[tauri::command]
pub async fn import_audio(state: State<'_, AppState>, project_id: String, path: String) -> Result<AudioFile> {
    let output_dir = {
        let db = state.db.lock().unwrap();
        let project = db.get_project(&project_id)?.ok_or_else(|| {
            Hoi4RadioError::ProjectNotFound(project_id.clone())
        })?;
        project.output_dir.clone()
    };

    let source_path = PathBuf::from(path);
    let meta = crate::audio::analyze_audio(&source_path).await?;
    let id = format!("audio_{}", Uuid::new_v4().to_string().replace('-', ""));
    let ogg_filename = format!("{}.ogg", id);
    let music_dir = output_dir.join("music");
    std::fs::create_dir_all(&music_dir)?;
    let ogg_path = music_dir.join(&ogg_filename);
    crate::audio::transcode_to_ogg(&source_path, &ogg_path).await?;

    let title = source_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| id.clone());

    let audio = AudioFile {
        id: id.clone(),
        title,
        artist: None,
        source_path,
        ogg_filename,
        duration_secs: meta.duration_secs,
        sample_rate: meta.sample_rate,
        channels: meta.channels,
        volume: 0.75,
        tags: vec![],
        notes: None,
    };

    AudioRepository::new(&state.db.lock().unwrap()).create(&project_id, &audio)?;
    Ok(audio)
}

#[tauri::command]
pub fn generate_project_mod(state: State<AppState>, project_id: String) -> Result<String> {
    let db = state.db.lock().unwrap();
    let project = db.get_project(&project_id)?.ok_or_else(|| {
        Hoi4RadioError::ProjectNotFound(project_id.clone())
    })?;
    let stations = StationRepository::new(&*db).list_by_project(&project_id)?;
    let audio_files = db.list_audio_files(&project_id)?;
    drop(db);
    generate_mod(&project, &stations, &audio_files, &project.output_dir)?;
    Ok(project.output_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn validate_mod(state: State<'_, AppState>, project_id: String) -> Result<crate::validator::ValidationReport> {
    let output_dir = {
        let db = state.db.lock().unwrap();
        let project = db.get_project(&project_id)?.ok_or_else(|| {
            Hoi4RadioError::ProjectNotFound(project_id.clone())
        })?;
        project.output_dir.clone()
    };
    validate_mod_output(&output_dir).await
}
```

- [ ] **Step 2: Update `src-tauri/src/lib.rs` to wire commands**

```rust
pub mod commands;

use commands::AppState;
use db::Db;
use std::path::PathBuf;
use std::sync::Mutex;

pub fn run() {
    let app_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("hoi4-radio-maker");
    std::fs::create_dir_all(&app_dir).ok();
    let db_path = app_dir.join("app.db");
    let db = Db::open(&db_path).expect("failed to open database");

    tauri::Builder::default()
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::delete_project,
            commands::list_stations,
            commands::create_station,
            commands::list_audio_files,
            commands::delete_audio_file,
            commands::import_audio,
            commands::generate_project_mod,
            commands::validate_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Add `dirs` dependency to `src-tauri/Cargo.toml`**

```toml
dirs = "5.0"
```

- [ ] **Step 4: Build to verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add Tauri command layer with project, station, audio, and validation"
```

---

## Task 8: Vue Frontend — Project Management

**Files:**
- Create: `src/stores/project.ts`, `src/views/WelcomeView.vue`, `src/views/ProjectView.vue`, `src/components/ProjectList.vue`
- Modify: `src/router.ts`, `src/main.ts`

- [ ] **Step 1: Create Pinia store**

`src/stores/project.ts`:

```typescript
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Project {
  id: string
  name: string
  version: string
  supported_version: string
  tags: string[]
  author?: string
  output_dir: string
}

export interface CreateProjectRequest {
  name: string
  version: string
  supported_version: string
  tags: string[]
  author?: string
  output_dir: string
}

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([])
  const currentProject = ref<Project | null>(null)

  async function loadProjects() {
    projects.value = await invoke<Project[]>('list_projects')
  }

  async function createProject(req: CreateProjectRequest) {
    const p = await invoke<Project>('create_project', { req })
    projects.value.unshift(p)
    return p
  }

  async function deleteProject(id: string) {
    await invoke('delete_project', { id })
    projects.value = projects.value.filter(p => p.id !== id)
  }

  function setCurrentProject(p: Project | null) {
    currentProject.value = p
  }

  return { projects, currentProject, loadProjects, createProject, deleteProject, setCurrentProject }
})
```

- [ ] **Step 2: Create WelcomeView**

`src/views/WelcomeView.vue`:

```vue
<template>
  <v-layout style="height: 100vh">
    <v-navigation-drawer permanent width="280">
      <ProjectList />
    </v-navigation-drawer>
    <v-main style="padding: 24px">
      <h1 class="text-h4 mb-4">HOI4 Radio Maker</h1>
      <p class="text-body-1">选择一个项目或创建新项目开始。</p>
    </v-main>
  </v-layout>
</template>

<script setup lang="ts">
import ProjectList from '../components/ProjectList.vue'
</script>
```

- [ ] **Step 3: Create ProjectList component**

`src/components/ProjectList.vue`:

```vue
<template>
  <div class="pa-4">
    <v-btn color="primary" block @click="showDialog = true">
      新建项目
    </v-btn>
    <v-divider class="my-4" />
    <v-list>
      <v-list-item
        v-for="p in projectStore.projects"
        :key="p.id"
        :title="p.name"
        :subtitle="p.version"
        @click="selectProject(p)"
      />
    </v-list>

    <v-dialog v-model="showDialog" max-width="500">
      <v-card>
        <v-card-title>新建项目</v-card-title>
        <v-card-text>
          <v-text-field v-model="form.name" label="名称" />
          <v-text-field v-model="form.version" label="版本" />
          <v-text-field v-model="form.supported_version" label="支持的游戏版本" />
          <v-text-field v-model="form.output_dir" label="输出目录" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn text @click="showDialog = false">取消</v-btn>
          <v-btn color="primary" @click="handleCreate">创建</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore, type Project } from '../stores/project'

const projectStore = useProjectStore()
const router = useRouter()
const showDialog = ref(false)
const form = reactive({
  name: 'My Radio Mod',
  version: '0.1.0',
  supported_version: '*',
  output_dir: '',
})

onMounted(() => {
  projectStore.loadProjects()
})

async function handleCreate() {
  const p = await projectStore.createProject({
    ...form,
    tags: ['Sound'],
    author: undefined,
  })
  if (p) {
    showDialog.value = false
    router.push({ name: 'project', params: { id: p.id } })
  }
}

function selectProject(p: Project) {
  projectStore.setCurrentProject(p)
  router.push({ name: 'project', params: { id: p.id } })
}
</script>
```

- [ ] **Step 4: Create ProjectView layout**

`src/views/ProjectView.vue`:

```vue
<template>
  <v-layout style="height: 100vh">
    <v-navigation-drawer permanent width="280">
      <ProjectList />
    </v-navigation-drawer>
    <v-main style="padding: 24px">
      <div class="d-flex justify-space-between align-center mb-4">
        <h2 class="text-h5">{{ projectStore.currentProject?.name }}</h2>
        <v-btn color="primary" @click="generate">生成 Mod</v-btn>
      </div>
      <v-tabs v-model="tab">
        <v-tab value="audio">音频库</v-tab>
        <v-tab value="stations">电台编辑</v-tab>
      </v-tabs>
      <v-window v-model="tab">
        <v-window-item value="audio">
          <AudioLibraryView />
        </v-window-item>
        <v-window-item value="stations">
          <StationEditorView />
        </v-window-item>
      </v-window>
    </v-main>
  </v-layout>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ProjectList from '../components/ProjectList.vue'
import AudioLibraryView from './AudioLibraryView.vue'
import StationEditorView from './StationEditorView.vue'
import { useProjectStore } from '../stores/project'
import { invoke } from '@tauri-apps/api/core'

const tab = ref('audio')
const projectStore = useProjectStore()

async function generate() {
  if (!projectStore.currentProject) return
  const out = await invoke<string>('generate_project_mod', { projectId: projectStore.currentProject.id })
  alert(`已生成到: ${out}`)
}
</script>
```

- [ ] **Step 5: Update router**

`src/router.ts`:

```typescript
import { createRouter, createWebHistory } from 'vue-router'
import WelcomeView from './views/WelcomeView.vue'
import ProjectView from './views/ProjectView.vue'
import SettingsView from './views/SettingsView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'welcome', component: WelcomeView },
    { path: '/project/:id', name: 'project', component: ProjectView, props: true },
    { path: '/settings', name: 'settings', component: SettingsView },
  ],
})

export default router
```

- [ ] **Step 6: Install Vuetify and update main.ts**

```bash
cd /home/ldsr/code/Rust/hoi4-radio-maker
bun add vuetify@^3.8.0
```

`src/plugins/vuetify.ts`:

```typescript
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import 'vuetify/styles'

const vuetify = createVuetify({
  components,
  directives,
  theme: {
    defaultTheme: 'dark',
    themes: {
      dark: {
        colors: {
          primary: '#D0BCFF',
          secondary: '#CCC2DC',
          surface: '#1C1B1F',
          background: '#121212',
          error: '#F2B8B5',
        },
      },
      light: {
        colors: {
          primary: '#6750A4',
          secondary: '#625B71',
          surface: '#FFFBFE',
          background: '#FFFFFF',
          error: '#B3261E',
        },
      },
    },
  },
})

export default vuetify
```

`src/main.ts`:

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import vuetify from './plugins/vuetify'
import App from './App.vue'
import router from './router'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(vuetify)
app.mount('#app')
```

- [ ] **Step 7: Create placeholder AudioLibraryView and StationEditorView**

`src/views/AudioLibraryView.vue`:

```vue
<template>
  <div>
    <h3>音频库</h3>
    <p>拖拽或导入音频文件。</p>
  </div>
</template>
```

`src/views/StationEditorView.vue`:

```vue
<template>
  <div>
    <h3>电台编辑</h3>
    <p>从音频库添加歌曲到电台。</p>
  </div>
</template>
```

- [ ] **Step 8: Run dev and manually test**

```bash
bun run tauri dev
```

Expected: Welcome screen shows with "新建项目" button; clicking creates a project and it appears in the list.

- [ ] **Step 9: Commit**

```bash
git add src/
git commit -m "feat: add Vue project management UI"
```


---

## Task 9: Vue Frontend — Audio Library and Station Editor

**Files:**
- Create: `src/components/AudioImporter.vue`, `src/components/StationList.vue`, `src/components/SongEntryEditor.vue`
- Modify: `src/views/AudioLibraryView.vue`, `src/views/StationEditorView.vue`

- [ ] **Step 1: Confirm audio commands are wired**

`import_audio`, `list_audio_files`, and `delete_audio_file` were added to `src-tauri/src/commands.rs` and wired into `src-tauri/src/lib.rs` in Task 7. No further backend changes are required here; the frontend below invokes them directly.

- [ ] **Step 2: Create AudioImporter component**

`src/components/AudioImporter.vue`:

```vue
<template>
  <v-btn prepend-icon="mdi-music-note-plus" @click="selectFiles">
    导入音频
  </v-btn>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

const emit = defineEmits<{
  (e: 'import', paths: string[]): void
}>()

async function selectFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg'] }],
  })
  if (selected && Array.isArray(selected)) {
    emit('import', selected)
  }
}
</script>
```

- [ ] **Step 3: Update AudioLibraryView**

`src/views/AudioLibraryView.vue`:

```vue
<template>
  <div class="pa-4">
    <div class="d-flex align-center mb-4">
      <h3 class="text-h6 mr-4">音频库</h3>
      <AudioImporter @import="onImport" />
    </div>
    <v-list>
      <v-list-item
        v-for="audio in audioFiles"
        :key="audio.id"
        :title="audio.title"
        :subtitle="`${audio.artist || '未知艺术家'} · ${audio.duration_secs}s · ${audio.sample_rate}Hz · ${audio.channels}ch`"
      >
        <template #append>
          <v-btn icon="mdi-delete" variant="text" color="error" @click="deleteAudio(audio.id)" />
        </template>
      </v-list-item>
    </v-list>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AudioImporter from '../components/AudioImporter.vue'
import { useProjectStore } from '../stores/project'

interface AudioFile {
  id: string
  title: string
  artist?: string
  duration_secs: number
  sample_rate: number
  channels: number
}

const projectStore = useProjectStore()
const audioFiles = ref<AudioFile[]>([])

onMounted(async () => {
  await loadAudio()
})

async function loadAudio() {
  if (!projectStore.currentProject) return
  audioFiles.value = await invoke<AudioFile[]>('list_audio_files', { projectId: projectStore.currentProject.id })
}

async function onImport(paths: string[]) {
  if (!projectStore.currentProject) return
  for (const path of paths) {
    await invoke('import_audio', { projectId: projectStore.currentProject.id, path })
  }
  await loadAudio()
}

async function deleteAudio(id: string) {
  await invoke('delete_audio_file', { id })
  await loadAudio()
}
</script>
```

- [ ] **Step 4: Update StationEditorView**

`src/views/StationEditorView.vue`:

```vue
<template>
  <div class="pa-4">
    <div class="d-flex align-center mb-4">
      <h3 class="text-h6 mr-4">电台编辑</h3>
      <v-btn prepend-icon="mdi-plus" @click="createStation">新建电台</v-btn>
    </div>
    <v-tabs v-model="activeTab">
      <v-tab v-for="station in stations" :key="station.id" :value="station.id">
        {{ station.name }}
      </v-tab>
    </v-tabs>
    <v-window v-model="activeTab">
      <v-window-item v-for="station in stations" :key="station.id" :value="station.id">
        <v-list>
          <v-list-item
            v-for="entry in station.entries"
            :key="entry.audio_file_id"
            :title="entry.audio_file_id"
            :subtitle="`factor: ${entry.chance.factor}`"
          />
        </v-list>
      </v-window-item>
    </v-window>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useProjectStore } from '../stores/project'

interface Station {
  id: string
  name: string
  entries: Array<{ audio_file_id: string; chance: { factor: number; modifiers: any[] } }>
}

const projectStore = useProjectStore()
const stations = ref<Station[]>([])
const activeTab = ref<string>('')

onMounted(async () => {
  if (projectStore.currentProject) {
    stations.value = await invoke<Station[]>('list_stations', { projectId: projectStore.currentProject.id })
    if (stations.value.length > 0) {
      activeTab.value = stations.value[0].id
    }
  }
})

async function createStation() {
  if (!projectStore.currentProject) return
  const name = prompt('电台名称') || 'New Station'
  const station = await invoke<Station>('create_station', { projectId: projectStore.currentProject.id, name })
  stations.value.push(station)
}
</script>
```

- [ ] **Step 5: Add dialog plugin**

```bash
bun add @tauri-apps/plugin-dialog
```

Update `src-tauri/tauri.conf.json` capabilities:

```json
{
  "permissions": [
    "core:default",
    "dialog:default"
  ]
}
```

- [ ] **Step 6: Commit**

```bash
git add src/ src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/tauri.conf.json
git commit -m "feat: add audio library and station editor UI skeleton"
```


---

## Task 9.5: Settings Backend and UI

**Files:**
- Create: `src-tauri/src/settings.rs`, `src/views/SettingsView.vue`
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Confirm settings table exists**

The `settings` table was created in Task 3 (`db.rs` migration). No schema change is needed.

- [ ] **Step 2: Implement settings backend**

`src-tauri/src/settings.rs`:

```rust
use crate::db::Db;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub hoi4_game_dir: Option<String>,
    pub theme: String,
}

impl Settings {
    pub const KEY: &'static str = "app_settings";

    pub fn get(db: &Db) -> Result<Self> {
        let mut stmt = db.conn().prepare("SELECT value FROM settings WHERE key = ?1")?;
        let row = stmt
            .query_row([Self::KEY], |row| {
                let value: String = row.get(0)?;
                Ok(serde_json::from_str(&value).unwrap_or_default())
            })
            .optional()?;
        Ok(row.unwrap_or_default())
    }

    pub fn save(&self, db: &Db) -> Result<()> {
        let value = serde_json::to_string(self)?;
        db.conn().execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (Self::KEY, value),
        )?;
        Ok(())
    }
}
```

- [ ] **Step 3: Add settings commands**

Extend `src-tauri/src/commands.rs`:

```rust
use crate::settings::Settings;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings> {
    Settings::get(&state.db.lock().unwrap())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: Settings) -> Result<()> {
    settings.save(&state.db.lock().unwrap())
}
```

- [ ] **Step 4: Wire settings commands**

Add to the `tauri::generate_handler!` list in `src-tauri/src/lib.rs`:

```rust
commands::get_settings,
commands::save_settings,
```

- [ ] **Step 5: Create SettingsView.vue**

`src/views/SettingsView.vue`:

```vue
<template>
  <v-container>
    <h2 class="text-h5 mb-4">设置</h2>
    <v-text-field v-model="settings.ffmpeg_path" label="ffmpeg 路径" placeholder="ffmpeg" />
    <v-text-field v-model="settings.ffprobe_path" label="ffprobe 路径" placeholder="ffprobe" />
    <v-text-field v-model="settings.hoi4_game_dir" label="HOI4 游戏目录" placeholder="..." />
    <v-select
      v-model="settings.theme"
      label="主题"
      :items="themeOptions"
      item-title="label"
      item-value="value"
    />
    <v-btn color="primary" @click="save">保存</v-btn>
  </v-container>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Settings {
  ffmpeg_path: string | null
  ffprobe_path: string | null
  hoi4_game_dir: string | null
  theme: string
}

const settings = reactive<Settings>({
  ffmpeg_path: null,
  ffprobe_path: null,
  hoi4_game_dir: null,
  theme: 'light',
})

const themeOptions = [
  { label: '浅色', value: 'light' },
  { label: '深色', value: 'dark' },
]

onMounted(async () => {
  const saved = await invoke<Settings>('get_settings')
  Object.assign(settings, saved)
})

async function save() {
  await invoke('save_settings', { settings })
}
</script>
```

- [ ] **Step 6: Build to verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/settings.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src/views/SettingsView.vue src/router.ts
git commit -m "feat: add settings backend and UI"
```


---

## Task 10: End-to-End Integration — Generate First Mod

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src/views/ProjectView.vue`

- [ ] **Step 1: Ensure generate command loads audio files from DB**

`generate_project_mod` was already updated in Task 7 to load audio files via `list_audio_files`. The final implementation should look like this:

```rust
#[tauri::command]
pub fn generate_project_mod(state: State<AppState>, project_id: String) -> Result<String> {
    let db = state.db.lock().unwrap();
    let project = db.get_project(&project_id)?.ok_or_else(|| {
        Hoi4RadioError::ProjectNotFound(project_id.clone())
    })?;
    let stations = StationRepository::new(&*db).list_by_project(&project_id)?;
    let audio_files = db.list_audio_files(&project_id)?;
    drop(db);
    generate_mod(&project, &stations, &audio_files, &project.output_dir)?;
    Ok(project.output_dir.to_string_lossy().to_string())
}
```

- [ ] **Step 2: Run dev and test**

1. Create a project
2. Import at least one audio file on the audio library tab
3. Create a station and add the audio file to it
4. Click "生成 Mod"
5. Check output directory contains `descriptor.mod`, `.mod` file, `music/` with the transcoded `.ogg`, and `localisation/`
6. Click "验证 Mod" (or invoke `validate_mod`) to confirm the validator reports the output as valid

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: wire end-to-end mod generation"
```

---

## Task 11: Testing and Quality

**Files:**
- Create: `src-tauri/tests/integration_test.rs`, `.github/workflows/ci.yml`

- [ ] **Step 1: Add integration test for full flow**

`src-tauri/tests/integration_test.rs`:

```rust
use hoi4_radio_maker_lib::db::Db;
use hoi4_radio_maker_lib::models::CreateProjectRequest;
use hoi4_radio_maker_lib::station::StationRepository;
use tempfile::TempDir;

#[test]
fn test_create_project_and_station() {
    let tmp = TempDir::new().unwrap();
    let db = Db::open(tmp.path().join("app.db")).unwrap();
    let project = db.create_project(&CreateProjectRequest {
        name: "Integration Test".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec!["Sound".into()],
        author: None,
        output_dir: tmp.path().join("out"),
    }).unwrap();

    let repo = StationRepository::new(&db);
    let station = repo.create(&project.id, "War Radio").unwrap();
    assert_eq!(station.name, "War Radio");

    let stations = repo.list_by_project(&project.id).unwrap();
    assert_eq!(stations.len(), 1);
}
```

- [ ] **Step 2: Run all Rust tests**

```bash
cd src-tauri && cargo test
```

Expected: all PASS.

- [ ] **Step 3: Add GitHub Actions CI**

`.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: bun/action-setup@v3
        with:
          version: 9
      - run: bun install
      - run: cd src-tauri && cargo test
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tests/integration_test.rs .github/workflows/ci.yml
git commit -m "test: add integration test and CI workflow"
```

---

## Self-Review

**1. Spec coverage:**
- 项目管理 ✅ Task 3, 7, 8
- 音频库 ✅ Task 4, 9
- 电台编辑 ✅ Task 5, 9
- Mod 生成 ✅ Task 6, 10
- 验证 ✅ Task 6.5 (`validator.rs`, `validate_mod` command, `validator_test.rs`)
- 设置 ✅ Task 9.5 (`settings.rs`, `SettingsView.vue`, `get_settings`/`save_settings` commands)

**2. Placeholder scan:**
- 计划层面没有遗留的待办或占位标记
- Task 7 和 Task 10 中的音频文件加载已改为从数据库读取（`list_audio_files`），不再使用空数组占位符

**3. Type consistency：**
- `Project` / `AudioFile` / `Station` / `StationEntry` / `ChanceConfig` / `Trigger` 在 models.rs、db.rs、station.rs、generator.rs 中一致
- Tauri command 参数命名在前后端需对齐：`projectId` 与 `project_id`

---

## Execution Handoff

Plan complete and saved to `/home/ldsr/code/Rust/hoi4-radio-maker/docs/superpowers/plans/2026-06-12-hoi4radio-implementation-plan.md`.

**Two execution options:**

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints for review.

**Which approach?**
