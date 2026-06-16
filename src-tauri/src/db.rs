use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{Hoi4RadioError, Result};
use crate::models::{CreateProjectRequest, Project};

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

            PRAGMA foreign_keys = ON;
            ",
        )?;
        Ok(())
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
