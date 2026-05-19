# Better Hayabusa/ChainSaw

A local desktop application that acts as a graphical UI for the command-line forensics tools **Hayabusa** and **Chainsaw**. By Merciless Software — *making your life suck less*.

**Windows-only** in v1; architecture is ready for cross-platform expansion later.

## Where things live

- **Design spec (source of truth):** [docs/superpowers/specs/2026-05-19-better-hayabusa-chainsaw-design.md](docs/superpowers/specs/2026-05-19-better-hayabusa-chainsaw-design.md). Read this first. Most "how does X work" questions are answered there.
- **Backlog (deferred work):** [docs/superpowers/backlog.md](docs/superpowers/backlog.md). Items deferred from earlier milestones, organized by target milestone. **When drafting a new milestone's plan, read this first** and either fold items in or push them to a later section. When closing an item, delete the line.
- **Frontend:** `src/` — SvelteKit + shadcn-svelte + Tailwind. View layer only.
- **Backend (Rust):** `src-tauri/src/` — Tauri command handlers, process orchestration, SQLite, HTTP downloads, tool registry.
- **Per-tool modules:** split into two layers — `src-tauri/src/tools/references/<executable>.rs` owns all option metadata for an executable (drives the reference page); `src-tauri/src/tools/jobs/<job_type>/` owns what's job-type-specific (settings, defaults, command builder, output paths). Frontend forms are at `src/lib/tools/<job_type>/Form.svelte`.
- **Migrations:** `src-tauri/migrations/{app,project}/` — numbered SQL files, one set per database.
- **Help content:** `src/lib/help/` — Markdown rendered in UI popovers and drawers.

## Conventions that won't change (don't fight these)

- **Rust owns all heavy work.** The frontend never spawns processes, never touches SQLite, never touches files, never makes HTTP requests. It calls Tauri commands via `invoke()`.
- **API-first Rust.** Every backend capability is designed as a plain Rust API (pure functions or `impl` methods on data types) *first*, then wrapped in a thin `#[tauri::command]` handler. The API is not exposed externally — this is a design discipline, not a deployment goal. Consequences: business logic lives in `app_db::`, `tools::`, `paths::`, `platform::`, etc.; `commands::*` modules only marshal arguments, lock state, call the API, and map errors to `String`. Tests target the API, not the command wrappers (commands need a live Tauri runtime and are awkward to unit test).
- **Single source of truth for tool option metadata.** The per-executable Rust `tools/references/<executable>.rs` catalog drives form tooltips, run-info text files, and the in-app CLI reference page. If you're editing tool option help anywhere, edit the catalog and let it propagate.
- **Single source of truth for cross-language types.** Rust structs derive `ts-rs` definitions emitted to `src/lib/generated/`. Do not hand-write TypeScript types that mirror Rust structs.
- **No Tauri plugin-sql / plugin-shell / plugin-http / plugin-fs.** Those concerns live in Rust (`rusqlite`, `tokio::process::Command`, `reqwest`, `std::fs`). Only `plugin-dialog` and `plugin-os` are used.
- **No path string-concatenation in frontend.** Paths come from Rust as already-resolved strings; the frontend displays them but does not build them. Rust uses `std::path::{Path, PathBuf}` exclusively.
- **Rust re-validates every input.** Frontend Zod is form-time UX only. Do not skip server-side validation because "the frontend already checked".
- **No shell wrappers.** Spawn executables directly via `tokio::process::Command`. No `cmd.exe`, no PowerShell.
- **Process stdout/stderr** is appended to disk on every line AND batched at ~10Hz for IPC to the frontend. Don't emit per-line events.
- **Help is opt-in.** Never use modals, banners, or "did you know?" popups. Info is available behind icons and a `?` drawer; the user must click.
- **The portable model is sacred.** No installer, no Program Files, no registry, no `%APPDATA%`. App state lives in `app.db` next to the exe.

## Running and building

- `pnpm install` — install frontend deps (run once per clone, and after `pnpm-lock.yaml` changes)
- `pnpm tauri dev` — dev mode (Rust + Vite + Svelte HMR, opens a window)
- `cd src-tauri && cargo test` — Rust tests (also runs `ts-rs` type generation)
- `pnpm tauri build --debug` — debug bundle (faster, includes debug symbols)
- `pnpm tauri build` — release bundle (slower, used for distribution)

> Frontend tests (Vitest) and E2E tests (Playwright) land in later milestones.

## License

MIT. See [LICENSE](LICENSE).
© 2026 Merciless Software.
