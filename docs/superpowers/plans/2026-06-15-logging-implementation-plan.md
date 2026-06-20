# 统一日志与 Playwright 调试桥实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用 Tauri 官方 `tauri-plugin-log` 替换现有自定义日志，实现前后端统一日志；通过 Cargo feature `e2e-testing` 引入 `tauri-plugin-playwright`，仅在调试时启用。

**Architecture：** 后端删除 `tracing-subscriber`/`tracing-appender` 自定义初始化，改用 `tauri-plugin-log` 统一输出到 stdout、日志文件和 Webview console；前端 `main.ts` 通过 `takeoverConsole()` 与 `attachConsole()` 实现前后端日志双向流通；Playwright 插件作为 optional dependency，通过 feature flag 控制，不进入 Release 产物。

**Tech Stack：** Rust (tauri-plugin-log, tracing/log feature, tauri-plugin-playwright), TypeScript/Vue (@tauri-apps/plugin-log, @srsholmes/tauri-playwright, @playwright/test)

---

## 文件变更清单

| 文件 | 操作 | 职责 |
|---|---|---|
| `src-tauri/Cargo.toml` | 修改 | 添加 `tauri-plugin-log`、`log`、optional `tauri-plugin-playwright`；定义 `e2e-testing` feature；移除 `tracing-subscriber`、`tracing-appender` |
| `src-tauri/src/lib.rs` | 修改 | 删除 `init_logging()`，注册 `tauri-plugin-log` 和条件化 Playwright 插件 |
| `src-tauri/tauri.conf.json` | 修改 | 增加 `"withGlobalTauri": true` |
| `src-tauri/capabilities/default.json` | 修改 | 添加 `"log:default"` |
| `src-tauri/capabilities/debug.json` | 创建 | 添加 `"playwright:default"`（debug 专用） |
| `package.json` | 修改 | 添加 `@tauri-apps/plugin-log`、 `@srsholmes/tauri-playwright`、`@playwright/test` |
| `src/main.ts` | 修改 | 初始化日志桥接：`takeoverConsole()` + `attachConsole()` |
| `src/utils/logger.ts` | 创建 | 前端日志封装 |
| `src/views/SettingsView.vue` | 修改 | 新增「日志与诊断」卡片 |

---

## Task 1: 调整后端依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

**目标：** 引入 `tauri-plugin-log` 和 `log`，将 `tauri-plugin-playwright` 设为 optional dependency 并通过 feature 控制；移除不再需要的 `tracing-subscriber` 和 `tracing-appender`。

- [ ] **Step 1: 修改 `src-tauri/Cargo.toml`**

```toml
[package]
name = "hoi4-radio-maker"
version = "0.1.0"
description = "A Tauri App"
authors = ["you"]
edition = "2021"

[lib]
name = "hoi4_radio_maker_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-opener = "2"
tauri-plugin-log = "2"
log = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2.0.18"
tracing = { version = "0.1.44", features = ["log"] }
chrono = { version = "0.4.45", features = ["serde"] }
uuid = { version = "1.23.3", features = ["v4"] }
rusqlite = { version = "0.40.1", features = ["bundled", "chrono", "serde_json"] }
tempfile = "3.27.0"
dirs = "6.0.0"
tokio = { version = "1.52.3", features = ["macros", "process", "rt-multi-thread", "fs", "io-util"] }
regex = "1.12.4"
blake3 = "1.8.2"
futures = "0.3.31"

# Playwright 调试桥：仅在启用 e2e-testing feature 时编译
tauri-plugin-playwright = { version = "0.1", optional = true }

[features]
e2e-testing = ["dep:tauri-plugin-playwright"]
```

- [ ] **Step 2: 验证 Cargo.toml 格式**

