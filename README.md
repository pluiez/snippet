# Snippet

> 中文版本：[`README.zh-CN.md`](README.zh-CN.md)

A Windows-first (macOS later) desktop text-template manager built on Tauri 2. Global hotkey → centered palette → fuzzy-search a template → fill variables (if any) → write to clipboard with optional auto-paste.

Static snippets and parameterized templates are the same data model — a static snippet is just a zero-variable template.

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
