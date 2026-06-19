use crate::db::{BatchImportResult, Db};
use crate::error::Result;
use crate::models::{AudioFile, BatchUpdateAudioFileRequest, UpdateAudioFileRequest};

/// Repository for audio file records in the global library.
pub struct AudioRepository<'a> {
    db: &'a Db,
}

impl<'a> AudioRepository<'a> {
    /// Create a new repository backed by the given database.
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Insert a new audio file into the global library.
    pub fn create(&self, audio: &AudioFile) -> Result<AudioFile> {
        self.db.create_audio_file(audio)
    }

    /// Add an existing audio file to a project.
    pub fn add_to_project(&self, project_id: &str, audio_file_id: &str) -> Result<()> {
        self.db.add_audio_to_project(project_id, audio_file_id)
    }

    /// Remove an audio file reference from a project.
    pub fn remove_from_project(&self, project_id: &str, audio_file_id: &str) -> Result<()> {
        self.db.remove_audio_from_project(project_id, audio_file_id)
    }

    /// List all audio files belonging to a project.
    pub fn list(&self, project_id: &str) -> Result<Vec<AudioFile>> {
        self.db.list_audio_files(project_id)
    }

    /// List all audio files in the global library.
    pub fn list_all(&self) -> Result<Vec<AudioFile>> {
        self.db.list_all_audio_files()
    }

    /// Fetch a single audio file by ID, if it exists.
    pub fn get(&self, id: &str) -> Result<Option<AudioFile>> {
        self.db.get_audio_file(id)
    }

    /// Fetch a single audio file by source hash, if it exists.
    pub fn get_by_hash(&self, hash: &str) -> Result<Option<AudioFile>> {
        self.db.get_audio_file_by_hash(hash)
    }

    /// Delete an audio file from the global library.
    pub fn delete(&self, id: &str) -> Result<()> {
        self.db.delete_audio_file(id)
    }

    /// Update a single audio file.
    pub fn update(&self, id: &str, req: &UpdateAudioFileRequest) -> Result<AudioFile> {
        self.db.update_audio_file(id, req)
    }

    /// Batch update multiple audio files.
    pub fn batch_update(
        &self,
        ids: &[String],
        req: &BatchUpdateAudioFileRequest,
    ) -> Result<Vec<AudioFile>> {
        self.db.batch_update_audio_files(ids, req)
    }

    /// Import multiple audio files into a project.
    pub fn import_batch(&self, project_id: &str, result: &BatchImportResult) -> Result<()> {
        for audio in &result.created {
            self.db.create_audio_file(audio)?;
        }
        for audio in result.created.iter().chain(result.existing.iter()) {
            self.db.add_audio_to_project(project_id, &audio.id)?;
        }
        Ok(())
    }
}
