# M2 — Projects + Recent-Projects Flow — Design

**Date:** 2026-05-19
**Status:** Draft for review
**Milestone:** M2 — Projects
**Supersedes parts of:** [2026-05-19-better-hayabusa-chainsaw-design.md](2026-05-19-better-hayabusa-chainsaw-design.md) (the main design spec) where M2-specific decisions refine or extend it.

## 1. Purpose

M2 delivers the project flow — creating, opening, switching, and deleting projects — plus the recent-projects list, the project.db bootstrap, and a cluster of cross-cutting polish items the user wanted in the same pass: license change, version-bump rule, breadcrumbs, partial Settings UI, About page fixes, and typed IPC errors.

After M2, the user can open the app, land in their last project, see the project dashboard (empty of jobs — that's M3), and switch between projects freely.

## 2. Scope

### In M2

- Project lifecycle: create / open / switch / delete
- Recent-projects rendering on Home + chooser table at `/projects/`
- `project.db` bootstrap (new migrations folder, `projects` table)
- Sticky session: auto-reopen last project on launch (with failure handling)
- Schema-version mismatch refusal (closes #23)
- Settings page partial UI: launch behavior, default timezone, recent-projects count
- Activity log (`.bhc/activity.log`) — append-only, used for "last modified" timestamps
- Breadcrumbs on every page
- About page fixes: external-browser links, GitHub link, acknowledgements update
- Home page rewrite: new copy, tagline, recent projects, empty state with funny image
- Sidebar bug fix: About no longer highlights Settings
- License: MIT → AGPL-3.0-or-later (everywhere)
- Version-bump rule in `/ship` + CLAUDE.md
- Typed `CommandError` enum via ts-rs (closes #6)
- Migration failure-path test (closes #3)
- Centralized repo URL constant (closes #5)
- Project dashboard skeleton at `/projects/current` (info + empty Jobs section)

### Out of M2

- Job CRUD, target lists, tool reference pages (M3+)
- Run execution + live output (M4)
- Full Settings UX (M5 — M2 ships 3 entries)
- Tool acquisition (M7)
- Portable distribution (M8)
- Multiple projects loaded simultaneously
- Active-run lifecycle on app-close / project-switch (deferred — see Issue #25)
- Project rename feature (filed as backlog Issue)
- Recent projects: pinning, tags, custom sort (filed as backlog Issue)
- IANA timezone picker (v1.1+ — M2 ships UTC / Local toggle only)
- Funny image assets (placeholder slots in M2; commissioning tracked by separate Issues)

## 3. State model

### Single project loaded at a time

Backend Rust holds `Option<CurrentProject>` in `AppState`. `CurrentProject` owns the path, name, and a `Mutex<rusqlite::Connection>` for `project.db`. Switching projects is transactional: drop the previous connection, open the new one.

The project is the visual context (sidebar reflects it; dashboard renders for it). Whether it also functions as a runtime gate when M4 introduces real runs — and what exactly happens to active runs on app-close or project-switch — is deferred to M4's brainstorming pass (see Issue #25). M2 ships no runs, so this is non-blocking.

### Sticky session (auto-reopen on launch)

On app launch:

1. Read setting `launch_behavior` from `app_state`. Default: `"last_project"`.
2. If `"last_project"`:
   - Read `last_open_project_path` from `app_state`. If unset → route to Home.
   - If set, try to open: folder exists, `.bhc/project.db` exists, schema version compatible.
   - If all checks pass → set `current_project`, route to `/projects/current`.
   - If any check fails → sticky-fail takeover screen (§ 7).
3. If `"home_page"` → route to Home.

The auto-reopen path is purely additive UX over the manual open flow — it calls `open_project` under the hood and behaves identically once loaded.

## 4. Routing and pages

### Routes

| Route | Purpose |
|---|---|
| `/` | Home |
| `/projects/` | Projects chooser table |
| `/projects/current` | Active project dashboard (404-equivalent if no project loaded) |
| `/settings/` | Settings (partial in M2) |
| `/settings/about` | About |
| `/tools/hayabusa`, `/tools/chainsaw` | Tool reference stubs (unchanged in M2) |

### Home (`/`)

- **Copy:** First sentence rewritten to "A graphical UI for Hayabusa and related tools (Chainsaw and Takajo)." Tagline "*Making your life suck a little less...*" placed visibly on the page (header subtitle or footer of the intro card).
- **Buttons:** "New project" (always enabled, opens sheet); "Open project" (disabled when no projects exist, navigates to `/projects/`).
- **Recent projects:** Card listing up to N most recent (N from `recent_projects_count` setting, default 5), ordered by `last_opened_at` DESC. Each row: name + last-opened timestamp. Click anywhere on the row opens that project (closes the current one if a different project is loaded).
- **Empty state:** when `recent_projects` is empty, replace the recent-projects card with the funny-image-#1 placeholder (§ 8).

### New Project sheet

Triggered from the Home "New project" button. shadcn-svelte `<Sheet>`, not a route.

- **Fields:**
  - Project name (text, required, unique-within-recents check is advisory not enforced)
  - Default folder path (folder picker via `@tauri-apps/plugin-dialog`, required)
  - Description (textarea, optional)
- **Help text** below the path field: *"Source evidence and output paths can be set per-job to anywhere — this is just where the project metadata and logs live, and the default starting point for new jobs."*
- **Conflict handling** (§ 4b of parent spec): if the chosen folder already contains `.bhc/project.db`, inline alert in the sheet: *"This folder is already a project — open it?"* with [Open] [Cancel]. No overwrite path.
- **Actions:** [Cancel] closes the sheet; [Create] runs `create_project`, closes sheet, routes to `/projects/current`.

### Projects chooser (`/projects/`)

Reached from sidebar "Projects" item OR Home "Open project" button. Both navigate here.

- **Header:** filter input (keyword filter across project name and folder path, client-side).
- **Table columns:**
  - Name
  - Last opened (formatted per timezone setting)
  - Last modified (mtime of `.bhc/activity.log`, formatted per timezone setting)
  - Open action (button) — switches to that project
  - Delete action (button) — opens confirmation dialog
- **Open behavior:** closes current project if any, opens this one, routes to `/projects/current`.
- **Empty state:** funny image #1 + lazy-guy copy (§ 8) when no entries exist in `recent_projects`.

### Delete confirmation dialog

Triggered by the Delete action in the chooser.

Wording: *"Delete project '<name>'? This will permanently delete the entire project folder at `<path>`, including any evidence files stored inside. Files outside this folder are not affected. This cannot be undone."*

Buttons: [Cancel] [Delete].

On confirm:

1. If this is the currently-open project, close it first (drop the DB connection, clear `current_project`).
2. Recursively delete the folder at `<path>` via `std::fs::remove_dir_all`.
3. Remove the row from `recent_projects`.
4. Refresh the chooser.

**Safety:** Rust will refuse to delete if `<path>` is not the path recorded in `recent_projects` for this row (no path mismatching). Failures (in-use files on Windows, permission denied) surface as a typed `CommandError::Io`.

### Project dashboard (`/projects/current`)

In M2, no jobs exist yet. The dashboard shows:

- **Header:** project name + breadcrumb (§ 6).
- **Project info card:** name, description, folder path (with "Open in Explorer" button — uses `plugin-opener`), created date, `app_schema_version`.
- **Jobs section:** "No jobs yet — coming in M3" empty-state message + disabled "New job" button.

M3 fills in the jobs table; M4 adds run history. M2 lays out the structure so those drop in cleanly.

If the user navigates to `/projects/current` while no project is loaded, redirect to Home.

### Settings page (`/settings/`)

Partial UI in M2 — full Settings UX is M5. M2 exposes three entries:

| Setting key (in `app_state`) | UI | Default |
|---|---|---|
| `launch_behavior` | Radio: "Open last project" / "Open Home page" | `last_project` |
| `default_timezone` | Radio: "UTC" / "Local" | `UTC` |
| `recent_projects_count` | Number input (min 1, max 50) | `5` |

Theme, log retention, and Tools subsections remain stubbed (M5 work). The page renders sections for them with "Coming in M5" notes.

### About page (`/settings/about`)

Existing page receives four updates:

1. **Link fix:** external links open in the default browser via `@tauri-apps/plugin-opener` (current bare `<a href>` does nothing in a Tauri webview).
2. **Add:** GitHub repo link (uses the new repo-URL constant — § 9).
3. **License update:** displayed license is AGPL-3.0-or-later with a link to the local `LICENSE` file and an external link to the AGPL text.
4. **Acknowledgements update:** credit organizations, not tool names:
   - Yamato Security (Hayabusa)
   - WithSecureLabs (Chainsaw)
   - Yamato Security (Takajo)
   - SigmaHQ (Sigma rules)
   - Tauri
   - Svelte
   - shadcn-svelte

## 5. Sidebar behavior

### Layout when no project open

```
Home
Projects
─────
Tools
  Hayabusa
  Chainsaw
─────
Settings
─────
Merciless Software v<x.y.z>
```

### Layout when project open

```
Home
Projects
─────
▸ <ProjectName>          ← new section
─────
Tools
  Hayabusa
  Chainsaw
─────
Settings
─────
Merciless Software v<x.y.z>
```

The project section appears between Projects and Tools. In M2 the project name is a single link to `/projects/current`. In M3+, Jobs and Queue links appear under it.

### Bug fix: Settings highlighted on About

The active-page matcher currently treats `/settings/*` as activating the Settings sidebar item, which incorrectly highlights Settings when on `/settings/about`. Fix: tighten the matcher to activate Settings only on exact `/settings` (not deeper). About is reached via the Merciless Software footer click, which doesn't go through Settings.

## 6. Breadcrumbs

shadcn-svelte `<Breadcrumb>` strip at the top of every page.

| Route | Breadcrumb |
|---|---|
| `/` | `Home` |
| `/projects/` | `Home › Projects` |
| `/projects/current` | `Home › Projects › <ProjectName>` |
| `/settings/` | `Home › Settings` |
| `/settings/about` | `Home › Settings › About` |
| `/tools/hayabusa` | `Home › Tools › Hayabusa` |

Last segment is non-clickable (current page); prior segments link to their routes.

M3+ paths extend naturally: `/projects/current/jobs/[id]` → `Home › Projects › <ProjectName> › Jobs › <JobName>`.

## 7. Sticky-session failure screen

When the launch-time auto-open fails (folder missing, project.db gone, schema-too-new, I/O error):

- **Dedicated takeover at app launch** — replaces the default Home landing for this session.
- **Layout:**
  - Funny image #2 (sad / broken vibe, distinct from image #1).
  - Headline: *"Something broke, and we can't find data associated with this project"*
  - Body: bulleted list of what was checked and what failed (project name, expected path, specific reason — e.g., "Folder does not exist," "project.db missing," "Schema version 4 is newer than this app (3)").
  - Single button: [OK, take me Home].
- **On click:** removes the dead entry from `recent_projects`, clears `last_open_project_path`, routes to Home.

If the failure was schema-too-new (specifically), show the schema-mismatch panel (§ 8) instead — it has more specific messaging.

## 8. Schema-version mismatch (closes #23)

When `projects.app_schema_version > current_app_schema_version` on open:

- Refuse to open. Do not run any migration logic — the app has no concept of forward-compat for newer schemas.
- Show a dedicated panel:
  - Headline: *"This project was created by a newer version of Better Hayabusa."*
  - Body: *"Project schema version is X; this app supports up to Y. Upgrade the app to open this project."*
  - Links: GitHub releases (via repo-URL constant); local `LICENSE` reference.
  - Button: [Take me Home].
- If reached via sticky session, the panel replaces the sticky-fail screen for this specific failure.

The reverse case (`app_schema_version < current_app_schema_version`) triggers normal forward migrations — already covered by the parent spec.

## 9. Funny image slots (placeholders in M2)

Two distinct images shipped as placeholder SVGs in M2 with the actual assets commissioned separately:

| ID | Where used | Copy |
|---|---|---|
| Image #1 | Home empty state + chooser empty state | "You have no projects. Looks like somebody better get off their ass and GET TO WORK!" |
| Image #2 | Sticky-session failure screen | "Something broke, and we can't find data associated with this project" |

Placeholders ship at `static/img/no-projects-placeholder.svg` and `static/img/sticky-fail-placeholder.svg`. Separate backlog Issues track commissioning the real assets.

## 10. Data model

### `app.db` changes

No new tables — all required tables (`app_state`, `recent_projects`, `global_tools`) already exist from M1's `001_init.sql`.

New `app_state` keys consumed in M2:

| Key | Type | Default | Purpose |
|---|---|---|---|
| `launch_behavior` | `"last_project"` \| `"home_page"` | `last_project` | Sticky session toggle |
| `default_timezone` | `"UTC"` \| `"Local"` | `UTC` | Display filter for all timestamps |
| `recent_projects_count` | integer string | `5` | Home page list size |
| `last_open_project_path` | path string | unset | Sticky session target |

### `project.db` (new in M2)

New migration folder: `src-tauri/migrations/project/001_init.sql`.

```sql
CREATE TABLE projects (
  id                  INTEGER PRIMARY KEY,
  name                TEXT NOT NULL,
  description         TEXT,
  created_at          TEXT NOT NULL,           -- UTC ISO 8601
  app_schema_version  INTEGER NOT NULL
);

-- Constrained to one row at the application layer.
-- Jobs, job_hosts, job_runs, job_run_hosts come in M3/M4.
```

`app_schema_version` is checked on open per § 8; updated when forward migrations run.

### Activity log (`.bhc/activity.log`)

Append-only plain text file in the project's `.bhc/` folder. The chooser table reads its `mtime` for the "Last modified" column.

**Format:**

```
<UTC ISO 8601> | <event_type> | <key=value details>
```

**Examples:**

```
2026-05-19T14:22:01Z | project_opened | name="APT-29 sweep"
2026-05-19T14:22:05Z | settings_changed | key=default_timezone value=Local
```

**Events captured in M2:** `project_opened`, `settings_changed` (only if any project-scoped settings exist — none yet, so this is effectively a hook for M3+).

**M3/M4 extend** with: `job_created`, `job_updated`, `job_deleted`, `target_added`, `target_removed`, `run_started`, `run_completed`, `run_cancelled`, etc.

**Write guarantees:** the log append is part of the same `rusqlite::Transaction` as the DB write it accompanies (where applicable). Both succeed or both roll back. For events without an associated DB write (e.g., `project_opened` is read-only), the log append is best-effort with errors logged via `tracing` (does not fail the operation).

**Rotation:** none in v1. Log grows unboundedly. Backlog Issue tracks a future rotation policy.

## 11. Backend (Tauri commands)

### Typed `CommandError` (closes #6)

Replaces `.map_err(|e| e.to_string())` across all commands.

New module `src-tauri/src/commands/error.rs`:

```rust
#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum CommandError {
    NotFound        { path: String },
    AlreadyExists   { path: String },
    NotAProject     { path: String },
    SchemaTooNew    { project_version: u32, app_version: u32 },
    Io              { message: String },
    Db              { message: String },
    Internal        { message: String },
}
```

`Result<T, CommandError>` is the return type for every command. Existing `get_app_version` / `get_app_state` / `set_app_state` are refactored to use it.

Frontend consumes via the ts-rs-generated discriminated union and branches on `kind`.

### New commands in M2

| Command | Purpose |
|---|---|
| `create_project(folder_path, name, description?)` | Create `.bhc/project.db`, write initial activity-log entry, insert into `recent_projects`, set `current_project`, set `last_open_project_path` |
| `open_project(folder_path)` | Open existing `project.db`, validate schema, update `recent_projects.last_opened_at`, set `current_project`, set `last_open_project_path`, log `project_opened` |
| `close_project()` | Drop the project DB connection, clear `current_project`, clear `last_open_project_path` |
| `get_current_project()` | Returns `Option<ProjectInfo>` for the loaded project |
| `list_recent_projects(limit?)` | Returns recents ordered by `last_opened_at` DESC; limit defaults to `recent_projects_count` setting |
| `list_all_projects()` | Returns every row in `recent_projects` + computed `last_modified` from each project's `.bhc/activity.log` mtime (for the chooser) |
| `remove_recent_project(folder_path)` | Removes from `recent_projects` only — no disk changes |
| `delete_project(folder_path)` | Recursively deletes the project folder; removes from `recent_projects`; if it's the current project, closes it first |
| `check_last_open_project()` | Used at launch — returns `LaunchResult` (`Loaded(ProjectInfo)`, `Failed { reason }`, `NoneSet`, or `Disabled`) |

### Updated commands

| Command | Change |
|---|---|
| `get_app_version`, `get_app_state`, `set_app_state` | Return `Result<T, CommandError>` instead of `Result<T, String>` |

## 12. Frontend boundary

- IPC wrappers in `src/lib/ipc/projects.ts` for all new commands. Each function is `Promise<T>` and throws typed `CommandError` on failure.
- `src/lib/ipc/app.ts` extended for the three new `app_state` keys.
- New ts-rs exports land in `src/lib/generated/`: `Project`, `ProjectInfo`, `RecentProject`, `RecentProjectListEntry` (includes computed `last_modified`), `LaunchResult`, `CommandError`.
- New Svelte store `src/lib/stores/currentProject.ts` mirrors the backend's `current_project` for reactivity. Updated when `create_project` / `open_project` / `close_project` / `delete_project` is invoked. The store is the only frontend source for "which project is open."

## 13. Cross-cutting M2 chores

### License: MIT → AGPL-3.0-or-later (closes #18)

Updated surfaces:

- `LICENSE` — replace MIT text with AGPL-3.0-or-later text
- `README.md` — license section
- `CLAUDE.md` — `## License` section
- `package.json` — `"license": "AGPL-3.0-or-later"`
- `src-tauri/Cargo.toml` — `license = "AGPL-3.0-or-later"`
- `src-tauri/tauri.conf.json` — license metadata if present
- About page UI (§ 4)
- GitHub repo metadata — auto-derives from `LICENSE` after merge (verify the badge updates)

Dependency compatibility note: MIT / Apache-2.0 / BSD remain fine; GPL-2.0-only is incompatible. Future deps need a license check.

### Version-bump rule

Documented in `CLAUDE.md` and enforced by `/ship`:

- **Patch bump** on every `/ship` — automatic, in `tauri.conf.json` (canonical). `Cargo.toml` and `package.json` mirror at build time via existing tooling; if they don't mirror, also bump them.
- **Minor bump** manually at milestone close — a one-line edit before invoking `/ship`. Pre-1.0 milestones map to minor bumps (M2 close → 0.2.0, M3 close → 0.3.0).
- **Major bump** manually — user's call. Pre-1.0 we're in 0.x.y experimentation; **M8 milestone close → 1.0.0**; post-1.0 follows SemVer (breaking changes or major feature waves).

`/ship` integration: before creating the PR, read current version from `tauri.conf.json`, increment patch, write back, include the bump in the PR's commit.

### Migration failure-path test (closes #3)

`src-tauri/tests/db_migrations.rs` extended with a test that runs a deliberately malformed `.sql` migration and asserts `MigrationError::Migration(name, e)` preserves the failing migration's name. The new `project/001_init.sql` gives the test a real second migration to interact with.

### Centralized repo URL constant (closes #5)

Extract `https://github.com/SpongeBobCodePants/better-hayabusa` to `src/lib/constants.ts`. Consumers updated:

- `src/lib/components/UnsupportedOs.svelte`
- `src/routes/settings/about/+page.svelte`
- New consumer: sticky-fail screen + schema-mismatch panel (release link)

`README.md` and `CLAUDE.md` references stay as literal Markdown (no TS import).

## 14. Migrations runner

The existing `db::migrations` runner from M1 currently handles `app.db` only. M2 generalizes it to:

- Take a migration directory (e.g., `migrations/app/` or `migrations/project/`) as input.
- Take a callable for reading and writing the schema version (different storage per DB — `app_state.schema_version` for `app.db`, `projects.app_schema_version` for `project.db`).
- Run all `.sql` files numerically ordered, in a transaction per file, rolling back on failure.

If the existing runner already takes parameters that make this clean, this is a one-call addition. If not, refactor as part of M2.

## 15. Testing approach

### Rust

- `tests/db_migrations.rs` — extended with project.db migrations + malformed-migration test (closes #3).
- `tests/project_lifecycle.rs` (NEW) — create/open/close round-trip; conflict detection; schema-mismatch refusal; sticky-fail path; delete-project recursive.
- `tests/activity_log.rs` (NEW) — transactional write with action; mtime correctness.
- `tests/command_error.rs` (NEW) — serialization round-trip per variant (ensures the ts-rs export stays consistent).

### Frontend

Vitest is not yet adopted (per parent spec §2 "Not yet adopted"); M2 does not introduce it. Frontend correctness verified manually before completion.

### "Done" criteria

Per CLAUDE.md's verification-before-completion rule:

- `cd src-tauri && cargo test` — all green
- `pnpm build` — green (requires cargo test first so ts-rs codegen runs)
- Manual smoke covering: new project creation, open existing project, switch between projects, delete with confirmation, sticky session enable/disable, sticky-fail screen on missing folder, schema-too-new refusal screen (by hand-editing `projects.app_schema_version`), breadcrumbs on every page, sidebar bug fix on About, About-page external links open in browser, recent-projects count setting honored, timezone toggle reformats displayed timestamps without app restart

## 16. Repository layout (M2 additions)

```
src-tauri/
├── migrations/
│   └── project/                          # NEW
│       └── 001_init.sql                  # NEW
└── src/
    ├── commands/
    │   ├── error.rs                      # NEW — CommandError
    │   ├── projects.rs                   # NEW
    │   └── (app.rs refactored for CommandError)
    ├── db/
    │   ├── project_db.rs                 # NEW
    │   └── (migrations.rs generalized)
    ├── project/                          # NEW module
    │   ├── activity_log.rs               # NEW
    │   ├── conflict.rs                   # NEW
    │   ├── delete.rs                     # NEW
    │   ├── lifecycle.rs                  # NEW
    │   └── mod.rs                        # NEW
    └── types/
        ├── project.rs                    # NEW
        └── launch_result.rs              # NEW

src/
├── routes/
│   ├── +page.svelte                      # rewrite Home
│   ├── projects/
│   │   ├── +page.svelte                  # implement chooser
│   │   └── current/+page.svelte          # NEW — dashboard
│   └── settings/
│       ├── +page.svelte                  # add 3 settings entries
│       └── about/+page.svelte            # license + GitHub + acknowledgements + link fix
└── lib/
    ├── components/
    │   ├── Breadcrumbs.svelte            # NEW
    │   ├── NewProjectSheet.svelte        # NEW
    │   ├── ProjectsTable.svelte          # NEW
    │   ├── ConfirmDeleteProject.svelte   # NEW
    │   ├── StickyFailScreen.svelte       # NEW
    │   └── SchemaTooNewScreen.svelte     # NEW
    ├── ipc/
    │   ├── projects.ts                   # NEW
    │   └── (app.ts extended)
    ├── stores/
    │   └── currentProject.ts             # NEW
    ├── help/pages/
    │   ├── home.md                       # NEW
    │   ├── projects-chooser.md           # NEW
    │   ├── project-dashboard.md          # NEW
    │   └── settings.md                   # NEW
    ├── constants.ts                      # NEW (repo URL, defaults)
    └── generated/                        # ts-rs adds new types

static/img/
├── no-projects-placeholder.svg           # NEW (asset commissioned separately)
└── sticky-fail-placeholder.svg           # NEW (asset commissioned separately)

LICENSE                                   # replace with AGPL-3.0
README.md                                 # license + Home copy aligned
CLAUDE.md                                 # license + version-bump rule
package.json                              # license field
src-tauri/Cargo.toml                      # license field
src-tauri/tauri.conf.json                 # license metadata if present
.claude/commands/ship.md                  # version-bump rule integrated
```

## 17. Issues affected by M2

### Closes

- #3 — Migration failure-path test
- #5 — Centralize repo URL constant
- #6 — Typed IPC error (CommandError via ts-rs)
- #18 — Re-confirm MIT license pre-release (superseded by AGPL-3.0-or-later decision)
- #23 — Schema-version mismatch UX

### Updates

- #25 — Title broadened to cover both app-close and project-switch with active runs; the design decision is deferred to M4 brainstorming

### Defers (no M2 changes)

- #1 — Wire or remove `platform::Os` enum + `current_os()` helper
- #2 — Wire or remove `getCachedPlatform()`
- #24 — Auto-run `cargo test --test ts_export` before `pnpm build`

### New backlog Issues filed alongside this spec

(see § 18)

## 18. Parked tangents (file as backlog Issues after spec approval)

- **Funny image #1 asset** — commission the no-projects empty-state illustration.
- **Funny image #2 asset** — commission the sticky-session failure illustration.
- **Project rename feature** — allow renaming a project after creation (no UI in M2; v1.1+).
- **Recent projects: pinning / tags / custom sort** — UX enhancements deferred (v1.1+).
- **IANA timezone picker** — replace UTC/Local toggle with full picker (v1.1+).
- **Activity log rotation policy** — bound `.bhc/activity.log` growth (v1.1+).

## 19. Open questions

None at design time. Implementation-time decisions:

- Whether to use TanStack Table or hand-roll for the chooser (depends on filter complexity and column-sort needs).
- Whether activity-log `mtime` requires re-stat on every chooser render or can be cached (likely cache + invalidate on `open_project` / explicit refresh).
- Exact shadcn-svelte components for the sheet, confirmation dialog, and breadcrumbs.

These will be made during implementation per the parent spec's convention.
