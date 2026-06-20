# HOI4 Radio Maker 项目设置与音频库重构设计

> 设计日期：2026-06-19  
> 状态：待用户审查  
> 分支：`feat/project-settings-audio-library`  
> 方案：方案 A（统一重构）

---

## 1. 目标

一次性实现以下 7 项需求：

1. **项目全局默认路径**：全局设置可指定默认项目目录；新建项目默认在 `<默认目录>/<项目名>/` 下创建；同时生成同级的 `.mod` 文件；输出目录即项目目录。
2. **项目信息改名**：项目内「项目设置」改为「项目信息」。
3. **持久化设置 + HOI4 目录**：设置已持久化到 SQLite；HOI4 游戏目录用于自动读取游戏版本，作为新建项目默认 `supported_version`。
4. **默认项目信息**：全局设置可预填作者、版本、支持版本、标签；作者未填时取系统当前登录用户。
5. **音频库内部滚动**：音频列表在卡片内部滚动，避免整个页面滚动导致侧边栏下移。
6. **音频库视图切换与编辑**：支持列表/网格视图切换；可编辑音频元信息；支持批量编辑 tags、artist、volume。
7. **电台删除与去重**：可删除电台；同一项目内电台名称唯一。

---

## 2. 关键确认

| 问题 | 决策 |
|---|---|
| `.mod` 文件内容 | 标准 HOI4 mod descriptor（`name`、`path`、`supported_version`、`tags`、`version`、`picture` 可选） |
| HOI4 目录用途 | 读取游戏根目录下 `launcher-settings.json` 的 `version` 字段，作为全局默认 `supported_version` |
| 批量编辑字段 | tags、artist、volume |
| 项目信息 output_dir | 只读显示，由全局默认路径 + 项目名决定 |
| 新建项目 output_dir | 默认自动计算，但允许用户手动选择其他目录 |
| 系统用户名 | Linux/macOS 取 `std::env::var("USER")`，Windows 取 `USERNAME`，失败则空 |

---

## 3. 架构

```text
┌─────────────────────────────────────────────────────────────┐
│                     Global Settings                          │
│  default_project_dir  default_author  default_version       │
│  default_supported_version  default_tags  hoi4_game_dir     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Project Creation Flow                      │
│  1. 用户填写项目名                                           │
│  2. 系统从全局设置预填：作者/版本/支持版本/标签              │
│  3. output_dir 默认 = default_project_dir/<项目名>           │
│  4. 用户可手动覆盖 output_dir                                │
│  5. 创建目录 + .mod 文件                                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Project Info View                        │
│  只读显示 output_dir；可修改 name/version/supported_version  │
│  tags/author；不移动目录                                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Audio Archive View                        │
│  内部滚动容器 + 列表/网格切换 + 搜索/标签筛选                │
│  单选编辑 / 多选批量编辑（tags/artist/volume）               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Station Editor                            │
│  删除电台按钮 + 创建/重命名时校验名称唯一                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. 后端改造

### 4.1 Settings 模型扩展

`src-tauri/src/settings.rs`：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub hoi4_game_dir: Option<String>,
    pub theme: String,
    pub import_concurrency: u32,
    // 新增
    pub default_project_dir: Option<String>,
    pub default_author: Option<String>,
    pub default_version: Option<String>,
    pub default_supported_version: Option<String>,
    pub default_tags: Vec<String>,
}
```

Default 新增：
- `default_project_dir`: None
- `default_author`: None（新建时 fallback 到系统用户）
- `default_version`: Some("0.1.0".to_string())
- `default_supported_version`: None（可从 HOI4 目录探测）
- `default_tags`: vec!["Sound".to_string()]

### 4.2 HOI4 版本探测

新增 `src-tauri/src/hoi4_version.rs`：

```rust
use std::path::Path;

pub fn detect_game_version(hoi4_dir: &Path) -> Option<String> {
    let settings_path = hoi4_dir.join("launcher-settings.json");
    let content = std::fs::read_to_string(settings_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("version")?.as_str().map(|s| s.to_string())
}
```

在 `commands::get_settings` 中，如果 `default_supported_version` 为 None 且 `hoi4_game_dir` 存在，则返回探测到的版本（不持久化，仅作为 UI 默认值）。

### 4.3 系统用户名

在 `commands::create_project` 中，如果请求 author 为 None/空且全局 `default_author` 也为空，则使用 `std::env::var("USER")` 或 `USERNAME`。

### 4.4 项目创建改造

`src-tauri/src/commands.rs` 中 `create_project`：

1. 读取全局 Settings。
2. 如果请求 `output_dir` 为空：
   - 使用 `default_project_dir/<sanitize(name)>`。
   - 如果 `default_project_dir` 未设置，返回错误提示用户先设置全局路径或在创建时手动选择。
