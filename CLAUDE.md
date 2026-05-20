# Better Hayabusa

A local desktop application that acts as a graphical UI for the command-line forensics tools **Hayabusa** and **Chainsaw**. By Merciless Software — *making your life suck less*.

**Windows-only** in v1; architecture is ready for cross-platform expansion later.

## Process

This project uses **Claude Superpowers**. TDD, brainstorming, plan-writing, plan-execution, code review, verification-before-completion, and parallel-agent dispatch are handled by skills — don't restate them here. If a skill applies, use it.

**Fallback if Superpowers is unavailable or a skill is missed:** follow `superpowers:test-driven-development` (write a failing test before implementation code) and `superpowers:verification-before-completion` (run the relevant tests/builds before declaring done). Load-bearing only when the skills don't fire.

**Shipping approved work:** when the user has explicitly approved a completed unit of work for merge to `main`, they run `/ship` ([.claude/commands/ship.md](.claude/commands/ship.md)). It handles PR creation, Codex review request with retry/poll/feedback loop, fix cycles (max 5), and merge. Do NOT run `/ship` proactively — wait for the user trigger.

## Behavioral guidelines

- **When the request is ambiguous:** enumerate interpretations; don't pick silently. If a simpler approach exists, propose it before implementing the asked-for one. If something is unclear, name what's confusing — asking is cheaper than guessing.
- **When editing existing code:** don't "improve" adjacent code, comments, or formatting while you're in the file. Pre-existing dead code: mention it, don't delete it. Only remove imports/variables/functions that *your* changes orphaned.
- **When reviewing diffs:** flag, don't fix. Code review output is for the implementer to act on — making changes during review pollutes the diff being reviewed.
- **When something looks inconsistent:** check the backlog (GitHub Issues) before "fixing" it. A lot of "inconsistencies" are intentional defers that already have an Issue.

## Where things live

