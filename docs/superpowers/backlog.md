# Backlog

Deferred work, organized by target milestone. Plan-writers MUST consult this when drafting a new milestone's plan and either fold items in or push them to a later milestone with a brief note. Anyone closing an item should delete it from this file (not check it off — keep the file short).

## M2 candidates

- **Wire or remove `platform::Os` enum + `current_os()` helper.** [src-tauri/src/platform/mod.rs](../../src-tauri/src/platform/mod.rs) is exported via ts-rs but referenced by nothing — frontend gets OS from `@tauri-apps/plugin-os` directly. Either add a `get_current_os` command (the ts-rs export earns its keep, frontend can prefer Rust-as-SoT in `OsGate`) or delete the module. Deferred from M1 to avoid second-guessing the plan's deliberate scaffolding.
- **Wire or remove `getCachedPlatform()`.** [src/lib/stores/platform.ts](../../src/lib/stores/platform.ts) sync accessor has no consumer. Decide once a synchronous-platform-check use case actually shows up (e.g., a non-async render path).
- **Migration failure-path test.** [src-tauri/tests/db_migrations.rs](../../src-tauri/tests/db_migrations.rs) covers happy paths; a malformed-SQL migration should also be exercised. Land alongside the first `002_*.sql` so the test has a real second migration to interact with.
- **`app_state` key validation.** [src-tauri/src/commands/app.rs](../../src-tauri/src/commands/app.rs) `get_app_state`/`set_app_state` accept any `String`. Keys are app-internal today, but when a real Settings UI lands the commands should reject empty/oversized/control-char keys — and a `CHECK` constraint should land in a migration.
- **Centralize repo URL.** `https://github.com/MercilessSoftware/better-hayabusa-chainsaw` is hardcoded in both [src/lib/components/UnsupportedOs.svelte](../../src/lib/components/UnsupportedOs.svelte) and [src/routes/settings/about/+page.svelte](../../src/routes/settings/about/+page.svelte). With a third consumer, extract to `src/lib/constants.ts`.
- **Typed IPC error.** Every command in [src-tauri/src/commands/app.rs](../../src-tauri/src/commands/app.rs) does `.map_err(|e| e.to_string())` — loses error structure at the IPC boundary. Introduce a `CommandError` enum exported via ts-rs once 3+ commands have distinct error semantics.

## M8 (Portable distribution)

- **Portable-zip bundle target.** [src-tauri/tauri.conf.json](../../src-tauri/tauri.conf.json) currently uses `bundle.targets: ["nsis"]`. Replace/supplement with a portable zip per the "portable model is sacred" rule in CLAUDE.md.

## Unscoped (next time someone touches the area)

- **Dark-mode token migration.** Frontend Svelte components hardcode `text-slate-X` / `bg-white` / `bg-slate-50`. Switch to shadcn-svelte theme tokens (`text-muted-foreground`, `bg-background`, etc.) once dark mode is in scope.
- **Scaffold leftovers.** [static/svelte.svg](../../static/svelte.svg), [static/vite.svg](../../static/vite.svg), [static/tauri.svg](../../static/tauri.svg) are unreferenced. [README.md](../../README.md) is generic Tauri+Svelte boilerplate; rewrite when the GitHub repo goes public.
- **`.gitattributes`.** Every commit produces `LF will be replaced by CRLF` warnings on Windows. A `.gitattributes` declaring `* text=auto eol=lf` (or `eol=crlf` if Windows-CRLF is the project policy) would silence them and prevent future churn.
- **External-link affordance.** Links with `target="_blank"` (About page, UnsupportedOs) have no visual cue or `sr-only` text indicating "opens in new tab" — minor a11y polish.
- **`CardTitle` heading semantics.** shadcn-svelte's `CardTitle` may render as `<div>` not `<h2>`; the About and Home pages have `<h1>` but card titles miss the heading outline. Verify and either override or accept.
- **Tauri `csp: null`.** [src-tauri/tauri.conf.json](../../src-tauri/tauri.conf.json) is scaffold-default. Harden before any future remote URL loading.
