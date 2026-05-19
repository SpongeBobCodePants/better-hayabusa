# Better Hayabusa/ChainSaw — Design

**Date:** 2026-05-19
**Status:** Draft for review
**Scope:** v1 — Windows-only desktop app, architecturally ready for cross-platform expansion later.

## 1. Purpose

A local desktop application that acts as a graphical UI for the command-line forensics tools **Hayabusa** and **Chainsaw**. The user supplies the executables (downloaded themselves), and the app lets them organize, configure, run, and review tool invocations as named "jobs" inside "projects".

The current workflow being replaced: hand-edited PowerShell `.txt` scripts that build up CLI arguments via shell variables.

## 2. Tech stack

### Architecture overview

The app is a Tauri application where **Rust owns all heavy work** and the frontend is a view layer. Rust handles process spawning + stdout/stderr piping, SQLite, HTTP downloads, zip extraction, file I/O, and (in future) result parsing. The Svelte frontend renders UI, manages form state, and calls Rust via Tauri `invoke()`.

Performance is a first-class goal — the tools being wrapped (Hayabusa, Chainsaw) often process gigabytes of EVTX. The architecture is chosen so the UI never blocks on data-plane work and so future result-viewing can stream-parse multi-GB files without OOM.

### Components

| Layer | Choice |
|---|---|
| Application shell | Tauri 2.x |
| **Backend (heavy work)** | **Rust (Tauri command handlers)** |
| Frontend framework | SvelteKit with `@sveltejs/adapter-static` |
| Component library | shadcn-svelte |
| Styling | Tailwind CSS 4 |
| Distribution | Portable zip (no installer) |
| Package manager | pnpm |

### Rust crates

| Concern | Crate |
|---|---|
| Local database | `rusqlite` (bundled-libsqlite, sync, single-process — matches our access pattern; we wrap calls behind Tauri commands so the frontend never blocks) |
| Async runtime | `tokio` |
| Process execution | `tokio::process::Command` (spawn, stream stdout/stderr) |
| HTTP (tool downloads) | `reqwest` (with `rustls` for TLS) |
| Zip extraction | `zip` |
| Serialization | `serde` + `serde_json` |
| CSV (future results viewer) | `csv` |
| TS type generation | `ts-rs` (generates `.ts` definitions from Rust structs so the frontend boundary stays type-safe) |
| Logging | `tracing` |

### Frontend libraries

| Concern | Choice |
|---|---|
| State | Svelte 5 runes + small typed stores |
| Form-time validation | Zod (UX only; Rust re-validates every input it receives) |

### Tauri plugins (UI/system only — heavy work is in Rust)

| Plugin | Use |
|---|---|
| `@tauri-apps/plugin-dialog` | Folder/file pickers |
| `@tauri-apps/plugin-os` | Platform detection (the OS gate at boot) |

Plugins **deliberately not used**: `plugin-sql`, `plugin-shell`, `plugin-http`, `plugin-fs` — all of those concerns live in Rust, where we get streaming, batched IPC, and direct control of process I/O.

**No** SSR runtime, no backend server, no telemetry, no auto-update in v1.

## 3. Platform support

### Supported in v1

- **Windows 10 / 11 (x64)** — the only target users can actually use the app on.

### Cross-platform readiness

The app is designed so adding macOS / Linux later is mechanical, not architectural. Specifics:

- **OS gate at boot.** On app start, `platform()` from `@tauri-apps/plugin-os` is checked. If not `windows`, the entire app renders an "Unsupported OS" screen explaining v1 is Windows-only, with a link to the project's issue tracker. No navigation, no functionality. The gate is a single component at the root of the layout.
- **Paths are never concatenated as strings.** Rust uses `std::path::{Path, PathBuf}` throughout. The frontend only displays paths it receives from Rust — never builds them.
- **Executable naming is centralized.** A Rust helper `executable_name(tool: Tool, os: Os) -> &'static str` returns the platform-correct binary name. For v1 it always returns `hayabusa.exe` / `chainsaw.exe`; the match arm gets new cases later.
- **Command builders take a `PlatformContext`.** A typed Rust argument (currently always `PlatformContext { os: Os::Windows }`) so per-OS quoting/escaping differences land in one place.
- **No shell wrappers.** Executables are spawned directly via `tokio::process::Command`. No `cmd.exe`, no PowerShell. The variable interpolation from the old `.txt` scripts is replaced with Rust-side path/string templating.
- **Tauri config is platform-neutral except bundle targets.** `tauri.conf.json` ships a Windows portable target for v1; adding macOS/Linux portable targets later is additive config.

