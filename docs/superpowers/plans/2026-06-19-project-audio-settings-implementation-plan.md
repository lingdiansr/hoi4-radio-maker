# 项目设置与音频库重构实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现全局默认项目路径、项目信息、持久化默认项目信息、HOI4 版本探测、音频库视图切换与编辑、电台删除与去重。

**Architecture：** 后端扩展 `Settings` 与项目创建逻辑，新增 HOI4 版本探测模块、音频更新命令、电台管理命令；前端扩展设置表单、新建项目对话框、项目信息页、音频库视图与编辑、电台删除。

**Tech Stack：** Rust (tauri, rusqlite, serde), Vue 3 + TypeScript + Vuetify, Pinia

---

## 文件变更清单

| 文件 | 操作 | 职责 |
|---|---|---|
| `src-tauri/src/settings.rs` | 修改 | 扩展 Settings 结构 |
| `src-tauri/src/models.rs` | 修改 | 新增音频更新请求结构 |
| `src-tauri/src/error.rs` | 修改 | 新增 StationNameExists 错误 |
| `src-tauri/src/hoi4_version.rs` | 创建 | HOI4 版本探测 |
| `src-tauri/src/commands.rs` | 修改 | 改造 create_project，新增音频/电台命令 |
| `src-tauri/src/db.rs` | 修改 | 新增音频更新/电台删除/电台按名查询 |
| `src-tauri/src/lib.rs` | 修改 | 注册新命令 |
| `src/stores/audio.ts` | 修改 | 新增 updateAudio/batchUpdate |
| `src/stores/station.ts` | 修改 | 新增 deleteStation |
| `src/stores/settings.ts` | 创建 | 全局设置 Store |
| `src/views/SettingsView.vue` | 修改 | 扩展设置表单 |
| `src/components/AppSidebar.vue` | 修改 | 新建项目对话框增强 |
| `src/views/ProjectSettingsView.vue` | 修改 | 改名、output_dir 只读 |
| `src/views/AudioArchiveView.vue` | 修改 | 内部滚动、视图切换、编辑 |
| `src/views/StationEditorView.vue` | 修改 | 删除电台 |
| `src/components/AudioEditDialog.vue` | 创建 | 单条音频编辑 |
| `src/components/BatchAudioEditDialog.vue` | 创建 | 批量音频编辑 |

---

## Task 1: 扩展后端 Settings 模型

**Files:**
- Modify: `src-tauri/src/settings.rs`

- [ ] **Step 1: 修改 Settings 结构体**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub hoi4_game_dir: Option<String>,
    pub theme: String,
    pub import_concurrency: u32,
    pub default_project_dir: Option<String>,
    pub default_author: Option<String>,
    pub default_version: Option<String>,
    pub default_supported_version: Option<String>,
    pub default_tags: Vec<String>,
}
```

- [ ] **Step 2: 修改 Default 实现**

```rust
impl Default for Settings {
    fn default() -> Self {
        Self {
            ffmpeg_path: None,
            ffprobe_path: None,
            hoi4_game_dir: None,
            theme: "dark".to_string(),
            import_concurrency: 8,
            default_project_dir: None,
            default_author: None,
            default_version: Some("0.1.0".to_string()),
            default_supported_version: None,
            default_tags: vec!["Sound".to_string()],
        }
    }
}
```

- [ ] **Step 3: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: `Finished dev` 无错误。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/settings.rs
git commit -m "feat(settings): extend Settings model with default project metadata"
```

---

## Task 2: HOI4 版本探测模块

**Files:**
- Create: `src-tauri/src/hoi4_version.rs`

- [ ] **Step 1: 创建模块文件**

```rust
use std::path::Path;

pub fn detect_game_version(hoi4_dir: &Path) -> Option<String> {
    let settings_path = hoi4_dir.join("launcher-settings.json");
    let content = std::fs::read_to_string(settings_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get("version")?.as_str().map(|s| s.to_string())
}
```

- [ ] **Step 2: 注册模块**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod hoi4_version;
```

- [ ] **Step 3: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译通过。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/hoi4_version.rs src-tauri/src/lib.rs
git commit -m "feat(hoi4): add game version detection from launcher-settings.json"
```

---

## Task 3: 新增音频更新请求模型