- **Design spec (source of truth):** [docs/superpowers/specs/2026-05-19-better-hayabusa-chainsaw-design.md](docs/superpowers/specs/2026-05-19-better-hayabusa-chainsaw-design.md). Read this first. Most "how does X work" questions are answered there.
- **Roadmap:** [docs/superpowers/roadmap.md](docs/superpowers/roadmap.md). The milestone breakdown (M1–M8) and pointers to GitHub Issues/Milestones for current work tracking.
- **Backlog and per-milestone work:** [GitHub Issues](https://github.com/SpongeBobCodePants/better-hayabusa/issues). Filter by Milestone (e.g., `is:open milestone:"M2 - Projects"`) to see committed work; `is:open no:milestone` for un-triaged items; the `needs-spec-decision` label flags items needing a design call before they can be planned. **When drafting a new milestone's plan, read the milestone's open Issues first** and either fold them in, push them to a later milestone, or close them as obsolete.
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
- **Display name vs short code.** Display strings (window title, page headers, About, README, this file) use **"Better Hayabusa"**. Internal short codes — Cargo package `bhc`, library `bhc_lib`, Tauri identifier `com.mercilesssoftware.bhc` — stay as `bhc` even though the app was renamed from "Better Hayabusa/ChainSaw". Don't propose renaming the internal codes unless asked.
- **Per-project metadata folder is `.bh/`** (not `.bhc/`). Lives inside each project's timestamped folder and holds `project.db` + `activity.log` (plus SQLite WAL artifacts).
- **Project folder layout.** When a user creates a project named `<name>` and picks a parent location, the backend creates `<parent>/<name>__YYYY.MM.DD_HHMMSS/` (UTC timestamp) and places `.bh/` inside that. The user-picked path is the *parent* directory, not the project root.
- **Public repo; never commit real engagement data.** Real client names, real hostnames, real IPs, real file paths from engagements MUST NOT be in tracked files. Synthetic test fixtures use placeholders like `WORKSTATION-01`, `CLIENT-A`, `192.0.2.0/24`. Anything resembling real data goes in the gitignored `/private/` folder (kept locally for ad-hoc testing). Before any first push of new public content, grep history for the data.

## Stack (do not swap or "just try" alternatives without asking)

- **Tauri 2.x** / **Rust** stable (no `rust-toolchain.toml` yet — uses whatever `rustup` default is; add one if version drift becomes an issue)
- **SQLite via rusqlite 0.31+** with the `bundled` feature (compiles SQLite from source — no system dep)
  - Chosen over sqlx because this is a single-user desktop app: no async concurrency story, no multi-DB portability need, sqlx's compile-time `query!` macro requires a live DB at build time which is awkward for a shipped desktop binary. Don't propose sqlx or `tauri-plugin-sql` without flagging it first.
- **SvelteKit 2.x** + **Svelte 5** (runes) + **TypeScript 5.6+** (strict)
- **Tailwind 4** + **shadcn-svelte** — reach for shadcn primitives before hand-rolling a component
- **ts-rs 10** for cross-language type generation (Rust → TS)
- **Tauri plugins:** `plugin-dialog`, `plugin-os` only. No `plugin-sql`, `plugin-shell`, `plugin-http`, `plugin-fs` — those concerns live in Rust.
- **Not yet adopted (will add when needed):** form library (formsnap + sveltekit-superforms + zod), tables (TanStack Table), icons (currently using @lucide/svelte via shadcn-svelte default).

## Dependencies

- **Package manager: pnpm.** Never run `npm` or `yarn`; never commit `package-lock.json` or `yarn.lock`.
- **Minimum package age: 7 days.** Never install a package whose newest version was published less than 7 days ago — pin to an older version or wait. Supply-chain hygiene: recent npm worms and crypto-stealer compromises both shipped via brand-new versions. Configure pnpm's `minimumReleaseAge` setting.
- **Before adding any new runtime dependency, ask.** Dev dependencies for tooling (eslint, prettier, vitest, etc.) are fine to add unprompted as long as the 7-day rule holds.

## Code rules

- **TypeScript:** `strict: true`. No `any` — use `unknown` and narrow. No `// @ts-ignore` (use `// @ts-expect-error` with a reason if truly necessary). No null/undefined checks unless the value can actually be null — trust the type system.
- **Rust:** no `.unwrap()` or `.expect()` outside tests and `build.rs`. **Exception:** the Tauri `setup` closure in `lib.rs` may use `.expect()` for unrecoverable startup failures (DB open, path resolution) — those happen exactly once at boot and a panic is more visible than silently broken state. `#[tauri::command]` handlers MUST return `Result<T, String>` (or a typed error enum, see Issue #6) and never panic — a panic in a command crashes the webview.
- **Error handling:** no `try/catch` (TS) or `match`-on-Result-just-to-rewrap (Rust) around internal function calls. Trust your own code. Handle errors only at system boundaries (user input, network, filesystem, IPC) and only when the failure has a meaningful response.
- **SQL:** parameterized queries only. No string interpolation into SQL, ever, even for "trusted" input.
- **Frontend ↔ backend boundary:** components never call `invoke()` directly. Every Tauri command gets a typed wrapper in `src/lib/ipc/` and is imported from there. This keeps the IPC surface auditable in one place.
- **Secrets:** if/when secrets enter the picture (download mirror credentials, future cloud integrations), never log, hardcode, or commit them. Use OS keychain (Tauri stronghold plugin or the `keyring` crate) — not SQLite, not env files, not localStorage. M1 has no secrets; this rule is preventive.
- **Svelte:** group related controls into a component when this would be more efficient and make code more readable and reusable.

## Repo setup (required for any clone)

This is a public repo published at https://github.com/SpongeBobCodePants/better-hayabusa. Before your first commit in a fresh clone, set the per-repo author email to the GitHub noreply alias:

```sh
git config user.email "70819570+SpongeBobCodePants@users.noreply.github.com"
```

If you skip this, GitHub will reject the push with `GH007: Your push would publish a private email address` (because the user's global `user.email` is their real personal address, which isn't verified on the `SpongeBobCodePants` account and is protected by the privacy block). Author NAME can stay as your global setting; only email needs the per-repo override.

## Running and building

- `pnpm install` — install frontend deps (run once per clone, and after `pnpm-lock.yaml` changes)
- `pnpm tauri dev` — dev mode (Rust + Vite + Svelte HMR, opens a window). **Blocks the shell — don't run in foreground when scripting.**
- `pnpm build` — frontend-only build (vite + adapter-static). Fast sanity check that frontend compiles.
- `cd src-tauri && cargo test` — Rust tests. **Also generates `src/lib/generated/*.ts` as a side effect** (ts-rs codegen lives in `tests/ts_export.rs`) — must run before `pnpm build` in a fresh clone or frontend imports break.
- `cd src-tauri && cargo check` — Rust type-check only (faster than `build`).
- `pnpm tauri build --debug` — debug bundle (faster, includes debug symbols).
- `pnpm tauri build` — release bundle (slower, used for distribution).

**"Done" requires `cd src-tauri && cargo test` AND `pnpm build` both green.** Don't claim work complete on visual inspection alone. CI ([.github/workflows/ci.yml](.github/workflows/ci.yml)) runs both on every push/PR.

> Frontend tests (Vitest) and E2E tests (Playwright) land in later milestones.

## Versioning

We use SemVer. `src-tauri/tauri.conf.json` is the canonical version source; `package.json` and `src-tauri/Cargo.toml` mirror it.

- **Patch bump** (`0.1.x` → `0.1.x+1`) happens automatically in `/ship` on every PR.
- **Minor bump** (`0.x.0` → `0.x+1.0`) is a manual one-line edit at milestone close. M2 close → `0.2.0`, M3 close → `0.3.0`, etc.
- **Major bump** (`x.0.0` → `x+1.0.0`) is manual. Pre-1.0 we're in experimentation mode; `1.0.0` is cut at M8 close (portable distribution + release). Post-1.0, major bumps follow SemVer — breaking changes or significant feature waves.

About page reads the version via `get_app_version` (Tauri command sourced from `tauri.conf.json`).

## Gotchas (lessons from earlier work)

- **shadcn-svelte `Button` is link-aware.** When you need a navigation link styled as a button, pass `href` to `<Button>` — don't wrap `<Button>` inside `<a>`. The nested form is invalid HTML (interactive content inside an anchor) and breaks a11y.
- **Tauri default `bundle.targets: ["nsis"]` ships an installer.** Issue #7 tracks switching to a portable zip per the "portable model is sacred" rule. Until then, `pnpm tauri build` produces an installer, not a portable artifact.
- **`pnpm tauri dev` opens a window and blocks the shell** until the window closes (and only then `cargo run` exits). Don't run it foreground when scripting; if you need to check the window programmatically, run with `run_in_background` and monitor logs.
- **ts-rs codegen runs as a "test".** `src-tauri/tests/ts_export.rs` calls `AppVersion::export_all()` / `Os::export_all()` to write `.ts` files to `src/lib/generated/`. A developer who runs `pnpm build` without first running `cargo test` will hit TS errors on missing generated bindings. CI runs cargo test first to avoid this; locally, see [Issue #24](https://github.com/SpongeBobCodePants/better-hayabusa/issues/24) about adding a `pretauri:dev` script.
- **Per-repo git config is required** (see "Repo setup" above). A fresh clone defaults to the global `user.email`, which is the user's real personal address, which GitHub blocks with `GH007`. Set the noreply alias before the first commit in any clone.
- **`.gitignore` shipped with the project ignores `private/`** — used as the local-only sandbox for engagement data and scratch test inputs (see "never commit real engagement data" in Conventions). The folder may or may not exist locally; create as needed.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
© 2026 Merciless Software.
