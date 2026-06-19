use crate::db::Db;
use crate::error::Result;
use crate::models::{ChanceConfig, Station, StationEntry};
use rusqlite::params;
use uuid::Uuid;

/// Repository for managing radio stations and their song entries.
pub struct StationRepository<'a> {
    db: &'a Db,
}

impl<'a> StationRepository<'a> {
    /// Create a new repository wrapping the given database.
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Check whether a station with the given name already exists in the project.
    pub fn find_by_name(&self, project_id: &str, name: &str) -> Result<Option<Station>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name FROM stations WHERE project_id = ?1 AND name = ?2",
        )?;
        let mut rows = stmt.query(params![project_id, name])?;
        match rows.next()? {
            Some(row) => {
                let id: String = row.get("id")?;
                let name: String = row.get("name")?;
                let entries = self.list_entries(&id)?;
                Ok(Some(Station { id, name, entries }))
            }
            None => Ok(None),
        }
    }

    /// Create a new station within a project.
    pub fn create(&self, project_id: &str, name: &str) -> Result<Station> {
        let id = format!("station_{}", Uuid::new_v4().to_string().replace('-', ""));

        self.db.conn().execute(
            "INSERT INTO stations (id, project_id, name) VALUES (?1, ?2, ?3)",
            params![&id, project_id, name],
        )?;

        Ok(Station {
            id,
            name: name.to_string(),
            entries: vec![],
        })
    }

    /// Fetch a single station by ID, including its entries.
    pub fn get(&self, id: &str) -> Result<Option<Station>> {
        let mut stmt = self
            .db
            .conn()
            .prepare("SELECT id, name FROM stations WHERE id = ?1")?;

        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => {
                let id: String = row.get("id")?;
                let name: String = row.get("name")?;
                let entries = self.list_entries(&id)?;
                Ok(Some(Station { id, name, entries }))
            }
            None => Ok(None),
        }
    }

    /// List all stations belonging to a project, ordered by sort_order then id.
    pub fn list_by_project(&self, project_id: &str) -> Result<Vec<Station>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT id, name FROM stations WHERE project_id = ?1 ORDER BY sort_order, id",
        )?;

        let mut rows = stmt.query(params![project_id])?;
        let mut stations = Vec::new();
        while let Some(row) = rows.next()? {
            let id: String = row.get("id")?;
            let name: String = row.get("name")?;
            let entries = self.list_entries(&id)?;
            stations.push(Station { id, name, entries });
        }
        Ok(stations)
    }

    /// Add a song entry to a station.
    pub fn add_entry(
        &self,
        station_id: &str,
        audio_file_id: &str,
        chance: ChanceConfig,
    ) -> Result<()> {
        let chance_json = serde_json::to_string(&chance)?;

        self.db.conn().execute(
            "INSERT INTO station_entries (station_id, audio_file_id, chance_config)
             VALUES (?1, ?2, ?3)",
            params![station_id, audio_file_id, chance_json],
        )?;

        Ok(())
    }

    /// Remove a song entry from a station.
    pub fn remove_entry(&self, station_id: &str, audio_file_id: &str) -> Result<()> {
        self.db.conn().execute(
            "DELETE FROM station_entries WHERE station_id = ?1 AND audio_file_id = ?2",
            params![station_id, audio_file_id],
        )?;
        Ok(())
    }

    /// Delete a station and its entries.
    pub fn delete(&self, id: &str) -> Result<()> {
        self.db
            .conn()
            .execute("DELETE FROM stations WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Rename an existing station.
    pub fn rename(&self, id: &str, name: &str) -> Result<()> {
        self.db
            .conn()
            .execute(
                "UPDATE stations SET name = ?1 WHERE id = ?2",
                params![name, id],
            )?;
        Ok(())
    }

    fn list_entries(&self, station_id: &str) -> Result<Vec<StationEntry>> {
        let mut stmt = self.db.conn().prepare(
            "SELECT audio_file_id, chance_config FROM station_entries WHERE station_id = ?1",
        )?;

        let mut rows = stmt.query(params![station_id])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let audio_file_id: String = row.get("audio_file_id")?;
            let chance_json: String = row.get("chance_config")?;
            let chance = serde_json::from_str(&chance_json).unwrap_or_else(|err| {
                tracing::warn!(
                    "failed to parse chance_config for station {} audio {}: {}",
                    station_id,
                    audio_file_id,
                    err
                );
                ChanceConfig {
                    factor: 1.0,
                    modifiers: vec![],
                }
            });
            entries.push(StationEntry {
                audio_file_id,
                chance,
            });
        }
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::models::CreateProjectRequest;
    use std::path::PathBuf;

    #[test]
    fn find_by_name_returns_existing_station() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        let project = db
            .create_project(&CreateProjectRequest {
                name: "Test".to_string(),
                version: "0.1.0".to_string(),
                supported_version: "*".to_string(),
                tags: vec![],
                author: None,
                output_dir: PathBuf::from("/tmp/test"),
            })
            .unwrap();

        let repo = StationRepository::new(&db);
        repo.create(&project.id, "Frontline").unwrap();

        let found = repo.find_by_name(&project.id, "Frontline").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Frontline");

        let missing = repo.find_by_name(&project.id, "Missing").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn delete_station_removes_station_and_entries() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("test.db")).unwrap();
        let project = db
            .create_project(&CreateProjectRequest {
                name: "Test".to_string(),
                version: "0.1.0".to_string(),
                supported_version: "*".to_string(),
                tags: vec![],
                author: None,
                output_dir: PathBuf::from("/tmp/test"),
            })
            .unwrap();

        let repo = StationRepository::new(&db);
        let station = repo.create(&project.id, "ToDelete").unwrap();
        repo.delete(&station.id).unwrap();

        let remaining = repo.list_by_project(&project.id).unwrap();
        assert!(remaining.is_empty());
    }
}