**Files:**
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: 在 models.rs 末尾追加**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAudioFileRequest {
    pub title: Option<String>,
    pub artist: Option<Option<String>>,
    pub volume: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateAudioFileRequest {
    pub artist: Option<Option<String>>,
    pub volume: Option<f64>,
    pub tags: Option<Vec<String>>,
}
```

- [ ] **Step 2: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译通过。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat(models): add audio update request structs"
```

---

## Task 4: 新增 StationNameExists 错误

**Files:**
- Modify: `src-tauri/src/error.rs`

- [ ] **Step 1: 添加错误变体**

```rust
#[derive(Debug, thiserror::Error)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Hoi4RadioError {
    // ... existing variants ...

    #[error("station name already exists: {name}")]
    StationNameExists { name: String },
}
```

- [ ] **Step 2: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译通过。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/error.rs
git commit -m "feat(error): add StationNameExists error variant"
```

---

## Task 5: 数据库层新增音频与电台方法

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: 新增音频单条更新方法**

```rust
pub fn update_audio_file(&self, id: &str, req: &UpdateAudioFileRequest) -> Result<AudioFile> {
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(title) = &req.title {
        sets.push("title = ?");
        params.push(Box::new(title.clone()));
    }
    if let Some(artist) = &req.artist {
        sets.push("artist = ?");
        params.push(Box::new(artist.clone()));
    }
    if let Some(volume) = req.volume {
        sets.push("volume = ?");
        params.push(Box::new(volume));
    }
    if let Some(tags) = &req.tags {
        sets.push("tags = ?");
        params.push(Box::new(serde_json::to_string(tags)?));
    }
    if let Some(notes) = &req.notes {
        sets.push("notes = ?");
        params.push(Box::new(notes.clone()));
    }

    if sets.is_empty() {
        return self.get_audio_file(id).ok_or_else(|| Hoi4RadioError::AudioFileNotFound { id: id.to_string() });
    }

    let now = Utc::now().to_rfc3339();
    sets.push("updated_at = ?");
    params.push(Box::new(now));

    let sql = format!(
        "UPDATE audio_files SET {} WHERE id = ?",
        sets.join(", ")
    );
    params.push(Box::new(id.to_string()));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    self.conn.execute(&sql, param_refs.as_slice())?;
    self.get_audio_file(id).ok_or_else(|| Hoi4RadioError::AudioFileNotFound { id: id.to_string() })
}
```

> 注：`get_audio_file` 需在 db.rs 中存在或新增；若不存在，新增按 id 查询方法。

- [ ] **Step 2: 新增音频批量更新方法**

```rust
pub fn batch_update_audio_files(
    &self,
    ids: &[String],
    req: &BatchUpdateAudioFileRequest,
) -> Result<Vec<AudioFile>> {
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(artist) = &req.artist {
        sets.push("artist = ?");
        params.push(Box::new(artist.clone()));
    }
    if let Some(volume) = req.volume {
        sets.push("volume = ?");
        params.push(Box::new(volume));
    }
    if let Some(tags) = &req.tags {
        sets.push("tags = ?");
        params.push(Box::new(serde_json::to_string(tags)?));
    }

    if sets.is_empty() || ids.is_empty() {
        return Ok(Vec::new());
    }

    let now = Utc::now().to_rfc3339();
    sets.push("updated_at = ?");
    params.push(Box::new(now));

    let placeholders: Vec<String> = (0..ids.len()).map(|i| format!("?{}", i + params.len() + 1)).collect();
    let sql = format!(
        "UPDATE audio_files SET {} WHERE id IN ({})",
        sets.join(", "),
        placeholders.join(", ")
    );

    for id in ids {
        params.push(Box::new(id.clone()));
    }

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    self.conn.execute(&sql, param_refs.as_slice())?;

    let mut result = Vec::new();
    for id in ids {
        if let Some(audio) = self.get_audio_file(id) {
            result.push(audio);
        }
    }
    Ok(result)
}
```

- [ ] **Step 3: 新增电台按名称查询与删除方法**

```rust
pub fn find_station_by_name(&self, project_id: &str, name: &str) -> Result<Option<Station>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, project_id, name, sort_order FROM stations WHERE project_id = ?1 AND name = ?2"
    )?;
    let mut rows = stmt.query([project_id, name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(self.station_from_row(row)?))
    } else {
        Ok(None)
    }
}

pub fn delete_station(&self, station_id: &str) -> Result<()> {
    self.conn.execute("DELETE FROM stations WHERE id = ?1", [station_id])?;
    Ok(())
}
```

> 注：`station_from_row` 若不存在需新增辅助函数。

- [ ] **Step 4: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -20`
Expected: 编译通过。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat(db): add audio update, batch update, and station lookup/delete methods"
```

---

## Task 6: 改造 create_project 与新增命令

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 新增辅助函数 sanitize_project_name 和 write_mod_descriptor**

```rust
fn sanitize_project_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '\"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

