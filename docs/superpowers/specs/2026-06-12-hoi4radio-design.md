# HOI4 Radio Maker 设计文档

> 项目路径：`/home/ldsr/code/Rust/hoi4-radio-maker`
> 设计日期：2026-06-12
> 状态：待实现

---

## 1. 项目概述

**目标：** 打造一个跨平台桌面 GUI 工具，帮助《钢铁雄心 IV》（Hearts of Iron IV）MOD 作者从零开始创建和管理音乐/电台 Mod。

**核心能力：**
- 项目管理：创建、打开、保存、删除多个电台 Mod 项目
- 音频库：导入本地音频，自动转码为 HOI4 要求的 Ogg Vorbis
- 电台编辑：一个项目可包含多个电台（station），每个电台独立配置歌曲和播放条件
- 一键生成：输出完整的 HOI4 Mod 目录结构
- 验证：检查文件完整性、ID 一致性、本地化缺失等问题

---

## 2. 技术选型

| 层级 | 技术 | 说明 |
|---|---|---|
| 桌面框架 | Tauri 2 | Rust 后端 + Web 前端，跨平台，包体积小 |
| 前端框架 | Vue 3 + TypeScript | `<script setup>` SFC |
| 前端构建 | Vite | 热更新与打包 |
| 包管理器 | Bun | 项目初始化时指定 |
| UI 组件库 | Naive UI | 适合桌面端，Vue 3 生态成熟 |
| 状态管理 | Pinia | Vue 3 官方推荐 |
| 路由 | Vue Router | 多视图切换 |
| 后端语言 | Rust | 与 hoi4skill 保持一致，性能与类型安全 |
| 数据库 | SQLite（rusqlite） | 项目元数据、音频库、电台配置持久化 |
| 音频处理 | ffmpeg + ffprobe | 转码与元数据探测 |
| 构建工具 | Tauri CLI + Cargo | 输出 Windows / Linux / macOS 安装包 |

---

## 3. 功能需求

### 3.1 Must Have（必须实现）

#### 项目管理
- 创建项目：指定名称、版本、支持的游戏版本、作者、输出目录
- 打开/关闭项目
- 删除项目
- 最近项目列表
- 项目元数据编辑

#### 音频库
- 导入本地音频文件（支持 mp3 / wav / flac / ogg）
- 自动转码为 Ogg Vorbis（44.1 kHz，立体声/单声道）
- 读取并显示音频元数据：时长、采样率、声道数、文件大小
- 编辑歌曲元数据：ID、标题、艺术家、音量、标签、注释
- 音频库搜索与筛选

#### 电台编辑
- 一个项目支持多个电台（music station）
- 创建 / 重命名 / 删除电台
- 从音频库向电台添加/移除歌曲
- 配置每个电台条目的播放概率：
  - 基础权重 `factor`
  - 国家条件 `tag = XXX`
  - 战争状态 `has_war = yes/no`
  - 阵营/意识形态等高级条件（可选）
- 电台内歌曲排序

#### Mod 生成
- 生成 `descriptor.mod`
- 生成启动器用 `.mod` 文件
- 为每个电台生成：
  - `music/<station_id>.asset` —— 歌曲注册
  - `music/<station_id>.txt` —— 电台分配与触发条件
- 复制转码后的 `.ogg` 到 `music/`
- 生成本地化文件 `localisation/simp_chinese/<project_id>_music_l_simp_chinese.yml`

#### 验证
- 检查所有 `.ogg` 文件存在且可解码
- 检查 `.asset` 中的 `name` 与 `.txt` 中的 `song` 一一对应
- 检查本地化键无缺失
- 检查 ID 命名合法（英文、数字、下划线）

#### 设置
- ffmpeg / ffprobe 路径配置
- HOI4 游戏目录配置（用于高级验证，可选）
- 主题切换（暗色/亮色）

### 3.2 Should Have（建议实现）

- 拖拽导入音频
- 音频预览播放
- 从 ID3 标签自动读取标题/艺术家
- 导出/导入项目模板
- 批量编辑歌曲属性

### 3.3 Nice to Have（可选）

- Steam Workshop 一键上传
- 多语言界面（中/英）
- 与 hoi4skill 共享 Clausewitz 索引进行 trigger 验证
- 音频波形可视化

---

## 4. 架构设计

### 4.1 整体分层