3. 创建 `output_dir` 目录。
4. 在 `output_dir` 父目录创建 `<项目名>.mod` 文件。
5. 持久化项目到数据库（`output_dir` 存为 PathBuf）。

`.mod` 文件示例：

```text
name="东方之声"
path="mod/东方之声"
supported_version="1.14.*"
tags={
    "Sound"
}
version="0.1.0"
```

> 注：`path` 字段指向项目文件夹相对于游戏 `mod` 目录的相对路径。若用户手动选择其他目录，`.mod` 文件仍生成在该目录的父级。

### 4.5 项目信息更新

`update_project`：
- 允许修改 name/version/supported_version/tags/author。
- `output_dir` 不修改（保持创建时的目录）。
- 如果 name 变化，**不移动目录**，避免路径漂移。项目显示名与目录名可不一致。

### 4.6 音频信息编辑

新增命令：

```rust
#[tauri::command]
pub fn update_audio_file(
    state: State<'_, AppState>,
    id: String,
    req: UpdateAudioFileRequest,
) -> Result<AudioFile>
```

`UpdateAudioFileRequest` 结构：

```rust
pub struct UpdateAudioFileRequest {
    pub title: Option<String>,
    pub artist: Option<Option<String>>,
    pub volume: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<Option<String>>,
}
```

新增批量更新命令：

```rust
#[tauri::command]
pub fn batch_update_audio_files(
    state: State<'_, AppState>,
    ids: Vec<String>,
    req: BatchUpdateAudioFileRequest,
) -> Result<Vec<AudioFile>>
```

`BatchUpdateAudioFileRequest` 只包含可批量字段：

```rust
pub struct BatchUpdateAudioFileRequest {
    pub artist: Option<Option<String>>,
    pub volume: Option<f64>,
    pub tags: Option<Vec<String>>,
}
```

### 4.7 电台删除与去重

新增命令：

```rust
#[tauri::command]
pub fn delete_station(state: State<'_, AppState>, station_id: String) -> Result<()>
```

在 `create_station` 和 `update_station`（如未来有重命名）中加入唯一性校验：

```rust
if repo.find_by_name(&project_id, &name)?.is_some() {
    return Err(Hoi4RadioError::StationNameExists { name });
}
```

新增错误类型 `StationNameExists`。

---

## 5. 前端改造

### 5.1 SettingsView.vue

扩展全局设置表单，新增字段：

- **项目默认目录**（PathField，directory）
- **默认作者**
- **默认 Mod 版本**
- **默认支持游戏版本**
- **默认标签**（v-combobox，multiple chips）
- HOI4 游戏目录旁增加「读取版本」按钮，自动填充默认支持版本

### 5.2 AppSidebar.vue 新建项目对话框

- 保留 output_dir 字段，但默认自动填充。
- 当用户输入项目名且 output_dir 为空/为默认路径时，自动更新 output_dir。
- 用户可点击 PathField 手动选择其他目录，选择后锁定不再自动更新。
- 预填作者/版本/支持版本/标签。

新增逻辑：

```ts
const isOutputDirManuallySet = ref(false)

watch(() => form.name, (name) => {
  if (isOutputDirManuallySet.value || !settingsStore.defaultProjectDir) return
  form.output_dir = `${settingsStore.defaultProjectDir}/${sanitize(name)}`
})
```

### 5.3 ProjectSettingsView.vue 改名与调整

- 标题「项目设置」→「项目信息」。
- `output_dir` 改为只读文本显示（或禁用输入框）。
- 其他字段保持可编辑。

### 5.4 AudioArchiveView.vue

**布局**：
- 外层 `v-card` 设置 `display: flex; flex-direction: column; height: calc(100vh - 48px)`（或固定高度）。
- 头部（标题、导入按钮、搜索、筛选）固定。
- 音频列表区域 `overflow-y: auto; flex: 1`。

**视图切换**：
- 标题右侧增加 `v-btn-toggle`：网格 / 列表。
- 网格视图：保持现有卡片布局。
- 列表视图：`v-data-table` 或自定义 `v-list`，列：标题、艺术家、时长、采样率、声道、音量、标签、操作。

**选择模式**：
- 增加「选择」开关或按住 Ctrl/Shift 多选。
- 选中后顶部显示批量操作栏：编辑 tags / artist / volume / 删除。

**编辑**：
- 点击音频卡片/行打开 `AudioEditDialog`。
- 可编辑 title、artist、volume、tags、notes。

### 5.5 新增组件

- `src/components/AudioEditDialog.vue`：单条音频编辑。
- `src/components/BatchAudioEditDialog.vue`：批量编辑 tags/artist/volume。

### 5.6 StationEditorView.vue