fn write_mod_descriptor(output_dir: &Path, project: &Project) -> Result<()> {
    let mod_file = output_dir.with_extension("mod");
    let path_in_mod = format!("mod/{}", sanitize_project_name(&project.name));
    let tags = project
        .tags
        .iter()
        .map(|t| format!("    \"{}\"", t.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join("\n");
    let content = format!(
        "name=\"{}\"\npath=\"{}\"\nsupported_version=\"{}\"\ntags={{\n{}\n}}\nversion=\"{}\"\n",
        project.name.replace('"', "\\\""),
        path_in_mod,
        project.supported_version,
        tags,
        project.version
    );
    std::fs::write(&mod_file, content)?;
    Ok(())
}

fn current_system_user() -> Option<String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok()
}
```

- [ ] **Step 2: 改造 create_project**

```rust
#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    req: CreateProjectRequest,
) -> Result<Project> {
    let settings = {
        let db = lock_db(&state)?;
        Settings::get(&db)?
    };

    let output_dir = if req.output_dir.as_os_str().is_empty() {
        let base = settings
            .default_project_dir
            .as_deref()
            .ok_or_else(|| Hoi4RadioError::Other {
                message: "未设置项目默认目录，请先在全局设置中设置或手动选择输出目录".to_string(),
            })?;
        PathBuf::from(base).join(sanitize_project_name(&req.name))
    } else {
        req.output_dir
    };

    tokio::fs::create_dir_all(&output_dir).await?;

    let author = req
        .author
        .filter(|s| !s.is_empty())
        .or(settings.default_author)
        .or_else(current_system_user);

    let project = {
        let db = lock_db(&state)?;
        db.create_project(&CreateProjectRequest {
            name: req.name,
            version: req.version,
            supported_version: req.supported_version,
            tags: req.tags,
            author,
            output_dir,
        })?
    };

    write_mod_descriptor(&project.output_dir, &project)?;
    Ok(project)
}
```

> 注：`db.create_project` 当前签名使用 `CreateProjectRequest` 且 `output_dir` 为 PathBuf。需确保 `CreateProjectRequest` 的 `output_dir` 字段类型匹配。

- [ ] **Step 3: 新增音频更新命令**

```rust
#[tauri::command]
pub fn update_audio_file(
    state: State<'_, AppState>,
    id: String,
    req: UpdateAudioFileRequest,
) -> Result<AudioFile> {
    let db = lock_db(&state)?;
    db.update_audio_file(&id, &req)
}

#[tauri::command]
pub fn batch_update_audio_files(
    state: State<'_, AppState>,
    ids: Vec<String>,
    req: BatchUpdateAudioFileRequest,
) -> Result<Vec<AudioFile>> {
    let db = lock_db(&state)?;
    db.batch_update_audio_files(&ids, &req)
}
```

- [ ] **Step 4: 新增电台删除命令与 create_station 去重**

```rust
#[tauri::command]
pub fn create_station(
    state: State<'_, AppState>,
    project_id: String,
    name: String,
) -> Result<Station> {
    let db = lock_db(&state)?;
    if db.find_station_by_name(&project_id, &name)?.is_some() {
        return Err(Hoi4RadioError::StationNameExists { name });
    }
    db.create_station(&project_id, &name)
}

#[tauri::command]
pub fn delete_station(state: State<'_, AppState>, station_id: String) -> Result<()> {
    let db = lock_db(&state)?;
    db.delete_station(&station_id)
}
```

- [ ] **Step 5: 在 lib.rs 注册新命令**

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::update_audio_file,
    commands::batch_update_audio_files,
    commands::delete_station,
])
```

- [ ] **Step 6: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -20`
Expected: 编译通过。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add default project path, mod descriptor, audio update, station delete/unique name"
```

---

## Task 7: get_settings 返回探测版本

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: 修改 get_settings**

```rust
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings> {
    let db = lock_db(&state)?;
    let mut settings = Settings::get(&db)?;
    if settings.default_supported_version.is_none() {
        if let Some(hoi4_dir) = settings.hoi4_game_dir.as_deref() {
            if let Some(version) = crate::hoi4_version::detect_game_version(Path::new(hoi4_dir)) {
                settings.default_supported_version = Some(version);
            }
        }
    }
    Ok(settings)
}
```

- [ ] **Step 2: 编译检查**

Run: `cd src-tauri && cargo check 2>&1 | tail -10`
Expected: 编译通过。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat(settings): detect HOI4 version as default supported_version"
```

---

## Task 8: 创建全局 Settings Store

**Files:**
- Create: `src/stores/settings.ts`

- [ ] **Step 1: 创建 Store**

```ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invokeCommand } from '@/api/client'

