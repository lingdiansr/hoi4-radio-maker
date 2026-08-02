# HOI4 Radio Maker Compatibility & Quality Implementation Plan

> **For agentic workers:** REQUIRED SUB-_SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the highest-impact gaps discovered by auditing real HOI4 and Steam Workshop music mods: make generated localisation files load in-game, ensure validation works with custom ffprobe paths, force compatible Ogg Vorbis output, and auto-fill ID3 metadata on import.

**Architecture:** All four tasks are backend-only changes in `src-tauri/src/`. Each change is isolated to one or two files and is covered by a focused unit/integration test. No database schema changes are required.

**Tech Stack:** Rust, Tauri, SQLite (rusqlite), ffmpeg/ffprobe.

---

## File Structure

- `src-tauri/src/generator.rs` — write UTF-8 BOM before localisation YAML.
- `src-tauri/src/validator.rs` — accept an optional ffprobe binary path.
- `src-tauri/src/commands.rs` — pass settings ffprobe path to validator; use ID3 metadata when available.
- `src-tauri/src/audio.rs` — force 2-channel output; expose ffprobe metadata parsing for tests.
- `src-tauri/src/models.rs` — extend `AudioMetadata` with optional `title`/`artist`.

---

### Task 1: Localisation files must have UTF-8 BOM

**Files:**
- Modify: `src-tauri/src/generator.rs`
- Test: `src-tauri/src/generator.rs` (new test in `mod tests`)

