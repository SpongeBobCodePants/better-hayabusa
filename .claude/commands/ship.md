---
description: Open a PR, get Codex to review it, merge if clean or loop fixes (max 5 cycles)
---

# /ship — PR + Codex review loop

**Use when:** the user has explicitly approved a completed unit of work for merge to `main`. Do NOT invoke this proactively — it's a user-controlled trigger that fires AFTER they say "ship it" or equivalent.

**Don't use when:** work isn't approved yet, tests are failing, or the user just wants a status check (use `gh pr list` directly).

## Preflight (HARD STOP if any fail)

1. **Branch state.** Must be on a non-`main` branch with commits ahead of `origin/main`. If on `main`, ask the user what feature branch the work lives on. If branch has no diff vs `main`, abort with "nothing to ship."
2. **Clean working tree.** `git status --porcelain` must be empty. If not, stop and report the dirty paths — don't auto-stash.
3. **Tests + build green.** Run `cd src-tauri && cargo test` AND `pnpm build`. Both must pass. If either fails, abort with the failure output. ("Done" verification gate per CLAUDE.md.)
4. **`gh` auth.** `gh auth status` must show a logged-in account with `repo` scope.
5. **Per-repo git config.** `git config user.email` must be the noreply alias from CLAUDE.md, NOT the global personal email. If it's wrong, fix it per CLAUDE.md before continuing.
6. **Version bump available.** Read current version from `src-tauri/tauri.conf.json`. Increment the patch number. Write back to `tauri.conf.json` AND mirror to `package.json` (`"version"` field) AND `src-tauri/Cargo.toml` (`version = "..."`). Stage these as part of the next commit — do NOT push a "version bump" commit separately.

If a check fails, report exactly which one and stop. Don't try to fix unless the user says so.

## Phase A — Open the PR

1. Push the current branch: `git push -u origin HEAD`.
2. Create the PR with `gh pr create --base main --title "<title>" --body "<body>"`:
   - **Title:** derive from the most-recent merge-base-relative commit subject if there's one obvious commit, otherwise ask the user for the title. Optionally prefix with the new version (e.g. `v0.1.5: …`).
   - **Body:** standard format — `## Summary` (1–3 bullets), `## Test plan` (checklist of what was verified), and link any closed Issues (`Closes #NN`).
3. Capture the PR number and URL. Report them to the user.

## Codex review mode (standard vs thorough)

Codex has a per-repo review-thoroughness setting in its cloud dashboard. Two modes seen in practice:

- **Standard mode:** acks within ~1 min, finishes review within ~6 min. The original timings in this command were tuned for this.
- **Thorough mode:** acks can take up to 5 min (one cycle on PR #35 took ~5 min before the 👀 appeared), finishes review in 12–17 min. Surfaces materially more findings per cycle (cycle 7 on PR #35: 7 inline comments vs. typical 1–3) and re-evaluates the whole PR from scratch each pass rather than just the latest diff — meaning it can resurface findings you rejected with a prior PR comment.

The timing windows below are sized for thorough mode (the slower of the two). They're slightly generous for standard mode but won't cause issues; if a session is clearly under standard timings, the loops just exit faster on the success branch.

If you don't know which mode is on, assume thorough (safer default — the timings are longer but not by much, and you won't false-positive on a slow ack).

## Phase B — Request Codex review (with retry)

1. Post a PR comment containing exactly `@codex review` via `gh pr comment <PR#> --body "@codex review"`. Capture the returned comment ID/URL.
2. Wait up to 5 minutes for the ack, polling once per minute. Codex typically acks with an eye emoji (👀) reaction on the trigger comment, OR with a fresh PR comment from a user matching `codex*` / `openai*`. Detection:
   ```
   gh api repos/{owner}/{repo}/issues/comments/{commentId}/reactions --jq '.[] | select(.content=="eyes") | .user.login'
   gh pr view <PR#> --json comments --jq '.comments[] | select(.author.login | test("(?i)codex|openai")) | .body'
   ```