## 4. Project storage model

A project is a **folder the user picks**. The app creates one hidden subdirectory inside it:

```
<user-chosen-project-folder>/
└── .bhc/
    ├── project.db                       # SQLite — all settings, jobs, run history
    └── runs/
        └── <run_id>/
            ├── run-info.txt             # overall run summary (see § 8b)
            └── hosts/
                └── <host_id>/
                    ├── stdout.log
                    ├── stderr.log
                    └── run-info.txt     # exact command + per-option explanation for this host
```

Note that the **tool's own output files** (CSVs, HTML reports, etc.) do NOT live here — those go to the per-host subdirectory under the job's `root_output_dir` (which the user picks per job, anywhere on disk). The `.bhc/runs/` tree only holds run metadata and captured stdout/stderr.

**Consequences of this design:**

- Projects are portable — copy the folder, it works elsewhere.
- No central registry to get out of sync.
- The user is **not required** to store executables, input files, or output files inside the project folder. Those paths are absolute and live wherever the user wants.

## 4a. App-level state

The app ships as a **portable zip**. The user extracts it anywhere (USB stick, incident response laptop, etc.) and runs the `.exe` directly — no installer, no Program Files, no registry entries.

App-level state lives in a small SQLite database **next to the executable**:

```
<install_dir>/
├── better-hayabusa-chainsaw.exe
├── app.db                    # SQLite — app-level state
└── tools/                    # optional, populated by global tool downloads (§ 9a)
    ├── hayabusa/<version>/...
    └── chainsaw/<version>/...
```

`app.db` schema is intentionally small:

```sql
CREATE TABLE app_state (
  key   TEXT PRIMARY KEY,    -- 'theme', 'log_retention_days', ...
  value TEXT NOT NULL
);

CREATE TABLE recent_projects (
  path           TEXT PRIMARY KEY,
  name           TEXT NOT NULL,
  last_opened_at TEXT NOT NULL
);

-- Global default executable paths (autofill for new jobs).
-- May point at tools/ subdirs after a global download, or anywhere else.
CREATE TABLE global_tools (
  tool_name       TEXT PRIMARY KEY,    -- 'hayabusa' | 'chainsaw'
  executable_path TEXT NOT NULL,
  version_string  TEXT,
  last_checked_at TEXT
);
```

The global default executable paths are used to **autofill** the exe path field when creating a new job. The actual path used at run time always comes from the job's own `settings_json` — so jobs are self-contained and portable, and per-project tool downloads (§ 9a) override globals on a per-job basis.

## 4b. Project storage conflict handling

If the user picks a folder for "New Project" that already contains `.bhc/project.db`, the app offers two choices: "Open it as an existing project" or "Cancel". It does **not** offer to overwrite.

## 5. Navigation and pages

### Layout

```
┌────────────┬──────────────────────────────────┐
│ Sidebar    │ Main content                     │
│            │                                  │
│ Home       │                                  │
│ Projects   │                                  │
│ ─────      │                                  │
│ ▸ <proj>   │                                  │
│   Jobs     │                                  │
│   Queue    │                                  │
│ ─────      │                                  │
│ Settings   │                                  │
└────────────┴──────────────────────────────────┘
```

The project section in the sidebar is only active when a project is open.

### Pages

