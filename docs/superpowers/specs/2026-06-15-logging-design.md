# HOI4 Radio Maker 统一日志与调试桥设计

> 设计日期：2026-06-15  
> 状态：已确认，待实现  
> 关联问题：音频导入后未在全局音频库显示，需要更完整的前后端日志来定位问题。

---

## 1. 目标

1. 建立前后端统一的日志通道：前端 `console`、后端 `tracing` 最终写入同一组日志文件。
2. 开发阶段可在 Webview DevTools 同时看到 Rust 日志与前端日志。
3. 提供用户可访问的日志入口：日志级别显示、打开日志文件夹、复制日志路径。
4. 引入 `tauri-plugin-playwright` + `@srsholmes/tauri-playwright`，使 AI/开发者能通过 Playwright 控制真实 Tauri webview 进行调试、截图、读取 console log。
5. **安全约束**：Playwright 调试桥通过 Cargo feature `e2e-testing` 控制，Release 构建默认不启用，完全剔除插件代码。

---

## 2. 技术选型

| 组件 | 选型 | 说明 |
|---|---|---|
| 统一日志插件 | `tauri-plugin-log` + `@tauri-apps/plugin-log` | Tauri 官方日志插件，支持 Stdout、LogDir、Webview 多 target，文件自动轮转 |
| 后端日志 crate | `tracing`（启用 `log` feature）+ `log` | 保留现有 `tracing::info!` 等宏，事件自动流入 `log` crate，被插件捕获 |
| 调试桥 | `tauri-plugin-playwright` + `@srsholmes/tauri-playwright` | 控制真实原生 webview，Playwright 兼容 API |
| 条件编译 / feature flag | `#[cfg(feature = "e2e-testing")]` | Playwright 插件作为 optional dep，通过 feature 开关；Release 默认不启用 |

---

## 3. 架构

```text
┌─────────────────────────────────────────────────────────────┐
│                        Frontend (Vue/TS)                     │
│  console.log/error ──► takeoverConsole() ──► @tauri-apps/    │
│  plugin-log ──► IPC invoke("plugin:log|log")                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 tauri-plugin-log (Rust)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ TargetKind  │  │ TargetKind  │  │ TargetKind::Webview │  │
│  │  ::Stdout   │  │  ::LogDir   │  │   (dev only)        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              ▲
                              │
┌─────────────────────────────────────────────────────────────┐
│                Backend Rust (commands/...)                   │
│  tracing::info! / tracing::error!（通过 log feature 流入）    │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. 后端改造

### 4.1 依赖变更

`src-tauri/Cargo.toml`：

```toml
[dependencies]
tauri-plugin-log = "2"
log = "0.4"
# tracing 启用 log feature，让现有 tracing 事件自动被 tauri-plugin-log 捕获
tracing = { version = "0.1", features = ["log"] }
# tracing-subscriber / tracing-appender 将被移除，由插件统一处理输出
# Playwright 插件作为 optional dep，通过 e2e-testing feature 控制
tauri-plugin-playwright = { version = "0.1", optional = true }

[features]
e2e-testing = ["dep:tauri-plugin-playwright"]
```

> 注：`tracing` 保留，`tracing-subscriber` 与 `tracing-appender` 移除。

### 4.2 初始化改造

`src-tauri/src/lib.rs`：

1. 删除 `init_logging()` 函数。
2. 在 `run()` 中注册 `tauri-plugin-log`：

```rust
use tauri_plugin_log::{Target, TargetKind};

pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
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
            .build()
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
        .invoke_handler(tauri::generate_handler![...])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4.3 Tauri 配置

`src-tauri/tauri.conf.json` 增加全局 Tauri 对象，供 Playwright 桥接使用：

```json
{
  "app": {
    "withGlobalTauri": true
  }
}
```

### 4.4 Capability 权限

`src-tauri/capabilities/default.json`：

```json
{
  "permissions": [
    "core:default",
    "dialog:default",
    "opener:default",
    "opener:allow-open-path",
    "log:default"
  ]
}
```

`tauri-plugin-playwright` 主要开后端 socket 供 Playwright 连接，不依赖额外 capability。

### 4.5 现有 tracing 调用

保留所有 `tracing::info!` / `tracing::debug!` / `tracing::error!` 调用不变。`tracing` 的 `log` feature 会自动把这些事件转成 `log` 记录，被 `tauri-plugin-log` 捕获。

---

## 5. 前端改造

### 5.1 依赖变更

`package.json`：

```json
{
  "dependencies": {
    "@tauri-apps/plugin-log": "^2"
  },
  "devDependencies": {
    "@srsholmes/tauri-playwright": "^0.4",
    "@playwright/test": "^1.50"
  }
}
```

### 5.2 日志桥接

`src/main.ts`：

```ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { attachConsole, debug, error, info, trace, warn } from '@tauri-apps/plugin-log'
import '@mdi/font/css/materialdesignicons.css'
import vuetify from '@/plugins/vuetify'
import App from './App.vue'
import router from './router'

function forwardConsole(
  fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
  logger: (message: string) => Promise<void>
) {
  const original = console[fnName]
  console[fnName] = (message: unknown) => {
    original(message)
    logger(String(message))
  }
}

async function bootstrap() {
  forwardConsole('log', trace)
  forwardConsole('debug', debug)
  forwardConsole('info', info)
  forwardConsole('warn', warn)
  forwardConsole('error', error)

  await attachConsole()

  const app = createApp(App)
  app.use(createPinia())
  app.use(router)
  app.use(vuetify)
  app.mount('#app')
}

bootstrap()
```

### 5.3 日志封装

新增 `src/utils/logger.ts`：

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

关键用户操作后续逐步接入 `logger.info`（如导入开始/完成、生成 Mod、删除项目等）。

### 5.4 Settings UI 扩展

在 `src/views/SettingsView.vue` 新增「日志与诊断」卡片：

- 显示当前默认日志级别（`Info`，开发时可在代码或环境变量调整）。
- 「打开日志文件夹」按钮：调用 `openPath` 打开 Tauri 日志目录。
- 「复制日志路径」按钮：把日志目录路径写入剪贴板。
- （可选）显示最近一条错误日志摘要。

---

## 6. Playwright 调试桥

### 6.1 启用条件

- 通过 Cargo feature `e2e-testing` 控制：
  - 本地开发/调试：`cargo tauri dev --features e2e-testing`
  - 常规开发：`cargo tauri dev` 不带 feature 时不启用
  - Release 构建：`tauri build` 默认不带 `e2e-testing`，插件代码完全剔除

### 6.2 使用方式

开发时启动带 feature 的 dev server：

```bash
# 终端 1
cargo tauri dev --features e2e-testing

# 终端 2
npx playwright test --project=tauri
```

测试示例：

```ts
import { createTauriTest } from '@srsholmes/tauri-playwright'

export const { test, expect } = createTauriTest({
  devUrl: 'http://localhost:1420',
})

test('import audio shows in archive', async ({ tauriPage }) => {
  await tauriPage.goto('/audio')
  // 读取 console log、点击按钮、验证列表等
})
```

> 需要先在 `tauri.conf.json` 设置 `"withGlobalTauri": true`。
>
> 具体 API 以 `tauri-plugin-playwright` 与 `@srsholmes/tauri-playwright` 官方文档为准。

---

## 7. 文件变更清单

- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src-tauri/capabilities/default.json`（增加 `log:default`、`opener:allow-open-path`）
- `src-tauri/tauri.conf.json`（增加 `withGlobalTauri: true`）
- `package.json`
- `package-lock.json` / `bun.lock`（由包管理器自动更新）
- `src/main.ts`
- `src/utils/logger.ts`（新增）
- `src/views/SettingsView.vue`
- 后续逐步在 `src/stores/*.ts` 关键操作中接入 `logger.info`

---

## 8. 风险与注意事项

1. **console 转发异步化**
   - 通过 `forwardConsole()` 将 `console.log/error` 等转发到 `@tauri-apps/plugin-log` 后，`console.log` 变成异步 IPC。日常 UI 日志无影响；极端高频日志（如循环内大量打印）可能产生性能开销或顺序偏差。
2. **tracing → log 桥接**
   - 启用 `tracing` 的 `log` feature 后，所有 tracing 事件会流入 `log`。若未来同时启用 `tracing-subscriber` 文件层，可能导致日志重复写入。本方案移除 tracing-subscriber，完全由 `tauri-plugin-log` 处理输出。
3. **Playwright 安全**
   - 该插件作为 optional dependency，仅当启用 Cargo feature `e2e-testing` 时注册；Release 构建（`tauri build`）默认不带该 feature，插件代码完全剔除。
4. **日志文件位置**
   - `tauri-plugin-log` 的 `LogDir` target 使用 Tauri 推荐的系统日志目录（如 Linux `~/.config/hoi4-radio-maker/logs/` 或 `~/.local/share/` 下，依 Tauri 版本而定）。用户可通过「打开日志文件夹」按钮直接定位。

---

## 9. 验收标准

- [ ] `tauri dev` 启动后，Webview console 能看到 Rust 日志。
- [ ] 前端 `console.error` 能在日志文件中看到对应记录。
- [ ] Settings 页面新增「日志与诊断」区域，可打开日志文件夹。
- [ ] `tauri build`（不带 `e2e-testing` feature）产物不包含 Playwright 插件代码。
- [ ] 现有 `cargo test` 全部通过。
- [ ] 前端 `npm run build` 通过。