```
┌─────────────────────────────────────────────┐
│  Vue 3 前端（UI / 状态管理）                   │
│  - ProjectView                               │
│  - AudioLibraryView                          │
│  - StationEditorView                         │
│  - SettingsView                              │
│  - Pinia Store                               │
├─────────────────────────────────────────────┤
│  Tauri Commands（IPC 层）                      │
│  - project::create / open / save / delete    │
│  - audio::import / transcode / analyze       │
│  - station::create / add_song / update       │
│  - generator::build_mod                      │
│  - validator::validate                       │
├─────────────────────────────────────────────┤
│  Rust Core Modules                           │
│  - project.rs     项目持久化                  │
│  - audio.rs       音频元数据 / 转码           │
│  - station.rs     电台 / 歌曲条目 / 触发条件   │
│  - generator.rs   HOI4 文件生成               │
│  - validator.rs   生成结果验证                │
│  - hoi4.rs        HOI4 数据结构               │
│  - error.rs       错误类型                    │
└─────────────────────────────────────────────┘
```

### 4.2 数据模型

```rust
/// 一个电台 Mod 项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub version: String,
    pub supported_version: String,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub output_dir: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 音频库中的单个音频文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub source_path: PathBuf,
    pub ogg_filename: String,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u32,
    pub volume: f64,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

/// 电台
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub entries: Vec<StationEntry>,
}

/// 电台中的歌曲条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationEntry {
    pub audio_file_id: String,
    pub chance: ChanceConfig,
}

/// 播放概率配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceConfig {
    pub factor: f64,
    pub modifiers: Vec<Modifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    pub factor: Option<f64>,
    pub add: Option<f64>,
    pub base: Option<f64>,
    pub triggers: Vec<Trigger>,
}

/// Clausewitz 触发条件（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    HasWar { value: bool },
    Tag { value: String },
    HasGovernment { ideology: String },
    IsInFaction { tag: String },
}
```

### 4.3 输出文件结构

生成后的 Mod 目录示例：

```
my_radio_mod/
├── descriptor.mod
├── my_radio_mod.mod
├── music/
│   ├── station_main.asset
│   ├── station_main.txt
│   ├── station_war.asset
│   ├── station_war.txt
│   ├── song_001.ogg
│   ├── song_002.ogg
│   └── song_003.ogg
└── localisation/
    └── simp_chinese/
        └── my_radio_mod_music_l_simp_chinese.yml
```

#### `music/station_main.asset`

```hoi4
music = {
    name = "song_001"
    file = "song_001.ogg"
    volume = 0.75
}

music = {
    name = "song_002"
    file = "song_002.ogg"
    volume = 0.80
}
```

#### `music/station_main.txt`

```hoi4
music_station = "station_main"

music = {
    song = "song_001"
    chance = {
        factor = 2
        modifier = {
            factor = 0
            has_war = no
        }
    }
}

music = {
    song = "song_002"
    chance = {
        factor = 1
        modifier = {
            factor = 3
            tag = CHI
        }
    }
}
```

#### `localisation/simp_chinese/my_radio_mod_music_l_simp_chinese.yml`

```yaml
l_simp_chinese:
  song_001:0 "歌曲一"
  song_002:0 "歌曲二"
```

---

## 5. 关键流程

### 5.1 导入音频

1. 用户选择文件或拖拽文件到音频库
2. Rust 调用 `ffprobe -v quiet -print_format json -show_streams <file>` 读取元数据
3. 生成唯一 ID（如 `song_<timestamp>_<hash>`）
4. 保存原始文件路径和元数据到 SQLite
5. （可选）立即调用 ffmpeg 转码为 `.ogg`；或延迟到生成 Mod 时

### 5.2 编辑电台

1. 用户在左侧选择电台
2. 从音频库拖拽歌曲到电台区域
3. 在右侧属性面板编辑 factor 和 trigger
4. 每次变更通过 Tauri command 保存到 SQLite
5. 前端 Pinia store 同步更新

### 5.3 生成 Mod

1. 用户点击"生成 Mod"按钮
2. Rust 后端：
   - 清空或创建输出目录
   - 为每个电台生成 `.asset` 和 `.txt`
   - 复制/转码所有需要的 `.ogg`
   - 生成本地化文件
   - 生成 `descriptor.mod` 和 `.mod`
3. 返回生成结果摘要

### 5.4 验证