3. **If acked:** proceed to Phase C.
4. **If no ack after 5 min:** post `@codex review` again as a NEW comment (don't edit the original — Codex may not re-trigger on edits). Wait another 5 min.
5. **If still no ack:** STOP and ask the user. Two no-acks usually means Codex isn't installed/enabled on the repo, the bot is down, or rate-limited. Don't keep spamming.

## Phase C — Wait for Codex to finish reviewing

1. Poll every 3 min, up to **25 min total** (8 polls max). Use Monitor with an `until ...; do sleep 180; done` loop. Thorough-mode reviews routinely run 12–17 min; the 15-min cap from the previous version of this spec was load-bearing in a way it wasn't designed to be — bump to 25 to give ~30% headroom.
2. "Done" = Codex has posted ONE of:
   - A formal PR review (`gh pr view <PR#> --json reviews --jq '.reviews[] | select(.author.login | test("(?i)codex|openai"))'`)
   - A summary comment after the ack (distinct from the ack itself)
   - Inline review comments (`gh api repos/{owner}/{repo}/pulls/{PR#}/comments`)
3. **If 25 min elapses with no completion:** STOP and ask the user (could be a long review queue, or Codex is silently stuck).

## Phase D — Evaluate feedback

Read everything Codex posted (formal review body, inline comments, summary comment). Classify each item:

- **Approved with no actionable feedback** (review state APPROVED, no inline comments, or only positive notes) → **Phase E (merge)**.
- **Actionable feedback present** → evaluate each item:
  - **Valid:** real bug, real maintainability issue, real spec/convention violation, clear improvement. → fix it.
  - **Invalid:** misunderstands the code, contradicts CLAUDE.md / project conventions, suggests pattern we explicitly avoid (e.g., `tauri-plugin-sql`), or is purely stylistic with no clear win. → leave alone, but be honest if you're uncertain.
  - **If ANY item is "I'm not sure":** STOP and ask the user for the judgment call. Don't auto-decide on ambiguous calls.

If all flagged items are invalid: post a brief PR comment explaining why you're not addressing them, then merge (Phase E).

If any are valid: implement the fixes locally, commit (one focused commit per fix or one combined "address review feedback" commit — your call based on size), push, and **loop back to Phase B** (re-trigger Codex on the updated PR).

## Phase E — Merge

1. Confirm CI is green: `gh pr checks <PR#>`. If any check is failing, STOP — don't merge red. Report which check failed.
2. Squash-merge with branch deletion: `gh pr merge <PR#> --squash --delete-branch`.
3. Sync local: `git checkout main && git pull origin main && git branch -d <feature-branch>` (use `-d`, not `-D` — if there's unmerged work locally, refuse and report).
4. Confirm `git status` is clean and `git log --oneline -1` shows the squashed commit on `main`.
5. Report the merge commit SHA and the GitHub PR URL.

## Cycle limit

- Max **5 PR cycles** (initial + 4 rounds of fixes) before mandatory user check-in.
- After 5 cycles, STOP and ask the user. Don't keep looping silently — something's wrong if a PR needs 5+ rounds of fixes.

## "Going in circles" detection

Before each Phase D evaluation, classify the cycle's findings into three buckets first, then check the signals:

- **Net-new findings.** Things Codex has not flagged before on this PR.
- **Repeats of previously-rejected findings.** Items you already rejected (with a PR comment explaining why) that Codex is surfacing again — common in thorough mode, which re-reviews the whole PR each pass and does not appear to read your prior rejection comments. **Discount these entirely from the circles signal.** Post a one-line reply to the new inline comment re-pointing at the original rejection, then move on. They are noise, not progress, but not loop evidence either.
- **Repeats of previously-validated findings.** Items Codex flagged before, you tried to fix, and it's flagging the SAME thing again. These ARE the strong loop signal — the fix didn't land or didn't address what Codex meant.

Heuristics — any ONE of these means stop and ask:

- A finding that was previously **validated and fixed** is back (the fix didn't take, or Codex disagrees with the fix). This is the canonical "going in circles" — Codex and the implementer are not converging on the same understanding.
- Among the **non-rejection findings** (i.e. exclude the discounted bucket above), net LOC change across the last 2 cycles is < 5 lines (lots of churn, no real progress).
- The cycle's feedback is dominantly (≥50% of net-new items) about something the user explicitly approved earlier — Codex is fighting CLAUDE.md or the spec, that's a Codex-vs-project disagreement, not a fix. A single such item is noise; half or more of the cycle is a signal.
- You can't articulate, in one sentence, what the next round of fixes is trying to accomplish.

When you stop for circles: summarize what's happening, link the relevant Codex comments, and propose the choice (keep iterating with adjustments, override Codex and merge, or close the PR and rethink).

## When you stop for user input

In every case where this command says "STOP", do this:
1. Don't push more commits or post more comments without instruction.
2. Tell the user: where in the workflow you are, what triggered the stop, what your read of the situation is, and what choices you see.
3. Wait for them to redirect.

## What to report at the end

- PR URL + final state (merged / waiting / aborted).
- **Cycle metrics:**
  - Total cycles run.
  - Total findings, split into: valid-and-fixed, rejected-with-comment, repeats-of-rejected (noise).
  - Total wall-clock elapsed from first `@codex review` to merge.
  - Review mode used (standard vs thorough), if known.
- Codex feedback summary (1–3 bullets — the substantive things caught and addressed; skip the noise).
- Any user decisions made along the way (rejected findings, scope deferrals, mid-loop check-ins).
- Local state (`git status`, current branch, last commit).
