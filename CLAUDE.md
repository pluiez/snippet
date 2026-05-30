# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository status

**Phase 2 complete through Slice 6.** Working app: scaffold + data layer + main-window CRUD + palette + variable fill + edit mode + colors + clipboard output with Windows autoPaste. See `PROGRESS.md` for per-slice notes (what landed, what was verified, known limitations, and spec decisions written back into SPEC/ARCHITECTURE).

**Next: Slice 7 — Onboarding + 完整设置页** (per `TASKS.md` §Phase 2). Current Settings page is a minimal stub with only autoPaste; Slice 7 adds hotkey (with conflict check + live re-register), theme (light/dark/system), dataFolderPath, plus the onboarding window for first-launch path selection.

## Picking up where the last session left off

1. Read `PROGRESS.md` first — it has the complete status of each slice including known limitations and unresolved punch-list items.
2. Project memory and global memory load automatically. Notable entries: project lives at `C:\dev\snippet` (Windows-native, not WSL), data folder convention uses `Snippet` subdir (not bundle identifier), repo language is Chinese.
3. Dev workflow is Windows-native — user runs `pnpm tauri dev` from PowerShell at `C:\dev\snippet`. Linux WSL is for editing only (this Claude session accesses via `/mnt/c/dev/snippet/`).

## The four documents and how to use them

The docs are layered — read them top-down when starting a task:

- **`SPEC.md`** — product fact-of-record. UI behavior, data contracts, interaction flows, and the 12 core invariants in §13 that MUST have unit tests. When SPEC and other docs disagree, SPEC wins.
- **`ARCHITECTURE.md`** — implementation skeleton: tech stack, process boundaries, module domains, key dependencies, startup/shutdown sequencing, and **§6 critical timing constraints** (HWND capture, clipboard read timing, write-event dedup). Specific file names, function signatures, and module organization are deliberately left to the implementer.
- **`TASKS.md`** — vertical-slice delivery plan. Each slice is end-to-end (UI + backend) and independently demoable. Slices have explicit "范围" (scope), "验收" (acceptance), and "不包含" (out-of-scope) — respect the out-of-scope to avoid scope creep across slices.
- **`DESIGN.md`** — a Pinterest design-system reference doc (tokens, components, do's/don'ts). It is *not* the spec for Snippet's UI. Treat it as input material for the `apply-design-system` / `audit-design-system` skills, or as a tokens/structure example, not as a description of what Snippet looks like. SPEC §6 (color system, OKLCh) and §4 (interaction flows) are the real UI source of truth.

The specs are written in Chinese; preserve that language when editing them.

## Product in one paragraph

Snippet is a Windows-first (macOS later) Tauri v2 desktop app: global hotkey → centered palette → fuzzy-search a template → if it has variables, fill a form → write the rendered result to clipboard (and optionally auto-paste to the previously-focused window). Static snippets and parameterized templates are the same data model — a static snippet is just a zero-variable template.

## Architecture invariants worth pinning

- **Backend is the source of truth.** Rust owns persistence, hotkey, clipboard, paste, search index, color generation, GC, file watching. Frontend is pure view + control — it never touches the filesystem or OS APIs directly.
- **IPC commands are business actions, not file ops** (`save_template`, `render_template` — not `write_file`).
- **Type contract is shared via `ts-rs`.** Rust types derive `ts-rs`; frontend imports the generated `.ts`. CI must verify sync.
- **Variables are GUID-stable.** Bodies store `{<guid>}` placeholders, not display names. Renaming a variable's display name must not lose filled values or break body placeholders. (SPEC invariants 1, 2.)
- **Two color maps, two files, never merged.** `variableColorMap` and `tagColorMap` are independent — `language` tag and `Language` variable must not collide.
- **Color map GC runs at startup and shutdown.** No reference counting; convergence is by periodic scan only.

## Critical timing (ARCHITECTURE §6 — get these wrong and the app breaks)

- **HWND capture must be the first synchronous step in the hotkey callback** — before any UI work, before any async dispatch. Otherwise the cached "previous window" is the palette itself.
- **Clipboard read for `fillFromClipboard` happens when the fill dialog opens** — not when the hotkey fires. Reading earlier loses what the user copied during palette navigation.
- **`notify` watcher events must be deduped against the app's own writes** (ignore-set or debounce) to prevent self-triggered refresh loops.

## Common workflows

All commands from repo root.

| Command | What it does |
|---|---|
| `pnpm install` | Install frontend deps. Run once per machine. |
| `pnpm tauri dev` | Dev server with hot reload (frontend) + Rust auto-rebuild. |
| `pnpm tauri build` | Production build (Windows: `.msi` / `.exe`). |
| `pnpm bindings` | Regenerate `src/lib/bindings/*.ts` from Rust types with `#[derive(TS)]` + `#[ts(export)]`. Wraps `cd src-tauri && cargo test`. |
| `cd src-tauri && cargo check` | Fast Rust typecheck without producing binaries. |
| `cd src-tauri && cargo test` | Rust unit tests; also regenerates ts-rs bindings as a side effect. |

### Dev platform

Target is Windows (WebView2). Convention: develop and run on Windows native. WSL is fine for editing but Linux dev would need `webkit2gtk-4.1` + `librsvg2` apt packages, and Windows-specific behavior (hotkey, paste, clipboard) won't fully match.

### After editing Rust types

Anything `#[derive(TS)]` + `#[ts(export, export_to = "../src/lib/bindings/")]` is regenerated by `pnpm bindings`. Bindings are committed to git so a fresh clone compiles without running cargo first. The build does NOT auto-regenerate — re-run `pnpm bindings` after changing a derived type.

### Bundle identifier

`app.snippet` in `src-tauri/tauri.conf.json`. Change to a real reverse-domain before first signed release.

### Logging

`tracing` + `tracing-subscriber` initialized in `lib.rs::init_tracing()`. Override the default filter via `RUST_LOG` env var (e.g. `RUST_LOG=snippet_lib=trace pnpm tauri dev`).

## When implementing a slice

1. Start from the slice's "范围" / "验收" / "不包含" in `TASKS.md`. Do not pull work in from later slices.
2. Cross-reference SPEC for behavioral details, ARCHITECTURE for structural constraints.
3. Cover the relevant SPEC §13 invariants with Rust unit tests in the same slice (remaining invariants land in Phase 3 工作流 C).
4. Slices are tagged in git on completion for easy rollback.
