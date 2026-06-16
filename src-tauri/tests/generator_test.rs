use std::path::PathBuf;

use chrono::Utc;
use hoi4_radio_maker_lib::generator::generate_mod;
use hoi4_radio_maker_lib::models::{
    AudioFile, ChanceConfig, Modifier, Project, Station, StationEntry, Trigger,
};

#[test]
fn generates_expected_mod_files() {
    let temp = tempfile::tempdir().expect("failed to create temp dir");
    let output_dir = temp.path().join("mod").join("test_radio");

    let project = Project {
        id: "test_radio".to_string(),
        name: "Test Radio Mod".to_string(),
        version: "1.0.0".to_string(),
        supported_version: "1.17.*".to_string(),
        tags: vec!["Sound".to_string(), "Music".to_string()],
        author: None,
        output_dir: output_dir.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let audio_files = vec![
        AudioFile {
            id: "song_one".to_string(),
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
        },
        AudioFile {
            id: "song_two".to_string(),
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
        },
    ];

    let station = Station {
        id: "test_station".to_string(),
        name: "Test Station".to_string(),
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
                            Trigger::Tag { value: "GER".to_string() },
                        ],
                    }],
                },
            },
        ],
    };

    generate_mod(&project, &[station], &audio_files, &output_dir).expect("generate_mod failed");

    // Assert file existence.
    assert!(output_dir.join("descriptor.mod").is_file());
    assert!(output_dir
        .parent()
        .unwrap()
        .join("test_radio.mod")
        .is_file());
    assert!(output_dir.join("music").join("test_station.asset").is_file());
    assert!(output_dir.join("music").join("test_station.txt").is_file());
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
    assert!(descriptor.contains("supported_version=\"1.17.*\""));
    assert!(descriptor.contains("\"Sound\""));

    // Assert launcher .mod content.
    let launcher = std::fs::read_to_string(output_dir.parent().unwrap().join("test_radio.mod"))
        .expect("failed to read launcher .mod");
    assert!(launcher.contains("name=\"Test Radio Mod\""));
    assert!(launcher.contains("path=\"mod/test_radio\""));

    // Assert station asset content.
    let asset = std::fs::read_to_string(output_dir.join("music").join("test_station.asset"))
        .expect("failed to read asset");
    assert!(asset.contains("name = \"song_one\""));
    assert!(asset.contains("file = \"song_one.ogg\""));
    assert!(asset.contains("name = \"song_two\""));
    assert!(asset.contains("volume = 0.8"));

    // Assert station txt content.
    let txt = std::fs::read_to_string(output_dir.join("music").join("test_station.txt"))
        .expect("failed to read txt");
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

    let project = Project {
        id: "ref_radio".to_string(),
        name: "Referenced Radio".to_string(),
        version: "1.0.0".to_string(),
        supported_version: "1.17.*".to_string(),
        tags: vec!["Sound".to_string()],
        author: None,
        output_dir: output_dir.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let audio_files = vec![
        AudioFile {
            id: "used".to_string(),
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
        },
        AudioFile {
            id: "unused".to_string(),
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
        },
    ];

    let station = Station {
        id: "only_used".to_string(),
        name: "Only Used".to_string(),
        entries: vec![StationEntry {
            audio_file_id: "used".to_string(),
            chance: ChanceConfig {
                factor: 1.0,
                modifiers: vec![],
            },
        }],
    };

    generate_mod(&project, &[station], &audio_files, &output_dir).expect("generate_mod failed");

    let asset = std::fs::read_to_string(output_dir.join("music").join("only_used.asset"))
        .expect("failed to read asset");
    assert!(asset.contains("name = \"used\""));
    assert!(!asset.contains("name = \"unused\""));
}