1. 扫描输出目录
2. 用 ffprobe 验证每个 `.ogg` 可解码
3. 检查 `.asset` 与 `.txt` ID 一致性
4. 检查本地化键存在
5. 输出验证报告

---

## 6. UI 布局

采用经典三栏布局：

```
┌─────────────────────────────────────────────────────────────┐
│  顶部工具栏：新建项目 | 打开 | 保存 | 生成 Mod | 设置         │
├───────────┬───────────────────────────────┬─────────────────┤
│           │                               │                 │
│ 项目列表   │        主编辑区                │    属性面板      │
│           │                               │                 │
│ - 项目 A  │  [音频库]  [电台编辑] [输出预览] │  选中项属性      │
│ - 项目 B  │                               │  - ID           │
│           │  音频库：                      │  - 标题          │
│           │  ┌─────┐ ┌─────┐ ┌─────┐     │  - 艺术家        │
│           │  │歌曲1│ │歌曲2│ │歌曲3│     │  - 音量          │
│           │  └─────┘ └─────┘ └─────┘     │  - 触发条件      │
│           │                               │                 │
│           │  电台编辑：                    │                 │
│           │  ┌───────────────────────┐   │                 │
│           │  │  station_main          │   │                 │
│           │  │ - 歌曲1  factor=2      │   │                 │
│           │  │ - 歌曲2  factor=1      │   │                 │
│           │  └───────────────────────┘   │                 │
│           │                               │                 │
└───────────┴───────────────────────────────┴─────────────────┘
```

---

## 7. 数据库 Schema（SQLite）

```sql
-- 项目表
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    supported_version TEXT NOT NULL,
    tags TEXT NOT NULL,           -- JSON 数组
    author TEXT,
    output_dir TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 音频文件表
CREATE TABLE audio_files (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    artist TEXT,
    source_path TEXT NOT NULL,
    ogg_filename TEXT NOT NULL,
    duration_secs REAL,
    sample_rate INTEGER,
    channels INTEGER,
    volume REAL NOT NULL DEFAULT 0.75,
    tags TEXT NOT NULL DEFAULT '[]',
    notes TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 电台表
CREATE TABLE stations (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 电台条目表
CREATE TABLE station_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    station_id TEXT NOT NULL,
    audio_file_id TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    chance_config TEXT NOT NULL,  -- JSON
    FOREIGN KEY (station_id) REFERENCES stations(id) ON DELETE CASCADE,
    FOREIGN KEY (audio_file_id) REFERENCES audio_files(id) ON DELETE CASCADE
);

-- 设置表
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

---

## 8. 错误处理策略

- 所有 Rust 后端函数返回 `Result<T, Hoi4RadioError>`
- 错误类型涵盖：IO、ffmpeg 调用、数据库、项目解析、生成失败、验证失败
- 前端统一显示错误提示，关键操作提供重试机制
- 日志写入 `logs/hoi4radio-<date>.log`

---

## 9. 测试策略

- **单元测试**：Rust 核心模块（generator、validator、hoi4 数据结构）
- **集成测试**：导入音频、生成 Mod、验证输出
- **端到端测试**：Tauri 前端关键用户流程（使用 Playwright 或 Tauri 测试工具）
- **手动测试**：在 HOI4 启动器中加载生成的 Mod，确认音乐正常播放

---

## 10. 发布与打包

- 使用 `tauri build` 输出：
  - Windows: `.msi` / `.exe`
  - Linux: `.AppImage` / `.deb`
  - macOS: `.dmg` / `.app`
- 首次启动检测 ffmpeg，若未安装则引导用户下载
- 可选：内置 ffmpeg 二进制到安装包中

---

## 11. 风险与注意事项

1. **ffmpeg 依赖**：需要检测系统 ffmpeg；首次使用若无则提示安装或下载。
2. **HOI4 版本兼容**：`supported_version` 和文件格式可能随游戏更新变化，保持可配置。
3. **版权问题**：工具不提供音乐，用户需自行准备合法音频素材。
4. **跨平台路径**：统一使用 `PathBuf` 处理路径，避免 Windows/Unix 路径差异。
5. **音频格式**：确保输出 Ogg Vorbis 44.1 kHz，避免游戏无法解码。

---

## 12. 后续演进方向

- 与 hoi4skill 共享 Clausewitz 索引，验证 trigger 合法性
- 支持更多音频源（如在线 URL、YouTube 下载）
- 社区模板市场
- Steam Workshop 自动上传
