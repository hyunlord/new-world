---
name: worldsim-docs-upkeep
description: Use at the end of every WorldSim pass (and before the final report or pushing v8/main) — the end-of-pass checklist that keeps CHANGELOG, ROADMAP, HISTORY, ADRs, ARCHITECTURE and the skills true, and how to write each entry.
---

# WorldSim docs upkeep

The repository is the project's memory (ADR 0009). A pass is not finished
until the docs say what it did. Owner-facing docs are Korean (`README.md`
first part, `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`);
agent-facing docs are English (`docs/ARCHITECTURE.md`, `docs/decisions/*`,
`CHANGELOG.md`, `AGENTS.md`, `CLAUDE.md`, skills).

## Rules

- **Do** update the docs in the same pass as the change. *Reason:* a later pass cannot reconstruct why; V7 lost months of context to chat transcripts.
- **Do** keep `docs/ARCHITECTURE.md`'s system table generated (`python3 tools/v8/system_table.py --write`). *Reason:* `check_docs.sh` fails when it drifts from the registration code.
- **Do** cite a commit hash or a path for every HISTORY line and every skill lesson. *Reason:* an unsourced lesson cannot be checked or retired.
- **Don't** commit evidence images, captures or probe output under `docs/` (only ≤ 12 curated images in `docs/images/`). *Reason:* V7's `docs/` grew to 1.5 GB of evidence; raw data lives outside the repo.
- **Do** keep one current contact sheet of the game (`docs/images/<pass>_contact_sheet.png`) linked from README. *Reason:* the owner had no picture of the V8 game after M1 replaced the whole view (V8-PLAY).
- **Don't** edit `docs/v8/V8_DIRECTION.md` or a pass's goal file after it is committed. *Reason:* they are the dated originals; `docs/VISION.md` is the living copy.
- **Don't** rewrite an ADR to change a decision; add a new ADR that supersedes it. *Reason:* the record of why must survive.

## Pass coordination (depth roadmap v2 §7)

- Only one performance measurement job may run on this Mac at a time; overlapping measurements are both invalid. Admit each repetition at 1-minute load < 8 and interleave old/new binaries in the same session.
- Use pass-local temporary issue IDs (for example M6A-1) before publication; at publication assign the next free ISS numbers after checking the current ledger.
- Preserve both sides of documentation conflicts; stop and report code/data conflicts. A red gate means diagnose, fix and re-run; stop for code/data conflicts, an unfixable identity failure or the pass time limit.
- A bar that scores 0/8 on the first calibration round is a suspected specification defect: stop, report the mechanism, and get the bar confirmed or withdrawn before freezing and spending a held-out set (owner decision O-2, 2026-10-31).
- Held-out seed register: 1200–1231 M-UX2; 1232–1263 M-CORE; 1264–1295 M-CORE P-3d; 1296–1327 M6a. Allocate subsequent passes 32 seeds at a time from 1328 and record the allocation before evaluation. Calibrate on seeds 0–7 for at most four logged rounds; freeze before held-out and never tune on its results.

## Procedures

### End-of-pass checklist (also in AGENTS.md)

1. **CHANGELOG** (`CHANGELOG.md`): add `## <pass> — <date>` with `Added` / `Changed` / `Removed` (Keep a Changelog). Removed items name the area and its file or line counts.
2. **ROADMAP** (`docs/ROADMAP.md`): set the pass's row status (and final commit); move the "다음" pointer.
3. **HISTORY** (`docs/HISTORY.md`): add a dated line when a milestone ends or something is deleted (what, commit, where the removed thing lives now). Keep the file under 400 lines; details stay at tags.
4. **ADR** (`docs/decisions/NNNN-slug.md`): one file per design decision made in the pass.
5. **ARCHITECTURE** (`docs/ARCHITECTURE.md`): regenerate the system table; update the crate/folder maps, the bridge API groups and the GDScript layout if structure changed. Every path you name must exist.
6. **Skills** (`.claude/skills/*/SKILL.md`): add a lesson row when a problem needed more than one attempt, a better method was found, or Godot/gdext behaved unexpectedly. Same pass, not later.
7. `tools/play.sh --headless-check main` passes, and the current `docs/images/<pass>_contact_sheet.png` (now `mux_contact_sheet.png`, with `mux_compare_sheet.png` beside it) is replaced by the new pass's sheet when the view changed (keep ≤ 12 images; delete the previous pass's sheet). Make the sheet with `tools/v8/capture.gd` and `tools/v8/contact_sheet.py` (`worldsim-godot`, procedure 4) and point README's "지금 화면" line at it.
8. Run `bash tools/v8/check_docs.sh` — it must print `docs: PASS` (it also requires a `docs/images/*_contact_sheet.png` that README links).

### Writing a HISTORY line (Korean)

`- <date> — <what changed, one clause> (<commit>); <measured result if any>.`
Deleted things: `- <date> — 지움: <path or area> (<file count>), <tag where it lives>.`

### Writing an ADR (English)

Copy an existing file in `docs/decisions/`. Sections, in order: Context,
Decision, Alternatives considered, Consequences, Date, Source. Number = the
next free four-digit number; slug = short kebab-case title.

### Writing a CHANGELOG entry

```
## M1 — 2026-10-20
### Added
- Chop orders: … (`rust/crates/sim-systems/src/runtime/jobs/`).
### Changed
- …
### Removed
- `scripts/world/nature_renderer.gd` (193 lines) — replaced by the tile layer.
```

### Writing a skill lesson

One table row: `| date | symptom | root cause | fix | source |`. Symptom =
what you saw; root cause = why; fix = what to do next time (imperative);
source = commit hash or `path` (at a tag if the file is gone). Put it in the
skill whose rules it changes: Godot/gdext → `worldsim-godot`; Rust, ECS,
determinism → `worldsim-architecture`; tests, identity, perf → `worldsim-testing`;
anything else that bit us → `worldsim-pitfalls`. Over 400 lines? Move older
rows to that skill's `references/lessons-archive.md`.

## Lessons

| date | symptom | root cause | fix | source |
|---|---|---|---|---|
| 2026-10-20 | `docs/` had grown to 1.5 GB, pack 5.25 GiB; reviewers could not find the decision record | every pass committed its evidence captures and probe output under `docs/` | keep raw evidence outside the repo; commit summaries only; ≤ 12 images in `docs/images/` | `docs/v8/V8_0_GOAL.md` §"Why this pass exists"; `v7-final-art:docs/revitalize/probe/` |
| 2026-10-20 | `CLAUDE.md` (994 lines) described files and crates that no longer existed (`sim-data` crate, GDScript-era SimulationBus, `scripts/ui/CLAUDE.md`) | instructions were appended per incident and never pruned | keep AGENTS.md ≤ 150 lines pointing at docs and skills; `check_docs.sh` verifies named paths exist | `83a4f2dd:docs/v8/harvest/architecture_raw.md` (V8-0), `v7-final-art:CLAUDE.md` |
| 2026-10-20 | Pass 1's report survives only as an old version of `REPORT.md` | Pass 2 overwrote the shared report file instead of adding its own | one report file per pass; never overwrite another pass's record | `v7-final-art:docs/revitalize/REPORT.md` at `c70b0c67` vs `0941e1b7` |