HOI4 only loads `.yml` localisation files when they start with a UTF-8 BOM (`EF BB BF`). Currently `write_localisation` writes plain UTF-8.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn generated_localisation_has_utf8_bom() {
    use crate::models::{AudioFile, ImportStatus, Project};
    use chrono::Utc;
    use std::path::PathBuf;

    let tmp = tempfile::tempdir().unwrap();
    let loc_dir = tmp.path().join("localisation").join("simp_chinese");
    std::fs::create_dir_all(&loc_dir).unwrap();

    let project = Project {
        id: "my_project".to_string(),
        name: "My Project".to_string(),
        version: "0.1.0".to_string(),
        supported_version: "1.14.*".to_string(),
        tags: vec!["Sound".to_string()],
        author: None,
        output_dir: PathBuf::from("/tmp/my_project"),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let audio = AudioFile {
        id: "audio_001".to_string(),
        source_hash: "abc".to_string(),
        title: "Test Song".to_string(),
        artist: Some("Test Artist".to_string()),
        source_path: PathBuf::from("/tmp/test.mp3"),
        ogg_filename: "audio_001.ogg".to_string(),
        duration_secs: 120.0,
        sample_rate: 44100,
        channels: 2,
        volume: 0.75,
        tags: vec![],
        notes: None,
        import_status: ImportStatus::Ready,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    write_localisation(&project, &[audio], &loc_dir).unwrap();

    let path = loc_dir.join("my_project_music_l_simp_chinese.yml");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd src-tauri && cargo test --lib generated_localisation_has_utf8_bom -- --nocapture
```
Expected: FAIL — assertion fails because the file starts with `l` not BOM.

- [ ] **Step 3: Write minimal implementation**

In `src-tauri/src/generator.rs`, update `write_localisation`:

```rust
fn write_localisation(
    project: &Project,
    audio_files: &[AudioFile],
    loc_dir: &Path,
) -> Result<()> {
    let path = loc_dir.join(format!("{}_music_l_simp_chinese.yml", project.id));
    let mut file = File::create(&path)?;

    // HOI4 requires UTF-8 BOM at the start of localisation files.
    file.write_all("\u{FEFF}".as_bytes())?;

    writeln!(file, "l_simp_chinese:")?;
    writeln!(
        file,
        " {}_music_TITLE:0 \"{}\"",
        project.id,
        escape_hoi4(&project.name)
    )?;

    for audio in audio_files {
        writeln!(
            file,
            " {}:0 \"{}\"",
            audio.id,
            escape_hoi4(&audio.title)
        )?;
    }

    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd src-tauri && cargo test --lib generated_localisation_has_utf8_bom -- --nocapture
```
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/generator.rs
git commit -m "fix(generator): write UTF-8 BOM for HOI4 localisation files"
```

---

### Task 2: Validator must use the configured ffprobe path

**Files:**
- Modify: `src-tauri/src/validator.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/validator.rs` (new test in `mod tests`)

`validate_mod_output` currently calls the hard-coded `ffprobe` binary. If the user has configured a custom ffprobe path, validation will fail or silently fall back to the system binary.

- [ ] **Step 1: Write the failing test**

Create a fake ffprobe executable in the test and assert it is invoked.

```rust
#[test]
#[cfg(unix)]
fn validator_uses_custom_ffprobe_path() {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    let tmp = tempfile::tempdir().unwrap();
    let music_dir = tmp.path().join("music");
    let loc_dir = tmp.path().join("localisation").join("simp_chinese");
    std::fs::create_dir_all(&music_dir).unwrap();
    std::fs::create_dir_all(&loc_dir).unwrap();

    // Minimal valid OGG that ffprobe would accept — we only need the validator
    // to call our fake binary and report success.
    let fake_ogg = music_dir.join("audio_001.ogg");
    std::fs::write(&fake_ogg, b"OggS").unwrap();

    let asset_path = music_dir.join("station.asset");
    std::fs::write(
        &asset_path,
        r#"music = { name = "audio_001" file = "audio_001.ogg" volume = 0.75 }"#,
    )
    .unwrap();

    let txt_path = music_dir.join("station.txt");
    std::fs::write(
        &txt_path,
        r#"music_station = "station" music = { song = "audio_001" }"#,
    )
    .unwrap();

    let loc_path = loc_dir.join("proj_music_l_simp_chinese.yml");
    std::fs::write(&loc_path, "\u{FEFF}l_simp_chinese:\n audio_001:0 \"Song\"\n").unwrap();

    let fake_ffprobe = tmp.path().join("fake_ffprobe");
    std::fs::write(
        &fake_ffprobe,
        "#!/bin/sh\necho '{\"streams\":[{\"codec_type\":\"audio\"}]}'\n",
    )
    .unwrap();
    let mut perm = std::fs::metadata(&fake_ffprobe).unwrap().permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(&fake_ffprobe, perm).unwrap();

    let report = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(validate_mod_output(tmp.path(), Some(fake_ffprobe.to_str().unwrap())))
        .unwrap();

    assert!(report.passed);
    assert_eq!(report.ogg_files_checked, 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd src-tauri && cargo test --lib validator_uses_custom_ffprobe_path -- --nocapture
```
Expected: FAIL — the custom binary path is not accepted by the current signature.

- [ ] **Step 3: Write minimal implementation**

In `src-tauri/src/validator.rs`:

```rust
pub async fn validate_mod_output(
    output_dir: &Path,
    ffprobe_path: Option<&str>,
) -> Result<ValidationReport> {
    // ... existing body, but pass ffprobe_path down to check_ogg_decodable
}

async fn check_ogg_decodable(path: &Path, ffprobe_path: Option<&str>) -> Result<bool> {
    let binary = ffprobe_path.unwrap_or("ffprobe");
    let output = tokio::process::Command::new(binary)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_streams",
            &path.to_string_lossy(),
        ])
        .output()
        .await
        .map_err(|e| Hoi4RadioError::Other {
            message: format!("ffprobe failed for {}: {}", path.display(), e),
        })?;

    Ok(output.status.success())
}
```

Update the single call site inside `validate_mod_output` from `check_ogg_decodable(&ogg_path).await` to `check_ogg_decodable(&ogg_path, ffprobe_path).await`.

In `src-tauri/src/commands.rs`, update `validate_project_mod`:

```rust
#[tauri::command]
pub async fn validate_project_mod(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<crate::validator::ValidationReport> {
    let (output_dir, ffprobe_path) = {
        let db = lock_db(&state)?;
        let settings = Settings::get(&db)?;
        match db.get_project(&project_id)? {
            Some(project) => (project.output_dir, settings.ffprobe_path),
            None => return Err(Hoi4RadioError::ProjectNotFound { id: project_id }),
        }
    };

    validate_mod_output(&output_dir, ffprobe_path.as_deref()).await
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd src-tauri && cargo test --lib validator_uses_custom_ffprobe_path -- --nocapture
```
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/validator.rs src-tauri/src/commands.rs
git commit -m "fix(validator): use configured ffprobe path instead of hard-coded binary"
```

---

### Task 3: Force 2-channel (stereo) Ogg Vorbis output

**Files:**
- Modify: `src-tauri/src/audio.rs`
- Test: `src-tauri/src/audio.rs` (new test in `mod tests`)

The design doc specifies 44.1 kHz and stereo/mono output. Current ffmpeg args only force the sample rate, leaving the channel count unchanged. Some source files may be multi-channel and fail to decode in HOI4.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn transcode_args_force_stereo() {
    let args = transcode_args("input.mp3", "output.ogg");
    assert!(args.windows(2).any(|pair| pair == ["-ac", "2"]));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd src-tauri && cargo test --lib transcode_args_force_stereo -- --nocapture
```
Expected: FAIL — `transcode_args` does not exist yet.

- [ ] **Step 3: Write minimal implementation**

In `src-tauri/src/audio.rs`, extract the argument list and add `-ac 2`:

```rust
fn transcode_args<'a>(input: &'a str, output: &'a str) -> Vec<&'a str> {
    vec![
        "-y",
        "-i", input,
        "-ar", "44100",
        "-ac", "2",
        "-c:a", "libvorbis",
        "-q:a", "4",
        output,
    ]
}
```

Then replace the inline `.args([...])` in `transcode_to_ogg` with:

```rust
let args = transcode_args(
    input.to_string_lossy().as_ref(),
    output.to_string_lossy().as_ref(),
);

let mut child = Command::new(binary)
    .args(&args)
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Hoi4RadioError::Transcoding {
            message: format!("{binary} not found. Please install ffmpeg or specify the path in settings."),
        },
        _ => Hoi4RadioError::Io {
            message: e.to_string(),
        },
    })?;
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd src-tauri && cargo test --lib transcode_args_force_stereo -- --nocapture
```
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/audio.rs
git commit -m "fix(audio): force 2-channel Ogg Vorbis output for HOI4 compatibility"
```

---

### Task 4: Auto-fill title and artist from ID3/metadata tags

**Files:**
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/audio.rs`
- Modify: `src-tauri/src/commands.rs`
- Test: `src-tauri/src/audio.rs` (new test in `mod tests`)

Currently the import title is taken from the file stem and artist is always `None`. ffprobe exposes `format.tags` which contains `title`/`artist` (and many variants).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn parse_ffprobe_extracts_id3_tags() {
    let json = r#"{
        "streams": [{"codec_type": "audio", "sample_rate": "44100", "channels": 2}],
        "format": {"duration": "123.45", "tags": {"title": "Test Title", "artist": "Test Artist"}}
    }"#;

    let meta = parse_ffprobe_output(json).unwrap();
    assert_eq!(meta.duration_secs, 123.45);
    assert_eq!(meta.sample_rate, 44100);
    assert_eq!(meta.channels, 2);
    assert_eq!(meta.title, Some("Test Title".to_string()));
    assert_eq!(meta.artist, Some("Test Artist".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cd src-tauri && cargo test --lib parse_ffprobe_extracts_id3_tags -- --nocapture
```
Expected: FAIL — `parse_ffprobe_output` and `AudioMetadata.title/artist` do not exist.

- [ ] **Step 3: Write minimal implementation**

In `src-tauri/src/models.rs`, extend `AudioMetadata`:

```rust
/// Metadata extracted from an audio file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u32,
    pub title: Option<String>,
    pub artist: Option<String>,
}
```

In `src-tauri/src/audio.rs`, update `FfprobeFormat`:

```rust
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    #[serde(default)]
    tags: HashMap<String, String>,
}
```

Add a parsing helper (also used by the existing `analyze_audio`):

```rust
pub(crate) fn parse_ffprobe_output(json: &str) -> Result<AudioMetadata> {
    let parsed: FfprobeOutput = serde_json::from_str(json)?;

    let audio_stream = parsed
        .streams
        .into_iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"))
        .ok_or_else(|| Hoi4RadioError::AudioAnalysis {
            message: "no audio stream found".to_string(),
        })?;

    let sample_rate = audio_stream
        .sample_rate
        .as_deref()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Hoi4RadioError::AudioAnalysis {
            message: "missing or invalid sample rate".to_string(),
        })?;

    let channels = audio_stream.channels.ok_or_else(|| Hoi4RadioError::AudioAnalysis {
        message: "missing channel count".to_string(),
    })?;

    let duration_secs = parsed
        .format
        .as_ref()
        .and_then(|f| f.duration.as_deref())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Hoi4RadioError::AudioAnalysis {
            message: "missing or invalid duration".to_string(),
        })?;

    let tags = parsed.format.map(|f| f.tags).unwrap_or_default();
    let get_tag = |keys: &[&str]| {
        keys.iter()
            .find_map(|k| tags.get(&k.to_ascii_lowercase()).cloned())
    };

    Ok(AudioMetadata {
        duration_secs,
        sample_rate,
        channels,
        title: get_tag(&["title", "name", "track"]),
        artist: get_tag(&["artist", "author", "performer"]),
    })
}
```

Then simplify `analyze_audio` to call `parse_ffprobe_output` on the JSON string:

```rust
pub async fn analyze_audio<P: AsRef<Path>>(
    path: P,
    ffprobe_path: Option<&str>,
) -> Result<AudioMetadata> {
    // ... existing Command setup, then:
    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)?;
    parse_ffprobe_output(&String::from_utf8_lossy(&output.stdout))
}
```

Actually, since `parse_ffprobe_output` takes `&str`, you can pass `std::str::from_utf8(&output.stdout)?`.

In `src-tauri/src/commands.rs`, update `process_import_file` so that after `analyze_audio` succeeds and before `start_processing`, title/artist are written to the pending record:

```rust
let metadata = match analyze_audio(&source_path, ffprobe_path).await { ... };

// Apply ID3 metadata if present.
{
    let db = match lock_db(state_ref) {
        Ok(db) => db,
        Err(e) => return mark_error(e.to_string()),
    };
    let repo = AudioRepository::new(&db);
    let mut update_req = UpdateAudioFileRequest {
        title: None,
        artist: None,
        volume: None,
        tags: None,
        notes: None,
    };
    if let Some(ref t) = metadata.title {
        if !t.trim().is_empty() {
            update_req.title = Some(t.clone());
        }
    }
    if metadata.artist.is_some() {
        update_req.artist = Some(metadata.artist.clone());
    }
    if update_req.title.is_some() || update_req.artist.is_some() {
        let _ = repo.update(&id, &update_req);
    }
}

let processing_audio = {
    let db = match lock_db(state_ref) { ... };
    ...
};
```

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cd src-tauri && cargo test --lib parse_ffprobe_extracts_id3_tags -- --nocapture
```
Expected: PASS.

Also run the full backend suite:
```bash
cd src-tauri && cargo test --lib
```
Expected: all existing tests still pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/audio.rs src-tauri/src/commands.rs
git commit -m "feat(audio): read title/artist from ID3 tags during import"
```

---

## Final Verification

After all tasks:

```bash
cd src-tauri && cargo clippy --lib --features dev-mcp-bridge -- -D warnings
cd .. && npm run build
```

Both should pass.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-15-hoi4-compatibility-and-quality-plan.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using `superpowers:executing-plans`, batch execution with checkpoints.

Which approach would you like?