| Route | Purpose |
|---|---|
| `/` (Home) | Brief explanation of what the app does. "New Project" and "Open Project" buttons. Recent-projects list. |
| `/projects/new` | Folder picker, name, optional description. Creates `.bhc/project.db`. |
| `/projects/open` | Folder picker. Looks for `.bhc/project.db` inside. |
| `/projects/[id]` | Project dashboard: table of jobs with last-run status. "New Job" button. |
| `/projects/[id]/jobs/new` | New-job wizard. Pick tool type → form pre-filled with defaults → add one or more **targets** (hostname + input path) → pick root output directory → save (and optionally run). |
| `/projects/[id]/jobs/[jobId]` | Job detail: full settings (view/edit), Targets table, per-run history, "Run now", per-host live status table during a run, **Run details** panel (rendered from `run-info.txt`). |
| `/projects/[id]/queue` | All jobs in current project grouped by status: Pending · Running · Completed · Failed. |
| `/settings` | Theme, log retention, and a **Tools** section showing installed version + "Update tool", "Update rules", "Open in Explorer" buttons for each tool (see § 9a). Global default executable paths are set here and used to autofill new job forms. |

## 6. Initial job types (v1 scope)

Three concrete tool types, each implemented as a Rust per-tool module with: a settings struct (serde + validators), a defaults provider, an option-metadata catalog (used for both form tooltips and `run-info.txt`), a command builder, and per-host output-path conventions. Each tool also has a matching `Form.svelte` in the frontend (rendering only).

Every job has at least one **target** (hostname + input path) — see § 7 for the data model. Per-tool "Required inputs" below are the *tool options*, not the per-host input path (which is always supplied by the target).

| Job type | Wraps | Required tool options |
|---|---|---|
| `hayabusa_csv_timeline` | `hayabusa.exe csv-timeline` | exe path, output format (csv/json), severities to report (critical/high/medium/low), exclude-status, profile (`-p`) — see note below |
| `chainsaw_hunt` | `chainsaw.exe hunt` | exe path, sigma directory, mapping file, rules directory, level(s), status |
| `chainsaw_analyse_shimcache` | `chainsaw.exe analyse shimcache` | exe path, Amcache hive (optional) — note: the SYSTEM hive comes from the target's input path |

> **Note on Hayabusa options:** the fields listed above are the v1 minimum (covers the existing `.txt` script + format/severity selection). Hayabusa exposes many more flags and we will expand this schema in a follow-up spec. The `settings_json` + serde design accommodates this growth without DDL changes.

### Per-host output paths

The job has one `root_output_dir`. For each target, output files are placed under `<root_output_dir>/<host_id>/`. Per-tool conventions for file naming inside that directory:

- **`hayabusa_csv_timeline`**: `hayabusa_<host_id>.csv` (or `.json`) and `hayabusa_<host_id>.html`
- **`chainsaw_hunt`**: chainsaw's own output goes directly into the host subdirectory (it produces multiple files; chainsaw is told `--output <root_output_dir>/<host_id>`)
- **`chainsaw_analyse_shimcache`**: `chainsaw_shimcache_amcache_<host_id>.csv`

### Defaults

A Rust `defaults` provider per tool ships preconfigured argument sets matching the existing `.txt` scripts. Concretely:

- **Hayabusa default**: `--exclude-status deprecated,unsupported,experimental -m high -p all-field-info-verbose -X -s -w` (input/output paths injected per target)
- **Chainsaw hunt default**: `--level high --level critical --status stable --csv --skip-errors`, with `--sigma`, `--mapping`, and `--rules` autofilled to `<exe_dir>\sigma`, `<exe_dir>\mappings\sigma-event-logs-all.yml`, and `<exe_dir>\rules` respectively (matching the layout of an unpacked Chainsaw release). `<exe_dir>` is resolved from the chosen executable path.
- **Chainsaw shimcache default**: `--tspair`

### Extensibility

A new tool type is a single-directory Rust module + one frontend form:

```
src-tauri/src/tools/<tool>/
├── mod.rs              # struct ToolImpl { ... } implementing the Tool trait
├── settings.rs         # serde-derived settings struct + validators
├── defaults.rs         # default settings
├── options.rs          # option-metadata catalog (name, description, type, choices)
├── command.rs          # build_command(settings, target, platform_ctx) -> Command
└── output_paths.rs     # per-host output file conventions

src/lib/tools/<tool>/
└── Form.svelte         # frontend form, type-driven by ts-rs-generated types
```

A central registry (`src-tauri/src/tools/registry.rs`) maps `tool_type` strings to implementations. The frontend gets the available tool types and their option metadata via the `list_tools` and `get_tool_metadata` Tauri commands.

