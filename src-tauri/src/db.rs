use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{Hoi4RadioError, Result};
use crate::models::{AudioFile, BatchImportFailedFile, BatchUpdateAudioFileRequest, CreateProjectRequest, ImportStatus, Project, UpdateAudioFileRequest, UpdateProjectRequest};

/// Result type for a batch import operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchImportResult {
    pub created: Vec<AudioFile>,
    pub existing: Vec<AudioFile>,
    pub failed: Vec<BatchImportFailedFile>,
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
                import_status TEXT NOT NULL DEFAULT 'ready',
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
            ",
        )?;

        // Migrate from version 1 to 2: add import_status column if missing.
        if user_version < 2 {
            let has_status: i32 = self.conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('audio_files') WHERE name = 'import_status'",
                [],
                |row| row.get(0),
            )?;
            if has_status == 0 {
                self.conn.execute(
                    "ALTER TABLE audio_files ADD COLUMN import_status TEXT NOT NULL DEFAULT 'ready'",
                    [],
                )?;
            }
            self.conn.execute("PRAGMA user_version = 2", [])?;
        }

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
                import_status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
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
                audio.import_status.as_str(),
                audio.created_at.to_rfc3339(),
                audio.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(audio.clone())
    }

    /// Update the import status of an audio file.
    pub fn update_audio_file_status(&self, id: &str, status: ImportStatus) -> Result<AudioFile> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE audio_files SET import_status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now, id],
        )?;
        self.get_audio_file(id)?.ok_or_else(|| Hoi4RadioError::Other {
            message: format!("audio file not found: {id}"),
        })
    }

    /// Promote a pending record to processing with real hash and metadata.
    pub fn start_audio_file_processing(
        &self,
        id: &str,
        source_hash: &str,
        metadata: &crate::models::AudioMetadata,
    ) -> Result<AudioFile> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE audio_files
             SET source_hash = ?1,
                 duration_secs = ?2,
                 sample_rate = ?3,
                 channels = ?4,
                 import_status = ?5,
                 updated_at = ?6
             WHERE id = ?7",
            params![
                source_hash,
                metadata.duration_secs,
                metadata.sample_rate,
                metadata.channels,
                ImportStatus::Processing.as_str(),
                now,
                id,
            ],
        )?;
        self.get_audio_file(id)?.ok_or_else(|| Hoi4RadioError::Other {
            message: format!("audio file not found: {id}"),
        })
    }

    /// Remove all project references to an audio file.
    pub fn remove_audio_from_all_projects(&self, audio_file_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM project_audio_files WHERE audio_file_id = ?1",
            params![audio_file_id],
        )?;
        Ok(())
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
                    a.channels, a.volume, a.tags, a.notes, a.import_status,
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
                    import_status, created_at, updated_at
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
                    import_status, created_at, updated_at
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
                    import_status, created_at, updated_at
             FROM audio_files
             WHERE source_hash = ?1",
        )?;

        let mut rows = stmt.query(params![hash])?;
        match rows.next()? {
            Some(row) => Ok(Some(audio_file_from_row(row)?)),
            None => Ok(None),
        }
    }

    /// Delete an audio file from the global library.
    ///
    /// By default, refuses if the file is referenced by any project. Pass
    /// `force = true` to remove references and delete anyway.
    pub fn delete_audio_file(&self, id: &str, force: bool) -> Result<()> {
        if force {
            self.remove_audio_from_all_projects(id)?;
        } else {
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
        }

        self.conn
            .execute("DELETE FROM audio_files WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Update a single audio file.
    pub fn update_audio_file(&self, id: &str, req: &UpdateAudioFileRequest) -> Result<AudioFile> {
        let mut sets = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(title) = &req.title {
            sets.push("title = ?".to_string());
            params.push(rusqlite::types::Value::Text(title.clone()));
        }
        if let Some(artist) = &req.artist {
            sets.push("artist = ?".to_string());
            params.push(match artist {
                Some(a) => rusqlite::types::Value::Text(a.clone()),
                None => rusqlite::types::Value::Null,
            });
        }
        if let Some(volume) = req.volume {
            sets.push("volume = ?".to_string());
            params.push(rusqlite::types::Value::Real(volume));
        }
        if let Some(tags) = &req.tags {
            sets.push("tags = ?".to_string());
            params.push(rusqlite::types::Value::Text(serde_json::to_string(tags)?));
        }
        if let Some(notes) = &req.notes {
            sets.push("notes = ?".to_string());
            params.push(match notes {
                Some(n) => rusqlite::types::Value::Text(n.clone()),
                None => rusqlite::types::Value::Null,
            });
        }

        if sets.is_empty() {
            return self
                .get_audio_file(id)?
                .ok_or_else(|| Hoi4RadioError::Other {
                    message: format!("audio file not found: {id}"),
                });
        }

        let now = Utc::now().to_rfc3339();
        sets.push("updated_at = ?".to_string());
        params.push(rusqlite::types::Value::Text(now));

        let sql = format!("UPDATE audio_files SET {} WHERE id = ?", sets.join(", "));
        params.push(rusqlite::types::Value::Text(id.to_string()));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        self.conn.execute(&sql, param_refs.as_slice())?;

        self.get_audio_file(id)?.ok_or_else(|| Hoi4RadioError::Other {
            message: format!("audio file not found: {id}"),
        })
    }

    /// Batch update multiple audio files.
    pub fn batch_update_audio_files(
        &self,
        ids: &[String],
        req: &BatchUpdateAudioFileRequest,
    ) -> Result<Vec<AudioFile>> {
        let mut sets = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(artist) = &req.artist {
            sets.push("artist = ?".to_string());
            params.push(match artist {
                Some(a) => rusqlite::types::Value::Text(a.clone()),
                None => rusqlite::types::Value::Null,
            });
        }
        if let Some(volume) = req.volume {
            sets.push("volume = ?".to_string());
            params.push(rusqlite::types::Value::Real(volume));
        }
        if let Some(tags) = &req.tags {
            sets.push("tags = ?".to_string());
            params.push(rusqlite::types::Value::Text(serde_json::to_string(tags)?));
        }

        if sets.is_empty() || ids.is_empty() {
            return Ok(Vec::new());
        }

        let now = Utc::now().to_rfc3339();
        sets.push("updated_at = ?".to_string());
        params.push(rusqlite::types::Value::Text(now));

        let id_placeholders: Vec<String> = (0..ids.len()).map(|_| "?".to_string()).collect();
        let sql = format!(
            "UPDATE audio_files SET {} WHERE id IN ({})",
            sets.join(", "),
            id_placeholders.join(", ")
        );

        for id in ids {
            params.push(rusqlite::types::Value::Text(id.clone()));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        self.conn.execute(&sql, param_refs.as_slice())?;

        let mut result = Vec::new();
        for id in ids {
            if let Some(audio) = self.get_audio_file(id)? {
                result.push(audio);
            }
        }
        Ok(result)
    }
}

fn audio_file_from_row(row: &rusqlite::Row) -> Result<AudioFile> {
    let tags_json: String = row.get("tags")?;
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;
    let import_status_str: String = row.get("import_status")?;

    let parse_dt = |s: &str| -> Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| Hoi4RadioError::Other {
                message: format!("invalid RFC 3339 timestamp: {e}"),
            })
    };

    let import_status = import_status_str.parse().map_err(|e: String| {
        Hoi4RadioError::Other {
            message: format!("invalid import_status in database: {e}"),
        }
    })?;

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
        import_status,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BatchUpdateAudioFileRequest, UpdateAudioFileRequest};
    use chrono::Utc;

    fn dummy_audio(id: &str, title: &str) -> AudioFile {
        use crate::models::ImportStatus;
        AudioFile {
            id: id.to_string(),
            source_hash: format!("hash_{id}"),
            title: title.to_string(),
            artist: None,
            source_path: PathBuf::from(format!("/tmp/{id}.mp3")),
            ogg_filename: format!("{id}.ogg"),
            duration_secs: 120.0,
            sample_rate: 44100,
            channels: 2,
            volume: 0.75,
            tags: vec!["Sound".to_string()],
            notes: None,
            import_status: ImportStatus::Ready,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn update_audio_file_changes_fields() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        let audio = dummy_audio("audio_1", "Old Title");
        db.create_audio_file(&audio).unwrap();

        let updated = db
            .update_audio_file(
                &audio.id,
                &UpdateAudioFileRequest {
                    title: Some("New Title".to_string()),
                    artist: Some(Some("Artist".to_string())),
                    volume: Some(0.5),
                    tags: Some(vec!["Radio".to_string()]),
                    notes: Some(Some("note".to_string())),
                },
            )
            .unwrap();

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.artist, Some("Artist".to_string()));
        assert_eq!(updated.volume, 0.5);
        assert_eq!(updated.tags, vec!["Radio".to_string()]);
        assert_eq!(updated.notes, Some("note".to_string()));
    }

    #[test]
    fn batch_update_audio_files_changes_common_fields() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        let a1 = dummy_audio("audio_a", "A");
        let a2 = dummy_audio("audio_b", "B");
        db.create_audio_file(&a1).unwrap();
        db.create_audio_file(&a2).unwrap();

        let updated = db
            .batch_update_audio_files(
                &[a1.id.clone(), a2.id.clone()],
                &BatchUpdateAudioFileRequest {
                    artist: Some(Some("Shared Artist".to_string())),
                    volume: Some(0.9),
                    tags: Some(vec!["Batch".to_string()]),
                },
            )
            .unwrap();

        assert_eq!(updated.len(), 2);
        for audio in updated {
            assert_eq!(audio.artist, Some("Shared Artist".to_string()));
            assert_eq!(audio.volume, 0.9);
            assert_eq!(audio.tags, vec!["Batch".to_string()]);
        }
    }

    #[test]
    fn batch_update_returns_empty_when_no_ids() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        let result = db
            .batch_update_audio_files(
                &[],
                &BatchUpdateAudioFileRequest {
                    artist: Some(Some("Artist".to_string())),
                    volume: None,
                    tags: None,
                },
            )
            .unwrap();
        assert!(result.is_empty());
    }
}