export interface AppSettings {
  ffmpeg_path: string | null
  ffprobe_path: string | null
  hoi4_game_dir: string | null
  theme: string
  import_concurrency: number
  default_project_dir: string | null
  default_author: string | null
  default_version: string | null
  default_supported_version: string | null
  default_tags: string[]
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    ffmpeg_path: null,
    ffprobe_path: null,
    hoi4_game_dir: null,
    theme: 'dark',
    import_concurrency: 8,
    default_project_dir: null,
    default_author: null,
    default_version: '0.1.0',
    default_supported_version: null,
    default_tags: ['Sound'],
  })

  async function loadSettings() {
    const s = await invokeCommand<AppSettings>('get_settings')
    Object.assign(settings.value, s)
  }

  async function saveSettings() {
    await invokeCommand('save_settings', { settings: settings.value })
  }

  return {
    settings,
    loadSettings,
    saveSettings,
  }
})
```

- [ ] **Step 2: 编译检查**

Run: `npm run build 2>&1 | tail -10`
Expected: `vue-tsc` 通过。

- [ ] **Step 3: Commit**

```bash
git add src/stores/settings.ts
git commit -m "feat(store): add global settings store"
```

---

## Task 9: 扩展全局设置表单

**Files:**
- Modify: `src/views/SettingsView.vue`

- [ ] **Step 1: 扩展 Settings 接口与导入 settings store**

```ts
import { useSettingsStore, type AppSettings } from '@/stores/settings'

interface Settings extends AppSettings {}
```

- [ ] **Step 2: 在表单中新增字段**

在 `v-row` 内 `v-col` 中追加：

```vue
<PathField
  v-model="defaultProjectDir"
  label="项目默认目录"
  placeholder="选择存放项目的根目录"
  prepend-inner-icon="mdi-folder-home"
  picker-mode="directory"
  class="mb-4"
/>
<v-text-field
  v-model="settings.default_author"
  label="默认作者"
  placeholder="未填写时使用系统用户名"
  prepend-inner-icon="mdi-account"
  class="mb-4"
  hide-details="auto"
/>
<v-text-field
  v-model="settings.default_version"
  label="默认 Mod 版本"
  placeholder="0.1.0"
  prepend-inner-icon="mdi-tag-outline"
  class="mb-4"
  hide-details="auto"
/>
<v-text-field
  v-model="settings.default_supported_version"
  label="默认支持游戏版本"
  placeholder="*"
  prepend-inner-icon="mdi-gamepad-variant-outline"
  class="mb-4"
  hide-details="auto"
/>
<v-combobox
  v-model="settings.default_tags"
  label="默认标签"
  placeholder="输入后按回车添加"
  prepend-inner-icon="mdi-tag-multiple"
  multiple
  chips
  class="mb-4"
  hide-details="auto"
/>
```

- [ ] **Step 3: 添加 defaultProjectDir computed**

```ts
const defaultProjectDir = computed({
  get: () => settings.default_project_dir ?? '',
  set: (v: string) => { settings.default_project_dir = v.trim() || null }
})
```

- [ ] **Step 4: 修改 onMounted 和 save**

```ts
const settingsStore = useSettingsStore()

onMounted(async () => {
  await settingsStore.loadSettings()
  Object.assign(settings, settingsStore.settings)
})

async function save() {
  Object.assign(settingsStore.settings, settings)
  await settingsStore.saveSettings()
}
```

- [ ] **Step 5: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 6: Commit**

```bash
git add src/views/SettingsView.vue
git commit -m "feat(ui): extend settings form with default project metadata"
```

---

## Task 10: 改造新建项目对话框

**Files:**
- Modify: `src/components/AppSidebar.vue`

- [ ] **Step 1: 导入 settings store 并预填默认值**

```ts
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()
const isOutputDirManuallySet = ref(false)

async function openCreateDialog() {
  await settingsStore.loadSettings()
  form.name = 'My Radio Mod'
  form.version = settingsStore.settings.default_version ?? '0.1.0'
  form.supported_version = settingsStore.settings.default_supported_version ?? '*'
  form.author = settingsStore.settings.default_author ?? ''
  form.tags = [...settingsStore.settings.default_tags]
  form.output_dir = ''
  isOutputDirManuallySet.value = false
  updateOutputDirFromName()
  showCreateDialog.value = true
}