**Anticipated future tools** (architecture handles trivially): **Takajo** (Yamato Security's Hayabusa output enrichment tool, https://github.com/Yamato-Security/takajo), additional Chainsaw subcommands (`analyse srum`, `analyse mft`, etc.).

## 7. Data model

One SQLite database per project, located at `<project_folder>/.bhc/project.db`.

```sql
-- Single project per database. Constrained to one row at the application layer.
CREATE TABLE projects (
  id                  INTEGER PRIMARY KEY,
  name                TEXT NOT NULL,
  description         TEXT,
  created_at          TEXT NOT NULL,
  app_schema_version  INTEGER NOT NULL
);

CREATE TABLE jobs (
  id              INTEGER PRIMARY KEY,
  name            TEXT NOT NULL,
  tool_type       TEXT NOT NULL,          -- 'hayabusa_csv_timeline' | 'chainsaw_hunt' | 'chainsaw_analyse_shimcache'
  settings_json   TEXT NOT NULL,          -- serde-validated on read/write; tool-specific options only
  root_output_dir TEXT NOT NULL,          -- absolute path; per-host outputs go into <root_output_dir>/<host_id>/
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);

-- Per-job target list. A job runs once per target. At least one row required per job.
CREATE TABLE job_hosts (
  id           INTEGER PRIMARY KEY,
  job_id       INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
  host_id      TEXT NOT NULL,             -- user-supplied unique-within-job identifier
  input_path   TEXT NOT NULL,
  sort_order   INTEGER NOT NULL,
  UNIQUE (job_id, host_id)
);

-- A run is one "Run now" click. It contains N host executions (one per row in job_run_hosts).
CREATE TABLE job_runs (
  id             INTEGER PRIMARY KEY,
  job_id         INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
  status         TEXT NOT NULL,           -- 'pending' | 'running' | 'completed' | 'partial' | 'failed' | 'cancelled'
  started_at     TEXT,
  completed_at   TEXT,
  host_count     INTEGER NOT NULL,        -- denormalized for cheap listing
  error_message  TEXT                     -- populated when the run itself fails (not per-host)
);

-- One row per (run, target) pair.
CREATE TABLE job_run_hosts (
  id             INTEGER PRIMARY KEY,
  run_id         INTEGER NOT NULL REFERENCES job_runs(id) ON DELETE CASCADE,
  host_id        TEXT NOT NULL,
  status         TEXT NOT NULL,           -- 'pending' | 'running' | 'completed' | 'failed' | 'skipped' | 'cancelled'
  command_line   TEXT NOT NULL,
  started_at     TEXT,
  completed_at   TEXT,
  exit_code      INTEGER,
  output_subdir  TEXT NOT NULL,           -- absolute path: <root_output_dir>/<host_id>
  stdout_path    TEXT,                    -- absolute path inside .bhc/runs/<run_id>/hosts/<host_id>/
  stderr_path    TEXT,
  error_message  TEXT
);

CREATE INDEX idx_job_runs_job_id    ON job_runs(job_id);
CREATE INDEX idx_job_runs_status    ON job_runs(status);
CREATE INDEX idx_job_hosts_job_id   ON job_hosts(job_id);
CREATE INDEX idx_run_hosts_run_id   ON job_run_hosts(run_id);
CREATE INDEX idx_run_hosts_status   ON job_run_hosts(status);
```

**Status semantics for `job_runs.status`:**

| Status | Condition |
|---|---|
| `pending` | Created, no host has started yet |
| `running` | At least one host is currently `running` (the rest may be `pending` or `completed`/`failed`) |
| `completed` | All hosts finished with `status='completed'` |
| `partial` | At least one host `completed` and at least one `failed`/`cancelled`/`skipped` |
| `failed` | All hosts ended in `failed` |
| `cancelled` | User cancelled mid-run; remaining hosts become `skipped` |

### Schema migrations

Numbered SQL files in `src-tauri/migrations/`, separated by target database:

```
src-tauri/migrations/
├── app/                 # applies to app.db (next to the exe)
│   └── 001_init.sql
└── project/             # applies to each project.db
    └── 001_init.sql
```

- **`project.db` migrations** run when opening a project; the app runs any migrations whose number is greater than `projects.app_schema_version`, then updates that value.
- **`app.db` migrations** run on app launch; the current version is tracked via `app_state.key = 'schema_version'`.

Tauri's SQL plugin has built-in migration support; we use it for both databases.

## 8. Process execution and live output

All process management lives in Rust. Frontend "Run now" → invoke `start_run(job_id)` → Rust orchestrates.

### Per-host execution loop (sequential within a run)

For each target in `job_hosts` (in `sort_order`):

1. Create `job_run_hosts` row with `status='running'`, `started_at=now()`.
2. Resolve `output_subdir = <root_output_dir>/<host_id>`; `mkdir -p` it.
3. Call the tool's Rust command builder: `build_command(settings, target, platform_ctx) -> Command`. Persist `command_line` (the exact resolved arg list, shell-quoted for display) into the row.
4. Write the per-host `run-info.txt` (§ 8b) into `<.bhc>/runs/<run_id>/hosts/<host_id>/`.
5. Spawn via `tokio::process::Command` with `stdout(Stdio::piped())`, `stderr(Stdio::piped())`.
6. Concurrently read stdout and stderr line streams. Each line is appended to its log file on disk *and* added to a per-host ring buffer; a `tracing`-driven debouncer flushes a batched `RunOutputEvent` to the frontend at ~10Hz (not every line — keeps IPC sane for chatty tools).
7. On process exit: write `exit_code`, `completed_at`, `status` (`completed` if exit 0, else `failed`). Update the host's `run-info.txt` to finalize duration/exit code.
8. After each host completes, recompute and persist `job_runs.status`.

When the host loop finishes, write the run-level `run-info.txt` summary and finalize `job_runs.completed_at` + `status`.

### Cancellation

`cancel_run(run_id)` Rust command:
- Sends SIGTERM (or equivalent on Windows: `Child::kill()`) to the currently-running child.
- Marks the current host `cancelled`; marks all remaining `pending` hosts `skipped`.
- Sets `job_runs.status = 'cancelled'`.

### Concurrency

- **Within a run:** sequential per host (no parallelism in v1).
- **Across runs:** sequential per project — only one run executes at a time. A second "Run now" while one is running creates the new run with `status='pending'`; it starts when the first finishes.
- Per-host parallelism within a run and cross-job parallelism are post-v1.

### Failure modes

| Cause | Outcome |
|---|---|
| Spawn fails (exe not found, permission denied) | host `status='failed'`, `error_message` set, stderr.log contains the spawn error |
| Process exits non-zero | host `status='failed'`, `exit_code` persisted |
| Disk I/O failure writing logs | host `status='failed'`, `error_message` describes the I/O error; process is killed |
| Frontend disconnects | execution continues; events buffer in Rust ring buffer and are sent when the frontend re-subscribes |

## 8a. Backend / frontend boundary

The frontend never touches process I/O, SQLite, files, or HTTP. All access goes through Tauri commands. Indicative surface (final names refined during implementation):

| Command | Purpose |
|---|---|
| **App** |
| `get_app_state()` / `set_app_state(key, value)` | Theme, retention, etc. |
| `list_recent_projects()` | Home page list |
| `get_global_tools()` / `set_global_tool_path(tool, path)` | Settings page |
| **Tools (registry & metadata)** |
| `list_tool_types()` | Available tool types for the new-job wizard |
| `get_tool_metadata(tool_type)` | Options, descriptions, defaults — used to build forms and to render `run-info.txt` |
| **Projects** |
| `create_project(folder_path, name, description)` | Bootstraps `.bhc/project.db` |
| `open_project(folder_path)` | Returns project metadata |
| `close_project()` | Releases the SQLite handle |
| **Jobs** |
| `list_jobs()` / `get_job(id)` | Project dashboard |
| `create_job(...)` / `update_job(...)` / `delete_job(id)` | Job CRUD; payload includes settings + targets |
| **Targets** |
| `add_target(job_id, host_id, input_path, sort_order)` | |
| `update_target(...)` / `delete_target(id)` / `reorder_targets(job_id, ordered_ids)` | |
| **Runs** |
| `start_run(job_id)` | Returns `run_id` immediately; execution proceeds async |
| `cancel_run(run_id)` | |
| `list_runs(job_id)` / `get_run(run_id)` | Run history |
| `get_run_info(run_id)` / `get_host_run_info(run_id, host_id)` | Returns parsed structured `RunInfo` for the UI panel |
| **Tools (download/update)** |
| `download_tool(tool, version, destination)` | Returns progress events |
| `update_tool_rules(tool, install_dir)` | |
| **Events** (frontend subscribes) |
| `run-output:{run_id}:{host_id}` | Batched stdout/stderr chunks (~10Hz) |
| `run-status` | Status transitions for any run or host |
| `download-progress:{download_id}` | Tool download progress |

**Type-safety:** Rust structs used as command inputs/outputs derive `ts-rs` definitions emitted to `src/lib/generated/`. Frontend imports these types. One source of truth.

**Validation:** every command re-validates its input. Frontend Zod is for form-time UX only — Rust does not trust frontend-side validation.

## 8b. Run-info text files

Two text files per run, written by Rust.

**Per-host file** (`<.bhc>/runs/<run_id>/hosts/<host_id>/run-info.txt`): written at host start, finalized at host exit.

```
Better Hayabusa/ChainSaw — Run Info
====================================
Run ID:        42
Job:           "APT-29 sweep" (id=7, tool=hayabusa_csv_timeline)
Host:          DC01
Input path:    C:\evidence\DC01\winevt\logs
Output dir:    D:\analysis\apt29\DC01

Started:       2026-05-19 14:22:01
Completed:     2026-05-19 14:24:17  (duration: 2m 16s)
Exit code:     0
Status:        completed

Command line:
  hayabusa.exe csv-timeline -d "C:\evidence\DC01\winevt\logs" -w
    --exclude-status deprecated,unsupported,experimental
    -m high -p all-field-info-verbose -X -s
    -o "D:\analysis\apt29\DC01\hayabusa_DC01.csv"
    -H "D:\analysis\apt29\DC01\hayabusa_DC01.html"

Options used:
  -d <path>                    Input directory containing .evtx files.
                               Set to: C:\evidence\DC01\winevt\logs
  -w                           Enable Windows-only rule profile.
                               (default for csv-timeline)
  --exclude-status <list>      Skip rules with the listed statuses.
                               Set to: deprecated,unsupported,experimental
  -m <level>                   Minimum severity to include. Only events at this
                               level or higher appear in output.
                               Set to: high
  -p <profile>                 Output profile controlling which event fields appear.
                               Set to: all-field-info-verbose
  -X                           Render extended UTC timestamps.
  -s                           Show statistics summary at the end.
  -o <path>                    CSV output file.
                               Set to: D:\analysis\apt29\DC01\hayabusa_DC01.csv
  -H <path>                    HTML report output.
                               Set to: D:\analysis\apt29\DC01\hayabusa_DC01.html
```

**Run-level file** (`<.bhc>/runs/<run_id>/run-info.txt`): written when the run finishes.

```
Better Hayabusa/ChainSaw — Run Info
====================================
Run ID:        42
Job:           "APT-29 sweep" (id=7, tool=hayabusa_csv_timeline)
Started:       2026-05-19 14:22:01
Completed:     2026-05-19 14:31:48  (duration: 9m 47s)
Status:        partial

Targets (3):
  DC01      completed   2m 16s   exit=0   D:\analysis\apt29\DC01
  DC02      completed   3m 02s   exit=0   D:\analysis\apt29\DC02
  WS-105    failed      4m 29s   exit=2   D:\analysis\apt29\WS-105
                                          (see per-host run-info.txt for details)
```

**Source of truth for option descriptions:** the per-tool Rust `options.rs` catalog. The same catalog drives form tooltips in the frontend (via `get_tool_metadata`), so the file and the UI cannot drift.

**UI rendering:** the Job Detail "Run details" panel calls `get_run_info(run_id)` / `get_host_run_info(...)`, which returns a structured `RunInfo` value (parsed, not raw text). The panel renders it natively with copy-to-clipboard. The on-disk text file is the human-readable, portable artifact.

## 9a. Tool acquisition and updates

Users are responsible for choosing where their tools live, but the app provides convenience operations to fetch and update them. All network operations fail gracefully.

### Installation destinations

| Destination | Where | When used |
|---|---|---|
| **Global** | `<install_dir>/tools/<tool>/<version>/` | "Download/Update tool" action in Settings; sets `global_tools.executable_path` |
| **Per-project** | `<project>/.bhc/tools/<tool>/<version>/` | "Download to this project" action in the New Job wizard; sets the job's exe path |

Both destinations use the same installer code path.

### Operations (provided by a Rust `tool_manager` module)

All operations run in Rust (using `reqwest` + `zip` + `tokio::process::Command`). The frontend invokes them via Tauri commands and subscribes to progress events.

| Operation | Behavior |
|---|---|
| `get_latest_version(tool)` | `GET https://api.github.com/repos/<owner>/<repo>/releases/latest`. Returns version tag + Windows asset URL. |
| `install_tool(tool, version, dest_dir)` | Downloads the Windows zip asset (with progress events), verifies size, extracts to a temp dir, then renames into `dest_dir` (atomic). Returns the resolved exe path. |
| `update_hayabusa_rules(install_dir)` | Spawns `hayabusa.exe update-rules` in `install_dir`, streams its output. |
| `update_chainsaw_sigma_rules(install_dir)` | Downloads `https://github.com/SigmaHQ/sigma/archive/refs/heads/master.zip`, extracts into a temp dir, then atomically swaps with `<install_dir>/sigma/`. Chainsaw's own bundled rules ship in the release zip — only the SigmaHQ community sigma rules need separate fetching. |

GitHub repos used:
- Hayabusa: `Yamato-Security/hayabusa`
- Chainsaw: `WithSecureLabs/chainsaw`

### Network failure handling (graceful, never crashes)

Every network call is wrapped in typed error handling. User-facing error states:

| Failure mode | User sees |
|---|---|
| No network / DNS fail | Banner: "Can't reach the internet. Check your connection." + "Retry" button + link to manual download instructions |
| GitHub API rate-limited (HTTP 403) | "GitHub rate limit reached. Try again in N minutes, or download manually from GitHub." |
| Asset download timeout | "Download timed out at X%. Retry?" |
| Corrupt / truncated zip | "Download was incomplete. Retry?" (partial file deleted) |
| Insufficient disk space | "Not enough disk space at `<destDir>`. Free at least X MB." |
| `update-rules` exit code != 0 | "Updating Hayabusa rules failed. See log." with the spawned process stderr in a collapsible panel |

Any error path leaves the previous (working) install in place — installs are atomic via "extract to temp dir, then rename" on success.

The app never blocks based on network state. If updates fail, the user can always set the exe path manually to a hand-downloaded copy of the tools.

### UI surfaces

- **Settings → Tools**: per tool, show installed version + "Update tool", "Update rules", and "Open in Explorer" buttons.
- **Job form**: next to the exe path field, "Download to this project" and "Update rules now" buttons. Optional "Update rules before each run" checkbox (off by default).

### HTTP scope

Outbound HTTP is initiated by Rust (`reqwest`), not the Tauri HTTP plugin. Rust enforces a hostname allowlist in code — only these hosts may be reached:

- `api.github.com`
- `github.com` (releases/download/archive paths only)
- `objects.githubusercontent.com` (where GitHub redirects asset downloads)
- `raw.githubusercontent.com` (in case sigma fetch needs it)

All other hostnames are rejected before the request is made.

## 9. Cross-cutting concerns

- **Path validation**: on save and immediately before run, Rust checks exe + each target's input path + `root_output_dir` exist. Show inline warnings in the UI; do **not** block save (paths may exist later when the user actually has the files).
- **Tool version detection**: when the user enters an executable path in a job form or in Settings, Rust runs `<exe> --version` once in the background. In the Settings page, the detected version is cached in `app.db.global_tools.version_string`. In job forms, the detected version is shown next to the path field but not persisted (a re-check happens whenever the path changes).
- **Error handling**: any spawn or I/O error during a host execution transitions that host to `failed`, with the error written both to `stderr.log` and `job_run_hosts.error_message`. A run-level error (e.g., can't create the output directory) sets `job_runs.error_message`.
- **Logging retention**: a global setting (default: keep all runs) lets users auto-delete run artifacts older than N days. Cleanup runs on project open.
- **No telemetry, no auto-update in v1.**

## 10. Out of scope for v1

- Cross-platform runtime (architecture is ready; build targets are not).
- **Takajo** and other future tool integrations (Hayabusa output enrichment, Chainsaw `analyse srum/mft`, etc.) — the per-tool Rust module pattern (§ 6) handles them as additive work.
- Multi-project parallel job runs.
- Per-host parallelism within a run (sequential per-host in v1).
- Re-running only the failed hosts of a previous run (v1.1 idea).
- Scheduled or recurring jobs.
- Importing existing PowerShell `.txt` scripts as jobs.
- Sharing or exporting jobs between projects.
- Authentication, multi-user, cloud sync.
- **In-app viewing of tool results.** Right now the user opens the CSV/JSON files Hayabusa and Chainsaw produce in external tools (Excel, jq, Timeline Explorer). A future version of the app will surface those results in-app: a table you can filter, sort, search, and pivot, scoped per host or aggregated across hosts. To make that fast on multi-GB outputs we'll have Rust stream-parse the files into additional tables in the per-project `project.db` (e.g., a `parsed_events` table indexed by host + timestamp + severity). v1 doesn't need any structural change to enable this — SQLite is already in place and Rust can do the parsing.

## 11. Repository layout (proposed)

```
better-hayabusa-chainsaw/
├── src/                                 # SvelteKit frontend (view layer only)
│   ├── routes/                          # pages from § 5
│   ├── lib/
│   │   ├── components/                  # shadcn-svelte and app-specific components
│   │   ├── ipc/                         # thin invoke() wrappers per Tauri command
│   │   ├── generated/                   # ts-rs-generated TS types (do not edit)
│   │   ├── tools/
│   │   │   ├── hayabusa_csv_timeline/Form.svelte
│   │   │   ├── chainsaw_hunt/Form.svelte
│   │   │   └── chainsaw_analyse_shimcache/Form.svelte
│   │   └── stores/
│   └── app.html
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── migrations/
│   │   ├── app/                         # app.db migrations
│   │   │   └── 001_init.sql
│   │   └── project/                     # project.db migrations
│   │       └── 001_init.sql
│   └── src/
│       ├── main.rs                      # entry, Tauri setup, command registration
│       ├── commands/                    # #[tauri::command] handlers grouped by domain
│       │   ├── app.rs
│       │   ├── projects.rs
│       │   ├── jobs.rs
│       │   ├── targets.rs
│       │   ├── runs.rs
│       │   └── tool_manager.rs
│       ├── db/
│       │   ├── app_db.rs                # app.db connection + queries
│       │   ├── project_db.rs            # project.db connection pool + queries
│       │   └── migrations.rs            # numbered-file runner
│       ├── platform/                    # OS detection, executable_name, path helpers
│       ├── process/
│       │   ├── runner.rs                # per-host execution loop, IPC batching
│       │   └── events.rs                # RunOutputEvent, RunStatusEvent
│       ├── tool_manager/
│       │   ├── installer.rs             # download + atomic extract
│       │   ├── github.rs                # GitHub releases API client (allowlisted)
│       │   ├── sigma.rs                 # sigma rules fetch
│       │   └── errors.rs                # user-facing error variants
│       ├── tools/
│       │   ├── registry.rs              # tool_type → impl
│       │   ├── traits.rs                # Tool trait, OptionMetadata, RunInfo, etc.
│       │   ├── hayabusa_csv_timeline/   # mod.rs + settings + defaults + options + command + output_paths
│       │   ├── chainsaw_hunt/
│       │   └── chainsaw_analyse_shimcache/
│       └── run_info/                    # RunInfo struct + text-file rendering
├── docs/superpowers/specs/
├── package.json
├── pnpm-lock.yaml
└── README.md
```

## 12. Open questions

None at design time. Implementation-time decisions (NSIS branding details, exact shadcn-svelte component picks per page, log-viewer virtualization library) will be made during implementation.
