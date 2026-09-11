use std::path::PathBuf;

use chrono::Utc;
use hoi4_radio_maker_lib::generator::generate_mod;
use hoi4_radio_maker_lib::models::{
    AudioFile, ChanceConfig, ImportStatus, Project, Station, StationEntry,
};
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
        load_vanilla_triggers: true,
        trigger_mod_dirs: vec![],
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
        import_status: ImportStatus::Ready,
        created_at: now,
        updated_at: now,
    }];

    let station = Station {
        id: "val_station".to_string(),
        name: "Validation Station".to_string(),
        subdir: None,
        entries: vec![StationEntry {
            audio_file_id: "missing_song".to_string(),
            chance: ChanceConfig {
                factor: 1.0,
                modifiers: vec![],
            },
        }],
    };

    generate_mod(
        &project,
        &[station],
        &audio_files,
        &output_dir,
        &audio_store_dir,
    )
    .expect("generate_mod failed");

    let report = validate_mod_output(&output_dir, None, &Default::default())
        .await
        .expect("validate_mod_output failed");

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|e| { e.contains("missing_song.ogg") && e.contains("does not exist") }),
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
        load_vanilla_triggers: true,
        trigger_mod_dirs: vec![],
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
        import_status: ImportStatus::Ready,
        created_at: now,
        updated_at: now,
    }];

    std::fs::write(audio_store_dir.join("dummy_song.ogg"), b"").unwrap();

    let station = Station {
        id: "complete_station".to_string(),
        name: "Complete Station".to_string(),
        subdir: None,
        entries: vec![StationEntry {
            audio_file_id: "dummy_song".to_string(),
            chance: ChanceConfig {
                factor: 1.0,
                modifiers: vec![],
            },
        }],
    };

    generate_mod(
        &project,
        &[station],
        &audio_files,
        &output_dir,
        &audio_store_dir,
    )
    .expect("generate_mod failed");

    // Create an empty dummy OGG file; ffprobe will report it as not decodable.
    let ogg_path = output_dir.join("music").join("dummy_song.ogg");
    std::fs::write(&ogg_path, b"").expect("failed to write dummy ogg");

    let report = validate_mod_output(&output_dir, None, &Default::default())
        .await
        .expect("validate_mod_output failed");

    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|e| { e.contains("dummy_song.ogg") && e.contains("not decodable") }),
        "expected an OGG decode error, got: {:?}",
        report.errors
    );
    assert_eq!(report.ogg_files_checked, 1);
}

#[cfg(unix)]
#[tokio::test]
async fn test_validate_scans_station_subdirectories() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("nested_radio");
    let audio_store_dir = temp.path().join("audio_store");
    std::fs::create_dir_all(&audio_store_dir).unwrap();

    let project = Project {
        id: "nested_radio".to_string(),
        name: "Nested Radio".to_string(),
        version: "1.0.0".to_string(),
        supported_version: "1.17.*".to_string(),
        tags: vec![],
        author: None,
        output_dir: output_dir.clone(),
        load_vanilla_triggers: true,
        trigger_mod_dirs: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let now = Utc::now();
    let audio = AudioFile {
        id: "nested_song".to_string(),
        source_hash: "hash_nested".to_string(),
        title: "Nested Song".to_string(),
        artist: None,
        source_path: PathBuf::from("/tmp/nested.ogg"),
        ogg_filename: "nested.ogg".to_string(),
        duration_secs: 60.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.75,
        tags: vec![],
        notes: None,
        import_status: ImportStatus::Ready,
        created_at: now,
        updated_at: now,
    };
    std::fs::write(audio_store_dir.join("nested.ogg"), b"dummy ogg").unwrap();

    let station = Station {
        id: "radio_chi".to_string(),
        name: "Radio CHI".to_string(),
        subdir: None,
        entries: vec![StationEntry {
            audio_file_id: "nested_song".to_string(),
            chance: ChanceConfig {
                factor: 1.0,
                modifiers: vec![],
            },
        }],
    };

    generate_mod(
        &project,
        &[station],
        &[audio],
        &output_dir,
        &audio_store_dir,
    )
    .expect("generate_mod failed");

    // The station folder comes from its name.
    let station_dir = output_dir.join("music").join("radio_chi");
    assert!(station_dir.join("radio_chi.asset").is_file());
    assert!(station_dir.join("nested.ogg").is_file());

    // A stand-in ffprobe that always reports success, so the test proves the
    // subdirectory layout resolves rather than depending on real decoders.
    let fake_ffprobe = temp.path().join("fake_ffprobe");
    std::fs::write(&fake_ffprobe, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perm = std::fs::metadata(&fake_ffprobe).unwrap().permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(&fake_ffprobe, perm).unwrap();
    let ffprobe = fake_ffprobe.to_str().unwrap();

    let report = validate_mod_output(&output_dir, Some(ffprobe), &Default::default())
        .await
        .expect("validate_mod_output failed");
    assert!(
        report.passed,
        "expected a passing report for a subdirectory layout, got: {:?}",
        report.errors
    );
    assert_eq!(report.ogg_files_checked, 1);

    // A missing OGG inside the subdirectory is still detected.
    std::fs::remove_file(station_dir.join("nested.ogg")).unwrap();
    let report = validate_mod_output(&output_dir, Some(ffprobe), &Default::default())
        .await
        .expect("validate_mod_output failed");
    assert!(!report.passed);
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("nested.ogg") && e.contains("does not exist")),
        "expected a missing-OGG error, got: {:?}",
        report.errors
    );
}

