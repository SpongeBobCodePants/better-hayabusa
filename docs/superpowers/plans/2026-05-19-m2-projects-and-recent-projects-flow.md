# M2 — Projects + Recent-Projects Flow — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement M2 — project create / open / switch / delete flow, recent-projects chooser table, `project.db` bootstrap, sticky session, partial Settings UI, breadcrumbs, About page fixes, license change to AGPL-3.0-or-later, version-bump rule, and typed `CommandError` enum.

**Architecture:** API-first Rust (per CLAUDE.md): every backend capability is a plain Rust function tested directly; `#[tauri::command]` handlers are thin wrappers. SQLite via `rusqlite`. Frontend is Svelte 5 with shadcn-svelte primitives. Cross-language types via `ts-rs`. New `CommandError` enum replaces lossy `Result<T, String>` returns.

**Tech Stack:** Tauri 2.x · Rust (stable) · SvelteKit 2.x · Svelte 5 (runes) · TypeScript 5.6+ (strict) · Tailwind 4 · shadcn-svelte · rusqlite 0.31 (bundled) · ts-rs 10 · `tauri-plugin-dialog`, `tauri-plugin-os`, **NEW** `tauri-plugin-opener`.

**Source spec:** [`docs/superpowers/specs/2026-05-19-m2-projects-design.md`](../specs/2026-05-19-m2-projects-design.md). The spec is the source of truth for WHAT; this plan is the sequenced HOW.

---

## Conventions for this plan

- All paths are repo-relative. Windows-only environment; PowerShell shell available but use the Bash tool for tests/git/gh.
- Rust tests: `cd src-tauri && cargo test` (also generates ts-rs bindings).
- Frontend build: `pnpm build` (requires `cargo test` to have run first in a fresh clone).
- Each task ends with a commit. Conventional commits, no `-i` flags. Squash-merge is the project default.
- TDD: failing test first → minimal implementation → green test → commit. For non-TDD tasks (config, assets), each step is a small edit.
- **API-first Rust:** business logic lives in modules like `db::`, `project::`, `paths::`. `commands::` are thin wrappers only.
- **Dependencies:** CLAUDE.md requires asking before any new runtime dep. This plan adds `tauri-plugin-opener` + `@tauri-apps/plugin-opener` (M2 spec authorized) and shadcn-svelte components on demand (devDep, no ask needed). Confirm with the user at the start of Task 1 if any doubt.

---

## Task 1: Add `tauri-plugin-opener` for external-browser links

**Why:** The About page (and `UnsupportedOs.svelte`) use bare `<a href>` for external URLs, which do nothing in a Tauri webview. `tauri-plugin-opener` provides `openUrl()` that delegates to the OS default browser.

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `package.json` (via `pnpm add`)

- [ ] **Step 1: Verify user OK on adding a new runtime dep**

Confirm with user: "M2 spec calls for `tauri-plugin-opener`. Adding it as a runtime dep — OK?" If yes, continue.

- [ ] **Step 2: Add Rust crate**

Edit `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
tauri-plugin-opener = "2"
```

- [ ] **Step 3: Register plugin in `lib.rs`**

In `src-tauri/src/lib.rs`, in the `tauri::Builder::default()` chain, add `.plugin(tauri_plugin_opener::init())` alongside the existing plugins:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_os::init())
    .plugin(tauri_plugin_opener::init())          // NEW
    .setup(|app| {
```

- [ ] **Step 4: Add npm package (pinned to satisfy 7-day rule)**

Run: `pnpm add @tauri-apps/plugin-opener@^2`

If pnpm rejects a brand-new version per `minimumReleaseAge`, pin to an older specific version. Verify by reading the resolved version in `package.json`.

- [ ] **Step 5: Compile check**

Run: `cd src-tauri && cargo check`
Expected: success, no warnings about unused plugin.

Run: `pnpm build`
Expected: success.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/Cargo.lock package.json pnpm-lock.yaml
git commit -m "$(cat <<'EOF'
feat: add tauri-plugin-opener for external links

M2 needs to open external URLs (GitHub, license docs) in the user's
default browser. Bare <a href> doesn't work in a Tauri webview;
plugin-opener delegates to the OS.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: License swap MIT → AGPL-3.0-or-later

**Why:** Closes #18 and reflects the user's call from the M2 brainstorm. License is named across the repo; all surfaces must agree.

**Files:**
- Modify: `LICENSE` (replace entire content)
- Modify: `README.md` (license section)
- Modify: `CLAUDE.md` (`## License` section)
- Modify: `package.json` (`license` field)
- Modify: `src-tauri/Cargo.toml` (add `license` field — currently absent)
- Modify: `src-tauri/tauri.conf.json` (add `license` if its schema accepts it; otherwise skip)

- [ ] **Step 1: Replace `LICENSE` content**

Write the canonical AGPL-3.0 license text. Fetch from `https://www.gnu.org/licenses/agpl-3.0.txt` or copy from any AGPL-licensed project's `LICENSE` file. Header line should match SPDX: the file is just the license text; the SPDX identifier `AGPL-3.0-or-later` is used in metadata fields.

Expected first lines of the file:

```
                    GNU AFFERO GENERAL PUBLIC LICENSE
                       Version 3, 19 November 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.
```

- [ ] **Step 2: Update `README.md`**

Find the License section at the bottom (matches "MIT. See [LICENSE](LICENSE).") and replace with:

```markdown
## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
© 2026 Merciless Software.
```

- [ ] **Step 3: Update `CLAUDE.md`**

Find the `## License` section (last section in the file) and replace its body with:

```markdown
## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
© 2026 Merciless Software.
```

- [ ] **Step 4: Update `package.json`**

Change line `"license": "MIT",` to `"license": "AGPL-3.0-or-later",`.

- [ ] **Step 5: Update `src-tauri/Cargo.toml`**

The current `[package]` table has no `license` field. Add one after the `description` line:

```toml
[package]
name = "bhc"
version = "0.1.0"
description = "A Tauri App"
authors = ["Merciless Software"]
edition = "2021"
license = "AGPL-3.0-or-later"
```

- [ ] **Step 6: Update `src-tauri/tauri.conf.json`**

Check the Tauri config schema. As of Tauri 2.x, `tauri.conf.json` does not have a top-level `license` field — licensing is metadata-only and lives in `Cargo.toml` / `package.json` / `LICENSE`. **Skip this file.** (Verify by running `pnpm tauri info` after the build; it should report license `AGPL-3.0-or-later` sourced from `Cargo.toml`.)

- [ ] **Step 7: Verify builds**

Run: `cd src-tauri && cargo check`
Expected: success. Cargo may emit a warning about license file format if it disagrees with our content — investigate if so.

Run: `pnpm build`
Expected: success.

- [ ] **Step 8: Commit**

```bash
git add LICENSE README.md CLAUDE.md package.json src-tauri/Cargo.toml
git commit -m "$(cat <<'EOF'
chore: relicense MIT to AGPL-3.0-or-later

User decision from M2 brainstorm. Updated all license-name surfaces:
LICENSE file (full AGPL-3.0 text), README, CLAUDE.md, package.json,
Cargo.toml. About page UI updates land in a later task.

Closes #18

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Centralize repo URL constant

**Why:** Closes #5. URL is hardcoded in `UnsupportedOs.svelte` and `About` page; M2 adds a third consumer (sticky-fail screen). Extract before that.

**Files:**
- Create: `src/lib/constants.ts`
- Modify: `src/lib/components/UnsupportedOs.svelte`
- Modify: `src/routes/settings/about/+page.svelte`

- [ ] **Step 1: Create `src/lib/constants.ts`**

```typescript
/**
 * Project-wide constants. Single source of truth for values referenced
 * across the frontend. Rust-side equivalents (where needed) live in
 * their own modules; do not cross-import.
 */

export const REPO_URL = 'https://github.com/SpongeBobCodePants/better-hayabusa';
export const RELEASES_URL = `${REPO_URL}/releases`;
export const ISSUES_URL = `${REPO_URL}/issues`;
```

- [ ] **Step 2: Update `UnsupportedOs.svelte`**

Find the `Button` with the hardcoded URL and replace:

Before:
```svelte
<Button
  href="https://github.com/SpongeBobCodePants/better-hayabusa/issues"
  ...
>
```

After (add import at top):
```svelte
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { ISSUES_URL } from '$lib/constants';
  let { detectedOs }: { detectedOs: string } = $props();
</script>
```

And the Button:
```svelte
<Button
  href={ISSUES_URL}
  target="_blank"
  rel="noopener noreferrer"
  variant="outline"
>
  Open the issue tracker
</Button>
```

(Link-mechanism fix to use plugin-opener lands in Task 32 — keep the existing `href` shape here.)

- [ ] **Step 3: Update `src/routes/settings/about/+page.svelte`**

Add import at top of script:
```typescript
import { REPO_URL } from '$lib/constants';
```

Replace the literal `https://github.com/SpongeBobCodePants/better-hayabusa` (in the Source row) with `{REPO_URL}`.

- [ ] **Step 4: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add src/lib/constants.ts src/lib/components/UnsupportedOs.svelte src/routes/settings/about/+page.svelte
git commit -m "$(cat <<'EOF'
refactor: centralize repo URL constant

UnsupportedOs and About referenced the GitHub URL as a literal. M2
adds a third consumer (sticky-fail screen). Extract to constants.ts.

Closes #5

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Typed `CommandError` enum (closes #6 — foundation)

**Why:** Existing 3 commands use `Result<T, String>`. M2 adds ~9 new commands with multiple structured failure modes the frontend needs to discriminate on (schema-too-new, not-a-project, already-exists, etc.). String errors lose this structure. Build the enum before all other command work so M2 commands use it from day one.

**Files:**
- Create: `src-tauri/src/commands/error.rs`
- Create: `src-tauri/tests/command_error.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Write failing test**

Create `src-tauri/tests/command_error.rs`:

```rust
use bhc_lib::commands::error::CommandError;

#[test]
fn not_found_round_trips_through_json() {
    let e = CommandError::NotFound { path: "C:\\nope".to_string() };
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"kind\":\"NotFound\""));
    assert!(json.contains("C:\\\\nope"));
    let parsed: CommandError = serde_json::from_str(&json).unwrap();
    matches!(parsed, CommandError::NotFound { .. });
}

#[test]
fn schema_too_new_includes_versions() {
    let e = CommandError::SchemaTooNew { project_version: 4, app_version: 3 };
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"project_version\":4"));
    assert!(json.contains("\"app_version\":3"));
}

#[test]
fn all_variants_serialize_with_kind_tag() {
    let cases = vec![
        CommandError::NotFound { path: "p".into() },
        CommandError::AlreadyExists { path: "p".into() },
        CommandError::NotAProject { path: "p".into() },
        CommandError::SchemaTooNew { project_version: 2, app_version: 1 },
        CommandError::Io { message: "m".into() },
        CommandError::Db { message: "m".into() },
        CommandError::Internal { message: "m".into() },
    ];
    for e in cases {
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"kind\":"), "missing kind tag in {json}");
    }
}
```

- [ ] **Step 2: Run test, verify failure**

Run: `cd src-tauri && cargo test --test command_error`
Expected: FAIL — `error.rs` module doesn't exist.

- [ ] **Step 3: Create `src-tauri/src/commands/error.rs`**

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Typed error returned from every Tauri command. Frontend discriminates
/// on `kind` to decide UI behavior (schema-mismatch screen vs. inline
/// alert vs. toast).
///
/// Add new variants here when a command needs to surface a structured
/// failure case to the UI — never funnel real errors through `Internal`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/lib/generated/")]
pub enum CommandError {
    NotFound        { path: String },
    AlreadyExists   { path: String },
    NotAProject     { path: String },
    SchemaTooNew    { project_version: u32, app_version: u32 },
    Io              { message: String },
    Db              { message: String },
    Internal        { message: String },
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError::Db { message: e.to_string() }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(e: std::io::Error) -> Self {
        CommandError::Io { message: e.to_string() }
    }
}

impl<E: std::error::Error> From<std::sync::PoisonError<E>> for CommandError {
    fn from(e: std::sync::PoisonError<E>) -> Self {
        CommandError::Internal { message: format!("mutex poisoned: {e}") }
    }
}
```

- [ ] **Step 4: Register module**

Edit `src-tauri/src/commands/mod.rs`:

```rust
pub mod app;
pub mod error;
```

- [ ] **Step 5: Run test, verify pass**

Run: `cd src-tauri && cargo test --test command_error`
Expected: 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/error.rs src-tauri/src/commands/mod.rs src-tauri/tests/command_error.rs
git commit -m "$(cat <<'EOF'
feat(commands): add typed CommandError enum

Replaces the lossy Result<T, String> pattern with a discriminated
union so the frontend can branch on error kind (schema-too-new,
not-a-project, already-exists, etc.). Derives ts-rs export.
Implements From for rusqlite, io, and PoisonError.

Closes #6 (existing commands refactored in next task)

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Refactor existing commands to use `CommandError`

**Files:**
- Modify: `src-tauri/src/commands/app.rs`
- Modify: `src-tauri/tests/commands_app.rs` (no change needed — round-trip test still works)
- Modify: `src-tauri/tests/commands_app_state.rs` (uses internal API, not affected)

- [ ] **Step 1: Update `commands/app.rs`**

```rust
use tauri::State;

use crate::commands::error::CommandError;
use crate::db::app_db;
use crate::types::AppVersion;
use crate::AppState;

#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> Result<AppVersion, CommandError> {
    Ok(AppVersion {
        version: app.package_info().version.to_string(),
    })
}

#[tauri::command]
pub fn get_app_state(state: State<'_, AppState>, key: String) -> Result<Option<String>, CommandError> {
    let conn = state.app_db.lock()?;
    app_db::get_state(&conn, &key)
        .map_err(|e| CommandError::Db { message: e.to_string() })
}

#[tauri::command]
pub fn set_app_state(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), CommandError> {
    let conn = state.app_db.lock()?;
    app_db::set_state(&conn, &key, &value)
        .map_err(|e| CommandError::Db { message: e.to_string() })
}
```

- [ ] **Step 2: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: all existing tests pass (command return-type refactor is source-compatible at the boundary; `tests/commands_app.rs` only tests the `AppVersion` struct).

- [ ] **Step 3: Verify frontend still compiles**

Frontend `src/lib/ipc/app.ts` declares return types using `string | null` for `getAppState`. The Promise contract still resolves to `string | null`; rejection is now a typed CommandError. The `getAppState` wrapper doesn't need to change yet — but its `.catch()` consumers (if any) would now get a `CommandError` object instead of a `string`. Audit consumers:

Run: `grep -rn "catch" src/`

If any callers handle errors, leave the types as `unknown` for now; they'll be updated in IPC wrapper work (Task 19). For M2 the existing `app.ts` callers (currently only `getAppVersion` in `SidebarFooter` + About) silently fall back to `?` — no catch handler unwraps the type. Safe to leave.

Run: `pnpm build`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/app.rs
git commit -m "$(cat <<'EOF'
refactor(commands): use CommandError in app commands

Replaces .map_err(|e| e.to_string()) with typed CommandError. Mutex
lock and DB errors now flow through From impls. Frontend wrappers
receive typed errors on rejection.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Types module refactor + new type stubs

**Why:** Current `types.rs` is one file with one struct (`AppVersion`). M2 adds 5+ types (`Project`, `ProjectInfo`, `RecentProject`, `RecentProjectListEntry`, `LaunchResult`). Split into a directory for clarity.

