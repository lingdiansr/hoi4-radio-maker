# Agent Notes: hoi4-radio-maker

## Project Overview

`hoi4-radio-maker` is a desktop application built with **Tauri 2**, **Vue 3**, **TypeScript**, and **Vite**.

- **Identifier**: `xyz.ldsr.hoi4-radio-maker`
- **Package Manager**: [Bun](https://bun.sh/)
- **Frontend**: Vue 3 + TypeScript + Vite (`src/`)
- **Backend**: Rust via Tauri 2 (`src-tauri/`)

## Common Commands

```bash
# Install dependencies
bun install

# Desktop development
bun run tauri dev

# Desktop build
bun run tauri build

# Android development (after `bun run tauri android init`)
bun run tauri android dev

# Android build
bun run tauri android build
```

## Key Directories

- `src/` — Vue frontend source code
- `src-tauri/` — Tauri Rust backend, Cargo project, and application assets
- `src-tauri/src/` — Rust source code
- `src-tauri/capabilities/` — Tauri capability definitions
- `src-tauri/icons/` — Application icons
- `public/` — Static assets served by Vite

## Error Handling & Logging

- Rust errors are centralized in `src-tauri/src/error.rs` as `Hoi4RadioError`.
- All Tauri commands return `Result<T, Hoi4RadioError>` so structured errors reach the frontend.
- Use `tracing::info!` / `warn!` / `error!` for logging; logs go to stdout and to `<data_dir>/hoi4-radio-maker/logs/hoi4-radio-maker.log`.
- Override log level with the `RUST_LOG` environment variable, e.g. `RUST_LOG=debug bun run tauri dev`.

## Build & Lint Notes

- TypeScript type checking is run as part of the build: `vue-tsc --noEmit && vite build`.
- Vite dev server is configured for Tauri on port `1420` with HMR on port `1421` when `TAURI_DEV_HOST` is set.
- Rust code lives under `src-tauri/`; build with `cargo` from within that directory if needed.

## Adding Dependencies

Use Bun for frontend dependencies:

```bash
bun add <package>
bun add -d <dev-package>
```

Use Cargo for Rust dependencies inside `src-tauri/`:

```bash
cd src-tauri && cargo add <crate>
```
