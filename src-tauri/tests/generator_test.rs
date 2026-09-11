use std::path::PathBuf;

use chrono::Utc;
use hoi4_radio_maker_lib::generator::generate_mod;
use hoi4_radio_maker_lib::models::{
    AudioFile, ChanceConfig, ImportStatus, Modifier, Project, Station, StationEntry, Trigger,
};

#[test]
fn generates_expected_mod_files() {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("test_radio");
    let audio_store_dir = temp.path().join("audio_store");
    std::fs::create_dir_all(&audio_store_dir).unwrap();

    let project = Project {
        id: "test_radio".to_string(),
        name: "Test Radio Mod".to_string(),
        version: "1.0.0".to_string(),
        supported_version: "1.17.*".to_string(),
        tags: vec!["Sound".to_string(), "Music".to_string()],
        author: None,
        output_dir: output_dir.clone(),
        load_vanilla_triggers: true,
        trigger_mod_dirs: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let now = Utc::now();
    let audio_files = vec![
        AudioFile {
            id: "song_one".to_string(),
            source_hash: "hash_one".to_string(),
            title: "First \"Song\"".to_string(),
            artist: None,
            source_path: PathBuf::from("/fake/song_one.ogg"),
            ogg_filename: "song_one.ogg".to_string(),
            duration_secs: 120.0,
            sample_rate: 44100,
            channels: 2,
            volume: 0.65,
            tags: vec![],
            notes: None,
            import_status: ImportStatus::Ready,
            created_at: now,
            updated_at: now,
        },
        AudioFile {
            id: "song_two".to_string(),
            source_hash: "hash_two".to_string(),
            title: "Second Song".to_string(),
            artist: None,
            source_path: PathBuf::from("/fake/song_two.ogg"),
            ogg_filename: "song_two.ogg".to_string(),
            duration_secs: 180.0,
            sample_rate: 44100,
            channels: 2,
            volume: 0.8,
            tags: vec![],
            notes: None,
            import_status: ImportStatus::Ready,
            created_at: now,
            updated_at: now,
        },
    ];

    let station = Station {
        id: "test_station".to_string(),
        name: "Test Station".to_string(),
        subdir: None,
        entries: vec![
            StationEntry {
                audio_file_id: "song_one".to_string(),
                chance: ChanceConfig {
                    factor: 1.0,
                    modifiers: vec![],
                },
            },
            StationEntry {
                audio_file_id: "song_two".to_string(),
                chance: ChanceConfig {
                    factor: 0.5,
                    modifiers: vec![Modifier {
                        factor: Some(2.0),
                        add: None,
                        base: None,
                        triggers: vec![
                            Trigger::HasWar { value: true },
                            Trigger::Tag {
                                value: "GER".to_string(),
                            },
                        ],
                    }],
                },
            },
        ],
    };

    for audio in &audio_files {
        std::fs::write(audio_store_dir.join(&audio.ogg_filename), b"dummy ogg").unwrap();
    }

    generate_mod(
        &project,
        &[station],
        &audio_files,
        &output_dir,
        &audio_store_dir,
    )
    .expect("generate_mod failed");

    // Assert file existence.
    assert!(output_dir.join("descriptor.mod").is_file());
    assert!(output_dir
        .parent()
        .unwrap()
        .join("test_radio.mod")
        .is_file());
    // Station files live under a folder derived from the station name.
    let station_dir = output_dir.join("music").join("test_station");
    assert!(station_dir.join("test_station.asset").is_file());
    assert!(station_dir.join("test_station.txt").is_file());
    assert!(output_dir
        .join("localisation")
        .join("simp_chinese")
        .join("test_radio_music_l_simp_chinese.yml")
        .is_file());

    // Assert descriptor.mod content.
    let descriptor = std::fs::read_to_string(output_dir.join("descriptor.mod"))
        .expect("failed to read descriptor.mod");
    assert!(descriptor.contains("name=\"Test Radio Mod\""));
    assert!(descriptor.contains("version=\"1.0.0\""));
    assert!(descriptor.contains("supported_version=\"v1.17.*\""));
    assert!(descriptor.contains("\"Sound\""));

    // Assert launcher .mod content.
    let launcher = std::fs::read_to_string(output_dir.parent().unwrap().join("test_radio.mod"))
        .expect("failed to read launcher .mod");
    assert!(launcher.contains("name=\"Test Radio Mod\""));
    assert!(launcher.contains("path=\"test_radio\""));

    // Assert station asset content.
    let asset = std::fs::read_to_string(station_dir.join("test_station.asset"))
        .expect("failed to read asset");
    assert!(asset.contains("name = \"song_one\""));
    assert!(asset.contains("file = \"song_one.ogg\""));
    assert!(asset.contains("name = \"song_two\""));
    assert!(asset.contains("volume = 0.8"));

    // Assert station txt content.
    let txt =
        std::fs::read_to_string(station_dir.join("test_station.txt")).expect("failed to read txt");
    assert!(txt.contains("music_station = \"test_station\""));
    assert!(txt.contains("song = \"song_one\""));
    assert!(txt.contains("factor = 1"));
    assert!(txt.contains("song = \"song_two\""));
    assert!(txt.contains("factor = 0.5"));
    assert!(txt.contains("modifier = {"));
    assert!(txt.contains("factor = 2"));
    assert!(txt.contains("has_war = yes"));
    assert!(txt.contains("tag = GER"));

    // Assert localisation content.
    let loc = std::fs::read_to_string(
        output_dir
            .join("localisation")
            .join("simp_chinese")
            .join("test_radio_music_l_simp_chinese.yml"),
    )
    .expect("failed to read localisation");
    assert!(loc.contains("l_simp_chinese:"));
    assert!(loc.contains("test_radio_music_TITLE:0 \"Test Radio Mod\""));
    assert!(loc.contains("song_one:0 \"First \\\"Song\\\"\""));
    assert!(loc.contains("song_two:0 \"Second Song\""));
}

#[test]
fn only_includes_referenced_audio_files() {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("ref_radio");
    let audio_store_dir = temp.path().join("audio_store");
    std::fs::create_dir_all(&audio_store_dir).unwrap();

    let now = Utc::now();
    let project = Project {
        id: "ref_radio".to_string(),
        name: "Referenced Radio".to_string(),
        version: "1.0.0".to_string(),
        supported_version: "1.17.*".to_string(),
        tags: vec!["Sound".to_string()],
        author: None,
        output_dir: output_dir.clone(),
        load_vanilla_triggers: true,
        trigger_mod_dirs: vec![],
        created_at: now,
        updated_at: now,
    };

    let audio_files = vec![
        AudioFile {
            id: "used".to_string(),
            source_hash: "hash_used".to_string(),
            title: "Used Song".to_string(),
            artist: None,
            source_path: PathBuf::from("/fake/used.ogg"),
            ogg_filename: "used.ogg".to_string(),
            duration_secs: 60.0,
            sample_rate: 44100,
            channels: 2,
            volume: 0.65,
            tags: vec![],
            notes: None,
            import_status: ImportStatus::Ready,
            created_at: now,
            updated_at: now,
        },
        AudioFile {
            id: "unused".to_string(),
            source_hash: "hash_unused".to_string(),
            title: "Unused Song".to_string(),
            artist: None,
            source_path: PathBuf::from("/fake/unused.ogg"),
            ogg_filename: "unused.ogg".to_string(),
            duration_secs: 60.0,
            sample_rate: 44100,
            channels: 2,
            volume: 0.65,
            tags: vec![],
            notes: None,
            import_status: ImportStatus::Ready,
            created_at: now,
            updated_at: now,
        },
    ];

    let station = Station {
        id: "only_used".to_string(),
        name: "Only Used".to_string(),
        subdir: None,
        entries: vec![StationEntry {
            audio_file_id: "used".to_string(),
            chance: ChanceConfig {
                factor: 1.0,
                modifiers: vec![],
            },
        }],
    };

    std::fs::write(audio_store_dir.join("used.ogg"), b"dummy ogg").unwrap();

    generate_mod(
        &project,
        &[station],
        &audio_files,
        &output_dir,
        &audio_store_dir,
    )
    .expect("generate_mod failed");

    let asset = std::fs::read_to_string(
        output_dir
            .join("music")
            .join("only_used")
            .join("only_used.asset"),
    )
    .expect("failed to read asset");
    assert!(asset.contains("name = \"used\""));
    assert!(!asset.contains("name = \"unused\""));
}

#[test]
fn stations_write_into_name_derived_subdirectories() {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("subdir_radio");
    let audio_store_dir = temp.path().join("audio_store");
    std::fs::create_dir_all(&audio_store_dir).unwrap();

    let project = Project {
        id: "subdir_radio".to_string(),
        name: "Subdir Radio".to_string(),
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
    let make_audio = |id: &str, file: &str| AudioFile {
        id: id.to_string(),
        source_hash: format!("hash_{id}"),
        title: id.to_string(),
        artist: None,
        source_path: PathBuf::from(format!("/tmp/{file}")),
        ogg_filename: file.to_string(),
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
    let audio_files = vec![
        make_audio("chi_song", "chi_song.ogg"),
        make_audio("derived_song", "derived_song.ogg"),
    ];
    for audio in &audio_files {
        std::fs::write(audio_store_dir.join(&audio.ogg_filename), b"dummy ogg").unwrap();
    }

    let entry = |audio_id: &str| StationEntry {
        audio_file_id: audio_id.to_string(),
        chance: ChanceConfig {
            factor: 1.0,
            modifiers: vec![],
        },
    };

    // Custom folder for the first station; name-derived folder for the second.
    let custom = Station {
        id: "radio_chi".to_string(),
        name: "Radio CHI".to_string(),
        subdir: Some("kmt_radio".to_string()),
        entries: vec![entry("chi_song")],
    };
    let derived = Station {
        id: "radio_prc".to_string(),
        name: "Radio PRC".to_string(),
        subdir: None,
        entries: vec![entry("derived_song")],
    };

    generate_mod(
        &project,
        &[custom, derived],
        &audio_files,
        &output_dir,
        &audio_store_dir,
    )
    .expect("generate_mod failed");

    let music_dir = output_dir.join("music");

    // Custom folder wins over the name.
    let custom_dir = music_dir.join("kmt_radio");
    let custom_asset =
        std::fs::read_to_string(custom_dir.join("radio_chi.asset")).expect("custom asset exists");
    assert!(custom_asset.contains("name = \"chi_song\""));
    assert!(custom_asset.contains("file = \"chi_song.ogg\""));
    assert!(custom_dir.join("radio_chi.txt").is_file());
    assert!(custom_dir.join("chi_song.ogg").is_file());

    // Unset folder falls back to a slug of the station name.
    let derived_dir = music_dir.join("radio_prc");
    let derived_asset =
        std::fs::read_to_string(derived_dir.join("radio_prc.asset")).expect("derived asset exists");
    assert!(derived_asset.contains("name = \"derived_song\""));
    assert!(derived_dir.join("derived_song.ogg").is_file());

    // Nothing is written flat into music/ any more.
    assert!(!music_dir.join("radio_chi.asset").exists());
    assert!(!music_dir.join("chi_song.ogg").exists());
    assert!(!music_dir.join("derived_song.ogg").exists());
}

#[test]
fn colliding_station_names_get_distinct_directories() {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("collide_radio");
    let audio_store_dir = temp.path().join("audio_store");
    std::fs::create_dir_all(&audio_store_dir).unwrap();

    let project = Project {
        id: "collide_radio".to_string(),
        name: "Collide Radio".to_string(),
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
        id: "alpha".to_string(),
        source_hash: "hash_alpha".to_string(),
        title: "Alpha".to_string(),
        artist: None,
        source_path: PathBuf::from("/tmp/alpha.ogg"),
        ogg_filename: "alpha.ogg".to_string(),
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
    std::fs::write(audio_store_dir.join("alpha.ogg"), b"dummy ogg").unwrap();

    let entry = || StationEntry {
        audio_file_id: "alpha".to_string(),
        chance: ChanceConfig {
            factor: 1.0,
            modifiers: vec![],
        },
    };

    // Both names slug to "front_line"; the second must get its own folder.
    let first = Station {
        id: "front_line".to_string(),
        name: "Front Line".to_string(),
        subdir: None,
        entries: vec![entry()],
    };
    let second = Station {
        id: "front_line_2".to_string(),
        name: "Front_Line".to_string(),
        subdir: None,
        entries: vec![entry()],
    };

    generate_mod(
        &project,
        &[first, second],
        &[audio],
        &output_dir,
        &audio_store_dir,
    )
    .expect("generate_mod failed");

    let music_dir = output_dir.join("music");
    assert!(music_dir
        .join("front_line")
        .join("front_line.asset")
        .is_file());
    assert!(music_dir
        .join("front_line_2")
        .join("front_line_2.asset")
        .is_file());
}
