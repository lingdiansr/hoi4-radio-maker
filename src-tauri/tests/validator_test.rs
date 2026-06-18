use std::path::PathBuf;

use chrono::Utc;
use hoi4_radio_maker_lib::generator::generate_mod;
use hoi4_radio_maker_lib::models::{AudioFile, ChanceConfig, Project, Station, StationEntry};
use hoi4_radio_maker_lib::validator::validate_mod_output;

#[tokio::test]
async fn test_validate_generated_mod_reports_missing_ogg() {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("val_radio");
    let audio_store_dir = temp.path().join("audio_store");
    std::fs::create_dir_all(&audio_store_dir).unwrap();

    let project = Project {
        id: "val_radio".to_string(),
        name: "Validation Radio".to_string(),
        version: "1.0.0".to_string(),
        supported_version: "1.17.*".to_string(),
        tags: vec!["Sound".to_string()],
        author: None,
        output_dir: output_dir.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let now = Utc::now();
    let audio_files = vec![AudioFile {
        id: "missing_song".to_string(),
        source_hash: "hash_missing".to_string(),
        title: "Missing Song".to_string(),
        artist: None,
        source_path: PathBuf::from("/fake/missing_song.ogg"),
        ogg_filename: "missing_song.ogg".to_string(),
        duration_secs: 120.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.65,
        tags: vec![],
        notes: None,
        created_at: now,
        updated_at: now,
    }];

    let station = Station {
        id: "val_station".to_string(),
        name: "Validation Station".to_string(),
        entries: vec![StationEntry {
            audio_file_id: "missing_song".to_string(),
            chance: ChanceConfig {
                factor: 1.0,
                modifiers: vec![],
            },
        }],
    };

    generate_mod(&project, &[station], &audio_files, &output_dir, &audio_store_dir)
        .expect("generate_mod failed");

    let report = validate_mod_output(&output_dir)
        .await
        .expect("validate_mod_output failed");

    assert!(!report.passed);
    assert!(
        report.errors.iter().any(|e| {
            e.contains("missing_song.ogg") && e.contains("does not exist")
        }),
        "expected an error about the missing OGG file, got: {:?}",
        report.errors
    );
}

#[tokio::test]
async fn test_validate_complete_mod_reports_ogg_decode_error() {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("complete_radio");
    let audio_store_dir = temp.path().join("audio_store");
    std::fs::create_dir_all(&audio_store_dir).unwrap();

    let project = Project {
        id: "complete_radio".to_string(),
        name: "Complete Radio".to_string(),
        version: "1.0.0".to_string(),
        supported_version: "1.17.*".to_string(),
        tags: vec!["Sound".to_string()],
        author: None,
        output_dir: output_dir.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let now = Utc::now();
    let audio_files = vec![AudioFile {
        id: "dummy_song".to_string(),
        source_hash: "hash_dummy".to_string(),
        title: "Dummy Song".to_string(),
        artist: None,
        source_path: PathBuf::from("/fake/dummy_song.ogg"),
        ogg_filename: "dummy_song.ogg".to_string(),
        duration_secs: 120.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.65,
        tags: vec![],
        notes: None,
        created_at: now,
        updated_at: now,
    }];

    std::fs::write(audio_store_dir.join("dummy_song.ogg"), b"").unwrap();

    let station = Station {
        id: "complete_station".to_string(),
        name: "Complete Station".to_string(),
        entries: vec![StationEntry {
            audio_file_id: "dummy_song".to_string(),
            chance: ChanceConfig {
                factor: 1.0,
                modifiers: vec![],
            },
        }],
    };

    generate_mod(&project, &[station], &audio_files, &output_dir, &audio_store_dir)
        .expect("generate_mod failed");

    // Create an empty dummy OGG file; ffprobe will report it as not decodable.
    let ogg_path = output_dir.join("music").join("dummy_song.ogg");
    std::fs::write(&ogg_path, b"").expect("failed to write dummy ogg");

    let report = validate_mod_output(&output_dir)
        .await
        .expect("validate_mod_output failed");

    assert!(!report.passed);
    assert!(
        report.errors.iter().any(|e| {
            e.contains("dummy_song.ogg") && e.contains("not decodable")
        }),
        "expected an OGG decode error, got: {:?}",
        report.errors
    );
    assert_eq!(report.ogg_files_checked, 1);
}
