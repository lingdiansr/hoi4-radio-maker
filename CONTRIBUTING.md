# 贡献指南

感谢你对 **hoi4-radio-maker** 的兴趣！本指南将帮助你快速了解项目技术栈、开发流程和贡献规范。

## 项目简介

`hoi4-radio-maker` 是一个基于 Tauri 2 的跨平台桌面应用，帮助《钢铁雄心 IV》MOD 作者创建和管理音乐/电台 Mod。

- **桌面框架**：Tauri 2（Rust 后端 + Web 前端）
- **前端**：Vue 3 + TypeScript + Vite
- **包管理器**：Bun
- **后端语言**：Rust

## 开始前准备

### 必需环境

- [Rust](https://www.rust-lang.org/tools/install)（最新稳定版）
- [Bun](https://bun.sh/docs/installation)
- [ffmpeg](https://ffmpeg.org/download.html) 与 [ffprobe](https://ffmpeg.org/download.html)（用于音频处理，开发阶段可选）

### 推荐工具

- [VS Code](https://code.visualstudio.com/)
  - [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 项目结构

```
hoi4-radio-maker/
├── src/                    # Vue 3 前端源码
│   ├── App.vue
│   ├── main.ts
│   └── assets/
├── src-tauri/              # Tauri Rust 后端
│   ├── src/                # Rust 源码
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── icons/
├── docs/                   # 设计文档与实现计划
│   └── superpowers/
├── package.json            # 前端依赖（Bun）
├── vite.config.ts          # Vite 配置
├── tsconfig.json           # TypeScript 配置
├── README.md
├── AGENTS.md               # 面向 Agent 的开发速查
└── LICENSE                 # GPL-3.0
```

## 开发环境搭建

```bash
# 1. 克隆仓库
cd /home/ldsr/code/Rust/hoi4-radio-maker

# 2. 安装前端依赖
bun install

# 3. 启动桌面开发模式
bun run tauri dev
```

如需 Android 开发，先执行：

```bash
bun run tauri android init
bun run tauri android dev
```

## 开发工作流

### 1. 创建分支

```bash
git checkout -b feature/your-feature-name
```

### 2. 编写代码

- **前端代码**放在 `src/`
- **后端代码**放在 `src-tauri/src/`
- 新增 Tauri Command 时，记得在 `src-tauri/src/lib.rs` 中注册
- 修改 capability 时，同步更新 `src-tauri/capabilities/`

### 3. 运行检查

前端类型检查：

```bash
bun run build
```

Rust 检查：

```bash
cd src-tauri && cargo check && cargo clippy
```

### 4. 测试

```bash
# 前端构建验证
bun run build

# Rust 单元测试与集成测试
cd src-tauri && cargo test
```

### 5. 提交代码

请遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>
```

常用类型：

- `feat`：新功能
- `fix`：修复 Bug
- `docs`：文档更新
- `style`：代码格式（不影响功能）
- `refactor`：重构
- `perf`：性能优化
- `test`：测试相关
- `chore`：构建/工具/依赖调整

示例：

```bash
git commit -m "feat(audio): add ffprobe metadata extraction"
git commit -m "fix(ui): resolve station list rendering on empty state"
git commit -m "docs: update setup instructions for Bun"
```

## 代码风格

### TypeScript / Vue

- 使用 `<script setup>` 单文件组件
- 优先使用 Composition API
- 类型定义明确，避免 `any`
- 组件名使用 PascalCase，文件名为多词形式

### Rust

- 遵循 `cargo fmt` 默认格式
- 使用 `cargo clippy` 检查常见代码问题
- 错误处理统一使用自定义 `Hoi4RadioError`，Tauri Command 返回 `Result<T, Hoi4RadioError>`
- 使用 `tracing::info!` / `warn!` / `error!` 记录日志，避免直接使用 `println!`
- 路径处理使用 `std::path::PathBuf`

### 错误与日志规范

- **后端**：所有错误通过 `src-tauri/src/error.rs` 中的 `Hoi4RadioError` 统一表达
- **后端**：日志写入 `stdout`（开发期）和 `<data_dir>/hoi4-radio-maker/logs/hoi4-radio-maker.log`（滚动日志）
- **前端**：调用 Tauri Command 时统一处理错误，使用 Vuetify 的 `v-snackbar` 配合全局 toast store 或全局错误处理显示提示
- **环境变量**：`RUST_LOG` 控制 Rust 日志级别，如 `RUST_LOG=debug bun run tauri dev`

## 添加依赖

### 前端依赖

```bash
bun add <package>
bun add -d <dev-package>
```

### Rust 依赖

```bash
cd src-tauri && cargo add <crate>
```

## 设计文档

较大改动前，请先阅读：

- `docs/superpowers/specs/2026-06-12-hoi4radio-design.md`
- `docs/superpowers/plans/2026-06-12-hoi4radio-implementation-plan.md`

如果你要新增功能或修改架构，建议先在 `docs/superpowers/` 中更新对应文档。

## 提交 Pull Request

1. 确保本地分支基于最新 `main`（或默认分支）
2. 运行构建与测试，确认无错误
3. 清晰描述 PR 目的、改动范围和测试方式
4. 关联相关 Issue（如有）

## 许可证

本项目采用 [GNU General Public License v3.0](LICENSE)。

提交代码即表示你同意在 GPL-3.0 许可证下发布你的贡献。
