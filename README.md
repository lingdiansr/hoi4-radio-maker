# hoi4-radio-maker

A Tauri desktop application for creating Hearts of Iron IV radio mods.

## Tech Stack

- **App Framework**: [Tauri 2](https://tauri.app/)
- **Frontend Framework**: [Vue 3](https://vuejs.org/) with `<script setup>` SFCs
- **Frontend Language**: [TypeScript](https://www.typescriptlang.org/)
- **Build Tool**: [Vite](https://vitejs.dev/)
- **Package Manager**: [Bun](https://bun.sh/)
- **Backend Language**: [Rust](https://www.rust-lang.org/)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Bun](https://bun.sh/docs/installation)

For system-specific dependencies, see the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Setup

```bash
bun install
```

For Android development, initialize the Android project:

```bash
bun run tauri android init
```

## Development

### Desktop

```bash
bun run tauri dev
```

### Android

```bash
bun run tauri android dev
```

## Build

### Desktop

```bash
bun run tauri build
```

### Android

```bash
bun run tauri android build
```

## Project Structure

- `src/` — Vue frontend source code
- `src-tauri/` — Tauri Rust backend source code

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, workflow, and code style guidelines.

## Design Docs

- [Design Specification](docs/superpowers/specs/2026-06-12-hoi4radio-design.md)
- [Implementation Plan](docs/superpowers/plans/2026-06-12-hoi4radio-implementation-plan.md)
- [Implementation Roadmap](docs/superpowers/roadmap.md)

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
