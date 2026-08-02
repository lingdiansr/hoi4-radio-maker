use chrono::Utc;
use hoi4_radio_maker_lib::db::Db;
use hoi4_radio_maker_lib::models::{AudioFile, ChanceConfig, CreateProjectRequest, ImportStatus};
use hoi4_radio_maker_lib::station::StationRepository;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_station_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let db = Db::open(tmp.path().join("app.db")).unwrap();

    let project = db
        .create_project(&CreateProjectRequest {
            name: "Test Project".into(),
            version: "0.1.0".into(),
            supported_version: "*".into(),
            tags: vec![],
            author: None,
            output_dir: tmp.path().join("out"),
        })
        .unwrap();

    let now = Utc::now();
    let audio = AudioFile {
        id: "song_001".into(),
        source_hash: "abc123".into(),
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
        import_status: ImportStatus::Ready,
        created_at: now,
        updated_at: now,
    };
    db.create_audio_file(&audio).unwrap();
    db.add_audio_to_project(&project.id, &audio.id).unwrap();

    let repo = StationRepository::new(&db);

    let station = repo.create(&project.id, "Main Station").unwrap();
    assert_eq!(station.name, "Main Station");
    assert!(station.id.starts_with("station_"));
    assert!(station.entries.is_empty());

    repo.add_entry(
        &station.id,
        &audio.id,
        ChanceConfig {
            factor: 0.5,
            modifiers: vec![],
        },
    )
    .unwrap();

    let fetched = repo.get(&station.id).unwrap().expect("station exists");
    assert_eq!(fetched.entries.len(), 1);
    assert_eq!(fetched.entries[0].audio_file_id, audio.id);
    assert!((fetched.entries[0].chance.factor - 0.5).abs() < f64::EPSILON);

    repo.remove_entry(&station.id, &audio.id).unwrap();

    let fetched = repo.get(&station.id).unwrap().expect("station exists");
    assert!(fetched.entries.is_empty());

    repo.delete(&station.id).unwrap();

    let stations = repo.list_by_project(&project.id).unwrap();
    assert!(stations.is_empty());
}
