# Repository Guidelines

## Project Overview

`hoi4-radio-maker` is a cross-platform Tauri 2 desktop app that helps Hearts of Iron IV mod authors build and manage music/radio mods. Users manage projects, import audio into a global library (transcoded to Ogg Vorbis), compose radio stations with per-entry chance/trigger configs, generate a complete HOI4 mod directory, and validate the output.

- **Frontend**: Vue 3 `<script setup>` SFC + TypeScript + Vite + Vuetify 3 (Material 3, custom `radioBureau` dark theme) + Pinia + Vue Router
- **Backend**: Rust (edition 2021, lib name `hoi4_radio_maker_lib`) via Tauri 2; SQLite via `rusqlite` (bundled); ffmpeg/ffprobe for audio; tokio for async pipelines
- **Identifier**: `xyz.ldsr.hoi4-radio-maker` · **License**: GPL-3.0

## Architecture & Data Flow

Layered call chain, one direction:

```
Vue SFC → Pinia store → src/api/client.ts (invokeCommand) → #[tauri::command] in commands.rs
        → lock_db() → Repository (AudioRepository / StationRepository) → Db (rusqlite)
```

- **32 Tauri commands** registered in `src-tauri/src/lib.rs` `invoke_handler` (project 5 / audio 9 / station 10 / generator+validator 2 / settings 3 / triggers 3). New commands MUST be registered there.
- **AppState** (`commands.rs`) holds `Mutex<Db>`, a `cancel_import` `AtomicBool`, and an `AsyncMutex<HashMap<String, watch::Sender<bool>>>` of active transcodes. Acquire the DB via the `lock_db(&state)` helper.
- **Import pipeline** (`import_audio_batch`): insert `Pending` records and emit initial events → Phase 1 BLAKE3 hash (`buffer_unordered(settings.import_concurrency)`, default 8) → Phase 2 dedup / analyze / transcode (`buffer_unordered(settings.import_concurrency / 2)`, default 4) via `process_hashed_file` (`analyze_audio` via ffprobe, ID3 title/artist applied to the record → `start_processing` → `transcode_to_ogg`) → emits `import:started` / `import:file` / `import:completed` / `import:cancelled` Tauri events keyed by `session_id`. Cancellation per file via `watch` channel; `transcode_to_ogg` selects on it and kills ffmpeg.
- **Mod generation** (`generate_project_mod`): `generator::generate_mod` clears/rebuilds `output_dir`, writes `descriptor.mod`, launcher `.mod` (in the parent dir, `path` field is the bare folder name), and writes each station's `.asset`/`.txt` plus its referenced OGGs into its own `music/<dir>/` folder. `<dir>` is the station's `subdir` when set, otherwise a slug of the station name (de-duplicated in station order). It also writes `localisation/simp_chinese/<project_id>_music_l_simp_chinese.yml`.
- **Validation** (`validate_project_mod`): `validator::validate_mod_output(output_dir, ffprobe_path)` recursively walks `music/` (including station subdirectories), parses `.asset`/`.txt` with regex, resolves each `file` relative to its `.asset` directory, and checks OGG decodability and ID naming; the configured ffprobe path from settings is passed through (falls back to `ffprobe`).
- **Settings**: single JSON blob under `settings` key in the `settings` table; `SettingsResponse` adds transient `detected_supported_version` + `ffmpeg_available`.
- **Errors**: every command returns `Result<T, Hoi4RadioError>`; the error serializes as `{ type, message }` (`serde(tag = "type", rename_all = "snake_case")`, 12 variants in `src-tauri/src/error.rs`).

## Key Directories

- `src/` — Vue frontend
  - `views/` — 6 pages: `WelcomeView` (`/`), `ProjectView` (`/project/:id`, tab shell), `StationEditorView` (stations), `AudioArchiveView` (global library), `ProjectSettingsView`, `SettingsView`
  - `components/` — `AppSidebar`, `ProjectList`, `AudioImporter`, `AudioPickerDialog`, `AudioEditDialog`, `BatchAudioEditDialog`, `PathField`
  - `stores/` — Pinia: `project`, `audio`, `station`, `settings`, `toast`
  - `api/client.ts` + `composables/useCommand.ts` — the only IPC surface
  - `utils/` — `sanitize.ts` (mirrors backend sanitizer), `logger.ts`, `errors.ts` (error → localized message)
  - `i18n/` — vue-i18n setup, locale detection, and `locales/{zh-CN,en}.ts` catalogues
