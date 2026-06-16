use hoi4_radio_maker_lib::audio_repo::AudioRepository;
use hoi4_radio_maker_lib::db::Db;
use hoi4_radio_maker_lib::models::{AudioFile, CreateProjectRequest};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_audio_file_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let db = Db::open(tmp.path().join("app.db")).unwrap();

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

    let repo = AudioRepository::new(&db);
    repo.create(&project.id, &audio).unwrap();

    let list = repo.list(&project.id).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].title, "Test Song");

    repo.delete(&audio.id).unwrap();
    let list = repo.list(&project.id).unwrap();
    assert!(list.is_empty());
}