Run: `cd src-tauri && cargo check --no-default-features 2>&1 | head -20`
Expected: 无 Cargo.toml 解析错误，可能提示缺少 `init_logging` 等（后续任务修复）。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore(deps): add tauri-plugin-log and optional playwright plugin"
```

---

## Task 2: 改造后端初始化

**Files:**
- Modify: `src-tauri/src/lib.rs`

**目标：** 删除自定义 `init_logging()`，在 `run()` 中注册 `tauri-plugin-log`，并条件化注册 Playwright 插件。

- [ ] **Step 1: 修改 `src-tauri/src/lib.rs`**

```rust
pub mod audio;
pub mod audio_repo;
pub mod commands;
pub mod db;
pub mod error;
pub mod ffmpeg_finder;
pub mod generator;
pub mod models;
pub mod settings;
pub mod station;
pub mod validator;

use crate::db::Db;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri_plugin_log::{Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("hoi4_radio_maker_lib::commands", log::LevelFilter::Debug)
                .level_for("hoi4_radio_maker_lib::audio", log::LevelFilter::Debug)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some("app".into()) }),
                    #[cfg(debug_assertions)]
                    Target::new(TargetKind::Webview),
                ])
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(feature = "e2e-testing")]
    {
        builder = builder.plugin(tauri_plugin_playwright::init());
    }

    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hoi4-radio-maker");
    std::fs::create_dir_all(&app_dir).ok();
    let db_path = app_dir.join("app.db");
    let db = Db::open(&db_path).expect("failed to open database");

    builder
        .manage(commands::AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::update_project,
            commands::delete_project,
            commands::list_audio_files,
            commands::list_all_audio_files,
            commands::add_audio_to_project,
            commands::remove_audio_from_project,
            commands::delete_audio_file,
            commands::import_audio_batch,
            commands::list_stations,
            commands::create_station,
            commands::add_station_entry,
            commands::remove_station_entry,
            commands::generate_project_mod,
            commands::validate_project_mod,
            commands::get_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -30`
Expected: `Finished dev` 无错误。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(logging): replace custom tracing subscriber with tauri-plugin-log; add conditional playwright plugin"
```

---

## Task 3: 配置 Tauri app 与 Capability

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/capabilities/default.json`
- Create: `src-tauri/capabilities/debug.json`

**目标：** 开启 `withGlobalTauri` 供 Playwright 使用；为日志插件和 Playwright 插件配置权限。

- [ ] **Step 1: 修改 `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "hoi4-radio-maker",
  "version": "0.1.0",
  "identifier": "xyz.ldsr.hoi4-radio-maker",
  "build": {
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "hoi4-radio-maker",
        "width": 800,
        "height": 600
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 2: 修改 `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "opener:default",
    "log:default"
  ]
}
```

- [ ] **Step 3: 创建 `src-tauri/capabilities/debug.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "debug",
  "description": "Debug-only capabilities for Playwright bridge",
  "windows": ["main"],
  "permissions": ["playwright:default"]
}
```

- [ ] **Step 4: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: `Finished dev` 无错误。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/capabilities/default.json src-tauri/capabilities/debug.json
git commit -m "feat(config): enable withGlobalTauri and add log/playwright capabilities"
```

---

## Task 4: 安装前端依赖

**Files:**
- Modify: `package.json`

**目标：** 安装 `@tauri-apps/plugin-log`、 `@srsholmes/tauri-playwright`、`@playwright/test`。

- [ ] **Step 1: 修改 `package.json`**

```json
{
  "name": "hoi4-radio-maker",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@fontsource/jetbrains-mono": "^5.2.8",
    "@fontsource/oranienbaum": "^5.2.8",
    "@fontsource/source-serif-4": "^5.2.9",
    "@mdi/font": "^7.4.47",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-log": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "pinia": "^3.0.4",
    "vue": "^3.5.13",
    "vue-router": "^5.1.0",
    "vuetify": "^4.1.2"
  },
  "devDependencies": {
    "@playwright/test": "^1.50.0",
    "@srsholmes/tauri-playwright": "^0.1.0",
    "@tauri-apps/cli": "^2",
    "@vitejs/plugin-vue": "^5.2.1",
    "typescript": "~5.6.2",
    "vite": "^6.0.3",
    "vue-tsc": "^2.1.10"
  }
}
```

- [ ] **Step 2: 安装依赖**

Run: `npm install --legacy-peer-deps`
Expected: 依赖安装成功，无 peer dependency 冲突。

- [ ] **Step 3: Commit**

```bash
git add package.json package-lock.json
git commit -m "chore(deps): add frontend log and playwright packages"
```

---

## Task 5: 前端日志桥接与封装

**Files:**
- Modify: `src/main.ts`
- Create: `src/utils/logger.ts`

**目标：** 在应用启动时桥接前后端日志；提供前端统一日志工具。

- [ ] **Step 1: 修改 `src/main.ts`**

```ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { attachConsole, takeoverConsole } from '@tauri-apps/plugin-log'
import '@mdi/font/css/materialdesignicons.css'
import vuetify from '@/plugins/vuetify'
import App from './App.vue'
import router from './router'

async function bootstrap() {
  await takeoverConsole()
  await attachConsole()

  const app = createApp(App)
  app.use(createPinia())
  app.use(router)
  app.use(vuetify)
  app.mount('#app')
}

bootstrap()
```

- [ ] **Step 2: 创建 `src/utils/logger.ts`**

```ts
import { trace, debug, info, warn, error } from '@tauri-apps/plugin-log'

export const logger = {
  trace: (msg: string) => trace(msg),
  debug: (msg: string) => debug(msg),
  info: (msg: string) => info(msg),
  warn: (msg: string) => warn(msg),
  error: (msg: string) => error(msg),
}
```

- [ ] **Step 3: 前端类型检查**

Run: `npm run build`
Expected: `vue-tsc --noEmit` 和 `vite build` 均通过。

- [ ] **Step 4: Commit**

```bash
git add src/main.ts src/utils/logger.ts
git commit -m "feat(logging): bridge frontend console to tauri-plugin-log and add logger util"
```

---

## Task 6: Settings 页面增加日志与诊断卡片

**Files:**
- Modify: `src/views/SettingsView.vue`

**目标：** 在 Settings 页面提供日志文件夹入口和日志路径复制功能。

- [ ] **Step 1: 读取当前 SettingsView.vue 结构**

Run: `cat src/views/SettingsView.vue | head -50`
Expected: 了解现有模板结构，找到插入「日志与诊断」卡片的位置（通常在 `<v-container>` 末尾或 settings 表单之后）。

- [ ] **Step 2: 在 `src/views/SettingsView.vue` 的 `<script setup>` 顶部增加导入**

```ts
import { logger } from '@/utils/logger'
import { open } from '@tauri-apps/plugin-opener'
import { appLogDir } from '@tauri-apps/api/path'
```

- [ ] **Step 3: 在 `<script setup>` 中增加日志相关函数**

```ts
async function openLogFolder() {
  try {
    const dir = await appLogDir()
    await open(dir)
    logger.info(`Opened log folder: ${dir}`)
  } catch (err) {
    logger.error(`Failed to open log folder: ${err}`)
  }
}

async function copyLogPath() {
  try {
    const dir = await appLogDir()
    await navigator.clipboard.writeText(dir)
    logger.info(`Copied log path: ${dir}`)
  } catch (err) {
    logger.error(`Failed to copy log path: ${err}`)
  }
}
```

- [ ] **Step 4: 在模板中插入「日志与诊断」卡片**

在 Settings 页面合适位置（如其他 `v-card` 之后）插入：

```vue
<v-card class="settings-card" variant="flat" rounded="xl">
  <v-card-title class="text-h6">日志与诊断</v-card-title>
  <v-card-text>
    <div class="text-body text-secondary mb-4">
      日志文件保存在应用日志目录中，遇到问题时可用于排查。
    </div>
    <div class="d-flex gap-3 flex-wrap">
      <v-btn color="primary" prepend-icon="mdi-folder-open-outline" @click="openLogFolder">
        打开日志文件夹
      </v-btn>
      <v-btn variant="outlined" prepend-icon="mdi-content-copy" @click="copyLogPath">
        复制日志路径
      </v-btn>
    </div>
  </v-card-text>
</v-card>
```

- [ ] **Step 5: 前端构建检查**

Run: `npm run build`
Expected: 构建通过。

- [ ] **Step 6: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "feat(ui): add logging diagnostics card in settings"
```

---

## Task 7: 验证后端测试与构建

**Files:**
- N/A（仅验证）

**目标：** 确保 Rust 测试和检查在改造后仍然通过。

- [ ] **Step 1: 运行 Rust 测试**

Run: `cd src-tauri && cargo test 2>&1 | tail -30`
Expected: 所有测试通过（`test result: ok`）。

- [ ] **Step 2: 检查 Release 构建（验证 Playwright 不进入 Release）**

Run: `cd src-tauri && cargo check --release 2>&1 | tail -10`
Expected: 编译通过，且不包含 `tauri-plugin-playwright` 相关编译输出。

- [ ] **Step 3: 检查带 feature 的 Debug 构建（验证 Playwright 可启用）**

Run: `cd src-tauri && cargo check --features e2e-testing 2>&1 | tail -10`
Expected: 编译通过。

- [ ] **Step 4: Commit（如有 Cargo.lock 更新）**

```bash
git add src-tauri/Cargo.lock
git commit -m "chore(lock): update Cargo.lock after logging deps" || echo "no changes to commit"
```

---

## Task 8: 端到端冒烟验证

**Files:**
- N/A（仅验证）

**目标：** 启动 `tauri dev`，验证 Webview console 能看到 Rust 日志，前端 console 能在日志文件中体现。

- [ ] **Step 1: 启动开发服务器**

Run: `npm run tauri dev`
Expected: 应用窗口出现，无启动错误。

- [ ] **Step 2: 打开 Webview DevTools**

在应用窗口内右键 → Inspect（若不可用，检查 `tauri` 依赖是否启用 `devtools` feature）。

- [ ] **Step 3: 验证 Rust 日志出现在 Console**

在 DevTools Console 中应能看到类似：

```
[2026-06-15][INFO][hoi4_radio_maker_lib::commands] ...
```

如果没有，检查 `tauri-plugin-log` 的 `Webview` target 是否仅在 `debug_assertions` 下启用，并确认当前是 dev 构建。

- [ ] **Step 4: 验证前端 console 进入日志文件**

在 DevTools Console 中执行：

```js
console.error('test frontend error')
```

然后打开日志文件夹（Settings → 打开日志文件夹），查看最新日志文件，应包含 `test frontend error`。

- [ ] **Step 5: 验证 Playwright 桥接（可选）**

Run:

```bash
cargo tauri dev --features e2e-testing
```

在另一个终端：

```bash
npx playwright install chromium
npx playwright test --project=tauri
```

Expected: Playwright 能连接到 Tauri webview（具体测试用例可在后续单独添加）。

---

## Self-Review

### Spec Coverage

| Spec 要求 | 对应 Task |
|---|---|
| 用 `tauri-plugin-log` 替换自定义日志 | Task 1, Task 2 |
| `tracing` 事件流入 `log` | Task 1（tracing/log feature） |
| 前端 `console` 转发到后端 | Task 5（takeoverConsole） |
| Webview 看到 Rust 日志 | Task 2（Webview target） |
| Settings 增加日志入口 | Task 6 |
| Playwright 调试桥 | Task 1, Task 3, Task 4, Task 8 |
| Release 不启用 Playwright | Task 1（optional dep + feature）, Task 7 |
| `withGlobalTauri: true` | Task 3 |

### Placeholder Scan

- 无 TBD/TODO。
- 所有代码片段包含完整文件路径和可运行命令。
- 所有函数名、导入路径与 spec 一致。

### Type Consistency

- `tauri-plugin-playwright` 版本使用 `0.1`（与调研一致）。
- npm 包使用 `@srsholmes/tauri-playwright` 与 `@playwright/test`。
- `appLogDir()` 来自 `@tauri-apps/api/path`，与现有 `@tauri-apps/api` 依赖一致。

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-15-logging-implementation-plan.md`.**

Two execution options:

**1. Subagent-Driven (recommended)** - Dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach would you like?