- `src-tauri/src/` — Rust backend (16 files): `main.rs` (entry), `lib.rs`, `commands.rs`, `db.rs`, `models.rs`, `error.rs`, `audio.rs`, `audio_repo.rs`, `station.rs`, `generator.rs`, `validator.rs`, `scripts.rs` (game/mod trigger vocabulary + value-kind index), `settings.rs`, `ffmpeg_finder.rs`, `hoi4_version.rs`, `naming.rs`
- `src-tauri/tests/` — Rust integration tests (8 files)
- `src-tauri/capabilities/` + `capabilities-dev/` — prod vs dev (mcp-bridge) capability files
- `docs/superpowers/` — `roadmap.md`, `specs/` (dated design docs), `plans/` (dated TDD implementation plans). Plans' checkboxes are NOT maintained; trust code, not checkmarks.
- `references/radio-mod-template/` — sample HOI4 radio mod for output-format reference

## Development Commands

```bash
bun install                     # install deps (Bun; bun.lock is the lockfile)
bun run tauri dev               # desktop dev: vite (port 1420) + cargo. NO MCP bridge
bun run tauri:dev               # desktop dev WITH the dev-mcp-bridge plugin (WebSocket on 9223)
bun run tauri build             # release bundle
bun run build                   # type gate: vue-tsc --noEmit && vite build
cd src-tauri && cargo fmt --check          # CI gate - run before pushing, it is not optional
cd src-tauri && cargo test      # all Rust tests
cd src-tauri && cargo test --lib           # unit tests only
cd src-tauri && cargo test --test generator_test   # one integration target
cd src-tauri && cargo clippy --all-targets -- -D warnings
RUST_LOG=debug bun run tauri dev            # debug-level backend logs
bun run tauri android dev | build           # Android targets
```

**Run the same four gates CI runs before pushing**: `bun run build`, `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`. `cargo fmt --check` is easy to
forget because `cargo clippy`/`cargo test` say nothing about formatting, and the failure
only shows up as a red CI run.

## Code Conventions & Common Patterns

### Rust backend
- All fallible fns return `Result<T, Hoi4RadioError>` (alias `crate::error::Result`). Add variants to `error.rs` rather than reusing `Other` for domain errors.
- Log with `tracing::{info,debug,warn,error}!` — never `println!`. Logs flow to stdout, `<data_dir>/hoi4-radio-maker/logs/hoi4-radio-maker.log`, and (dev) webview via `tauri-plugin-log`.
- Persistence pattern: `Db` (in `db.rs`) owns the `rusqlite::Connection` and all CRUD; `AudioRepository`/`StationRepository` are thin wrappers over `&Db` that commands call. Add repo methods that forward to `Db`.
- Schema: 6 tables — `projects`, `audio_files`, `project_audio_files`, `stations` (has optional `subdir`), `station_entries`, `settings`. Migrations live in `db.rs` (`migrate` + legacy-schema detection), keyed by `PRAGMA user_version` (currently 3).
- Transcode contract: Ogg Vorbis, `-ar 44100`, `-ac 2` (forced stereo), libvorbis quality 4. Source formats: mp3/flac/wav/ogg/m4a/aac/wma.
- `ImportStatus`: `pending | processing | ready | error | cancelled`; only `ready` audio may enter projects/stations.
- ASCII-safe IDs (`audio_<uuid>` / `station_<uuid>`); `sanitize_folder_name` in `commands.rs` mirrors `src/utils/sanitize.ts`.

