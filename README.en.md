# HOI4 Radio Maker

[简体中文](README.md) | [**English**](README.en.md)

> A desktop tool for creating music/radio mods for _Hearts of Iron IV_.

Create and manage HOI4 radio mods from scratch: import audio and transcode it to game-compatible Ogg Vorbis, compose radio stations with playback chances and trigger conditions, generate a complete mod directory in one click, and validate the output.

![tech-badge](https://img.shields.io/badge/Tauri%202-Rust-5b5f66)
![tech-badge](https://img.shields.io/badge/Vue%203-TypeScript-42b883)
![license-badge](https://img.shields.io/badge/license-GPL--3.0-blue)

## Features

- **Project management**: create / edit / delete projects; defaults for directory, author, version, tags; supported game version auto-detected from the HOI4 install dir (`launcher-settings.json`)
- **Global audio library**: batch import mp3 / flac / wav / ogg / m4a / aac / wma; BLAKE3 content-hash dedup — re-imports just create references, no re-transcoding
- **Automatic transcoding**: converts to HOI4-compatible Ogg Vorbis on import (44.1 kHz, forced stereo, libvorbis quality 4); concurrent imports with per-file progress and cancellation
- **Metadata**: reads embedded title/artist tags (ID3 etc.); single and batch editing (title / artist / volume / tags / notes)
- **Station editor**: multiple stations per project; per-song playback weight (`factor`) and triggers (`tag`, `has_war`, `has_government`, `is_in_faction`); in-station ordering
- **One-click generation**: emits a complete mod directory — `descriptor.mod`, launcher `.mod`, `music/<station>.asset` + `.txt`, and Simplified Chinese localisation (UTF-8 BOM)
- **Output validation**: OGG decodability, `.asset`/`.txt` ID consistency, localisation key coverage, ID naming rules (configurable ffprobe path)
- **Diagnostics**: unified frontend/backend logging (stdout + app data dir + dev webview); open the log folder from Settings

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop framework | [Tauri 2](https://tauri.app/) |
| Frontend | Vue 3 (`<script setup>`) + TypeScript + [Vite](https://vitejs.dev/) |
| UI | [Vuetify 3](https://vuetifyjs.com/) (Material Design 3, custom dark theme) |
| State / routing | Pinia · Vue Router |
| Backend | Rust (edition 2021) |
| Database | SQLite (rusqlite) |
| Audio processing | ffmpeg / ffprobe |

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Bun](https://bun.sh/docs/installation)
- System `ffmpeg` / `ffprobe` (auto-detected on first launch; can be set manually in Settings)

For system-level dependencies (WebKitGTK etc.), see the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

## Quick Start

```bash
bun install          # install frontend deps (Bun; bun.lock is the lockfile)
bun run tauri dev    # desktop dev: Vite (port 1420) + Rust backend
```

## Development

```bash
bun run build        # type gate: vue-tsc --noEmit && vite build
cd src-tauri && cargo test          # all Rust tests
cd src-tauri && cargo test --lib    # unit tests only
cd src-tauri && cargo clippy -- -D warnings
RUST_LOG=debug bun run tauri dev    # debug-level backend logs
```

## Build

```bash
bun run tauri build                 # desktop bundles (Windows / Linux / macOS)
bun run tauri android dev | build   # Android targets
```

## Project Structure

```
src/           Vue frontend (views / components / stores / api)
src-tauri/     Rust backend (commands / db / generator / validator / audio …)
  └─ tests/    integration tests
docs/superpowers/  design specs, implementation plans, roadmap
references/radio-mod-template/    sample HOI4 radio mod for output reference
```

## Docs

- [Design specification](docs/superpowers/specs/2026-06-12-hoi4radio-design.md)
- [Implementation roadmap](docs/superpowers/roadmap.md)
- [Implementation plan](docs/superpowers/plans/2026-06-12-hoi4radio-implementation-plan.md)
- [Contributing](CONTRIBUTING.md)

## License

Licensed under the [GNU General Public License v3.0](LICENSE).
