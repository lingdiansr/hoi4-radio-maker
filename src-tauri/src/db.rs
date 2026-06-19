use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{Hoi4RadioError, Result};
use crate::models::{AudioFile, CreateProjectRequest, Project, UpdateProjectRequest};

/// Result type for a batch import operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchImportResult {
    pub created: Vec<AudioFile>,
    pub existing: Vec<AudioFile>,
}

/// Wraps a SQLite connection and provides typed access to application data.
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open a database at `path`, creating the file and schema if necessary.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Access the underlying SQLite connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Create tables and enable foreign-key support.
    fn migrate(&self) -> Result<()> {
        let user_version: i32 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;

        if user_version == 0 && self.is_old_audio_schema()? {
            tracing::warn!(
                "detected old audio_files schema; dropping audio tables and recreating with new schema"
            );
            self.conn.execute_batch(
                "
                PRAGMA foreign_keys = OFF;
                DROP TABLE IF EXISTS station_entries;
                DROP TABLE IF EXISTS stations;
                DROP TABLE IF EXISTS project_audio_files;
                DROP TABLE IF EXISTS audio_files;
                PRAGMA foreign_keys = ON;
                ",
            )?;
        }

        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
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
                source_hash TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                artist TEXT,
                source_path TEXT NOT NULL,
                ogg_filename TEXT NOT NULL UNIQUE,
                duration_secs REAL,
                sample_rate INTEGER,
                channels INTEGER,
                volume REAL NOT NULL DEFAULT 0.75,
                tags TEXT NOT NULL DEFAULT '[]',
                notes TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_audio_files (
                project_id TEXT NOT NULL,
                audio_file_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (project_id, audio_file_id),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (audio_file_id) REFERENCES audio_files(id) ON DELETE CASCADE
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

            PRAGMA foreign_keys = ON;
            PRAGMA user_version = 1;
            ",
        )?;
        Ok(())
    }

    /// Detects the pre-global-audio-library schema where `audio_files`
    /// contained an embedded `project_id` and no `source_hash` column.
    fn is_old_audio_schema(&self) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'table' AND name = 'audio_files'",
            [],
            |row| row.get(0),
        )?;
        if count == 0 {
            return Ok(false);
        }
        let has_source_hash: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('audio_files') WHERE name = 'source_hash'",
            [],
            |row| row.get(0),
        )?;
        Ok(has_source_hash == 0)
    }

    /// Insert a new project and return the created record.
    pub fn create_project(&self, req: &CreateProjectRequest) -> Result<Project> {
        let id = format!("proj_{}", Uuid::new_v4().as_simple());
        let now = Utc::now();

        self.conn.execute(
            "INSERT INTO projects (
                id, name, version, supported_version, tags, author,
                output_dir, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &id,
                &req.name,
                &req.version,
                &req.supported_version,
                serde_json::to_string(&req.tags)?,
                req.author.as_deref(),
                req.output_dir.to_string_lossy(),
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        Ok(Project {
            id,
            name: req.name.clone(),
            version: req.version.clone(),
            supported_version: req.supported_version.clone(),
            tags: req.tags.clone(),
            author: req.author.clone(),
            output_dir: req.output_dir.clone(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Fetch a single project by ID, if it exists.
    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, supported_version, tags, author,
                    output_dir, created_at, updated_at
             FROM projects
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(project_from_row(row)?)),
            None => Ok(None),
        }
    }

    /// Update an existing project and return the updated record.
    pub fn update_project(&self, id: &str, req: &UpdateProjectRequest) -> Result<Project> {
        let now = Utc::now();

        self.conn.execute(
            "UPDATE projects
             SET name = ?1,
                 version = ?2,
                 supported_version = ?3,
                 tags = ?4,
                 author = ?5,
                 output_dir = ?6,
                 updated_at = ?7
             WHERE id = ?8",
            params![
                &req.name,
                &req.version,
                &req.supported_version,
                serde_json::to_string(&req.tags)?,
                req.author.as_deref(),
                req.output_dir.to_string_lossy(),
                now.to_rfc3339(),
                id,
            ],
        )?;

        let rows_affected = self.conn.changes();
        if rows_affected == 0 {
            return Err(Hoi4RadioError::ProjectNotFound { id: id.to_string() });
        }

        self.get_project(id)?.ok_or_else(|| Hoi4RadioError::ProjectNotFound {
            id: id.to_string(),
        })
    }

    /// List all projects ordered by creation time, newest first.
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, version, supported_version, tags, author,
                    output_dir, created_at, updated_at
             FROM projects
             ORDER BY created_at DESC",
        )?;

        let mut rows = stmt.query([])?;
        let mut projects = Vec::new();
        while let Some(row) = rows.next()? {
            projects.push(project_from_row(row)?);
        }
        Ok(projects)
    }

    /// Delete a project and all of its dependent data.
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Insert a new audio file into the global library.
    pub fn create_audio_file(&self, audio: &AudioFile) -> Result<AudioFile> {
        self.conn.execute(
            "INSERT INTO audio_files (
                id, source_hash, title, artist, source_path, ogg_filename,
                duration_secs, sample_rate, channels, volume, tags, notes,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                &audio.id,
                &audio.source_hash,
                &audio.title,
                audio.artist.as_deref(),
                audio.source_path.to_string_lossy(),
                &audio.ogg_filename,
                audio.duration_secs,
                audio.sample_rate,
                audio.channels,
                audio.volume,
                serde_json::to_string(&audio.tags)?,
                audio.notes.as_deref(),
                audio.created_at.to_rfc3339(),
                audio.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(audio.clone())
    }

    /// Add an existing audio file to a project's reference list.
    pub fn add_audio_to_project(&self, project_id: &str, audio_file_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO project_audio_files (
                project_id, audio_file_id, added_at
            ) VALUES (?1, ?2, ?3)",
            params![project_id, audio_file_id, now],
        )?;
        Ok(())
    }

    /// Remove an audio file reference from a project.
    pub fn remove_audio_from_project(&self, project_id: &str, audio_file_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM project_audio_files
             WHERE project_id = ?1 AND audio_file_id = ?2",
            params![project_id, audio_file_id],
        )?;
        Ok(())
    }

    /// List all audio files in a project.
    pub fn list_audio_files(&self, project_id: &str) -> Result<Vec<AudioFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.source_hash, a.title, a.artist, a.source_path,
                    a.ogg_filename, a.duration_secs, a.sample_rate,
                    a.channels, a.volume, a.tags, a.notes,
                    a.created_at, a.updated_at
             FROM audio_files a
             JOIN project_audio_files pa ON a.id = pa.audio_file_id
             WHERE pa.project_id = ?1
             ORDER BY a.title",
        )?;

        let mut rows = stmt.query(params![project_id])?;
        let mut files = Vec::new();
        while let Some(row) = rows.next()? {
            files.push(audio_file_from_row(row)?);
        }
        Ok(files)
    }

    /// List all audio files in the global library.
    pub fn list_all_audio_files(&self) -> Result<Vec<AudioFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_hash, title, artist, source_path, ogg_filename,
                    duration_secs, sample_rate, channels, volume, tags, notes,
                    created_at, updated_at
             FROM audio_files
             ORDER BY title",
        )?;

        let mut rows = stmt.query([])?;
        let mut files = Vec::new();
        while let Some(row) = rows.next()? {
            files.push(audio_file_from_row(row)?);
        }
        Ok(files)
    }

    /// Fetch a single audio file by ID, if it exists.
    pub fn get_audio_file(&self, id: &str) -> Result<Option<AudioFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_hash, title, artist, source_path, ogg_filename,
                    duration_secs, sample_rate, channels, volume, tags, notes,
                    created_at, updated_at
             FROM audio_files
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(audio_file_from_row(row)?)),
            None => Ok(None),
        }
    }

    /// Fetch an audio file by source hash, if it exists.
    pub fn get_audio_file_by_hash(&self, hash: &str) -> Result<Option<AudioFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_hash, title, artist, source_path, ogg_filename,
                    duration_secs, sample_rate, channels, volume, tags, notes,
                    created_at, updated_at
             FROM audio_files
             WHERE source_hash = ?1",
        )?;

        let mut rows = stmt.query(params![hash])?;
        match rows.next()? {
            Some(row) => Ok(Some(audio_file_from_row(row)?)),
            None => Ok(None),
        }
    }

    /// Delete an audio file from the global library if it is not referenced.
    pub fn delete_audio_file(&self, id: &str) -> Result<()> {
        let references: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM project_audio_files WHERE audio_file_id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        if references > 0 {
            return Err(Hoi4RadioError::Other {
                message: "audio file is still referenced by one or more projects".to_string(),
            });
        }

        self.conn
            .execute("DELETE FROM audio_files WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn audio_file_from_row(row: &rusqlite::Row) -> Result<AudioFile> {
    let tags_json: String = row.get("tags")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let parse_dt = |s: &str| -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| Hoi4RadioError::Other {
                message: format!("invalid RFC 3339 timestamp: {e}"),
            })
    };

    Ok(AudioFile {
        id: row.get("id")?,
        source_hash: row.get("source_hash")?,
        title: row.get("title")?,
        artist: row.get("artist")?,
        source_path: PathBuf::from(row.get::<_, String>("source_path")?),
        ogg_filename: row.get("ogg_filename")?,
        duration_secs: row.get("duration_secs")?,
        sample_rate: row.get("sample_rate")?,
        channels: row.get("channels")?,
        volume: row.get("volume")?,
        tags: serde_json::from_str(&tags_json)?,
        notes: row.get("notes")?,
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}

fn project_from_row(row: &rusqlite::Row) -> Result<Project> {
    let tags_json: String = row.get("tags")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let parse_dt = |s: &str| -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| Hoi4RadioError::Other {
                message: format!("invalid RFC 3339 timestamp: {e}"),
            })
    };

    Ok(Project {
        id: row.get("id")?,
        name: row.get("name")?,
        version: row.get("version")?,
        supported_version: row.get("supported_version")?,
        tags: serde_json::from_str(&tags_json)?,
        author: row.get("author")?,
        output_dir: PathBuf::from(row.get::<_, String>("output_dir")?),
        created_at: parse_dt(&created_at_str)?,
        updated_at: parse_dt(&updated_at_str)?,
    })
}