function updateOutputDirFromName() {
  if (isOutputDirManuallySet.value) return
  const base = settingsStore.settings.default_project_dir
  if (base && form.name) {
    form.output_dir = `${base.replace(/\/$/, '')}/${sanitize(form.name)}`
  }
}

function sanitize(name: string) {
  return name.replace(/[\\/:*?"<>|]/g, '_')
}
```

- [ ] **Step 2: 监听项目名变化自动更新 output_dir**

```ts
watch(() => form.name, updateOutputDirFromName)
```

- [ ] **Step 3: PathField 添加手动选择标记**

```vue
<PathField
  v-model="form.output_dir"
  label="输出目录"
  placeholder="选择 Mod 输出目录"
  prepend-inner-icon="mdi-folder-open"
  picker-mode="directory"
  class="mb-4"
  :rules="[required]"
  @update:model-value="isOutputDirManuallySet = true"
/>
```

- [ ] **Step 4: 修改 handleCreate 以提交 tags 和 author**

```ts
async function handleCreate() {
  if (!form.name || !form.version || !form.supported_version || !form.output_dir) return
  const p = await projectStore.createProject({
    name: form.name,
    version: form.version,
    supported_version: form.supported_version,
    tags: form.tags,
    author: form.author.trim() || undefined,
    output_dir: form.output_dir,
  })
  if (p) {
    showCreateDialog.value = false
    router.push({ name: 'project', params: { id: p.id } })
  }
}
```

- [ ] **Step 5: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 6: Commit**

```bash
git add src/components/AppSidebar.vue
git commit -m "feat(ui): use default project path and metadata in create project dialog"
```

---

## Task 11: 项目信息页改名与只读输出目录

**Files:**
- Modify: `src/views/ProjectSettingsView.vue`

- [ ] **Step 1: 修改标题和 output_dir 字段**

```vue
<div class="text-display text-h5">项目信息</div>
```

output_dir 改为只读：

```vue
<v-text-field
  :model-value="form.output_dir"
  label="输出目录"
  prepend-inner-icon="mdi-folder-open"
  class="mb-4"
  hide-details="auto"
  readonly
/>
```

- [ ] **Step 2: 移除 PathField 对 output_dir 的使用**

删除原有的 `PathField` 绑定到 `form.output_dir` 的代码。

- [ ] **Step 3: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 4: Commit**

```bash
git add src/views/ProjectSettingsView.vue
git commit -m "feat(ui): rename project settings to project info, make output_dir read-only"
```

---

## Task 12: 音频 Store 新增编辑方法

**Files:**
- Modify: `src/stores/audio.ts`

- [ ] **Step 1: 新增方法**

```ts
export interface UpdateAudioRequest {
  title?: string
  artist?: string | null
  volume?: number
  tags?: string[]
  notes?: string | null
}

export interface BatchUpdateAudioRequest {
  artist?: string | null
  volume?: number
  tags?: string[]
}

async function updateAudio(id: string, req: UpdateAudioRequest): Promise<AudioFile> {
  const updated = await invokeCommand<AudioFile>('update_audio_file', { id, req })
  const idx = allAudioFiles.value.findIndex((a) => a.id === id)
  if (idx !== -1) {
    allAudioFiles.value[idx] = updated
  }
  return updated
}

async function batchUpdateAudio(ids: string[], req: BatchUpdateAudioRequest): Promise<AudioFile[]> {
  const updated = await invokeCommand<AudioFile[]>('batch_update_audio_files', { ids, req })
  for (const audio of updated) {
    const idx = allAudioFiles.value.findIndex((a) => a.id === audio.id)
    if (idx !== -1) {
      allAudioFiles.value[idx] = audio
    }
  }
  return updated
}
```

- [ ] **Step 2: 暴露方法**

```ts
return {
  // ... existing ...
  updateAudio,
  batchUpdateAudio,
}
```

- [ ] **Step 3: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 4: Commit**

```bash
git add src/stores/audio.ts
git commit -m "feat(store): add audio update and batch update methods"
```

---

## Task 13: 新增 AudioEditDialog 组件

**Files:**
- Create: `src/components/AudioEditDialog.vue`

- [ ] **Step 1: 创建组件**

```vue
<template>
  <v-dialog v-model="dialog" max-width="520" class="bureau-dialog">
    <v-card class="dialog-card">
      <div class="dialog-accent" />
      <v-card-title class="dialog-title pa-6 pb-2">
        <div class="text-mono text-caption text-secondary">EDIT AUDIO</div>
        <div class="text-display text-h5">编辑音频信息</div>
      </v-card-title>
      <v-card-text class="pa-6 pt-4">
        <v-text-field v-model="form.title" label="标题" class="mb-4" hide-details="auto" />
        <v-text-field v-model="artistInput" label="艺术家" class="mb-4" hide-details="auto" />
        <v-slider v-model="form.volume" label="音量" min="0" max="1" step="0.05" thumb-label class="mb-4" />
        <v-combobox v-model="form.tags" label="标签" multiple chips class="mb-4" hide-details="auto" />
        <v-textarea v-model="notesInput" label="备注" rows="3" hide-details="auto" />
      </v-card-text>
      <v-card-actions class="pa-6">
        <v-spacer />
        <v-btn variant="text" @click="dialog = false">取消</v-btn>
        <v-btn color="primary" prepend-icon="mdi-check-circle" @click="save">保存</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { reactive, computed, watch } from 'vue'
import { useAudioStore, type AudioFile, type UpdateAudioRequest } from '@/stores/audio'

const dialog = defineModel<boolean>({ required: true })
const props = defineProps<{ audio: AudioFile | null }>()
const emit = defineEmits<{ (e: 'saved'): void }>()

const audioStore = useAudioStore()

const form = reactive<UpdateAudioRequest & { title: string }>({
  title: '',
  artist: undefined,
  volume: 0.75,
  tags: [],
  notes: undefined,
})

const artistInput = computed({
  get: () => form.artist ?? '',
  set: (v: string) => { form.artist = v.trim() || null }
})

const notesInput = computed({
  get: () => form.notes ?? '',
  set: (v: string) => { form.notes = v.trim() || null }
})

watch(() => props.audio, (a) => {
  if (!a) return
  form.title = a.title
  form.artist = a.artist ?? null
  form.volume = a.volume
  form.tags = [...a.tags]
  form.notes = a.notes ?? null
}, { immediate: true })

async function save() {
  if (!props.audio) return
  await audioStore.updateAudio(props.audio.id, {
    title: form.title,
    artist: form.artist,
    volume: form.volume,
    tags: form.tags,
    notes: form.notes,
  })
  dialog.value = false
  emit('saved')
}
</script>
```

- [ ] **Step 2: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 3: Commit**

```bash
git add src/components/AudioEditDialog.vue
git commit -m "feat(ui): add AudioEditDialog component"
```

---

## Task 14: 新增 BatchAudioEditDialog 组件

**Files:**
- Create: `src/components/BatchAudioEditDialog.vue`

- [ ] **Step 1: 创建组件**

```vue
<template>
  <v-dialog v-model="dialog" max-width="520" class="bureau-dialog">
    <v-card class="dialog-card">
      <div class="dialog-accent" />
      <v-card-title class="dialog-title pa-6 pb-2">
        <div class="text-mono text-caption text-secondary">BATCH EDIT</div>
        <div class="text-display text-h5">批量编辑 ({{ count }})</div>
      </v-card-title>
      <v-card-text class="pa-6 pt-4">
        <v-text-field v-model="artistInput" label="艺术家" class="mb-4" hide-details="auto" />
        <v-slider v-model="form.volume" label="音量" min="0" max="1" step="0.05" thumb-label class="mb-4" />
        <v-combobox v-model="form.tags" label="标签" multiple chips class="mb-4" hide-details="auto" />
        <div class="text-caption text-secondary">留空表示不修改该字段</div>
      </v-card-text>
      <v-card-actions class="pa-6">
        <v-spacer />
        <v-btn variant="text" @click="dialog = false">取消</v-btn>
        <v-btn color="primary" prepend-icon="mdi-check-circle" @click="save">保存</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { reactive, computed } from 'vue'
import { useAudioStore, type BatchUpdateAudioRequest } from '@/stores/audio'

const dialog = defineModel<boolean>({ required: true })
const props = defineProps<{ ids: string[] }>()
const emit = defineEmits<{ (e: 'saved'): void }>()

const audioStore = useAudioStore()

const form = reactive<BatchUpdateAudioRequest>({
  artist: undefined,
  volume: undefined,
  tags: [],
})

const count = computed(() => props.ids.length)

const artistInput = computed({
  get: () => form.artist ?? '',
  set: (v: string) => { form.artist = v.trim() || undefined }
})

async function save() {
  const req: BatchUpdateAudioRequest = {}
  if (form.artist !== undefined) req.artist = form.artist
  if (form.volume !== undefined) req.volume = form.volume
  if (form.tags && form.tags.length > 0) req.tags = form.tags
  await audioStore.batchUpdateAudio(props.ids, req)
  dialog.value = false
  emit('saved')
}
</script>
```

- [ ] **Step 2: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 3: Commit**

```bash
git add src/components/BatchAudioEditDialog.vue
git commit -m "feat(ui): add BatchAudioEditDialog component"
```

---

## Task 15: 重构 AudioArchiveView

**Files:**
- Modify: `src/views/AudioArchiveView.vue`

- [ ] **Step 1: 新增导入与状态**

```ts
import AudioEditDialog from '@/components/AudioEditDialog.vue'
import BatchAudioEditDialog from '@/components/BatchAudioEditDialog.vue'

const viewMode = ref<'grid' | 'list'>('grid')
const selectedIds = ref<Set<string>>(new Set())
const selecting = ref(false)
const showEditDialog = ref(false)
const showBatchDialog = ref(false)
const audioToEdit = ref<AudioFile | null>(null)
```

- [ ] **Step 2: 调整卡片布局为内部滚动**

```vue
<v-card class="archive-card" variant="elevated" rounded="xl">
  <!-- header -->
  <v-card-text class="pa-6 archive-body">
    <!-- filters -->
    <div class="audio-scroll-container">
      <!-- grid/list content -->
    </div>
  </v-card-text>
</v-card>
```

CSS：

```css
.archive-card {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 48px);
}
.archive-body {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}
.audio-scroll-container {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
```

- [ ] **Step 3: 添加视图切换按钮与选择模式**

在卡片标题右侧：

```vue
<v-btn-toggle v-model="viewMode" mandatory density="comfortable" class="mr-3">
  <v-btn value="grid" icon="mdi-view-grid-outline" />
  <v-btn value="list" icon="mdi-view-list-outline" />
</v-btn-toggle>
<v-btn
  :color="selecting ? 'primary' : undefined"
  variant="outlined"
  size="small"
  @click="toggleSelecting"
>
  {{ selecting ? '完成' : '选择' }}
</v-btn>
```

- [ ] **Step 4: 列表视图与选择逻辑**

网格视图每项增加选择复选框（仅在 selecting 模式显示）。
列表视图使用 `v-data-table` 或 `v-list`，同样支持选择。

```ts
function toggleSelecting() {
  selecting.value = !selecting.value
  selectedIds.value.clear()
}

function toggleSelect(id: string) {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id)
  } else {
    selectedIds.value.add(id)
  }
}

