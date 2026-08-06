# HOI4 Radio Maker

[**简体中文**](README.md) | [English](README.en.md)

> 为《钢铁雄心 IV》（Hearts of Iron IV）模组作者打造的音乐/电台 Mod 制作桌面工具。

从零开始创建和管理 HOI4 音乐/电台 Mod：导入音频自动转码为游戏兼容的 Ogg Vorbis，组合电台并配置播放概率与触发条件，一键生成完整的 Mod 目录，并验证输出质量。

![tech-badge](https://img.shields.io/badge/Tauri%202-Rust-5b5f66)
![tech-badge](https://img.shields.io/badge/Vue%203-TypeScript-42b883)
![license-badge](https://img.shields.io/badge/license-GPL--3.0-blue)

## 功能特性

- **项目管理**：创建 / 编辑 / 删除项目；默认目录、作者、版本、标签可全局预填；支持游戏版本自动从 HOI4 安装目录（`launcher-settings.json`）探测
- **全局音频库**：批量导入 mp3 / flac / wav / ogg / m4a / aac / wma；BLAKE3 内容哈希去重，重复导入直接建立引用、不重复转码
- **自动转码**：导入即转码为 HOI4 兼容的 Ogg Vorbis（44.1 kHz、强制立体声、libvorbis 质量 4）；并发导入、逐文件进度与取消
- **元数据**：自动读取 ID3 等嵌入标签的标题 / 艺术家；支持单条与批量编辑（标题 / 艺术家 / 音量 / 标签 / 备注）
- **电台编辑**：一个项目多个电台；每首歌曲配置播放权重（`factor`）与触发条件（`tag`、`has_war`、`has_government`、`is_in_faction`）；电台内排序
- **一键生成**：输出完整 Mod 目录——`descriptor.mod`、启动器 `.mod`、`music/<station>.asset` + `.txt`、简体中文本地化文件（UTF-8 BOM）
- **输出验证**：检查 OGG 可解码性、`.asset` 与 `.txt` ID 一致性、本地化键完整性、ID 命名合法性（可配置 ffprobe 路径）
- **诊断**：前后端统一日志（stdout + 应用数据目录 + 开发期 Webview），设置页可一键打开日志目录

## 技术栈

| 层级 | 技术 |
|---|---|
| 桌面框架 | [Tauri 2](https://tauri.app/) |
| 前端 | Vue 3（`<script setup>`）+ TypeScript + [Vite](https://vitejs.dev/) |
| UI | [Vuetify 3](https://vuetifyjs.com/)（Material Design 3，自定义暗色主题） |
| 状态管理 | Pinia · 路由 Vue Router |
| 后端 | Rust（edition 2021） |
| 数据库 | SQLite（rusqlite） |
| 音频处理 | ffmpeg / ffprobe |

## 环境要求

- [Rust](https://www.rust-lang.org/tools/install)
- [Bun](https://bun.sh/docs/installation)
- 系统 `ffmpeg` / `ffprobe`（首次启动自动探测，也可在设置中手动指定路径）

系统级依赖（WebKitGTK 等）参见 [Tauri 环境要求](https://tauri.app/start/prerequisites/)。

## 快速开始

```bash
bun install          # 安装前端依赖（Bun，锁文件 bun.lock）
bun run tauri dev    # 桌面开发：Vite (端口 1420) + Rust 后端
```

## 开发

```bash
bun run build        # 类型门禁：vue-tsc --noEmit && vite build
cd src-tauri && cargo test          # 全部 Rust 测试
cd src-tauri && cargo test --lib    # 仅单元测试
cd src-tauri && cargo clippy -- -D warnings
RUST_LOG=debug bun run tauri dev    # 后端 debug 级日志
```

## 构建

```bash
bun run tauri build                 # 桌面发布包（Windows / Linux / macOS）
bun run tauri android dev | build   # Android 目标
```

## 项目结构

```
src/           Vue 前端（views / components / stores / api）
src-tauri/     Rust 后端（commands / db / generator / validator / audio …）
  └─ tests/    集成测试
docs/superpowers/  设计文档（specs）、实现计划（plans）、路线图（roadmap）
references/radio-mod-template/    HOI4 电台 Mod 输出格式参考
```

## 文档

- [设计规格](docs/superpowers/specs/2026-06-12-hoi4radio-design.md)
- [实施路线图](docs/superpowers/roadmap.md)
- [实现计划](docs/superpowers/plans/2026-06-12-hoi4radio-implementation-plan.md)
- [参与贡献](CONTRIBUTING.md)

## 许可证

本项目基于 [GNU General Public License v3.0](LICENSE) 发布。