/// A `.txt` whose trigger is absent from the loaded vocabulary, or whose tag /
/// ideology does not exist in it, is reported as a warning; a correct one is
/// not. An empty vocabulary disables the check entirely.
#[tokio::test]
async fn test_validate_checks_triggers_against_vocabulary() {
    use hoi4_radio_maker_lib::scripts::{ScriptVocabulary, TriggerDef};
    use hoi4_radio_maker_lib::validator::validate_mod_output;
    use std::collections::{BTreeMap, BTreeSet};

    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("trig_radio");
    let music_dir = output_dir.join("music").join("radio_chi");
    std::fs::create_dir_all(&music_dir).unwrap();

    std::fs::write(music_dir.join("radio_chi.ogg"), b"dummy ogg").unwrap();
    std::fs::write(
        music_dir.join("radio_chi.asset"),
        "music = { name = \"song_a\" file = \"radio_chi.ogg\" volume = 1.0 }",
    )
    .unwrap();
    std::fs::write(
        music_dir.join("radio_chi.txt"),
        r#"music_station = "radio_chi"
music = {
    song = "song_a"
    chance = {
        factor = 1
        modifier = {
            factor = 2
            tag = CHI
            has_war = yes
            made_up_trigger = yes
        }
    }
}
"#,
    )
    .unwrap();
    std::fs::write(
        output_dir.join("descriptor.mod"),
        "name=\"Trig\"\n",
    )
    .unwrap();
    let loc_dir = output_dir.join("localisation").join("simp_chinese");
    std::fs::create_dir_all(&loc_dir).unwrap();
    std::fs::write(
        loc_dir.join("trig_radio_music_l_simp_chinese.yml"),
        "\u{FEFF}l_simp_chinese:\n song_a:0 \"Song A\"\n",
    )
    .unwrap();

    // A vocabulary that knows has_war/tag and CHI, but not made_up_trigger.
    let mut vocab = ScriptVocabulary::default();
    let mut triggers: BTreeMap<String, TriggerDef> = BTreeMap::new();
    for name in ["has_war", "tag", "is_in_faction_with", "has_government"] {
        triggers.insert(
            name.to_string(),
            TriggerDef {
                name: name.to_string(),
                scopes: vec!["COUNTRY".to_string()],
                targets: vec!["none".to_string()],
            },
        );
    }
    vocab.triggers = triggers;
    let mut tags = BTreeSet::new();
    tags.insert("CHI".to_string());
    vocab.country_tags = tags;

    let report = validate_mod_output(&output_dir, None, &vocab)
        .await
        .expect("validate failed");

    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("made_up_trigger") && w.contains("Unknown trigger")),
        "expected an unknown-trigger warning, got: {:?}",
        report.warnings
    );
    // The known trigger and known tag produce no vocabulary warnings.
    assert!(
        !report
            .warnings
            .iter()
            .any(|w| w.contains("'tag'") || w.contains("'has_war'")),
        "known triggers must not warn, got: {:?}",
        report.warnings
    );

    // A wrong tag is caught once the vocabulary carries country tags.
    let mut bad = vocab.clone();
    let mut tags = BTreeSet::new();
    tags.insert("GER".to_string());
    bad.country_tags = tags;
    let report = validate_mod_output(&output_dir, None, &bad)
        .await
        .expect("validate failed");
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("Unknown country tag 'CHI'")),
        "expected an unknown-tag warning, got: {:?}",
        report.warnings
    );

    // Without a vocabulary the check is skipped.
    let report = validate_mod_output(&output_dir, None, &ScriptVocabulary::default())
        .await
        .expect("validate failed");
    assert!(
        !report.warnings.iter().any(|w| w.contains("Unknown trigger")),
        "empty vocabulary must skip the check, got: {:?}",
        report.warnings
    );

}