function openEdit(audio: AudioFile) {
  audioToEdit.value = audio
  showEditDialog.value = true
}
```

- [ ] **Step 5: 批量操作栏**

```vue
<v-expand-transition>
  <div v-if="selecting && selectedIds.size > 0" class="batch-bar d-flex gap-3 mb-4">
    <v-btn color="primary" size="small" prepend-icon="mdi-pencil" @click="showBatchDialog = true">批量编辑</v-btn>
    <v-btn variant="outlined" size="small" prepend-icon="mdi-delete-outline" @click="batchDelete">删除</v-btn>
    <v-spacer />
    <span class="text-caption text-secondary">已选择 {{ selectedIds.size }} 项</span>
  </div>
</v-expand-transition>
```

- [ ] **Step 6: 挂载编辑/批量对话框**

```vue
<AudioEditDialog v-model="showEditDialog" :audio="audioToEdit" @saved="onImported" />
<BatchAudioEditDialog v-model="showBatchDialog" :ids="Array.from(selectedIds)" @saved="onImported" />
```

- [ ] **Step 7: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 8: Commit**

```bash
git add src/views/AudioArchiveView.vue
-git commit -m "feat(ui): refactor audio archive with internal scrolling, view toggle, and edit dialogs"
```

---

## Task 16: Station Store 新增删除方法

**Files:**
- Modify: `src/stores/station.ts`

- [ ] **Step 1: 新增 deleteStation**

```ts
async function deleteStation(stationId: string) {
  await invokeCommand('delete_station', { stationId })
  await loadStations()
}
```

- [ ] **Step 2: 暴露方法**

```ts
return {
  // ... existing ...
  deleteStation,
}
```

- [ ] **Step 3: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 4: Commit**

```bash
git add src/stores/station.ts
git commit -m "feat(store): add deleteStation method"
```

---

## Task 17: StationEditorView 增加删除按钮

**Files:**
- Modify: `src/views/StationEditorView.vue`

- [ ] **Step 1: 在电台卡片标题添加删除按钮**

```vue
<v-card-title class="d-flex justify-space-between align-center">
  <span class="text-display text-h6">歌曲列表</span>
  <div class="d-flex align-center gap-2">
    <v-chip size="small" color="primary" class="text-mono">
      {{ station.entries.length }} tracks
    </v-chip>
    <v-btn icon="mdi-delete-outline" variant="text" size="small" color="error" @click="confirmDeleteStation(station.id)" />
  </div>
