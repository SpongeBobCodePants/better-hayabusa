# Better Hayabusa

A local desktop application that acts as a graphical UI for the command-line forensics tools [Hayabusa](https://github.com/Yamato-Security/hayabusa) and [Chainsaw](https://github.com/WithSecureLabs/chainsaw). By [Merciless Software](https://github.com/SpongeBobCodePants) — *making your life suck less*.

> **Status: Early development.** Milestone 1 (Foundation) is complete: Tauri shell, OS gate, SQLite-backed app state, sidebar shell, About page. Projects, jobs, runs, tool downloads, and the in-app CLI reference are not implemented yet. See [docs/superpowers/roadmap.md](docs/superpowers/roadmap.md) for what's planned and [the open Issues](../../issues) for current work.

**Windows-only in v1.** Architecture is ready for cross-platform expansion later.

## Why this exists

Hayabusa and Chainsaw are excellent at what they do, but their CLIs are dense and the typical analyst workflow is a sprawl of one-off PowerShell scripts per engagement. Better Hayabusa replaces those scripts with a small project-and-jobs UI: define a job once, point it at one or more hosts' EVTX folders, run it, browse the results.

The app is intentionally **portable**: no installer, no registry writes, no `%APPDATA%`. State lives in `app.db` next to the executable. Drop the folder on a USB stick, run it from there.

## Status detail

| Milestone | Status | Scope |
|---|---|---|
| **M1 — Foundation** | ✅ Done | Tauri scaffold, OS gate, `app.db` + migrations, sidebar, About |
| **M2 — Projects** | Next | Create / open / switch projects, recent-projects list |
| **M3 — Jobs + targets** | | Configure named jobs with target host lists (no execution yet) |
| **M4 — Run execution** | | Process spawning, live stdout, run history, per-host status |
| **M5 — Help system + Settings** | | `?` drawer, info icons, theme switcher, log retention |
| **M6 — Tools reference pages** | | Searchable in-app reference for Hayabusa and Chainsaw |
| **M7 — Tool acquisition** | | Download / update Hayabusa, Chainsaw, and their rule packs |
| **M8 — Portable distribution + release polish** | | Real portable Windows zip; v1.0 cut |

Track current work in [GitHub Milestones](../../milestones) and [open Issues](../../issues).

## Building from source

### Prerequisites
- Node.js 20+ and [pnpm](https://pnpm.io/) 9+
- Rust toolchain (`rustup`) with the `x86_64-pc-windows-msvc` target
- Microsoft Edge WebView2 Runtime (pre-installed on Windows 10 1803+ and Windows 11)
- Visual Studio Build Tools with "Desktop development with C++"

See [Tauri's prerequisites](https://v2.tauri.app/start/prerequisites/) for full details.

### Commands

```sh
pnpm install              # install frontend deps
pnpm tauri dev            # dev mode (Rust + Vite + Svelte HMR, opens a window)
cd src-tauri && cargo test  # Rust tests (also runs ts-rs type generation)
pnpm tauri build --debug  # debug bundle (faster, includes debug symbols)
pnpm tauri build          # release bundle
```

Frontend tests (Vitest) and E2E tests (Playwright) land in later milestones.

## Architecture

- **Frontend** (`src/`) — SvelteKit + shadcn-svelte + Tailwind 4. View layer only; never spawns processes, touches SQLite, or makes HTTP requests.
- **Backend** (`src-tauri/src/`) — Rust. Owns all heavy work: process orchestration, SQLite (via `rusqlite`), HTTP (via `reqwest`), filesystem.
- **Cross-language types** — Rust structs derive [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs) bindings emitted to `src/lib/generated/`. The frontend never hand-writes types that mirror Rust.
- **IPC** — frontend calls Rust via `invoke()` against `#[tauri::command]` handlers. The data layer is API-first: pure Rust functions are tested directly; command wrappers stay thin.

Full design rationale: [docs/superpowers/specs/](docs/superpowers/specs/). Build conventions: [CLAUDE.md](CLAUDE.md).

## Contributing

This is a small project still finding its shape. If you're interested in contributing, browse the [open Issues](../../issues) — items tagged `good first issue` are the friendliest entry points. PRs welcome against `main`; please ensure `cargo test` and `pnpm build` both pass before opening.

## Acknowledgements

- [Hayabusa](https://github.com/Yamato-Security/hayabusa) (Yamato Security) — the underlying Sigma-rule-driven EVTX analyzer
- [Chainsaw](https://github.com/WithSecureLabs/chainsaw) (WithSecure Labs) — fast EVTX/registry hunting
- [SigmaHQ Sigma rules](https://github.com/SigmaHQ/sigma)
- [Tauri](https://tauri.app), [Svelte](https://svelte.dev), [shadcn-svelte](https://www.shadcn-svelte.com)

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
© 2026 Merciless Software.
