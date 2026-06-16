use crate::db::Db;
use crate::error::Result;
use crate::models::AudioFile;

/// Repository for audio file records in a project.
pub struct AudioRepository<'a> {
    db: &'a Db,
}

impl<'a> AudioRepository<'a> {
    /// Create a new repository backed by the given database.
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Insert a new audio file into the project.
    pub fn create(&self, project_id: &str, audio: &AudioFile) -> Result<AudioFile> {
        self.db.create_audio_file(project_id, audio)
    }

    /// List all audio files belonging to a project.
    pub fn list(&self, project_id: &str) -> Result<Vec<AudioFile>> {
        self.db.list_audio_files(project_id)
    }

    /// Fetch a single audio file by ID, if it exists.
    pub fn get(&self, id: &str) -> Result<Option<AudioFile>> {
        self.db.get_audio_file(id)
    }

    /// Delete an audio file by ID.
    pub fn delete(&self, id: &str) -> Result<()> {
        self.db.delete_audio_file(id)
    }
}