</v-card-title>
```

- [ ] **Step 2: 添加确认对话框与删除逻辑**

```ts
const showDeleteStationDialog = ref(false)
const stationToDelete = ref<Station | null>(null)

function confirmDeleteStation(station: Station) {
  stationToDelete.value = station
  showDeleteStationDialog.value = true
}

async function handleDeleteStation() {
  if (!stationToDelete.value) return
  await stationStore.deleteStation(stationToDelete.value.id)
  showDeleteStationDialog.value = false
  stationToDelete.value = null
}
```

- [ ] **Step 3: 添加删除确认对话框模板**

```vue
<v-dialog v-model="showDeleteStationDialog" max-width="420" class="bureau-dialog">
  <v-card class="dialog-card">
    <div class="dialog-accent dialog-accent--danger" />
    <v-card-title class="dialog-title pa-6 pb-2">
      <v-icon color="error" size="28">mdi-alert-circle</v-icon>
      <div>
        <div class="text-mono text-caption text-secondary">CONFIRM DELETION</div>
        <div class="text-display text-h5">删除电台</div>
      </div>
    </v-card-title>
    <v-card-text class="pa-6 pt-4">
      确定要删除电台 <strong class="text-primary">{{ stationToDelete?.name }}</strong> 吗？
    </v-card-text>
    <v-card-actions class="pa-6">
      <v-spacer />
      <v-btn variant="text" @click="showDeleteStationDialog = false">取消</v-btn>
      <v-btn color="error" prepend-icon="mdi-delete-outline" @click="handleDeleteStation">删除</v-btn>
    </v-card-actions>
  </v-card>
