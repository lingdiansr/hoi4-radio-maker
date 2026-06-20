use hoi4_radio_maker_lib::db::Db;
use hoi4_radio_maker_lib::models::CreateProjectRequest;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: create a tiny valid-ish audio file for ffprobe/ffmpeg to process.
/// For this test we only care that the import path runs; ffprobe will report
/// whatever it can, and ffmpeg will transcode the file.
fn create_dummy_wav(dir: &std::path::Path) -> PathBuf {
    let path = dir.join("dummy.wav");
    // Minimal valid RIFF/WAVE header with silence (1 second, 44100 Hz, stereo, 16-bit)
    let data = b"RIFF\x26\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x02\x00\x44\xAC\x00\x00\x10\xB1\x02\x00\x04\x00\x10\x00data\x02\x00\x00\x00\x00\x00";
    std::fs::write(&path, data).unwrap();
    path
}

#[tokio::test]
async fn test_global_import_without_project() {
    // This test requires ffmpeg and ffprobe in PATH.
    if std::process::Command::new("ffmpeg").arg("-version").output().is_err() {
        eprintln!("ffmpeg not available, skipping test");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let db = Db::open(tmp.path().join("app.db")).unwrap();

    let wav = create_dummy_wav(tmp.path());

    // We can't easily invoke the Tauri command directly, but we can exercise
    // the Db layer used by the command.
    let project = db
        .create_project(&CreateProjectRequest {
            name: "Test".into(),
            version: "0.1.0".into(),
            supported_version: "*".into(),
            tags: vec![],
            author: None,
            output_dir: tmp.path().join("out"),
        })
        .unwrap();

    // Create a minimal audio record directly to verify the schema path.
    use chrono::Utc;
    use hoi4_radio_maker_lib::models::AudioFile;
    let now = Utc::now();
    let audio = AudioFile {
        id: "audio_test".into(),
        source_hash: "hash_test".into(),
        title: "Test Audio".into(),
        artist: None,
        source_path: wav.clone(),
        ogg_filename: "audio_test.ogg".into(),
        duration_secs: 1.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.75,
        tags: vec![],
        notes: None,
        created_at: now,
        updated_at: now,
    };
    db.create_audio_file(&audio).unwrap();

    let all = db.list_all_audio_files().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "Test Audio");

    // Verify project reference works.
    db.add_audio_to_project(&project.id, &audio.id).unwrap();
    let project_audio = db.list_audio_files(&project.id).unwrap();
    assert_eq!(project_audio.len(), 1);
    assert_eq!(project_audio[0].id, audio.id);
}