**Files:**
- Delete: `src-tauri/src/types.rs`
- Create: `src-tauri/src/types/mod.rs`
- Create: `src-tauri/src/types/app_version.rs` (moved)
- Create: `src-tauri/src/types/project.rs` (new)
- Create: `src-tauri/src/types/launch_result.rs` (new)
- Modify: `src-tauri/src/commands/app.rs` (re-imports `AppVersion`)
- Modify: `src-tauri/tests/ts_export.rs` (export new types)
- Modify: `src-tauri/tests/commands_app.rs` (still imports `AppVersion`; no change needed)

- [ ] **Step 1: Create `src-tauri/src/types/mod.rs`**

```rust
pub mod app_version;
pub mod project;
pub mod launch_result;

pub use app_version::AppVersion;
pub use project::{Project, ProjectInfo, RecentProject, RecentProjectListEntry};
pub use launch_result::LaunchResult;
```

- [ ] **Step 2: Move `AppVersion` to `src-tauri/src/types/app_version.rs`**

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct AppVersion {
    pub version: String,
}
```

- [ ] **Step 3: Create `src-tauri/src/types/project.rs`**

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One row from `projects` in project.db. Single row per DB,
/// constrained at the application layer.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,             // UTC ISO 8601
    pub app_schema_version: u32,
}

/// What the frontend gets back when a project is opened — `Project` plus
/// the resolved folder path so the UI can display it without re-querying.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct ProjectInfo {
    pub project: Project,
    pub folder_path: String,            // absolute path the user picked
}

/// One row from `recent_projects` in app.db.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,         // UTC ISO 8601
}

/// Same as RecentProject plus a computed `last_modified` (mtime of
/// `.bhc/activity.log` in the project folder, ISO 8601 UTC).
/// Used by the chooser table only.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
pub struct RecentProjectListEntry {
    pub path: String,
    pub name: String,
    pub last_opened_at: String,
    pub last_modified: Option<String>,  // None if activity.log missing
}
```

- [ ] **Step 4: Create `src-tauri/src/types/launch_result.rs`**

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::types::ProjectInfo;

/// Result of `check_last_open_project()` at app boot. Drives whether
/// the frontend lands on the dashboard, on Home, or on the sticky-fail
/// takeover screen.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, export_to = "../../src/lib/generated/")]
pub enum LaunchResult {
    /// Sticky session succeeded; this project was loaded into AppState
    /// and the frontend should route to `/projects/current`.
    Loaded { info: ProjectInfo },

    /// Sticky session failed; frontend shows the sticky-fail takeover.
    /// `reason` is a human-readable description; `path` and `name` are
    /// from the dead recent_projects entry (which has been removed).
    Failed {
        path: String,
        name: String,
        reason: String,
    },

    /// `app_schema_version` of the project is newer than the app's.
    /// Frontend shows the schema-mismatch takeover screen.
    SchemaTooNew {
        path: String,
        name: String,
        project_version: u32,
        app_version: u32,
    },

    /// No `last_open_project_path` is set in app.db. Frontend lands on Home.
    NoneSet,

    /// `launch_behavior` setting is `"home_page"`. Frontend lands on Home.
    Disabled,
}
```

- [ ] **Step 5: Delete the old single-file `types.rs`**

Run: `rm src-tauri/src/types.rs`

- [ ] **Step 6: Update `tests/ts_export.rs`**

```rust
//! Not a real test — a codegen trigger. Calling `export_all()` writes TS
//! definitions to `../../src/lib/generated/`. Must run before `pnpm build`.

use bhc_lib::commands::error::CommandError;
use bhc_lib::platform::Os;
use bhc_lib::types::{AppVersion, LaunchResult, Project, ProjectInfo, RecentProject, RecentProjectListEntry};
use ts_rs::TS;

#[test]
fn export_ts_types() {
    AppVersion::export_all().expect("export AppVersion");
    Os::export_all().expect("export Os");
    Project::export_all().expect("export Project");
    ProjectInfo::export_all().expect("export ProjectInfo");
    RecentProject::export_all().expect("export RecentProject");
    RecentProjectListEntry::export_all().expect("export RecentProjectListEntry");
    LaunchResult::export_all().expect("export LaunchResult");
    CommandError::export_all().expect("export CommandError");
}
```

- [ ] **Step 7: Run tests + verify codegen**

Run: `cd src-tauri && cargo test`
Expected: all green; `src/lib/generated/` now contains `Project.ts`, `ProjectInfo.ts`, `RecentProject.ts`, `RecentProjectListEntry.ts`, `LaunchResult.ts`, `CommandError.ts` (plus existing `AppVersion.ts`, `Os.ts`).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/types/ src-tauri/tests/ts_export.rs
git rm src-tauri/src/types.rs
git commit -m "$(cat <<'EOF'
refactor(types): split into directory; add M2 type stubs

Splits types.rs into types/mod.rs + types/app_version.rs (existing) +
types/project.rs + types/launch_result.rs (new). Adds Project,
ProjectInfo, RecentProject, RecentProjectListEntry, LaunchResult.
ts-rs exports updated.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Project.db bootstrap — migration + connection module + #3 test

**Why:** M2 needs `project.db` per-project. The existing migration runner already takes a `&[Migration]` slice, so adding `PROJECT_MIGRATIONS` is mechanical. Closes #3 by adding a malformed-migration test that uses the new project migrations.

**Files:**
- Create: `src-tauri/migrations/project/001_init.sql`
- Modify: `src-tauri/src/db/migrations.rs` (add `PROJECT_MIGRATIONS` constant)
- Create: `src-tauri/src/db/project_db.rs`
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/tests/db_migrations.rs` (add project tests + malformed test)

- [ ] **Step 1: Write failing tests**

Add to `src-tauri/tests/db_migrations.rs` (append at end):

```rust
use bhc_lib::db::migrations::{Migration, PROJECT_MIGRATIONS};

#[test]
fn run_project_migrations_creates_projects_table() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("project.db");
    let conn = Connection::open(&db_path).unwrap();

    run_migrations(&conn, PROJECT_MIGRATIONS).expect("run project migrations");

    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    assert!(tables.contains(&"projects".to_string()));
    assert!(tables.contains(&"_migrations".to_string()));
}

#[test]
fn project_open_or_create_works_on_fresh_path() {
    use bhc_lib::db::project_db::open_or_create;

    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("project.db");
    assert!(!db_path.exists());

    let conn = open_or_create(&db_path).expect("open or create");
    assert!(db_path.exists());

    let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
    assert_eq!(fk, 1);
}

#[test]
fn malformed_migration_preserves_name_in_error() {
    use bhc_lib::db::migrations::MigrationError;

    let tmp = tempdir().unwrap();
    let conn = Connection::open(tmp.path().join("test.db")).unwrap();

    let bad = &[
        Migration { name: "001_ok", sql: "CREATE TABLE ok (id INTEGER);" },
        Migration { name: "002_bad", sql: "CREATE TABEL whoops (oops INTEGER);" }, // typo
    ];

    let result = run_migrations(&conn, bad);
    match result {
        Err(MigrationError::Migration(name, _)) => {
            assert_eq!(name, "002_bad");
        }
        other => panic!("expected MigrationError::Migration with '002_bad', got {other:?}"),
    }

    // 001_ok should still be recorded as applied.
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM _migrations WHERE name = '001_ok'",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 1);
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cd src-tauri && cargo test --test db_migrations`
Expected: FAIL — `PROJECT_MIGRATIONS` and `project_db` don't exist yet.

- [ ] **Step 3: Create `src-tauri/migrations/project/001_init.sql`**

```sql
CREATE TABLE projects (
  id                  INTEGER PRIMARY KEY,
  name                TEXT NOT NULL,
  description         TEXT,
  created_at          TEXT NOT NULL,
  app_schema_version  INTEGER NOT NULL
);
```

- [ ] **Step 4: Add `PROJECT_MIGRATIONS` to `migrations.rs`**

Append to `src-tauri/src/db/migrations.rs`:

```rust
pub const PROJECT_MIGRATIONS: &[Migration] = &[Migration {
    name: "001_init",
    sql: include_str!("../../migrations/project/001_init.sql"),
}];

/// Current app-side schema version. Stored in `projects.app_schema_version`
/// on create; compared on open to detect "project too new for this app."
pub const CURRENT_PROJECT_SCHEMA_VERSION: u32 = 1;
```

- [ ] **Step 5: Create `src-tauri/src/db/project_db.rs`**

```rust
use std::path::Path;

use rusqlite::Connection;

use crate::db::migrations::{run_migrations, MigrationError, PROJECT_MIGRATIONS};

#[derive(Debug, thiserror::Error)]
pub enum ProjectDbError {
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("migration: {0}")]
    Migration(#[from] MigrationError),
}

/// Opens (creating if missing) the project.db at the given path and runs
/// migrations. Does NOT check schema version against the app — that's the
/// caller's responsibility (see project::lifecycle::open_project).
pub fn open_or_create(db_path: &Path) -> Result<Connection, ProjectDbError> {
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    run_migrations(&conn, PROJECT_MIGRATIONS)?;
    Ok(conn)
}

/// Reads `projects.app_schema_version` from the given project.db connection.
/// Returns None if the projects row doesn't exist yet (a freshly migrated,
/// not-yet-bootstrapped DB).
pub fn read_schema_version(conn: &Connection) -> Result<Option<u32>, rusqlite::Error> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT app_schema_version FROM projects LIMIT 1",
        [],
        |row| row.get::<_, u32>(0),
    )
    .optional()
}
```

- [ ] **Step 6: Register module in `src-tauri/src/db/mod.rs`**

```rust
pub mod app_db;
pub mod migrations;
pub mod project_db;
```

- [ ] **Step 7: Run tests, verify pass**

Run: `cd src-tauri && cargo test --test db_migrations`
Expected: all green (8 tests including the 3 new ones).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/migrations/project/001_init.sql src-tauri/src/db/migrations.rs src-tauri/src/db/project_db.rs src-tauri/src/db/mod.rs src-tauri/tests/db_migrations.rs
git commit -m "$(cat <<'EOF'
feat(db): add project.db bootstrap with projects table

Adds migrations/project/001_init.sql with projects table (single
row per DB, app_schema_version tracked). New db/project_db.rs
mirrors app_db pattern. Adds CURRENT_PROJECT_SCHEMA_VERSION
constant for compatibility checks. Migration runner is reused
as-is (already takes &[Migration]).

Also adds a malformed-migration test that asserts MigrationError
preserves the failing migration's name.

Closes #3

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Activity log module

**Why:** Per spec § 10, `.bhc/activity.log` is an append-only plain-text file. Its mtime is the "Last modified" timestamp shown in the chooser table. Built before lifecycle because every lifecycle operation appends to it.

**Files:**
- Create: `src-tauri/src/project/mod.rs`
- Create: `src-tauri/src/project/activity_log.rs`
- Create: `src-tauri/tests/activity_log.rs`
- Modify: `src-tauri/src/lib.rs` (register `project` module)

- [ ] **Step 1: Write failing test**

Create `src-tauri/tests/activity_log.rs`:

```rust
use bhc_lib::project::activity_log::{append_event, ActivityEvent};
use std::fs;
use tempfile::tempdir;

#[test]
fn append_event_creates_log_file_with_header_line() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("activity.log");

    append_event(
        &log_path,
        ActivityEvent::ProjectOpened { name: "Test Project".to_string() },
    )
    .expect("append");

    let contents = fs::read_to_string(&log_path).unwrap();
    assert!(contents.contains("project_opened"));
    assert!(contents.contains("name=\"Test Project\""));
    // ISO 8601 UTC pattern
    assert!(contents.contains("Z |"));
}

#[test]
fn append_event_appends_to_existing_log() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("activity.log");

    append_event(&log_path, ActivityEvent::ProjectOpened { name: "P1".into() }).unwrap();
    append_event(&log_path, ActivityEvent::SettingsChanged { key: "x".into(), value: "y".into() }).unwrap();

    let contents = fs::read_to_string(&log_path).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("project_opened"));
    assert!(lines[1].contains("settings_changed"));
    assert!(lines[1].contains("key=x"));
    assert!(lines[1].contains("value=y"));
}

#[test]
fn append_event_quoting_handles_names_with_quotes() {
    let tmp = tempdir().unwrap();
    let log_path = tmp.path().join("activity.log");

    append_event(
        &log_path,
        ActivityEvent::ProjectOpened { name: "She said \"hi\"".to_string() },
    )
    .expect("append");

    let contents = fs::read_to_string(&log_path).unwrap();
    // Inner quotes escaped to \"
    assert!(contents.contains("name=\"She said \\\"hi\\\"\""));
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cd src-tauri && cargo test --test activity_log`
Expected: FAIL — `project::activity_log` module doesn't exist.

- [ ] **Step 3: Create `src-tauri/src/project/mod.rs`**

```rust
pub mod activity_log;
```

- [ ] **Step 4: Create `src-tauri/src/project/activity_log.rs`**

```rust
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Error)]
pub enum ActivityLogError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("timestamp: {0}")]
    Timestamp(#[from] time::error::Format),
}

/// Events that can be appended to a project's `.bhc/activity.log`.
/// M2 ships project_opened and settings_changed; M3/M4 extend with
/// job and run events.
#[derive(Debug, Clone)]
pub enum ActivityEvent {
    ProjectOpened    { name: String },
    SettingsChanged  { key: String, value: String },
}

impl ActivityEvent {
    fn event_type(&self) -> &'static str {
        match self {
            ActivityEvent::ProjectOpened { .. }    => "project_opened",
            ActivityEvent::SettingsChanged { .. }  => "settings_changed",
        }
    }

    fn details(&self) -> String {
        match self {
            ActivityEvent::ProjectOpened { name } => format!("name={}", quote(name)),
            ActivityEvent::SettingsChanged { key, value } => format!("key={key} value={value}"),
        }
    }
}

fn quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Appends an event line to the given log path. Creates the file if
/// missing. Format: `<UTC ISO 8601> | <event_type> | <details>`.
///
/// Best-effort: if the write fails, the error bubbles up. Callers may
/// choose to log-and-continue (for read-only events like project_opened)
/// or roll back the accompanying DB transaction (for state-changing
/// events).
pub fn append_event(log_path: &Path, event: ActivityEvent) -> Result<(), ActivityLogError> {
    let now = OffsetDateTime::now_utc().format(&Rfc3339)?;
    let line = format!("{} | {} | {}\n", now, event.event_type(), event.details());

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}
```

- [ ] **Step 5: Add `time` crate to dependencies**

The `time` crate is needed for proper RFC 3339 UTC formatting. Add to `src-tauri/Cargo.toml`:

```toml
time = { version = "0.3", features = ["formatting", "macros"] }
```