- 每个电台卡片标题右侧增加删除按钮（带确认对话框）。
- 新建电台对话框中校验名称非空；后端校验唯一性。

---

## 6. 数据流

### 6.1 新建项目

```
AppSidebar.vue 填写表单
  ↓
projectStore.createProject(req)
  ↓
invoke('create_project', { req })
  ↓
commands::create_project
  ├─ 读取 Settings
  ├─ 计算 output_dir（默认或用户指定）
  ├─ fs::create_dir_all
  ├─ 写入 .mod 文件
  ├─ Db::create_project
  └─ 返回 Project
  ↓
projectStore 更新列表并跳转
```

### 6.2 编辑音频

```
AudioArchiveView.vue 点击音频 / 批量选择
  ↓
AudioEditDialog / BatchAudioEditDialog
  ↓
audioStore.updateAudio(id, req) / batchUpdate(ids, req)
  ↓
invoke('update_audio_file') / invoke('batch_update_audio_files')
  ↓
Db::update_audio_file
  ↓
audioStore.loadAllAudio()
```

### 6.3 删除电台

```
StationEditorView.vue 点击删除
  ↓
stationStore.deleteStation(stationId)
  ↓
invoke('delete_station', { stationId })
  ↓
Db::delete_station
  ↓
stationStore.loadStations()
```

---

## 7. 文件变更清单

### 后端

- `src-tauri/src/settings.rs`：扩展 Settings 结构。
- `src-tauri/src/models.rs`：新增 `UpdateAudioFileRequest`、`BatchUpdateAudioFileRequest`。
- `src-tauri/src/error.rs`：新增 `StationNameExists` 错误。
- `src-tauri/src/hoi4_version.rs`（新增）：HOI4 版本探测。
- `src-tauri/src/commands.rs`：改造 `create_project`、新增 `update_audio_file`、`batch_update_audio_files`、`delete_station`、修改 `create_station`。
- `src-tauri/src/db.rs`：新增音频更新/批量更新/电台删除/按名称查询方法。
- `src-tauri/src/lib.rs`：注册新命令。
- `src-tauri/Cargo.toml`：无需新增依赖（已有 serde_json、dirs）。

### 前端

- `src/stores/project.ts`：可能无需改动，或增加默认值辅助函数。
- `src/stores/audio.ts`：新增 `updateAudio`、`batchUpdate`。
- `src/stores/settings.ts`（新增）或扩展现有设置加载逻辑：集中管理全局设置。
- `src/views/SettingsView.vue`：扩展表单字段。
- `src/views/AppSidebar.vue`：新建项目对话框增强。
- `src/views/ProjectSettingsView.vue`：改名、output_dir 只读。
- `src/views/AudioArchiveView.vue`：内部滚动、视图切换、选择、编辑。
- `src/views/StationEditorView.vue`：删除电台按钮。
- `src/components/AudioEditDialog.vue`（新增）。
- `src/components/BatchAudioEditDialog.vue`（新增）。

---

## 8. 风险与注意事项

1. **路径规范化**
   - 项目名需要 sanitize，避免非法字符（`/`、`\`、`.` 等）。
2. **.mod 文件编码**
   - HOI4 mod descriptor 使用 CP1252 或 UTF-8-BOM 兼容性最好，这里使用 UTF-8（现代 HOI4 支持）。
3. **output_dir 移动**
   - 项目信息中不修改 output_dir，避免音频引用和 .mod 文件路径失效。
4. **批量编辑 volume**
   - volume 范围 0.0~1.0，UI 使用 slider，后端校验。
5. **电台删除确认**
   - 删除电台仅删除电台和条目关系，不删除全局音频。
6. **HOI4 版本探测失败**
   - 如果 `launcher-settings.json` 不存在或解析失败，保持 `supported_version` 为空或用户手动填写。

---

## 9. 验收标准

- [ ] 全局设置可保存/读取默认项目目录、默认作者、版本、支持版本、标签。
- [ ] 新建项目时 output_dir 默认按 `<默认目录>/<项目名>` 填充，且自动创建目录和 `.mod` 文件。
- [ ] 用户可在创建对话框手动修改 output_dir。
- [ ] HOI4 目录选择后可读取游戏版本填入默认支持版本。
- [ ] 作者未填时默认使用系统当前登录用户。
- [ ] 项目内「项目设置」改为「项目信息」，output_dir 只读。
- [ ] 音频库列表在卡片内部滚动，侧边栏不随内容滚动。
- [ ] 音频库支持网格/列表视图切换。
- [ ] 可编辑单条音频 title/artist/volume/tags/notes。
- [ ] 可批量编辑多条音频的 tags/artist/volume。
- [ ] 可删除电台，同一项目内电台名称唯一。
- [ ] `cargo test` 和 `npm run build` 通过。