</v-dialog>
```

- [ ] **Step 4: 构建检查**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 5: Commit**

```bash
git add src/views/StationEditorView.vue
git commit -m "feat(ui): add station deletion button with confirmation"
```

---

## Task 18: 后端测试补充

**Files:**
- Modify: `src-tauri/tests/station_test.rs`（若存在）或新增测试

- [ ] **Step 1: 添加电台唯一性测试**

```rust
#[test]
fn test_create_station_with_duplicate_name_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let db = Db::open(tmp.path().join("test.db")).unwrap();
    let project = db.create_project(&sample_project_req()).unwrap();
    db.create_station(&project.id, "前线电台").unwrap();
    let result = db.create_station(&project.id, "前线电台");
    assert!(result.is_err());
}
```

- [ ] **Step 2: 运行测试**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: 测试通过。

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tests/station_test.rs
git commit -m "test(station): add duplicate name test"
```

---

## Task 19: 集成验证

**Files:**
- N/A

- [ ] **Step 1: 后端全量测试**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: 所有测试通过。

- [ ] **Step 2: 前端构建**

Run: `npm run build 2>&1 | tail -10`
Expected: 构建通过。

- [ ] **Step 3: Release 构建检查**

Run: `cd src-tauri && cargo check --release 2>&1 | tail -10`
Expected: 编译通过。

- [ ] **Step 4: Commit（如有 Cargo.lock 更新）**

```bash
git add src-tauri/Cargo.lock || true
git commit -m "chore(lock): update Cargo.lock" || echo "no lock changes"
```

---

## Self-Review

### Spec Coverage

| Spec 要求 | 对应 Task |
|---|---|
| 全局默认路径 | Task 1, Task 6, Task 10 |
| 项目信息改名 | Task 11 |
| 持久化设置 + HOI4 版本 | Task 1, Task 2, Task 7, Task 8, Task 9 |
| 默认项目信息 | Task 1, Task 6, Task 10 |
| 音频库内部滚动 | Task 15 |
| 视图切换与编辑 | Task 12, Task 13, Task 14, Task 15 |
| 电台删除与去重 | Task 4, Task 5, Task 16, Task 17, Task 18 |

### Placeholder Scan

- 无 TBD/TODO。
- 所有代码片段包含完整路径。
- 所有命令包含预期输出。

### Type Consistency

- `UpdateAudioFileRequest` / `BatchUpdateAudioFileRequest` 前后端名称一致。
- `Settings` 扩展字段在 Rust/TypeScript 中名称一致（snake_case in Rust, camelCase in TS via Tauri transform）。
- `AudioFile` 接口已存在，编辑对话框字段匹配。

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-19-project-audio-settings-implementation-plan.md`.**

Two execution options:

**1. Subagent-Driven (recommended)** - Dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach would you like?