### Frontend
- `<script setup>` + Composition API, no `any`. Vue 3.5, Vuetify 3, Pinia setup-style stores: `defineStore(name, () => {...})`.
- Never call `invoke` directly: `invokeCommand<T>(cmd, args)` (`src/api/client.ts`) normalizes errors to `AppError { type, message }`; components use `useCommand().run(cmd, args, { successMsg?, silent? })` for toast feedback.
- Command names are `snake_case`. Arg-key convention is inconsistent — project commands take `{ id, req }`, audio/station take camelCase (`{ projectId, stationId, audioFileId, chance }`); `delete_project` is the odd one out with `{ id, delete_files }`.
- **All user-facing text goes through vue-i18n** (`src/i18n/`, `zh-CN` primary + `en`): template `$t('key')`, script `const { t } = useI18n()`. Never hardcode display strings. Add keys to both catalogues. Interpolate with `$t('key', { name })`; when a value must stay wrapped in markup, use `<i18n-t keypath="…">` + a named slot. Options arrays that render labels must be `computed`, not module-level `const`.
- Backend errors are localized in the frontend: `errorMessage(err)` (`src/utils/errors.ts`) maps the `Hoi4RadioError` wire tag to `errors.<type>` using the variant's own fields, falling back to the backend `message`. Use it instead of reading `err.message` directly.
- Backend fields are snake_case in TS interfaces; local refs/actions camelCase; action verbs: `loadX / createX / updateX / deleteX / addX / removeX / reorderX`.
- Import progress is pushed via Tauri events; `audio.ts` store has `ensureListening()`/`stopListening()` and upserts + removes `cancelled` entries.
- Dialogs share classes `.dialog-card` / `.dialog-accent`. Vuetify ships two themes (`radioBureau` dark, `radioBureauLight`); `src/plugins/theme.ts` maps `settings.theme` (`dark` / `light` / `system`) onto the running instance, caches the resolved id for startup, and follows OS changes under `system`.
- `main.ts` boot order: forward `console.*` to plugin-log → createApp + Pinia + router + vuetify → mount.

### Trigger vocabulary (`scripts.rs`)
- The project does **not** hardcode trigger lists and does **not** depend on hoi4skill. `build_vocabulary` parses the game the user already installed: `documentation/triggers_documentation.md` (only `## <name>` blocks carrying a `* Supported Scopes:` line), `common/scripted_triggers/*.txt`, `common/country_tags/*.txt`, `common/ideologies/*.txt` (first-level children of the `ideologies` block, excluding nested `types`).
- Sources are per project: `load_vanilla_triggers` (default on) plus `trigger_mod_dirs` (Workshop mods, default off), resolved to roots by `script_roots` alone — do not reassemble vanilla-vs-mod roots at a call site.
- **Strip trailing comments before any structural check** (`strip_comment`, quote-aware). Real mods write `totalitarian_socialist = { #社` and use CRLF; an `ends_with('{')` test silently dropped ideologies. There are regression tests for this.
- Value kinds are **measured from usage**, never from the docs: `Supported Targets` does not predict them (`stockpile_ratio` is documented `none` yet takes a number). `ValueKindIndex` classifies every assignment in **one pass** and caches per source set on `AppState`; never rescan per name — a per-name scan cost 1.98 s on a 120 MB corpus and blocked the IPC thread.
- `lookup` answers are advisory: an unknown or never-assigned trigger yields `None` and the UI falls back to a free-text control.

## Important Files

| File | Why |
|---|---|
| `src-tauri/src/lib.rs` | Module list, `run()` init (plugins, DB path, AppState), all 32 command registrations, dev-capability injection |
| `src-tauri/src/commands.rs` | `AppState` + every Tauri command + the whole import pipeline |
| `src-tauri/src/db.rs` | Schema, migrations, all CRUD (largest backend file) |
| `src-tauri/src/models.rs` | Domain models + all request structs (contract between layers and frontend) |
| `src-tauri/src/error.rs` | `Hoi4RadioError` variants; error wire format |
| `src/api/client.ts` | `invokeCommand` + `AppError` normalization |
| `src/composables/useCommand.ts` | Command invocation + toast wrapper |
| `src/stores/audio.ts` | Global/project audio lists, import progress, event subscription |
| `src/plugins/vuetify.ts` | `radioBureau` theme tokens + component defaults |
| `docs/superpowers/roadmap.md` | Phase status + prioritized backlog (section 9) |

## Runtime/Tooling Preferences

