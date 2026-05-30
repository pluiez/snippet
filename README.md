# Snippet

A Windows-first (macOS later) desktop text-template manager built on Tauri 2. Global hotkey → centered palette → fuzzy-search a template → fill variables (if any) → write to clipboard with optional auto-paste.

Static snippets and parameterized templates are the same data model — a static snippet is just a zero-variable template.

> [中文版本见下方 / Chinese version below ↓](#中文)

## Status

Phase 2 complete (Slice 0 through 7c). Phase 3 工作流 A (animations) and 工作流 B (error handling) complete. Phase 3 工作流 C: §13 invariant unit-test infrastructure + storage-layer integration tests complete; Windows `.msi` packaging and manual smoke test remain.

`cargo test --lib` = 78 passing.

See [`HANDOFF.md`](HANDOFF.md) for the full current handoff document.

## Tech Stack

- **Frontend**: Tauri 2.x · React 19 · TypeScript · Tailwind CSS v4 · Vite · framer-motion
- **Backend**: Rust 2021 with `nucleo-matcher` (fuzzy search), `pinyin` (Chinese pinyin), `enigo` (Windows-only keyboard simulation for auto-paste), `regex`, `chrono`, `rand`
- **Type sync**: `ts-rs` generates TypeScript bindings from Rust types on `cargo test`
- **Tauri plugins**: `global-shortcut`, `dialog`, `clipboard-manager`, `single-instance`

## Dev Quick Start

Target runtime is **Windows native** (WebView2). Development convention: edit from **Ubuntu on WSL 2** at `/mnt/c/dev/snippet/`, run from PowerShell at `C:\dev\snippet`. WSL is fine for editing and `cargo test --lib`; `pnpm tauri dev` must run from Windows because of WebView2 / hotkey / clipboard requirements.

```powershell
# From C:\dev\snippet in Windows PowerShell
pnpm install       # once per machine
pnpm bindings      # rebuild Rust + regenerate TS bindings (= cd src-tauri && cargo test)
pnpm tauri dev     # dev server with hot reload + Rust auto-rebuild
pnpm tauri build   # production build (.msi / .exe — Windows)
```

First-launch state lives in `%APPDATA%\Snippet\` (bootstrap.json, settings.json, templates/, etc.). Clear that directory to retrigger the onboarding flow.

## Documentation Map

Read in priority order:

1. **[`CLAUDE.md`](CLAUDE.md)** — AI coding agent guide (dev commands, architecture invariants, key conventions). Start here.
2. **[`PROGRESS.md`](PROGRESS.md)** — per-slice implementation log: what landed, verification scenarios, known caveats, patch history. Highest information density.
3. **[`SPEC.md`](SPEC.md)** (Chinese) — product spec: UI behavior, data contracts, interaction flows, §13 core invariants. **SPEC is the source of truth.**
4. **[`ARCHITECTURE.md`](ARCHITECTURE.md)** (Chinese) — implementation skeleton: tech stack, process boundaries, module domains, §6 critical timing constraints.
5. **[`TASKS.md`](TASKS.md)** (Chinese) — vertical-slice delivery plan.
6. **[`HANDOFF.md`](HANDOFF.md)** (Chinese) — current handoff document for the next AI agent.

The specs are written in Chinese; preserve that language when editing them.

## License

MIT — see [`LICENSE`](LICENSE).

---

<a name="中文"></a>

## 中文

Snippet 是一个 Windows-first（macOS later）桌面文本模板管理工具，基于 Tauri 2 构建。全局热键 → 居中 palette → 模糊搜索模板 → 填变量 → 写剪贴板（可选自动粘贴）。

静态片段和带参数的模板是同一种数据模型 —— 静态片段就是零变量的模板。

## 项目状态

Phase 2 完成（Slice 0 到 7c）。Phase 3 工作流 A（动画）和工作流 B（错误处理）已落地。工作流 C：§13 不变量单测基础设施 + storage 层集成测试已落地；剩 Windows `.msi` 打包和手动 smoke test。

`cargo test --lib` = 78 通过。

完整交接信息见 [`HANDOFF.md`](HANDOFF.md)。

## 技术栈

- **前端**：Tauri 2.x · React 19 · TypeScript · Tailwind CSS v4 · Vite · framer-motion
- **后端**：Rust 2021；关键 crate：`nucleo-matcher`（fuzzy）、`pinyin`（中文）、`enigo`（Windows 键盘模拟）、`regex`、`chrono`、`rand`
- **类型同步**：`ts-rs` 在 `cargo test` 时从 Rust 类型生成 TypeScript bindings
- **Tauri 插件**：`global-shortcut`、`dialog`、`clipboard-manager`、`single-instance`

## 开发快速启动

目标运行环境是 **Windows native**（WebView2）。开发约定：从 **Ubuntu on WSL 2** 的 `/mnt/c/dev/snippet/` 编辑，从 Windows PowerShell 的 `C:\dev\snippet` 运行。WSL 适合编辑和跑 `cargo test --lib`；`pnpm tauri dev` 必须在 Windows 跑（依赖 WebView2 / 热键 / 剪贴板）。

```powershell
# 在 Windows PowerShell 的 C:\dev\snippet 下
pnpm install       # 每台机器一次
pnpm bindings      # 重建 Rust + 生成 TS bindings（= cd src-tauri && cargo test）
pnpm tauri dev     # 开发服务器 + Rust 热重建
pnpm tauri build   # 生产构建（.msi / .exe）
```

首次启动状态存在 `%APPDATA%\Snippet\`（bootstrap.json、settings.json、templates/ 等）。清空该目录可重新触发 onboarding 流程。

## 文档地图

按优先级阅读：

1. **[`CLAUDE.md`](CLAUDE.md)** — AI coding agent 指南（dev 命令、架构不变量、关键约定）。**先看这个。**
2. **[`PROGRESS.md`](PROGRESS.md)** — 每个 slice 的实施记录：实现了什么、验证场景、已知坑、patch 历史。信息密度最高。
3. **[`SPEC.md`](SPEC.md)**（中文）— 产品 spec：UI 行为、数据契约、交互流程、§13 核心不变量。**SPEC 是事实标准。**
4. **[`ARCHITECTURE.md`](ARCHITECTURE.md)**（中文）— 实现骨架：技术栈、进程边界、模块域、§6 关键时序约束。
5. **[`TASKS.md`](TASKS.md)**（中文）— 切片交付计划。
6. **[`HANDOFF.md`](HANDOFF.md)**（中文）— 当前交接文档。

Spec 和 UI 文字使用中文；编辑时请保持。

## 许可证

MIT — 见 [`LICENSE`](LICENSE)。
