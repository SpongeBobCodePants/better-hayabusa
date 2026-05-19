# Roadmap

Better Hayabusa is sliced into eight milestones. Each one is a coherent slice of user-visible value (~2–6 weeks of work for one developer + AI).

| | Title | Status | Scope summary |
|---|---|---|---|
| **M1** | Foundation | ✅ Done | Tauri shell, OS gate, `app.db` + migrations, sidebar, About |
| **M2** | Projects | Next | Create / open / switch projects; recent-projects list; `project.db` bootstrap |
| **M3** | Jobs + targets | | Configure named jobs with target host lists (no execution yet); per-tool reference catalogs |
| **M4** | Run execution + live output | | Click Run; processes spawn; stdout streams; run history; per-host status |
| **M5** | Help system + Settings | | `?` drawer, info icons, theme switcher, log retention |
| **M6** | Tools reference pages | | Searchable in-app reference for Hayabusa and Chainsaw; Settings → Tools surface |
| **M7** | Tool acquisition + updates | | Download / update tools and rule packs with graceful failure |
| **M8** | Portable distribution + release polish | | Real portable Windows zip; v1.0 cut |

## Current work tracking

**Authoritative source for "what's in milestone X":** [GitHub Milestones](https://github.com/SpongeBobCodePants/better-hayabusa/milestones) — each Milestone has a longer description and a filterable list of its open Issues.

**Authoritative source for individual work items:** [GitHub Issues](https://github.com/SpongeBobCodePants/better-hayabusa/issues). Filter by milestone (e.g., `is:open milestone:"M2 - Projects"`) to see what's planned. The `needs-spec-decision` label marks items that need a design call before they can be implemented.

## For AI plan-writers

When drafting a new milestone's plan:

1. Read this file for milestone context.
2. Run `gh issue list --milestone "M<n> - <title>"` to see committed work.
3. Run `gh issue list --no-milestone` to see backlog candidates that might fit.
4. Consider `gh issue list --label needs-spec-decision --milestone "M<n> - <title>"` — those gaps need to be resolved before or during planning.
5. Read [docs/superpowers/specs/](specs/) for the design intent.
6. Write the plan under [docs/superpowers/plans/](plans/) following the `superpowers:writing-plans` skill.

## Out of scope (v1)

Cross-platform builds, Takajo + additional Chainsaw subcommands, multi-project parallel runs, per-host parallelism, scheduled jobs, importing `.txt` scripts, job sharing/export, auth/multi-user/cloud, in-app result viewing with stream-parsed events. See design spec §10.