(Verify with user that adding `time` is OK — it's a small, well-established crate, but per CLAUDE.md ask before runtime deps. If declined, use `chrono` if already in tree, or build a tiny ISO 8601 formatter using `std::time::SystemTime`.)

- [ ] **Step 6: Register `project` module in `src-tauri/src/lib.rs`**

Add `pub mod project;` near the other `pub mod` lines:

```rust
pub mod commands;
pub mod db;
pub mod paths;
pub mod platform;
pub mod project;       // NEW
pub mod types;
```

- [ ] **Step 7: Run tests, verify pass**

Run: `cd src-tauri && cargo test --test activity_log`
Expected: 3 tests pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/project/ src-tauri/tests/activity_log.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "$(cat <<'EOF'
feat(project): add activity log append API

Append-only plain-text log at .bhc/activity.log. M2 events:
project_opened, settings_changed. Format:
<UTC ISO 8601> | <event_type> | <key=value details>

mtime of this file feeds the chooser's "Last modified" column.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Project conflict-check API

**Why:** Per spec § 4 (New Project sheet conflict handling), we need to know if a chosen folder is empty, already a project, or has some other state. Used by the create flow's inline alert.

**Files:**
- Create: `src-tauri/src/project/conflict.rs`
- Create: `src-tauri/tests/project_conflict.rs`
- Modify: `src-tauri/src/project/mod.rs`

- [ ] **Step 1: Write failing test**

Create `src-tauri/tests/project_conflict.rs`:

```rust
use bhc_lib::project::conflict::{check_folder, FolderState};
use std::fs;
use tempfile::tempdir;

#[test]
fn empty_folder_is_eligible() {
    let tmp = tempdir().unwrap();
    let state = check_folder(tmp.path()).unwrap();
    assert_eq!(state, FolderState::Eligible);
}

#[test]
fn folder_with_bhc_project_db_is_existing_project() {
    let tmp = tempdir().unwrap();
    fs::create_dir(tmp.path().join(".bhc")).unwrap();
    fs::write(tmp.path().join(".bhc").join("project.db"), b"").unwrap();

    let state = check_folder(tmp.path()).unwrap();
    assert_eq!(state, FolderState::ExistingProject);
}

#[test]
fn folder_with_other_files_but_no_bhc_is_eligible() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("readme.txt"), b"hi").unwrap();
    let state = check_folder(tmp.path()).unwrap();
    assert_eq!(state, FolderState::Eligible);
}

#[test]
fn nonexistent_folder_returns_not_found() {
    let result = check_folder(std::path::Path::new("C:\\definitely-does-not-exist-xyz"));
    assert!(matches!(result, Err(_)));
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cd src-tauri && cargo test --test project_conflict`
Expected: FAIL — `project::conflict` module doesn't exist.

- [ ] **Step 3: Create `src-tauri/src/project/conflict.rs`**

```rust
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderState {
    /// Folder is empty, or has unrelated files but no `.bhc/project.db`.
    /// Safe target for `create_project`.
    Eligible,
    /// Folder already has `.bhc/project.db`. `create_project` would error;
    /// caller should offer "Open it instead?" UX.
    ExistingProject,
}

#[derive(Debug, thiserror::Error)]
pub enum ConflictCheckError {
    #[error("folder does not exist: {0}")]
    NotFound(String),
    #[error("path is not a directory: {0}")]
    NotADirectory(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Inspects a folder to decide whether `create_project` would conflict
/// with an existing project. Does NOT decide eligibility for any other
/// reason (write permissions, disk space, etc.) — those surface as I/O
/// errors at create time.
pub fn check_folder(folder: &Path) -> Result<FolderState, ConflictCheckError> {
    if !folder.exists() {
        return Err(ConflictCheckError::NotFound(folder.display().to_string()));
    }
    if !folder.is_dir() {
        return Err(ConflictCheckError::NotADirectory(folder.display().to_string()));
    }

    let project_db = folder.join(".bhc").join("project.db");
    if project_db.exists() {
        Ok(FolderState::ExistingProject)
    } else {
        Ok(FolderState::Eligible)
    }
}
```

- [ ] **Step 4: Register in `project/mod.rs`**

```rust
pub mod activity_log;
pub mod conflict;
```

- [ ] **Step 5: Run tests, verify pass**

Run: `cd src-tauri && cargo test --test project_conflict`
Expected: 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/project/conflict.rs src-tauri/src/project/mod.rs src-tauri/tests/project_conflict.rs
git commit -m "$(cat <<'EOF'
feat(project): add folder conflict check

check_folder returns Eligible or ExistingProject. Used by the New
Project sheet to surface "this folder is already a project — open
it instead?" inline.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Project lifecycle — `create_project`

**Why:** Per spec § 11. Creates `.bhc/project.db`, writes the `projects` row, logs the open event. Pure Rust API per "API-first Rust" convention.

**Files:**
- Create: `src-tauri/src/project/lifecycle.rs`
- Create: `src-tauri/tests/project_lifecycle.rs`
- Modify: `src-tauri/src/project/mod.rs`
- Modify: `src-tauri/src/db/app_db.rs` (add `insert_recent_project`, `touch_recent_project`)

- [ ] **Step 1: Write failing test**

Create `src-tauri/tests/project_lifecycle.rs`:

```rust
use bhc_lib::db::app_db;
use bhc_lib::project::lifecycle::create_project;
use tempfile::tempdir;

#[test]
fn create_project_writes_db_and_log_and_recents_row() {
    let app_tmp = tempdir().unwrap();
    let app_db_path = app_tmp.path().join("app.db");
    let app_conn = app_db::open_or_create(&app_db_path).unwrap();

    let project_tmp = tempdir().unwrap();
    let project_folder = project_tmp.path();

    let info = create_project(
        &app_conn,
        project_folder,
        "Test Project",
        Some("A description"),
    )
    .expect("create_project");

    // project.db exists at the expected path
    let project_db = project_folder.join(".bhc").join("project.db");
    assert!(project_db.exists(), "project.db should exist");

    // activity.log exists
    let activity_log = project_folder.join(".bhc").join("activity.log");
    assert!(activity_log.exists(), "activity.log should exist");
    let log_contents = std::fs::read_to_string(&activity_log).unwrap();
    assert!(log_contents.contains("project_opened"));
    assert!(log_contents.contains("Test Project"));

    // recent_projects row inserted
    let count: i64 = app_conn.query_row(
        "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
        [project_folder.to_str().unwrap()],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 1);

    // ProjectInfo has the right shape
    assert_eq!(info.project.name, "Test Project");
    assert_eq!(info.project.description.as_deref(), Some("A description"));
    assert_eq!(info.project.app_schema_version, 1);
    assert_eq!(info.folder_path, project_folder.display().to_string());
}

#[test]
fn create_project_in_folder_that_already_has_project_errors() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "First", None).unwrap();
    let result = create_project(&app_conn, folder, "Second", None);

    // Should be a LifecycleError::AlreadyExists
    use bhc_lib::project::lifecycle::LifecycleError;
    assert!(matches!(result, Err(LifecycleError::AlreadyExists { .. })));
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cd src-tauri && cargo test --test project_lifecycle`
Expected: FAIL — `project::lifecycle` module doesn't exist.

- [ ] **Step 3: Add app_db helpers for `recent_projects`**

Append to `src-tauri/src/db/app_db.rs`:

```rust
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Insert or update a recent_projects row, bumping last_opened_at to now.
pub fn upsert_recent_project(
    conn: &Connection,
    path: &str,
    name: &str,
) -> Result<(), AppDbError> {
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| AppDbError::Sql(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
    conn.execute(
        "INSERT INTO recent_projects (path, name, last_opened_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(path) DO UPDATE SET name = excluded.name, last_opened_at = excluded.last_opened_at",
        rusqlite::params![path, name, now],
    )?;
    Ok(())
}

/// Delete a recent_projects row by path. Returns true if a row was removed.
pub fn remove_recent_project(conn: &Connection, path: &str) -> Result<bool, AppDbError> {
    let n = conn.execute("DELETE FROM recent_projects WHERE path = ?1", [path])?;
    Ok(n > 0)
}

/// Read all recent_projects rows ordered by last_opened_at DESC.
pub fn list_recent_projects(conn: &Connection) -> Result<Vec<(String, String, String)>, AppDbError> {
    let mut stmt = conn.prepare(
        "SELECT path, name, last_opened_at FROM recent_projects ORDER BY last_opened_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
```

- [ ] **Step 4: Create `src-tauri/src/project/lifecycle.rs`**

```rust
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::db::{app_db, project_db};
use crate::db::migrations::CURRENT_PROJECT_SCHEMA_VERSION;
use crate::project::activity_log::{append_event, ActivityEvent};
use crate::project::conflict::{check_folder, ConflictCheckError, FolderState};
use crate::types::{Project, ProjectInfo};

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("folder already a project: {path}")]
    AlreadyExists { path: String },
    #[error("folder is not a project: {path}")]
    NotAProject { path: String },
    #[error("folder not found: {path}")]
    NotFound { path: String },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("project db: {0}")]
    ProjectDb(#[from] project_db::ProjectDbError),
    #[error("app db: {0}")]
    AppDb(#[from] app_db::AppDbError),
    #[error("activity log: {0}")]
    ActivityLog(#[from] crate::project::activity_log::ActivityLogError),
    #[error("conflict check: {0}")]
    Conflict(#[from] ConflictCheckError),
}

/// Path helpers — keep `.bhc/` layout in one place.
pub fn bhc_dir(folder: &Path) -> PathBuf { folder.join(".bhc") }
pub fn project_db_path(folder: &Path) -> PathBuf { bhc_dir(folder).join("project.db") }
pub fn activity_log_path(folder: &Path) -> PathBuf { bhc_dir(folder).join("activity.log") }

/// Creates a new project at `folder`. Bootstraps `.bhc/project.db`,
/// inserts the `projects` row, writes the first activity log entry, and
/// adds an entry to app.db's `recent_projects`.
///
/// Returns the loaded `ProjectInfo` ready for the caller to install as
/// the current project.
pub fn create_project(
    app_conn: &Connection,
    folder: &Path,
    name: &str,
    description: Option<&str>,
) -> Result<ProjectInfo, LifecycleError> {
    // 1. Conflict check.
    match check_folder(folder)? {
        FolderState::Eligible => {}
        FolderState::ExistingProject => {
            return Err(LifecycleError::AlreadyExists {
                path: folder.display().to_string(),
            });
        }
    }

    // 2. Create .bhc/ directory.
    fs::create_dir_all(bhc_dir(folder))?;

    // 3. Open project.db (runs migrations).
    let project_conn = project_db::open_or_create(&project_db_path(folder))?;

    // 4. Insert projects row.
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("RFC3339 format never fails for current_utc time");
    project_conn.execute(
        "INSERT INTO projects (name, description, created_at, app_schema_version) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, description, now, CURRENT_PROJECT_SCHEMA_VERSION],
    )?;

    // 5. Read the inserted row.
    let project: Project = project_conn.query_row(
        "SELECT id, name, description, created_at, app_schema_version FROM projects LIMIT 1",
        [],
        |row| Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            app_schema_version: row.get::<_, u32>(4)?,
        }),
    )?;

    // 6. Activity log.
    append_event(
        &activity_log_path(folder),
        ActivityEvent::ProjectOpened { name: name.to_string() },
    )?;

    // 7. Upsert recent_projects.
    let folder_str = folder.display().to_string();
    app_db::upsert_recent_project(app_conn, &folder_str, name)?;

    Ok(ProjectInfo { project, folder_path: folder_str })
}
```

- [ ] **Step 5: Register lifecycle in project mod**

Update `src-tauri/src/project/mod.rs`:

```rust
pub mod activity_log;
pub mod conflict;
pub mod lifecycle;
```

- [ ] **Step 6: Run tests, verify pass**

Run: `cd src-tauri && cargo test --test project_lifecycle`
Expected: 2 tests pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/project/lifecycle.rs src-tauri/src/project/mod.rs src-tauri/src/db/app_db.rs src-tauri/tests/project_lifecycle.rs
git commit -m "$(cat <<'EOF'
feat(project): create_project lifecycle

Bootstraps .bhc/project.db, inserts projects row, writes first
activity log entry, upserts recent_projects in app.db. Errors with
AlreadyExists when folder already has .bhc/project.db.

Adds app_db helpers: upsert_recent_project, remove_recent_project,
list_recent_projects.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: Project lifecycle — `open_project` with schema check

**Files:**
- Modify: `src-tauri/src/project/lifecycle.rs`
- Modify: `src-tauri/tests/project_lifecycle.rs`

- [ ] **Step 1: Write failing tests**

Append to `src-tauri/tests/project_lifecycle.rs`:

```rust
use bhc_lib::db::migrations::CURRENT_PROJECT_SCHEMA_VERSION;
use bhc_lib::project::lifecycle::{open_project, OpenOutcome};

#[test]
fn open_project_returns_loaded_for_compatible_schema() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "Test", None).unwrap();

    let outcome = open_project(&app_conn, folder).expect("open_project");
    match outcome {
        OpenOutcome::Loaded { info, .. } => {
            assert_eq!(info.project.name, "Test");
            assert_eq!(info.project.app_schema_version, CURRENT_PROJECT_SCHEMA_VERSION);
        }
        OpenOutcome::SchemaTooNew { .. } => panic!("expected Loaded"),
    }
}

#[test]
fn open_project_on_missing_bhc_errors_not_a_project() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let empty_tmp = tempdir().unwrap();

    let result = open_project(&app_conn, empty_tmp.path());
    assert!(matches!(result, Err(bhc_lib::project::lifecycle::LifecycleError::NotAProject { .. })));
}

#[test]
fn open_project_with_too_new_schema_returns_schema_too_new() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "Test", None).unwrap();

    // Forcibly bump the schema version in project.db to simulate a future app.
    let db_path = folder.join(".bhc").join("project.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute("UPDATE projects SET app_schema_version = 99", []).unwrap();
    drop(conn);

    let outcome = open_project(&app_conn, folder).expect("open returns Ok");
    match outcome {
        OpenOutcome::SchemaTooNew { project_version, app_version, .. } => {
            assert_eq!(project_version, 99);
            assert_eq!(app_version, CURRENT_PROJECT_SCHEMA_VERSION);
        }
        OpenOutcome::Loaded { .. } => panic!("expected SchemaTooNew"),
    }
}

#[test]
fn open_project_updates_last_opened_at_in_recents() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path();

    create_project(&app_conn, folder, "Test", None).unwrap();

    let before: String = app_conn.query_row(
        "SELECT last_opened_at FROM recent_projects WHERE path = ?1",
        [folder.to_str().unwrap()],
        |r| r.get(0),
    ).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1100)); // RFC3339 is sec precision

    open_project(&app_conn, folder).unwrap();

    let after: String = app_conn.query_row(
        "SELECT last_opened_at FROM recent_projects WHERE path = ?1",
        [folder.to_str().unwrap()],
        |r| r.get(0),
    ).unwrap();

    assert_ne!(before, after, "open_project should bump last_opened_at");
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cd src-tauri && cargo test --test project_lifecycle`
Expected: 2 existing pass, 4 new FAIL (`open_project` not defined).

- [ ] **Step 3: Add `open_project` to `lifecycle.rs`**

Append to `src-tauri/src/project/lifecycle.rs`:

```rust
use rusqlite::OptionalExtension;

/// Possible outcomes of `open_project`. The schema-version mismatch is
/// not an Err because it's a designed UX state (user-recoverable: upgrade
/// the app), not a system failure.
pub enum OpenOutcome {
    Loaded {
        info: ProjectInfo,
        connection: Connection,         // caller installs into AppState
    },
    SchemaTooNew {
        path: String,
        name: String,
        project_version: u32,
        app_version: u32,
    },
}

/// Opens an existing project. Validates that `.bhc/project.db` exists,
/// runs forward migrations (no-op if up to date), checks the project's
/// stored schema version against the app's, logs the open event, bumps
/// `recent_projects.last_opened_at`.
pub fn open_project(
    app_conn: &Connection,
    folder: &Path,
) -> Result<OpenOutcome, LifecycleError> {
    let db_path = project_db_path(folder);
    if !db_path.exists() {
        return Err(LifecycleError::NotAProject {
            path: folder.display().to_string(),
        });
    }

    let project_conn = project_db::open_or_create(&db_path)?;

    // Read schema version.
    let project_version = project_db::read_schema_version(&project_conn)?
        .ok_or_else(|| LifecycleError::NotAProject {
            path: folder.display().to_string(),
        })?;

    if project_version > CURRENT_PROJECT_SCHEMA_VERSION {
        let name: String = project_conn.query_row(
            "SELECT name FROM projects LIMIT 1",
            [],
            |r| r.get(0),
        ).unwrap_or_else(|_| String::from("(unknown)"));

        return Ok(OpenOutcome::SchemaTooNew {
            path: folder.display().to_string(),
            name,
            project_version,
            app_version: CURRENT_PROJECT_SCHEMA_VERSION,
        });
    }

    // Read the project row.
    let project: Project = project_conn.query_row(
        "SELECT id, name, description, created_at, app_schema_version FROM projects LIMIT 1",
        [],
        |row| Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            app_schema_version: row.get::<_, u32>(4)?,
        }),
    )?;

    let folder_str = folder.display().to_string();

    // Log + upsert recents (best-effort log; recents upsert is required).
    let _ = append_event(
        &activity_log_path(folder),
        ActivityEvent::ProjectOpened { name: project.name.clone() },
    );
    app_db::upsert_recent_project(app_conn, &folder_str, &project.name)?;

    Ok(OpenOutcome::Loaded {
        info: ProjectInfo { project, folder_path: folder_str },
        connection: project_conn,
    })
}
```

- [ ] **Step 4: Run tests, verify pass**

Run: `cd src-tauri && cargo test --test project_lifecycle`
Expected: 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/project/lifecycle.rs src-tauri/tests/project_lifecycle.rs
git commit -m "$(cat <<'EOF'
feat(project): open_project with schema version check

Opens existing project.db, validates schema version (returns
SchemaTooNew outcome when project version exceeds app version),
logs project_opened, bumps recent_projects.last_opened_at.

OpenOutcome is an enum because schema-too-new is a designed UX
state, not a system error — frontend takes a different branch
to show the upgrade screen.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 12: Project lifecycle — `close_project` + `check_last_open_project`

**Files:**
- Modify: `src-tauri/src/project/lifecycle.rs`
- Modify: `src-tauri/tests/project_lifecycle.rs`

- [ ] **Step 1: Write failing tests**

Append to `src-tauri/tests/project_lifecycle.rs`:

```rust
use bhc_lib::project::lifecycle::check_last_open_project;
use bhc_lib::types::LaunchResult;

#[test]
fn check_last_open_when_none_set_returns_none_set() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    assert!(matches!(result, LaunchResult::NoneSet));
}

#[test]
fn check_last_open_when_disabled_returns_disabled() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    app_db::set_state(&app_conn, "launch_behavior", "home_page").unwrap();
    app_db::set_state(&app_conn, "last_open_project_path", "C:\\whatever").unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    assert!(matches!(result, LaunchResult::Disabled));
}

#[test]
fn check_last_open_with_loadable_project_returns_loaded() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let project_tmp = tempdir().unwrap();
    create_project(&app_conn, project_tmp.path(), "Test", None).unwrap();
    app_db::set_state(
        &app_conn,
        "last_open_project_path",
        project_tmp.path().to_str().unwrap(),
    ).unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    assert!(matches!(result, LaunchResult::Loaded { .. }));
}

#[test]
fn check_last_open_with_missing_folder_returns_failed_and_cleans_recents() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    let missing = "C:\\definitely-does-not-exist-xyz";
    app_db::upsert_recent_project(&app_conn, missing, "Ghost").unwrap();
    app_db::set_state(&app_conn, "last_open_project_path", missing).unwrap();

    let result = check_last_open_project(&app_conn).unwrap();
    match result {
        LaunchResult::Failed { path, name, .. } => {
            assert_eq!(path, missing);
            assert_eq!(name, "Ghost");
        }
        _ => panic!("expected Failed"),
    }

    // Recents entry should be gone.
    let count: i64 = app_conn.query_row(
        "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
        [missing],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 0);

    // last_open_project_path setting should be cleared.
    let v = app_db::get_state(&app_conn, "last_open_project_path").unwrap();
    assert!(v.is_none());
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cd src-tauri && cargo test --test project_lifecycle`
Expected: 6 existing pass; 4 new FAIL (`check_last_open_project` not defined).

- [ ] **Step 3: Add `close_project` + `check_last_open_project` to `lifecycle.rs`**

Append to `src-tauri/src/project/lifecycle.rs`:

```rust
use crate::types::LaunchResult;

/// Drops the AppState current_project handle. Called from the command
/// layer (this Rust API doesn't own AppState).
///
/// Side effect: clears `last_open_project_path` in app.db, so a subsequent
/// launch with sticky session enabled lands on Home.
pub fn clear_sticky_session(app_conn: &Connection) -> Result<(), LifecycleError> {
    app_conn.execute(
        "DELETE FROM app_state WHERE key = 'last_open_project_path'",
        [],
    )?;
    Ok(())
}

/// Sets `last_open_project_path` so the next launch can sticky-restore.
pub fn set_sticky_session(app_conn: &Connection, folder: &Path) -> Result<(), LifecycleError> {
    app_db::set_state(app_conn, "last_open_project_path", &folder.display().to_string())?;
    Ok(())
}

/// Run at app launch. Decides whether to sticky-restore, land on Home,
/// or show a failure screen.
pub fn check_last_open_project(app_conn: &Connection) -> Result<LaunchResult, LifecycleError> {
    // Honor the launch_behavior setting.
    let behavior = app_db::get_state(app_conn, "launch_behavior")?
        .unwrap_or_else(|| "last_project".to_string());
    if behavior == "home_page" {
        return Ok(LaunchResult::Disabled);
    }

    let last_path = match app_db::get_state(app_conn, "last_open_project_path")? {
        Some(p) => p,
        None => return Ok(LaunchResult::NoneSet),
    };

    let folder = PathBuf::from(&last_path);

    // Look up the friendly name from recents (if any) for error reporting.
    let name = app_conn
        .query_row(
            "SELECT name FROM recent_projects WHERE path = ?1",
            [&last_path],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .unwrap_or_else(|| String::from("(unknown)"));

    // Folder still exists?
    if !folder.exists() {
        app_db::remove_recent_project(app_conn, &last_path)?;
        clear_sticky_session(app_conn)?;
        return Ok(LaunchResult::Failed {
            path: last_path,
            name,
            reason: "Folder no longer exists.".to_string(),
        });
    }

    // project.db still there?
    if !project_db_path(&folder).exists() {
        app_db::remove_recent_project(app_conn, &last_path)?;
        clear_sticky_session(app_conn)?;
        return Ok(LaunchResult::Failed {
            path: last_path,
            name,
            reason: "Project metadata (.bhc/project.db) is missing.".to_string(),
        });
    }

    // Try opening.
    match open_project(app_conn, &folder)? {
        OpenOutcome::Loaded { info, connection: _ } => Ok(LaunchResult::Loaded { info }),
        OpenOutcome::SchemaTooNew { path, name, project_version, app_version } => {
            // Don't auto-remove on schema-too-new — user can upgrade the app.
            Ok(LaunchResult::SchemaTooNew { path, name, project_version, app_version })
        }
    }
}
```

Note: `check_last_open_project` calls `open_project` which returns a `Connection`. We discard the connection here — the Tauri command layer wraps this and installs the connection into AppState. Refactor in Task 14 if this awkwardness shows up.

Actually — better: extract the open logic so we can use it without producing a Connection we throw away. **Revise**: don't discard. Return the connection up through `LaunchResult::Loaded { info, connection }` — but `LaunchResult` is exported via ts-rs and can't carry a Connection. Solution: keep two separate APIs. `check_last_open_project` for the boot decision, and have the command layer call `open_project` again if `Loaded`. Slightly wasteful but clean. Document in a code comment.

Revised `check_last_open_project` last block:

```rust
    // Try opening (this returns OpenOutcome; we drop the connection here
    // — the Tauri command layer re-opens to install in AppState).
    match open_project(app_conn, &folder)? {
        OpenOutcome::Loaded { info, .. } => Ok(LaunchResult::Loaded { info }),
        OpenOutcome::SchemaTooNew { path, name, project_version, app_version } => {
            Ok(LaunchResult::SchemaTooNew { path, name, project_version, app_version })
        }
    }
```

- [ ] **Step 4: Run tests, verify pass**

Run: `cd src-tauri && cargo test --test project_lifecycle`
Expected: 10 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/project/lifecycle.rs src-tauri/tests/project_lifecycle.rs
git commit -m "$(cat <<'EOF'
feat(project): close + check_last_open_project (sticky session)

Adds clear_sticky_session and set_sticky_session helpers (operate on
app.db only — Connection lifecycle is the command layer's job).

check_last_open_project reads launch_behavior + last_open_project_path
from app.db and returns a LaunchResult variant the command layer
exposes to the frontend. Auto-cleans dead recents entries on
NotFound/NotAProject; does NOT auto-clean on SchemaTooNew so the
user can upgrade and try again.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 13: Project delete (recursive folder removal)

**Files:**
- Create: `src-tauri/src/project/delete.rs`
- Create: `src-tauri/tests/project_delete.rs`
- Modify: `src-tauri/src/project/mod.rs`

- [ ] **Step 1: Write failing test**

Create `src-tauri/tests/project_delete.rs`:

```rust
use bhc_lib::db::app_db;
use bhc_lib::project::lifecycle::create_project;
use bhc_lib::project::delete::delete_project;
use tempfile::tempdir;

#[test]
fn delete_project_removes_folder_and_recents_row() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path().to_path_buf();

    create_project(&app_conn, &folder, "Test", None).unwrap();
    assert!(folder.exists());
    assert!(folder.join(".bhc").join("project.db").exists());

    delete_project(&app_conn, &folder).expect("delete");

    assert!(!folder.exists(), "project folder should be gone");

    let count: i64 = app_conn.query_row(
        "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
        [folder.to_str().unwrap()],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn delete_project_with_evidence_files_deletes_them_too() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();
    let project_tmp = tempdir().unwrap();
    let folder = project_tmp.path().to_path_buf();

    create_project(&app_conn, &folder, "Test", None).unwrap();

    // Drop some "evidence" files in the folder.
    std::fs::write(folder.join("evidence-DC01.evtx"), b"fake").unwrap();
    std::fs::create_dir(folder.join("subfolder")).unwrap();
    std::fs::write(folder.join("subfolder").join("more.evtx"), b"fake").unwrap();

    delete_project(&app_conn, &folder).expect("delete");
    assert!(!folder.exists());
}

#[test]
fn delete_project_on_missing_folder_still_cleans_recents() {
    let app_tmp = tempdir().unwrap();
    let app_conn = app_db::open_or_create(&app_tmp.path().join("app.db")).unwrap();

    // Insert a stale recents entry.
    app_db::upsert_recent_project(&app_conn, "C:\\does-not-exist", "Ghost").unwrap();

    delete_project(&app_conn, std::path::Path::new("C:\\does-not-exist"))
        .expect("delete should succeed (recents-only cleanup)");

    let count: i64 = app_conn.query_row(
        "SELECT COUNT(*) FROM recent_projects WHERE path = ?1",
        ["C:\\does-not-exist"],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 0);
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cd src-tauri && cargo test --test project_delete`
Expected: FAIL — `project::delete` doesn't exist.

- [ ] **Step 3: Create `src-tauri/src/project/delete.rs`**

```rust
use std::fs;
use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

use crate::db::app_db;

#[derive(Debug, Error)]
pub enum DeleteError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("app db: {0}")]
    AppDb(#[from] app_db::AppDbError),
}

/// Recursively deletes the project folder at `folder` and removes the
/// corresponding `recent_projects` entry from app.db.
///
/// If the folder doesn't exist, just cleans up recents (no error).
///
/// **Safety:** the caller MUST first close any open Connection to this
/// project's project.db, or the file lock will prevent deletion on
/// Windows.
pub fn delete_project(app_conn: &Connection, folder: &Path) -> Result<(), DeleteError> {
    if folder.exists() {
        fs::remove_dir_all(folder)?;
    }
    app_db::remove_recent_project(app_conn, &folder.display().to_string())?;
    Ok(())
}
```

- [ ] **Step 4: Register in `project/mod.rs`**

```rust
pub mod activity_log;
pub mod conflict;
pub mod delete;
pub mod lifecycle;
```

- [ ] **Step 5: Run tests, verify pass**

Run: `cd src-tauri && cargo test --test project_delete`
Expected: 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/project/delete.rs src-tauri/src/project/mod.rs src-tauri/tests/project_delete.rs
git commit -m "$(cat <<'EOF'
feat(project): recursive project delete

Removes the entire project folder including evidence files inside it
(per spec — confirmation dialog text spells this out for the user).
Cleans recents on success; tolerant of already-missing folders.

Caller MUST close any open project.db Connection first (Windows
file locking) — documented in the function comment.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 14: Extend `AppState` for `current_project`

**Why:** AppState currently only holds `app_db` and `paths`. Add an `Option<CurrentProject>` field. Don't expose externally — only the project commands touch it.

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add CurrentProject type + extend AppState**

Update `src-tauri/src/lib.rs`:

```rust
use std::sync::Mutex;

pub mod commands;
pub mod db;
pub mod paths;
pub mod platform;
pub mod project;
pub mod types;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

/// Holds the currently-open project's metadata + DB connection.
/// Single project at a time per M2 design.
pub struct CurrentProject {
    pub info: types::ProjectInfo,
    pub db: Mutex<rusqlite::Connection>,
}

pub struct AppState {
    pub app_db: Mutex<rusqlite::Connection>,
    pub paths: paths::AppPaths,
    pub current_project: Mutex<Option<CurrentProject>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_paths = paths::AppPaths::from_current_exe()
                .expect("resolve app paths");
            let conn = db::app_db::open_or_create(&app_paths.app_db)
                .expect("open or create app.db");
            app.manage(AppState {
                app_db: Mutex::new(conn),
                paths: app_paths,
                current_project: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_version,
            commands::app::get_app_state,
            commands::app::set_app_state,
            // project commands registered in Task 16
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify build**

Run: `cd src-tauri && cargo test`
Expected: all green; nothing references `current_project` yet so no behavior change.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(app-state): add current_project slot

AppState now holds Mutex<Option<CurrentProject>>. Commands in the
next task will read/write through this slot. The Connection is
held in its own Mutex (separate from app_db) so they can be locked
independently.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 15: Tauri commands — project lifecycle wrappers

**Files:**
- Create: `src-tauri/src/commands/projects.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Create `src-tauri/src/commands/projects.rs` — Part A (lifecycle commands)**

```rust
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::State;

use crate::commands::error::CommandError;
use crate::db::app_db;
use crate::project::lifecycle::{
    self, activity_log_path, check_last_open_project as check_last,
    clear_sticky_session, create_project as create_p, open_project as open_p,
    set_sticky_session, LifecycleError, OpenOutcome,
};
use crate::types::{LaunchResult, ProjectInfo, RecentProject, RecentProjectListEntry};
use crate::{AppState, CurrentProject};

impl From<LifecycleError> for CommandError {
    fn from(e: LifecycleError) -> Self {
        match e {
            LifecycleError::AlreadyExists { path } => CommandError::AlreadyExists { path },
            LifecycleError::NotAProject { path } => CommandError::NotAProject { path },
            LifecycleError::NotFound { path } => CommandError::NotFound { path },
            LifecycleError::Io(e) => CommandError::Io { message: e.to_string() },
            LifecycleError::Sql(e) => CommandError::Db { message: e.to_string() },
            LifecycleError::ProjectDb(e) => CommandError::Db { message: e.to_string() },
            LifecycleError::AppDb(e) => CommandError::Db { message: e.to_string() },
            LifecycleError::ActivityLog(e) => CommandError::Io { message: e.to_string() },
            LifecycleError::Conflict(e) => CommandError::Internal { message: e.to_string() },
        }
    }
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    folder_path: String,
    name: String,
    description: Option<String>,
) -> Result<ProjectInfo, CommandError> {
    let folder = PathBuf::from(&folder_path);
    let app_conn = state.app_db.lock()?;

    let info = create_p(&app_conn, &folder, &name, description.as_deref())?;

    // Re-open the project to grab its Connection for AppState.
    let connection = crate::db::project_db::open_or_create(&lifecycle::project_db_path(&folder))
        .map_err(|e| CommandError::Db { message: e.to_string() })?;

    *state.current_project.lock()? = Some(CurrentProject {
        info: info.clone(),
        db: Mutex::new(connection),
    });

    set_sticky_session(&app_conn, &folder)?;
    Ok(info)
}

#[tauri::command]
pub fn open_project(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<LaunchResult, CommandError> {
    let folder = PathBuf::from(&folder_path);
    let app_conn = state.app_db.lock()?;

    match open_p(&app_conn, &folder)? {
        OpenOutcome::Loaded { info, connection } => {
            *state.current_project.lock()? = Some(CurrentProject {
                info: info.clone(),
                db: Mutex::new(connection),
            });
            set_sticky_session(&app_conn, &folder)?;
            Ok(LaunchResult::Loaded { info })
        }
        OpenOutcome::SchemaTooNew { path, name, project_version, app_version } => {
            // Don't install; let frontend show the upgrade screen.
            Ok(LaunchResult::SchemaTooNew { path, name, project_version, app_version })
        }
    }
}

#[tauri::command]
pub fn close_project(state: State<'_, AppState>) -> Result<(), CommandError> {
    *state.current_project.lock()? = None;
    let app_conn = state.app_db.lock()?;
    clear_sticky_session(&app_conn)?;
    Ok(())
}

#[tauri::command]
pub fn get_current_project(state: State<'_, AppState>) -> Result<Option<ProjectInfo>, CommandError> {
    let current = state.current_project.lock()?;
    Ok(current.as_ref().map(|cp| cp.info.clone()))
}

#[tauri::command]
pub fn check_last_open_project_cmd(state: State<'_, AppState>) -> Result<LaunchResult, CommandError> {
    let app_conn = state.app_db.lock()?;
    let result = check_last(&app_conn)?;

    // If Loaded, install into AppState (re-open to get the connection).
    if let LaunchResult::Loaded { ref info } = result {
        drop(app_conn); // release lock before re-acquiring through open_p
        let app_conn = state.app_db.lock()?;
        let folder = PathBuf::from(&info.folder_path);
        match open_p(&app_conn, &folder)? {
            OpenOutcome::Loaded { info: i, connection } => {
                *state.current_project.lock()? = Some(CurrentProject {
                    info: i,
                    db: Mutex::new(connection),
                });
            }
            OpenOutcome::SchemaTooNew { .. } => {
                // Shouldn't happen — check_last would have returned SchemaTooNew already.
                return Err(CommandError::Internal {
                    message: "open_project returned SchemaTooNew after check_last said Loaded".into(),
                });
            }
        }
    }
    Ok(result)
}
```

- [ ] **Step 2: Append Part B (chooser / list / delete commands)**

```rust
#[tauri::command]
pub fn list_recent_projects(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<RecentProject>, CommandError> {
    let app_conn = state.app_db.lock()?;

    // If no explicit limit, read recent_projects_count from settings (default 5).
    let lim = match limit {
        Some(n) => n,
        None => app_db::get_state(&app_conn, "recent_projects_count")
            .ok()
            .flatten()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5),
    };

    let rows = app_db::list_recent_projects(&app_conn)?;
    let out: Vec<RecentProject> = rows
        .into_iter()
        .take(lim as usize)
        .map(|(path, name, last_opened_at)| RecentProject { path, name, last_opened_at })
        .collect();
    Ok(out)
}

#[tauri::command]
pub fn list_all_projects(state: State<'_, AppState>) -> Result<Vec<RecentProjectListEntry>, CommandError> {
    let app_conn = state.app_db.lock()?;
    let rows = app_db::list_recent_projects(&app_conn)?;

    let mut out = Vec::with_capacity(rows.len());
    for (path, name, last_opened_at) in rows {
        let log = std::path::Path::new(&path).join(".bhc").join("activity.log");
        let last_modified = std::fs::metadata(&log)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| {
                use time::OffsetDateTime;
                use time::format_description::well_known::Rfc3339;
                OffsetDateTime::from(t).format(&Rfc3339).ok()
            });
        out.push(RecentProjectListEntry { path, name, last_opened_at, last_modified });
    }
    Ok(out)
}

#[tauri::command]
pub fn remove_recent_project(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<(), CommandError> {
    let app_conn = state.app_db.lock()?;
    app_db::remove_recent_project(&app_conn, &folder_path)
        .map(|_| ())
        .map_err(|e| CommandError::Db { message: e.to_string() })
}

#[tauri::command]
pub fn delete_project(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<(), CommandError> {
    let folder = PathBuf::from(&folder_path);

    // If this is the currently-open project, close it first (drops DB Connection).
    {
        let mut current = state.current_project.lock()?;
        if let Some(cp) = current.as_ref() {
            if cp.info.folder_path == folder_path {
                *current = None;
            }
        }
    }

    let app_conn = state.app_db.lock()?;
    crate::project::delete::delete_project(&app_conn, &folder)
        .map_err(|e| match e {
            crate::project::delete::DeleteError::Io(e) => CommandError::Io { message: e.to_string() },
            crate::project::delete::DeleteError::Sql(e) => CommandError::Db { message: e.to_string() },
            crate::project::delete::DeleteError::AppDb(e) => CommandError::Db { message: e.to_string() },
        })?;
    Ok(())
}
```

- [ ] **Step 3: Register module + commands**

`src-tauri/src/commands/mod.rs`:

```rust
pub mod app;
pub mod error;
pub mod projects;
```

`src-tauri/src/lib.rs` — extend the `invoke_handler` list:

```rust
.invoke_handler(tauri::generate_handler![
    commands::app::get_app_version,
    commands::app::get_app_state,
    commands::app::set_app_state,
    commands::projects::create_project,
    commands::projects::open_project,
    commands::projects::close_project,
    commands::projects::get_current_project,
    commands::projects::check_last_open_project_cmd,
    commands::projects::list_recent_projects,
    commands::projects::list_all_projects,
    commands::projects::remove_recent_project,
    commands::projects::delete_project,
])
```

- [ ] **Step 4: Verify build**

Run: `cd src-tauri && cargo test`
Expected: all green. New ts-rs exports for `RecentProjectListEntry` etc. are written to `src/lib/generated/`.

Run: `pnpm build`
Expected: success (no frontend consumers yet, but type imports compile).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/projects.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(commands): expose project lifecycle to frontend

Adds 9 Tauri commands wrapping the project Rust API: create_project,
open_project, close_project, get_current_project,
check_last_open_project_cmd, list_recent_projects, list_all_projects,
remove_recent_project, delete_project.

All return Result<T, CommandError> with typed error variants. The
lifecycle API stays UI-agnostic; these wrappers marshal AppState
mutex locks and install/uninstall the current project's DB Connection.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 16: Frontend IPC wrappers + current-project store + date formatter

**Files:**
- Create: `src/lib/ipc/projects.ts`
- Modify: `src/lib/ipc/app.ts` (extend for new app_state keys)
- Create: `src/lib/stores/currentProject.ts`
- Create: `src/lib/helpers/formatDate.ts`

- [ ] **Step 1: Create `src/lib/ipc/projects.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { ProjectInfo } from '$lib/generated/ProjectInfo';
import type { RecentProject } from '$lib/generated/RecentProject';
import type { RecentProjectListEntry } from '$lib/generated/RecentProjectListEntry';
import type { LaunchResult } from '$lib/generated/LaunchResult';

export async function createProject(
  folderPath: string,
  name: string,
  description?: string
): Promise<ProjectInfo> {
  return await invoke<ProjectInfo>('create_project', { folderPath, name, description });
}

export async function openProject(folderPath: string): Promise<LaunchResult> {
  return await invoke<LaunchResult>('open_project', { folderPath });
}

export async function closeProject(): Promise<void> {
  await invoke<void>('close_project');
}

export async function getCurrentProject(): Promise<ProjectInfo | null> {
  return await invoke<ProjectInfo | null>('get_current_project');
}

export async function checkLastOpenProject(): Promise<LaunchResult> {
  return await invoke<LaunchResult>('check_last_open_project_cmd');
}

export async function listRecentProjects(limit?: number): Promise<RecentProject[]> {
  return await invoke<RecentProject[]>('list_recent_projects', { limit });
}

export async function listAllProjects(): Promise<RecentProjectListEntry[]> {
  return await invoke<RecentProjectListEntry[]>('list_all_projects');
}

export async function removeRecentProject(folderPath: string): Promise<void> {
  await invoke<void>('remove_recent_project', { folderPath });
}

export async function deleteProject(folderPath: string): Promise<void> {
  await invoke<void>('delete_project', { folderPath });
}
```

- [ ] **Step 2: Extend `src/lib/ipc/app.ts`**

Add typed helpers for the new settings keys:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { AppVersion } from '$lib/generated/AppVersion';

export async function getAppVersion(): Promise<AppVersion> {
  return await invoke<AppVersion>('get_app_version');
}

export async function getAppState(key: string): Promise<string | null> {
  return await invoke<string | null>('get_app_state', { key });
}

export async function setAppState(key: string, value: string): Promise<void> {
  await invoke<void>('set_app_state', { key, value });
}

// Typed convenience wrappers for the M2 settings.

export type LaunchBehavior = 'last_project' | 'home_page';
export type TimezoneMode = 'UTC' | 'Local';

export async function getLaunchBehavior(): Promise<LaunchBehavior> {
  const v = await getAppState('launch_behavior');
  return v === 'home_page' ? 'home_page' : 'last_project';
}

export async function setLaunchBehavior(v: LaunchBehavior): Promise<void> {
  await setAppState('launch_behavior', v);
}

export async function getDefaultTimezone(): Promise<TimezoneMode> {
  const v = await getAppState('default_timezone');
  return v === 'Local' ? 'Local' : 'UTC';
}

export async function setDefaultTimezone(v: TimezoneMode): Promise<void> {
  await setAppState('default_timezone', v);
}

export async function getRecentProjectsCount(): Promise<number> {
  const v = await getAppState('recent_projects_count');
  if (v === null) return 5;
  const n = parseInt(v, 10);
  return Number.isFinite(n) && n >= 1 && n <= 50 ? n : 5;
}

export async function setRecentProjectsCount(n: number): Promise<void> {
  await setAppState('recent_projects_count', String(n));
}
```

- [ ] **Step 3: Create `src/lib/stores/currentProject.ts`**

```typescript
import { writable, type Readable } from 'svelte/store';
import type { ProjectInfo } from '$lib/generated/ProjectInfo';
import {
  closeProject as ipcClose,
  createProject as ipcCreate,
  getCurrentProject as ipcGet,
  openProject as ipcOpen,
} from '$lib/ipc/projects';

const { subscribe, set } = writable<ProjectInfo | null>(null);

export const currentProject: Readable<ProjectInfo | null> = { subscribe };

/** Hydrate the store from Rust on app boot. */
export async function loadCurrentProject(): Promise<void> {
  set(await ipcGet());
}

/** Create + auto-install as current. */
export async function createAndInstall(
  folderPath: string,
  name: string,
  description?: string
): Promise<ProjectInfo> {
  const info = await ipcCreate(folderPath, name, description);
  set(info);
  return info;
}

/** Open + auto-install as current. Returns the LaunchResult so callers
 *  can branch on schema-too-new etc. */
export async function openAndInstall(folderPath: string) {
  const result = await ipcOpen(folderPath);
  if (result.kind === 'Loaded') {
    set(result.info);
  }
  return result;
}

/** Close + clear store. */
export async function close(): Promise<void> {
  await ipcClose();
  set(null);
}
```

- [ ] **Step 4: Create `src/lib/helpers/formatDate.ts`**

```typescript
import { getDefaultTimezone, type TimezoneMode } from '$lib/ipc/app';

/**
 * Formats a UTC ISO 8601 timestamp string per the user's default_timezone
 * setting. Cached per-render where the caller uses the same mode.
 *
 * Async to allow reading the setting from app.db; in hot paths, callers
 * should fetch the mode once via getDefaultTimezone and call formatDateSync.
 */
export async function formatDate(isoUtc: string): Promise<string> {
  const mode = await getDefaultTimezone();
  return formatDateSync(isoUtc, mode);
}

export function formatDateSync(isoUtc: string, mode: TimezoneMode): string {
  const d = new Date(isoUtc);
  if (isNaN(d.getTime())) return isoUtc; // fall back to raw string

  if (mode === 'UTC') {
    // YYYY-MM-DD HH:MM:SS UTC
    const iso = d.toISOString();
    return `${iso.slice(0, 10)} ${iso.slice(11, 19)} UTC`;
  }
  // Local timezone via Intl
  return new Intl.DateTimeFormat(undefined, {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
    hour12: false,
  }).format(d);
}
```

- [ ] **Step 5: Verify build**

Run: `pnpm build`
Expected: success. All generated types resolve.

- [ ] **Step 6: Commit**

```bash
git add src/lib/ipc/projects.ts src/lib/ipc/app.ts src/lib/stores/currentProject.ts src/lib/helpers/formatDate.ts
git commit -m "$(cat <<'EOF'
feat(frontend): IPC wrappers + currentProject store + date helper

projects.ts wraps all 9 project commands with typed Promise<T> returns.
app.ts adds typed wrappers for the new settings keys (launch_behavior,
default_timezone, recent_projects_count) with sensible defaults.
currentProject store mirrors the backend's current_project for reactive
UI updates. formatDate honors the default_timezone setting.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 17: Breadcrumbs component + wire into layout

**Files:**
- Create: `src/lib/components/Breadcrumbs.svelte`
- Modify: `src/routes/+layout.svelte`
- Add shadcn-svelte breadcrumb primitives via CLI (see Step 1)

- [ ] **Step 1: Add shadcn-svelte breadcrumb component**

Run: `pnpm dlx shadcn-svelte@latest add breadcrumb`
Expected: adds `src/lib/components/ui/breadcrumb/` with the primitives.

If `pnpm dlx` complains about minimum release age, pin the version: `pnpm dlx shadcn-svelte@1.x.x add breadcrumb` (check published versions ≥ 7 days old).

- [ ] **Step 2: Create `src/lib/components/Breadcrumbs.svelte`**

```svelte
<script lang="ts">
  import { page } from '$app/state';
  import { currentProject } from '$lib/stores/currentProject';
  import {
    Breadcrumb,
    BreadcrumbList,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbPage,
    BreadcrumbSeparator,
  } from '$lib/components/ui/breadcrumb';

  type Crumb = { label: string; href: string | null };

  function deriveCrumbs(pathname: string, projectName: string | undefined): Crumb[] {
    const crumbs: Crumb[] = [{ label: 'Home', href: '/' }];

    if (pathname === '/') {
      crumbs[0].href = null; // current page
      return crumbs;
    }

    const parts = pathname.split('/').filter(Boolean);

    if (parts[0] === 'projects') {
      crumbs.push({ label: 'Projects', href: '/projects/' });
      if (parts[1] === 'current') {
        crumbs.push({ label: projectName ?? 'Project', href: null });
      } else if (parts.length === 1) {
        crumbs[crumbs.length - 1].href = null;
      }
    } else if (parts[0] === 'settings') {
      crumbs.push({ label: 'Settings', href: '/settings' });
      if (parts[1] === 'about') {
        crumbs.push({ label: 'About', href: null });
      } else {
        crumbs[crumbs.length - 1].href = null;
      }
    } else if (parts[0] === 'tools') {
      crumbs.push({ label: 'Tools', href: null });
      if (parts[1] === 'hayabusa') {
        crumbs.push({ label: 'Hayabusa', href: null });
      } else if (parts[1] === 'chainsaw') {
        crumbs.push({ label: 'Chainsaw', href: null });
      }
    }
    return crumbs;
  }

  let crumbs = $derived(deriveCrumbs(page.url.pathname, $currentProject?.project.name));
</script>

<Breadcrumb class="border-b bg-white px-6 py-3">
  <BreadcrumbList>
    {#each crumbs as crumb, i}
      <BreadcrumbItem>
        {#if crumb.href}
          <BreadcrumbLink href={crumb.href}>{crumb.label}</BreadcrumbLink>
        {:else}
          <BreadcrumbPage>{crumb.label}</BreadcrumbPage>
        {/if}
      </BreadcrumbItem>
      {#if i < crumbs.length - 1}
        <BreadcrumbSeparator />
      {/if}
    {/each}
  </BreadcrumbList>
</Breadcrumb>
```

- [ ] **Step 3: Wire into `src/routes/+layout.svelte`**

```svelte
<script lang="ts">
  import '../app.css';
  import OsGate from '$lib/components/OsGate.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Breadcrumbs from '$lib/components/Breadcrumbs.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<OsGate>
  <div class="flex h-screen">
    <Sidebar />
    <main class="flex flex-1 flex-col overflow-hidden bg-slate-50">
      <Breadcrumbs />
      <div class="flex-1 overflow-y-auto">
        {@render children()}
      </div>
    </main>
  </div>
</OsGate>
```

- [ ] **Step 4: Verify build**

Run: `pnpm build`
Expected: success. Breadcrumb component renders.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/Breadcrumbs.svelte src/lib/components/ui/breadcrumb/ src/routes/+layout.svelte package.json pnpm-lock.yaml
git commit -m "$(cat <<'EOF'
feat(ui): breadcrumbs on every page

Classic shadcn-svelte breadcrumb strip at the top of every route.
Project name is pulled from currentProject store when on
/projects/current; falls back to "Project" if not loaded.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 18: Sidebar — fix Settings highlight bug + add project section

**Files:**
- Modify: `src/lib/components/Sidebar.svelte`

- [ ] **Step 1: Update `src/lib/components/Sidebar.svelte`**

```svelte
<script lang="ts">
  import { page } from '$app/state';
  import { Separator } from '$lib/components/ui/separator';
  import SidebarFooter from './SidebarFooter.svelte';
  import { currentProject } from '$lib/stores/currentProject';

  type NavItem = { href: string; label: string };
  const topItems: NavItem[] = [
    { href: '/', label: 'Home' },
    { href: '/projects', label: 'Projects' }
  ];
  const toolsItems: NavItem[] = [
    { href: '/tools/hayabusa', label: 'Hayabusa' },
    { href: '/tools/chainsaw', label: 'Chainsaw' }
  ];
  const bottomItems: NavItem[] = [{ href: '/settings', label: 'Settings' }];

  // Bug fix: previously startsWith match caused /settings/about to highlight
  // Settings. Now we require an exact match OR a strict subpath match for
  // items that legitimately have children (currently only /tools/*).
  function isActive(href: string) {
    const path = page.url.pathname;
    if (href === '/') return path === '/';
    if (href === '/settings') return path === '/settings'; // exact only
    if (href === '/projects') return path === '/projects' || path.startsWith('/projects/');
    if (href.startsWith('/tools/')) return path === href;
    return path === href;
  }
</script>

<aside class="flex h-screen w-56 flex-col border-r bg-white">
  <nav class="flex-1 overflow-y-auto py-4">
    {#each topItems as item}
      <a
        href={item.href}
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={isActive(item.href)}
        class:font-medium={isActive(item.href)}
      >
        {item.label}
      </a>
    {/each}

    {#if $currentProject}
      <Separator class="my-2" />
      <div class="px-4 py-1 text-xs font-semibold uppercase text-slate-500">
        Project
      </div>
      <a
        href="/projects/current"
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={page.url.pathname === '/projects/current'}
        class:font-medium={page.url.pathname === '/projects/current'}
      >
        ▸ {$currentProject.project.name}
      </a>
    {/if}

    <Separator class="my-2" />

    <div class="px-4 py-1 text-xs font-semibold uppercase text-slate-500">Tools</div>
    {#each toolsItems as item}
      <a
        href={item.href}
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={isActive(item.href)}
        class:font-medium={isActive(item.href)}
      >
        {item.label}
      </a>
    {/each}

    <Separator class="my-2" />

    {#each bottomItems as item}
      <a
        href={item.href}
        class="block px-4 py-2 text-sm hover:bg-slate-100"
        class:bg-slate-100={isActive(item.href)}
        class:font-medium={isActive(item.href)}
      >
        {item.label}
      </a>
    {/each}
  </nav>

  <SidebarFooter />
</aside>
```

- [ ] **Step 2: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 3: Manual smoke (defer until orchestration is wired in Task 26 — for now, just confirm compile)**

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/Sidebar.svelte
git commit -m "$(cat <<'EOF'
fix(sidebar): exact-match Settings + add project section

Bug: /settings/about activated the Settings sidebar item because
isActive used startsWith. Now exact match for Settings; startsWith
preserved for /projects (legitimate sub-routes) and tools.

Adds a Project section that appears only when currentProject is
loaded. Links to /projects/current with active state. M3+ adds
Jobs and Queue sub-links here.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 19: Funny image placeholder SVGs

**Files:**
- Create: `static/img/no-projects-placeholder.svg`
- Create: `static/img/sticky-fail-placeholder.svg`

- [ ] **Step 1: Create `static/img/no-projects-placeholder.svg`**

A simple placeholder (real asset commissioned via Issue #26):

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width="200" height="200">
  <rect width="200" height="200" rx="12" fill="#f1f5f9"/>
  <text x="100" y="100" text-anchor="middle" dominant-baseline="middle"
        font-family="system-ui, sans-serif" font-size="14" fill="#64748b">
    [no-projects placeholder]
  </text>
  <text x="100" y="120" text-anchor="middle" dominant-baseline="middle"
        font-family="system-ui, sans-serif" font-size="10" fill="#94a3b8">
    asset commissioned via Issue #26
  </text>
</svg>
```

- [ ] **Step 2: Create `static/img/sticky-fail-placeholder.svg`**

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width="200" height="200">
  <rect width="200" height="200" rx="12" fill="#fef2f2"/>
  <text x="100" y="100" text-anchor="middle" dominant-baseline="middle"
        font-family="system-ui, sans-serif" font-size="14" fill="#b91c1c">
    [sticky-fail placeholder]
  </text>
  <text x="100" y="120" text-anchor="middle" dominant-baseline="middle"
        font-family="system-ui, sans-serif" font-size="10" fill="#dc2626">
    asset commissioned via Issue #27
  </text>
</svg>
```

- [ ] **Step 3: Verify file presence**

Run: `pnpm build`
Expected: SVGs are picked up as static assets.

- [ ] **Step 4: Commit**

```bash
git add static/img/no-projects-placeholder.svg static/img/sticky-fail-placeholder.svg
git commit -m "$(cat <<'EOF'
chore(assets): add funny-image placeholders

Two SVG placeholders for the no-projects and sticky-fail empty
states. Real assets commissioned via Issues #26 and #27 will swap
these out.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 20: Home page rewrite

**Files:**
- Modify: `src/routes/+page.svelte`

- [ ] **Step 1: Replace Home page content**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { listRecentProjects } from '$lib/ipc/projects';
  import { openAndInstall } from '$lib/stores/currentProject';
  import { formatDateSync, type TimezoneMode } from '$lib/helpers/formatDate';
  import { getDefaultTimezone, getRecentProjectsCount } from '$lib/ipc/app';
  import NewProjectSheet from '$lib/components/NewProjectSheet.svelte';
  import type { RecentProject } from '$lib/generated/RecentProject';

  let recents = $state<RecentProject[]>([]);
  let tzMode = $state<TimezoneMode>('UTC');
  let count = $state<number>(5);
  let loaded = $state(false);
  let newSheetOpen = $state(false);

  onMount(async () => {
    [recents, tzMode, count] = await Promise.all([
      listRecentProjects(),
      getDefaultTimezone(),
      getRecentProjectsCount(),
    ]);
    loaded = true;
  });

  async function handleOpenRecent(path: string) {
    const result = await openAndInstall(path);
    if (result.kind === 'Loaded') {
      goto('/projects/current');
    } else if (result.kind === 'SchemaTooNew') {
      // SchemaTooNew handled by the sticky-fail/upgrade screen elsewhere.
      // For now, surface an alert. Improve in Task 25 once screen exists.
      alert(`Project requires app v${result.app_version}+; you have v${result.project_version}.`);
    }
  }

  function refreshRecents() {
    listRecentProjects().then((r) => (recents = r));
  }
</script>

<div class="mx-auto max-w-3xl space-y-6 p-8">
  <header>
    <h1 class="text-3xl font-bold">Better Hayabusa</h1>
    <p class="mt-2 text-slate-600">
      A graphical UI for Hayabusa and related tools (Chainsaw and Takajo).
      Organize your investigations as projects, configure tool runs as named
      jobs, and review run history in one place.
    </p>
    <p class="mt-2 text-sm italic text-slate-500">
      Making your life suck a little less...
    </p>
  </header>

  <Card>
    <CardHeader>
      <CardTitle>Get started</CardTitle>
    </CardHeader>
    <CardContent class="flex gap-3">
      <Button onclick={() => (newSheetOpen = true)}>New project</Button>
      <Button
        variant="outline"
        disabled={loaded && recents.length === 0}
        href="/projects/"
      >
        Open project
      </Button>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Recent projects</CardTitle>
    </CardHeader>
    <CardContent>
      {#if !loaded}
        <p class="text-sm text-slate-500">Loading...</p>
      {:else if recents.length === 0}
        <div class="flex flex-col items-center gap-3 py-6 text-center">
          <img
            src="/img/no-projects-placeholder.svg"
            alt="No projects"
            class="h-32 w-32"
          />
          <p class="text-sm font-medium text-slate-700">
            You have no projects. Looks like somebody better get off their ass
            and GET TO WORK!
          </p>
        </div>
      {:else}
        <ul class="divide-y">
          {#each recents.slice(0, count) as r}
            <li>
              <button
                class="flex w-full items-center justify-between px-2 py-2 text-left hover:bg-slate-100"
                onclick={() => handleOpenRecent(r.path)}
              >
                <span class="text-sm font-medium">{r.name}</span>
                <span class="text-xs text-slate-500">
                  {formatDateSync(r.last_opened_at, tzMode)}
                </span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </CardContent>
  </Card>
</div>

<NewProjectSheet bind:open={newSheetOpen} oncreate={refreshRecents} />
```

(`NewProjectSheet` component lands in Task 21 — Svelte 5 accepts undefined components at parse but they need to exist at compile. So either order tasks so 21 lands first, OR add a stub import here. **Plan as-is**: defer building this Home until after Task 21. Reorder execution if needed.)

- [ ] **Step 2: Verify build (deferred until after Task 21)**

- [ ] **Step 3: Commit (deferred until Task 21)**

> **Reorder note:** the engineer should execute **Task 21 (NewProjectSheet)** before this task. The plan retains the original order for narrative flow, but `pnpm build` will fail on `<NewProjectSheet />` until Task 21 is committed. Acceptable to do Tasks 20+21+22+23+24 as one combined commit if iterating tightly.

---

## Task 21: `NewProjectSheet.svelte` component

**Files:**
- Create: `src/lib/components/NewProjectSheet.svelte`
- Add shadcn-svelte sheet, input, textarea, alert components via CLI

- [ ] **Step 1: Add shadcn-svelte components**

Run: `pnpm dlx shadcn-svelte@latest add sheet input textarea label alert`

- [ ] **Step 2: Create `src/lib/components/NewProjectSheet.svelte`**

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Label } from '$lib/components/ui/label';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetFooter,
    SheetHeader,
    SheetTitle,
  } from '$lib/components/ui/sheet';
  import { createAndInstall, openAndInstall } from '$lib/stores/currentProject';
  import type { CommandError } from '$lib/generated/CommandError';

  type Props = {
    open: boolean;
    oncreate?: () => void;
  };
  let { open = $bindable(false), oncreate }: Props = $props();

  let name = $state('');
  let folder = $state('');
  let description = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let conflictPath = $state<string | null>(null);

  function reset() {
    name = '';
    folder = '';
    description = '';
    error = null;
    conflictPath = null;
  }

  async function pickFolder() {
    const picked = await openDialog({ directory: true, multiple: false });
    if (typeof picked === 'string') {
      folder = picked;
      conflictPath = null;
      error = null;
    }
  }

  async function handleCreate() {
    if (!name.trim() || !folder.trim()) {
      error = 'Name and folder are required.';
      return;
    }
    busy = true;
    error = null;
    conflictPath = null;
    try {
      await createAndInstall(folder, name.trim(), description.trim() || undefined);
      oncreate?.();
      open = false;
      reset();
      goto('/projects/current');
    } catch (e) {
      const err = e as CommandError;
      if (err.kind === 'AlreadyExists') {
        conflictPath = err.path;
      } else if (err.kind === 'Io') {
        error = `I/O error: ${err.message}`;
      } else if (err.kind === 'Db') {
        error = `Database error: ${err.message}`;
      } else {
        error = `Failed to create project (${err.kind}).`;
      }
    } finally {
      busy = false;
    }
  }

  async function handleOpenInstead() {
    if (!conflictPath) return;
    busy = true;
    try {
      const result = await openAndInstall(conflictPath);
      if (result.kind === 'Loaded') {
        open = false;
        reset();
        goto('/projects/current');
      } else if (result.kind === 'SchemaTooNew') {
        error = `That project requires a newer app version.`;
      }
    } finally {
      busy = false;
    }
  }
</script>

<Sheet bind:open>
  <SheetContent side="right" class="sm:max-w-md">
    <SheetHeader>
      <SheetTitle>New project</SheetTitle>
      <SheetDescription>
        Create a new investigation project. You can change settings later.
      </SheetDescription>
    </SheetHeader>

    <div class="space-y-4 py-4">
      <div>
        <Label for="name">Project name</Label>
        <Input id="name" bind:value={name} placeholder="APT-29 sweep" />
      </div>

      <div>
        <Label for="folder">Default folder</Label>
        <div class="flex gap-2">
          <Input id="folder" bind:value={folder} readonly placeholder="Click Browse to pick a folder" />
          <Button variant="outline" onclick={pickFolder}>Browse</Button>
        </div>
        <p class="mt-1 text-xs text-slate-500">
          Source evidence and output paths can be set per-job to anywhere — this
          is just where the project metadata and logs live, and the default
          starting point for new jobs.
        </p>
      </div>

      <div>
        <Label for="description">Description (optional)</Label>
        <Textarea id="description" bind:value={description} rows={3} />
      </div>

      {#if conflictPath}
        <Alert>
          <AlertDescription>
            This folder is already a project — open it?
            <div class="mt-2 flex gap-2">
              <Button size="sm" onclick={handleOpenInstead}>Open</Button>
              <Button size="sm" variant="outline" onclick={() => (conflictPath = null)}>
                Cancel
              </Button>
            </div>
          </AlertDescription>
        </Alert>
      {/if}

      {#if error}
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      {/if}
    </div>

    <SheetFooter class="flex gap-2">
      <Button variant="outline" onclick={() => { open = false; reset(); }}>Cancel</Button>
      <Button onclick={handleCreate} disabled={busy}>
        {busy ? 'Creating...' : 'Create'}
      </Button>
    </SheetFooter>
  </SheetContent>
</Sheet>
```

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: success (Home page from Task 20 + this Sheet compile together).

- [ ] **Step 4: Commit (combine with Task 20)**

```bash
git add src/lib/components/NewProjectSheet.svelte src/lib/components/ui/sheet/ src/lib/components/ui/input/ src/lib/components/ui/textarea/ src/lib/components/ui/label/ src/lib/components/ui/alert/ src/routes/+page.svelte package.json pnpm-lock.yaml
git commit -m "$(cat <<'EOF'
feat(ui): Home rewrite + NewProjectSheet

Home now shows the new copy + tagline + New/Open buttons (Open
disabled when no projects exist) + recent-projects list (limit
from setting). Empty state shows the funny-image placeholder
and lazy-guy copy.

NewProjectSheet handles folder-pick, name+description input, and
the conflict alert ("already a project — open it instead?").
Routes to /projects/current on success.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 22: `ConfirmDeleteProject.svelte` component

**Files:**
- Create: `src/lib/components/ConfirmDeleteProject.svelte`
- Add shadcn-svelte alert-dialog

- [ ] **Step 1: Add shadcn-svelte alert-dialog**

Run: `pnpm dlx shadcn-svelte@latest add alert-dialog`

- [ ] **Step 2: Create `src/lib/components/ConfirmDeleteProject.svelte`**

```svelte
<script lang="ts">
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';

  type Props = {
    open: boolean;
    name: string;
    path: string;
    onconfirm: () => void;
  };
  let { open = $bindable(false), name, path, onconfirm }: Props = $props();
</script>

<AlertDialog bind:open>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Delete project '{name}'?</AlertDialogTitle>
      <AlertDialogDescription>
        This will permanently delete the entire project folder at
        <code class="rounded bg-slate-100 px-1">{path}</code>, including any
        evidence files stored inside. Files outside this folder are not
        affected. This cannot be undone.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction onclick={onconfirm} class="bg-red-600 hover:bg-red-700">
        Delete
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
```

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/ConfirmDeleteProject.svelte src/lib/components/ui/alert-dialog/ package.json pnpm-lock.yaml
git commit -m "$(cat <<'EOF'
feat(ui): ConfirmDeleteProject dialog

Standalone confirmation dialog with the spec-defined wording.
Caller passes name + path + onconfirm; dialog handles open state.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 23: `ProjectsTable.svelte` + chooser page

**Files:**
- Create: `src/lib/components/ProjectsTable.svelte`
- Modify: `src/routes/projects/+page.svelte`
- Add shadcn-svelte table component

- [ ] **Step 1: Add shadcn-svelte table component**

Run: `pnpm dlx shadcn-svelte@latest add table`

- [ ] **Step 2: Create `src/lib/components/ProjectsTable.svelte`**

```svelte
<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { deleteProject as ipcDelete } from '$lib/ipc/projects';
  import { openAndInstall } from '$lib/stores/currentProject';
  import { formatDateSync, type TimezoneMode } from '$lib/helpers/formatDate';
  import ConfirmDeleteProject from './ConfirmDeleteProject.svelte';
  import type { RecentProjectListEntry } from '$lib/generated/RecentProjectListEntry';

  type Props = {
    projects: RecentProjectListEntry[];
    tzMode: TimezoneMode;
    onchange?: () => void;
  };
  let { projects, tzMode, onchange }: Props = $props();

  let filter = $state('');
  let confirmOpen = $state(false);
  let toDelete = $state<RecentProjectListEntry | null>(null);

  let filtered = $derived(
    filter.trim() === ''
      ? projects
      : projects.filter((p) => {
          const q = filter.toLowerCase();
          return p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q);
        })
  );

  async function handleOpen(p: RecentProjectListEntry) {
    const result = await openAndInstall(p.path);
    if (result.kind === 'Loaded') {
      goto('/projects/current');
    } else if (result.kind === 'SchemaTooNew') {
      alert(`'${p.name}' requires app v${result.app_version}+; you have v${result.project_version}.`);
    } else if (result.kind === 'Failed') {
      alert(`Failed to open '${p.name}': ${result.reason}`);
      onchange?.();
    }
  }

  function requestDelete(p: RecentProjectListEntry) {
    toDelete = p;
    confirmOpen = true;
  }

  async function confirmDelete() {
    if (!toDelete) return;
    try {
      await ipcDelete(toDelete.path);
      onchange?.();
    } catch (e) {
      alert(`Delete failed: ${JSON.stringify(e)}`);
    } finally {
      toDelete = null;
      confirmOpen = false;
    }
  }
</script>

<div class="space-y-4">
  <Input bind:value={filter} placeholder="Filter projects..." class="max-w-sm" />

  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Name</TableHead>
        <TableHead>Last opened</TableHead>
        <TableHead>Last modified</TableHead>
        <TableHead class="text-right">Actions</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each filtered as p}
        <TableRow>
          <TableCell class="font-medium">{p.name}</TableCell>
          <TableCell>{formatDateSync(p.last_opened_at, tzMode)}</TableCell>
          <TableCell>
            {p.last_modified ? formatDateSync(p.last_modified, tzMode) : '—'}
          </TableCell>
          <TableCell class="space-x-2 text-right">
            <Button size="sm" onclick={() => handleOpen(p)}>Open</Button>
            <Button
              size="sm"
              variant="outline"
              class="text-red-600 hover:bg-red-50"
              onclick={() => requestDelete(p)}
            >
              Delete
            </Button>
          </TableCell>
        </TableRow>
      {/each}
    </TableBody>
  </Table>
</div>

{#if toDelete}
  <ConfirmDeleteProject
    bind:open={confirmOpen}
    name={toDelete.name}
    path={toDelete.path}
    onconfirm={confirmDelete}
  />
{/if}
```

- [ ] **Step 3: Replace `src/routes/projects/+page.svelte`**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { listAllProjects } from '$lib/ipc/projects';
  import { getDefaultTimezone } from '$lib/ipc/app';
  import ProjectsTable from '$lib/components/ProjectsTable.svelte';
  import NewProjectSheet from '$lib/components/NewProjectSheet.svelte';
  import { formatDateSync, type TimezoneMode } from '$lib/helpers/formatDate';
  import type { RecentProjectListEntry } from '$lib/generated/RecentProjectListEntry';

  let projects = $state<RecentProjectListEntry[]>([]);
  let tzMode = $state<TimezoneMode>('UTC');
  let loaded = $state(false);
  let newSheetOpen = $state(false);

  async function refresh() {
    [projects, tzMode] = await Promise.all([listAllProjects(), getDefaultTimezone()]);
    loaded = true;
  }

  onMount(refresh);
</script>

<div class="mx-auto max-w-5xl space-y-6 p-8">
  <header class="flex items-center justify-between">
    <h1 class="text-2xl font-bold">Projects</h1>
    <Button onclick={() => (newSheetOpen = true)}>New project</Button>
  </header>

  {#if !loaded}
    <p class="text-sm text-slate-500">Loading...</p>
  {:else if projects.length === 0}
    <div class="flex flex-col items-center gap-3 py-12 text-center">
      <img src="/img/no-projects-placeholder.svg" alt="No projects" class="h-40 w-40" />
      <p class="text-base font-medium text-slate-700">
        You have no projects. Looks like somebody better get off their ass and
        GET TO WORK!
      </p>
    </div>
  {:else}
    <ProjectsTable {projects} {tzMode} onchange={refresh} />
  {/if}
</div>

<NewProjectSheet bind:open={newSheetOpen} oncreate={refresh} />
```

- [ ] **Step 4: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/ProjectsTable.svelte src/routes/projects/+page.svelte src/lib/components/ui/table/ package.json pnpm-lock.yaml
git commit -m "$(cat <<'EOF'
feat(ui): projects chooser table + page

ProjectsTable renders the list with Name / Last opened / Last
modified columns + per-row Open and Delete actions. Filter input
above the table does live client-side keyword matching on name +
path. Delete action opens the confirmation dialog.

/projects/ page wires the table to listAllProjects + Open project
flow (closes current if any, opens this one, routes to dashboard).
Empty state matches Home (funny-image + lazy-guy copy).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 24: Project dashboard at `/projects/current`

**Files:**
- Create: `src/routes/projects/current/+page.svelte`

- [ ] **Step 1: Create `src/routes/projects/current/+page.svelte`**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { currentProject } from '$lib/stores/currentProject';
  import { formatDateSync, type TimezoneMode } from '$lib/helpers/formatDate';
  import { getDefaultTimezone } from '$lib/ipc/app';

  let tzMode = $state<TimezoneMode>('UTC');

  onMount(async () => {
    tzMode = await getDefaultTimezone();
    if (!$currentProject) {
      goto('/');
    }
  });
</script>

{#if $currentProject}
  <div class="mx-auto max-w-4xl space-y-6 p-8">
    <header>
      <h1 class="text-3xl font-bold">{$currentProject.project.name}</h1>
      {#if $currentProject.project.description}
        <p class="mt-1 text-sm text-slate-600">{$currentProject.project.description}</p>
      {/if}
    </header>

    <Card>
      <CardHeader>
        <CardTitle>Project info</CardTitle>
      </CardHeader>
      <CardContent class="space-y-2 text-sm">
        <div>
          <span class="font-medium">Folder:</span>
          <code class="ml-2 rounded bg-slate-100 px-1">{$currentProject.folder_path}</code>
        </div>
        <div>
          <span class="font-medium">Created:</span>
          {formatDateSync($currentProject.project.created_at, tzMode)}
        </div>
        <div>
          <span class="font-medium">Schema version:</span>
          {$currentProject.project.app_schema_version}
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Jobs</CardTitle>
      </CardHeader>
      <CardContent>
        <p class="mb-3 text-sm text-slate-500">No jobs yet — coming in M3.</p>
        <Button disabled>New job</Button>
      </CardContent>
    </Card>
  </div>
{/if}
```

- [ ] **Step 2: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add src/routes/projects/current/+page.svelte
git commit -m "$(cat <<'EOF'
feat(ui): project dashboard skeleton

/projects/current renders the project header, info card (folder
path, created date, schema version), and an empty Jobs section
with a disabled "New job" button. M3 fleshes out jobs; this
establishes the layout.

Redirects to Home if currentProject is null (e.g., user typed the
URL directly without opening a project first).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 25: Sticky-fail + schema-too-new takeover screens

**Files:**
- Create: `src/lib/components/StickyFailScreen.svelte`
- Create: `src/lib/components/SchemaTooNewScreen.svelte`

- [ ] **Step 1: Create `src/lib/components/StickyFailScreen.svelte`**

```svelte
<script lang="ts">
  import { Button } from '$lib/components/ui/button';

  type Props = {
    path: string;
    name: string;
    reason: string;
    onContinue: () => void;
  };
  let { path, name, reason, onContinue }: Props = $props();
</script>

<div class="flex h-full items-center justify-center p-8">
  <div class="max-w-md space-y-4 rounded-lg border bg-white p-8 text-center shadow-sm">
    <img
      src="/img/sticky-fail-placeholder.svg"
      alt="Project failed to load"
      class="mx-auto h-32 w-32"
    />
    <h1 class="text-xl font-bold text-slate-900">
      Something broke, and we can't find data associated with this project
    </h1>
    <ul class="space-y-1 text-left text-sm text-slate-600">
      <li><span class="font-medium">Project:</span> {name}</li>
      <li><span class="font-medium">Path:</span> <code class="rounded bg-slate-100 px-1">{path}</code></li>
      <li><span class="font-medium">Reason:</span> {reason}</li>
    </ul>
    <Button onclick={onContinue}>OK, take me Home</Button>
  </div>
</div>
```

- [ ] **Step 2: Create `src/lib/components/SchemaTooNewScreen.svelte`**

```svelte
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { RELEASES_URL } from '$lib/constants';

  type Props = {
    path: string;
    name: string;
    project_version: number;
    app_version: number;
    onContinue: () => void;
  };
  let { path, name, project_version, app_version, onContinue }: Props = $props();
</script>

<div class="flex h-full items-center justify-center p-8">
  <div class="max-w-md space-y-4 rounded-lg border bg-white p-8 text-center shadow-sm">
    <h1 class="text-xl font-bold text-slate-900">
      This project was created by a newer version of Better Hayabusa
    </h1>
    <p class="text-sm text-slate-600">
      Project '<strong>{name}</strong>' was created with schema version
      <strong>{project_version}</strong>; this app supports up to version
      <strong>{app_version}</strong>. Upgrade the app to open this project.
    </p>
    <p class="text-xs text-slate-500">
      Path: <code class="rounded bg-slate-100 px-1">{path}</code>
    </p>
    <div class="flex justify-center gap-2">
      <Button href={RELEASES_URL} target="_blank" rel="noopener noreferrer" variant="outline">
        Open releases
      </Button>
      <Button onclick={onContinue}>Take me Home</Button>
    </div>
  </div>
</div>
```

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/StickyFailScreen.svelte src/lib/components/SchemaTooNewScreen.svelte
git commit -m "$(cat <<'EOF'
feat(ui): sticky-fail + schema-too-new takeover screens

Two dedicated screens for the failure modes of sticky-session
restore. StickyFail shows the funny image, the missing path/name,
and the reason. SchemaTooNew shows the version mismatch and links
to GitHub releases.

Both expose an onContinue callback so the parent (Task 26) routes
back to Home after dismissal.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 26: Sticky session orchestration on boot

**Files:**
- Modify: `src/routes/+layout.svelte`
- Create: `src/lib/components/SessionLoader.svelte` (encapsulates the boot logic)

- [ ] **Step 1: Create `src/lib/components/SessionLoader.svelte`**

```svelte
<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { checkLastOpenProject } from '$lib/ipc/projects';
  import { loadCurrentProject } from '$lib/stores/currentProject';
  import StickyFailScreen from './StickyFailScreen.svelte';
  import SchemaTooNewScreen from './SchemaTooNewScreen.svelte';
  import type { LaunchResult } from '$lib/generated/LaunchResult';

  type Props = { children: Snippet };
  let { children }: Props = $props();

  type State =
    | { kind: 'loading' }
    | { kind: 'ready' }
    | { kind: 'sticky_fail'; path: string; name: string; reason: string }
    | { kind: 'schema_too_new'; path: string; name: string; project_version: number; app_version: number };

  let state = $state<State>({ kind: 'loading' });

  onMount(async () => {
    const result: LaunchResult = await checkLastOpenProject();
    switch (result.kind) {
      case 'Loaded':
        await loadCurrentProject(); // hydrate store from backend
        state = { kind: 'ready' };
        goto('/projects/current');
        break;
      case 'Failed':
        state = {
          kind: 'sticky_fail',
          path: result.path,
          name: result.name,
          reason: result.reason,
        };
        break;
      case 'SchemaTooNew':
        state = {
          kind: 'schema_too_new',
          path: result.path,
          name: result.name,
          project_version: result.project_version,
          app_version: result.app_version,
        };
        break;
      case 'NoneSet':
      case 'Disabled':
        state = { kind: 'ready' };
        break;
    }
  });

  function dismissToHome() {
    state = { kind: 'ready' };
    goto('/');
  }
</script>

{#if state.kind === 'loading'}
  <div class="flex h-full items-center justify-center text-sm text-slate-500">
    Loading...
  </div>
{:else if state.kind === 'sticky_fail'}
  <StickyFailScreen
    path={state.path}
    name={state.name}
    reason={state.reason}
    onContinue={dismissToHome}
  />
{:else if state.kind === 'schema_too_new'}
  <SchemaTooNewScreen
    path={state.path}
    name={state.name}
    project_version={state.project_version}
    app_version={state.app_version}
    onContinue={dismissToHome}
  />
{:else}
  {@render children()}
{/if}
```

- [ ] **Step 2: Wire SessionLoader into `+layout.svelte`**

```svelte
<script lang="ts">
  import '../app.css';
  import OsGate from '$lib/components/OsGate.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Breadcrumbs from '$lib/components/Breadcrumbs.svelte';
  import SessionLoader from '$lib/components/SessionLoader.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<OsGate>
  <div class="flex h-screen">
    <Sidebar />
    <main class="flex flex-1 flex-col overflow-hidden bg-slate-50">
      <Breadcrumbs />
      <div class="flex-1 overflow-y-auto">
        <SessionLoader>
          {@render children()}
        </SessionLoader>
      </div>
    </main>
  </div>
</OsGate>
```

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/SessionLoader.svelte src/routes/+layout.svelte
git commit -m "$(cat <<'EOF'
feat(ui): sticky-session boot orchestration

SessionLoader runs check_last_open_project on mount and branches:
Loaded -> goto /projects/current, Failed -> StickyFailScreen,
SchemaTooNew -> SchemaTooNewScreen, NoneSet/Disabled -> render
children normally.

Wraps the route content in +layout.svelte so the takeover screens
sit inside the main area (sidebar + breadcrumbs still visible).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 27: Settings page partial UI (3 entries)

**Files:**
- Modify: `src/routes/settings/+page.svelte`
- Add shadcn-svelte select component

- [ ] **Step 1: Add shadcn-svelte select component**

Run: `pnpm dlx shadcn-svelte@latest add select`

- [ ] **Step 2: Replace Settings page content**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '$lib/components/ui/select';
  import {
    getLaunchBehavior, setLaunchBehavior, type LaunchBehavior,
    getDefaultTimezone, setDefaultTimezone, type TimezoneMode,
    getRecentProjectsCount, setRecentProjectsCount,
  } from '$lib/ipc/app';

  let launchBehavior = $state<LaunchBehavior>('last_project');
  let tzMode = $state<TimezoneMode>('UTC');
  let count = $state<number>(5);
  let loaded = $state(false);

  onMount(async () => {
    [launchBehavior, tzMode, count] = await Promise.all([
      getLaunchBehavior(),
      getDefaultTimezone(),
      getRecentProjectsCount(),
    ]);
    loaded = true;
  });

  async function onLaunchBehaviorChange(v: string) {
    launchBehavior = v as LaunchBehavior;
    await setLaunchBehavior(launchBehavior);
  }

  async function onTzChange(v: string) {
    tzMode = v as TimezoneMode;
    await setDefaultTimezone(tzMode);
  }

  async function onCountChange() {
    const n = Math.max(1, Math.min(50, count | 0));
    count = n;
    await setRecentProjectsCount(n);
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 p-8">
  <h1 class="text-2xl font-bold">Settings</h1>

  {#if loaded}
    <Card>
      <CardHeader>
        <CardTitle>General</CardTitle>
      </CardHeader>
      <CardContent class="space-y-6">
        <div class="space-y-2">
          <Label for="launch">On launch</Label>
          <Select type="single" value={launchBehavior} onValueChange={onLaunchBehaviorChange}>
            <SelectTrigger id="launch" class="w-64">
              <SelectValue placeholder="Choose..." />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="last_project">Open last project</SelectItem>
              <SelectItem value="home_page">Open Home page</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="space-y-2">
          <Label for="tz">Default timezone</Label>
          <Select type="single" value={tzMode} onValueChange={onTzChange}>
            <SelectTrigger id="tz" class="w-64">
              <SelectValue placeholder="Choose..." />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="UTC">UTC</SelectItem>
              <SelectItem value="Local">Local</SelectItem>
            </SelectContent>
          </Select>
          <p class="text-xs text-slate-500">
            All displayed timestamps use this. The DB always stores UTC.
          </p>
        </div>

        <div class="space-y-2">
          <Label for="count">Recent projects to show on Home</Label>
          <Input
            id="count"
            type="number"
            min="1"
            max="50"
            bind:value={count}
            onblur={onCountChange}
            class="w-32"
          />
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>Theme, log retention, tools</CardTitle>
      </CardHeader>
      <CardContent>
        <p class="text-sm text-slate-500">Coming in M5.</p>
      </CardContent>
    </Card>
  {/if}
</div>
```

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add src/routes/settings/+page.svelte src/lib/components/ui/select/ package.json pnpm-lock.yaml
git commit -m "$(cat <<'EOF'
feat(ui): Settings page partial UI

Three M2 settings exposed: launch behavior (last project / home),
default timezone (UTC / Local), recent projects to show (number
1-50). Reads + writes via app_state. Other Settings sections
stubbed for M5.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 28: About page — license, GitHub link, acknowledgements, link-fix via plugin-opener

**Files:**
- Modify: `src/routes/settings/about/+page.svelte`

- [ ] **Step 1: Replace About page content**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { getAppVersion } from '$lib/ipc/app';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { REPO_URL } from '$lib/constants';

  let version = $state<string>('…');

  onMount(async () => {
    try {
      const v = await getAppVersion();
      version = v.version;
    } catch {
      version = '?';
    }
  });

  const acknowledgements = [
    { name: 'Yamato Security (Hayabusa)', url: 'https://github.com/Yamato-Security/hayabusa' },
    { name: 'WithSecureLabs (Chainsaw)', url: 'https://github.com/WithSecureLabs/chainsaw' },
    { name: 'Yamato Security (Takajo)', url: 'https://github.com/Yamato-Security/takajo' },
    { name: 'SigmaHQ (Sigma rules)', url: 'https://github.com/SigmaHQ/sigma' },
    { name: 'Tauri', url: 'https://tauri.app' },
    { name: 'Svelte', url: 'https://svelte.dev' },
    { name: 'shadcn-svelte', url: 'https://www.shadcn-svelte.com' }
  ];

  function open(url: string) {
    void openUrl(url);
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 p-8">
  <header>
    <h1 class="text-3xl font-bold">Better Hayabusa</h1>
    <p class="mt-1 text-lg italic text-slate-600">Making your life suck a little less…</p>
  </header>

  <Card>
    <CardHeader>
      <CardTitle>About</CardTitle>
    </CardHeader>
    <CardContent class="space-y-3 text-sm">
      <div>
        <span class="font-medium">Version:</span> <code>{version}</code>
      </div>
      <div>
        <span class="font-medium">License:</span> AGPL-3.0-or-later
        <button
          type="button"
          onclick={() => open('https://www.gnu.org/licenses/agpl-3.0.html')}
          class="ml-2 text-blue-600 underline"
        >
          read
        </button>
      </div>
      <div>
        <span class="font-medium">Source:</span>
        <button
          type="button"
          onclick={() => open(REPO_URL)}
          class="text-blue-600 underline"
        >
          GitHub
        </button>
      </div>
      <div class="pt-2 text-xs text-slate-500">© 2026 Merciless Software</div>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Acknowledgements</CardTitle>
    </CardHeader>
    <CardContent>
      <ul class="space-y-1 text-sm">
        {#each acknowledgements as ack}
          <li>
            <button
              type="button"
              onclick={() => open(ack.url)}
              class="text-blue-600 underline"
            >
              {ack.name}
            </button>
          </li>
        {/each}
      </ul>
    </CardContent>
  </Card>
</div>
```

- [ ] **Step 2: Verify build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add src/routes/settings/about/+page.svelte
git commit -m "$(cat <<'EOF'
feat(ui): About page updates

- License row updated to AGPL-3.0-or-later with link to GNU AGPL page
- Acknowledgements credit organizations (Yamato Security, WithSecureLabs,
  SigmaHQ) not tool names; adds Takajo
- All external links now use plugin-opener (was bare <a href> which
  does nothing in a Tauri webview)
- Tagline updated to "Making your life suck a little less..."

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 29: Document version-bump rule in CLAUDE.md + integrate into `/ship`

**Files:**
- Modify: `CLAUDE.md`
- Modify: `.claude/commands/ship.md`

- [ ] **Step 1: Append version-bump section to `CLAUDE.md`**

Add a new section after `## Running and building` (before `## Gotchas`):

```markdown
## Versioning

We use SemVer. `src-tauri/tauri.conf.json` is the canonical version source; `package.json` and `src-tauri/Cargo.toml` mirror it.

- **Patch bump** (`0.1.x` → `0.1.x+1`) happens automatically in `/ship` on every PR.
- **Minor bump** (`0.x.0` → `0.x+1.0`) is a manual one-line edit at milestone close. M2 close → `0.2.0`, M3 close → `0.3.0`, etc.
- **Major bump** (`x.0.0` → `x+1.0.0`) is manual. Pre-1.0 we're in experimentation mode; `1.0.0` is cut at M8 close (portable distribution + release). Post-1.0, major bumps follow SemVer — breaking changes or significant feature waves.

About page reads the version via `get_app_version` (Tauri command sourced from `tauri.conf.json`).
```

- [ ] **Step 2: Add version-bump steps to `.claude/commands/ship.md`**

In the Preflight section, add a step:

```markdown
6. **Version bump available.** Read current version from `src-tauri/tauri.conf.json`. Increment the patch number. Write back to `tauri.conf.json` AND mirror to `package.json` (`"version"` field) AND `src-tauri/Cargo.toml` (`version = "..."`). Stage these as part of the next commit — do NOT push a "version bump" commit separately.
```

(Insert before Phase A.)

In Phase A, expand Step 2 to include the version in the PR title/body:

```markdown
2. Create the PR with `gh pr create --base main --title "<title>" --body "<body>"`:
   - **Title:** derive from the most-recent merge-base-relative commit subject if there's one obvious commit, otherwise ask the user for the title. Optionally prefix with the new version (e.g. `v0.1.5: …`).
```

- [ ] **Step 3: Verify nothing breaks**

Run: `pnpm build`
Expected: success (CLAUDE.md / ship.md don't affect compile).

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md .claude/commands/ship.md
git commit -m "$(cat <<'EOF'
docs: document version-bump rule + integrate into /ship

Adds a Versioning section to CLAUDE.md. Patch is automatic in
/ship; minor is a manual one-line edit at milestone close; major
is user's call (M8 close -> 1.0.0).

/ship preflight gains step 6: read current version, bump patch,
mirror across tauri.conf.json + package.json + Cargo.toml. Version
bump rides in the PR commit (no separate "bump" commit).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 30: Final verification (cargo test + pnpm build + manual smoke)

**Files:** none modified

- [ ] **Step 1: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: all green. New ts-rs exports are written to `src/lib/generated/`.

- [ ] **Step 2: Run frontend build**

Run: `pnpm build`
Expected: success.

- [ ] **Step 3: Manual smoke — happy paths**

Run: `pnpm tauri dev` (in background, monitor window)

Verify each:
- App opens to Home (first launch — no sticky session set).
- "New project" sheet opens; folder picker works; Create lands in `/projects/current` with project info card filled.
- Sidebar shows the project section with the project name.
- Breadcrumb shows `Home › Projects › <name>` on dashboard.
- Navigate to Settings; sidebar highlights Settings; breadcrumb is `Home › Settings`.
- Navigate to Merciless Software footer → About; sidebar does NOT highlight Settings (bug fix verified).
- About page: license shows AGPL-3.0-or-later, GitHub link opens browser, acknowledgements list orgs.
- Close project (deferred — no UI button per spec). Close app and reopen → lands back in last project (sticky session works).
- Toggle Settings → On launch → Home; close + reopen app → lands on Home.
- Toggle Settings → Default timezone → Local; project info card's Created timestamp reformats without restart.
- Set Recent projects count = 3 in Settings; Home recent-projects list shows max 3.

- [ ] **Step 4: Manual smoke — error paths**

- Try to create a project in a folder that already has one → inline conflict alert with "Open it" button works.
- Open project chooser (`/projects/`); use filter; click Delete on a project; confirm dialog wording matches spec; delete actually removes the folder (verify on disk).
- Hand-edit `app.db` to set `projects.app_schema_version` higher in a project's project.db, then restart → SchemaTooNewScreen shows.
- Manually rename a project folder externally, then restart → StickyFailScreen shows the funny image + path + reason; dismiss returns to Home.

- [ ] **Step 5: Stop the dev server**

Close the Tauri window.

- [ ] **Step 6: One final cargo test + pnpm build**

Run: `cd src-tauri && cargo test && cd .. && pnpm build`
Expected: both green.

- [ ] **Step 7: No commit (verification only)**

If any failure during smoke testing, file a fix task and address before marking M2 complete.

---

## Self-review checklist (run after all tasks committed)

- [ ] Every section of [the spec](../specs/2026-05-19-m2-projects-design.md) maps to a task above. Specifically verify:
  - § 3 sticky session → Task 26
  - § 4 routing + pages → Tasks 20, 21, 23, 24, 27, 28
  - § 5 sidebar → Task 18
  - § 6 breadcrumbs → Task 17
  - § 7 sticky-fail screen → Tasks 25, 26
  - § 8 schema mismatch → Tasks 25, 26
  - § 9 image placeholders → Task 19
  - § 10 data model + activity log → Tasks 7, 8
  - § 11 commands + CommandError → Tasks 4, 5, 15
  - § 12 frontend boundary → Task 16
  - § 13 cross-cutting chores → Tasks 1 (opener), 2 (license), 3 (URL constant), 7 (#3), 29 (version-bump)
  - § 14 migrations runner — already general; verified in Task 7
- [ ] No placeholders, no "TBD", no "implement appropriate X" in any step.
- [ ] All `CommandError` usages reference the variants defined in Task 4.
- [ ] All ts-rs exports listed in Task 6 are imported wherever they're used (Tasks 16, 17, 21, 23, 24, 25).
- [ ] All git commit messages end with the Co-Authored-By trailer.