- **Bun** is the package manager (`bun.lock`; `bunfig.toml` pins the npmmirror registry). Do not use npm — the stray root `package-lock.json` is dead weight. `bun add` / `bun add -d` for frontend deps; `cargo add` inside `src-tauri/` for Rust.
- Vite dev server: port 1420 (strict), HMR ws on 1421 when `TAURI_DEV_HOST` is set; alias `@` → `src/`.
- Cargo features: `dev-mcp-bridge` (adds the MCP debugging plugin, WebSocket on port 9223) and `e2e-testing` (optional `tauri-plugin-playwright`). Release builds exclude both. **Neither is a default feature**: the bridge only loads via the `tauri:dev` script (`tauri dev --features dev-mcp-bridge`), so plain `bun run tauri dev` runs an app with no bridge — attaching to it will never work.
- Capabilities: prod set in `capabilities/default.json` (`core:default`, `dialog:default`, `opener:default`, `opener:allow-open-path`, `log:default`); dev-mcp-bridge capability is injected at runtime from `capabilities-dev/`. No fs/shell/http permissions — don't add them casually.
- Requires system `ffmpeg`/`ffprobe`; auto-detected on first `get_settings` and stored in settings.
- `.vscode/extensions.json`: Vue.volar, tauri-apps.tauri-vscode, rust-lang.rust-analyzer.

## Testing & QA

- **Unit tests** (in-module `#[cfg(test)]`, 47 across 9 modules): `scripts.rs` (13: trigger-doc parsing, scripted triggers, tags, ideologies, CRLF + trailing-comment regressions, value classification, index build), `hoi4_version.rs` (10, version extraction), `commands.rs` (9: sanitize/transcode concurrency/subdir normalization/remove-files, trigger-index cache keying), `naming.rs` (4, slugify/unique-name suffixing), `generator.rs` (3, escape_hoi4, localisation UTF-8 BOM, value quoting), `db.rs` (3, audio update/batch), `station.rs` (2, find_by_name/delete), `audio.rs` (2, transcode args force stereo, ffprobe ID3 parsing), `validator.rs` (1, custom ffprobe path). All synchronous `#[test]`.
- **Integration tests** (`src-tauri/tests/`, 16 tests in 8 files): lifecycle tests (`db_test` x3, incl. the v2→v3 migration check; `station_test` x1, incl. subdir round-trip; `audio_repo_test` x1; `models_test` x1), output tests (`generator_test` x4, incl. custom and name-derived folders), validation (`validator_test` x4, all `#[tokio::test]`), import (`import_batch_test` x1), audio (`audio_test` x1). Pattern: `tempfile::tempdir()` + `Db::open(tmp/...app.db)`; dummy OGG/WAV fixtures are hand-written bytes.
- No `[dev-dependencies]` — `tempfile`/`tokio`/`chrono` are regular deps.
- Real ffmpeg/ffprobe required for: `import_batch_test` (has skip guard) and `validator_test::test_validate_complete_mod_reports_ogg_decode_error` (implicit, no guard).
- **Remaining coverage notes**: transcode and ID3 tests assert argument lists / JSON parsing, not real ffmpeg round-trips; `analyze_audio`'s `Some(ffprobe_path)` branch is untested (the validator's is).
- Frontend has no automated tests; `bun run build` (vue-tsc) is the only type gate. E2E debugging is possible via the dev-mcp-bridge feature + Playwright tooling (see `docs/superpowers/specs/2026-06-15-logging-design.md`).
- **CI** (`.github/workflows/ci.yml`) runs on every push to `main` and every PR, in two jobs: a frontend job (`bun install --frozen-lockfile`, then `bun run build`) and a Rust job (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, with the `rustfmt`/`clippy` components installed). Green local `cargo test` + `clippy` does **not** imply a green CI run — neither checks formatting, so run `cargo fmt --check` too. Releases are tag-driven (`v*`, `.github/workflows/release.yml`).
- Commits follow Conventional Commits (`feat:`/`fix:`/`refactor:`/`chore:`); the compat plan at `docs/superpowers/plans/2026-06-15-hoi4-compatibility-and-quality-plan.md` defines the TDD (failing test → impl → pass) workflow used for backend fixes.
