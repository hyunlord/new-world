# GOAL: WorldSim — V8-0: a clean slate for the colony sim (pin the past, cut the god game, restructure without changing the simulation, living docs in the repo, skills that accumulate lessons)

Repository `hyunlord/new-world`. Base: **`codex/art-production-2026-10` = `b1ca032f8fa84cee87d68a185e471b8e2c2bddce`** (Art 13b). It already contains the whole simulation line `codex/revitalize-2026-10` = `354e4a13b21a20b28c65d84fab103c40b5143fa0` (Pass 21, `game_v26`): merge-base(354e4a13, b1ca032f) = 354e4a13, so the art head is 286 commits ahead and 0 behind.

This run is executed by **Claude Code**, unattended, on the Mac, in `/Users/rexxa/work/new-world-pass4`. No DGX. No other run uses the Mac during this pass.

## Why this pass exists

On 2026-10-20 the owner changed the game's direction. The full decision record is `docs/v8/V8_DIRECTION.md`, copied in by the owner; read it completely before Part 1.
- WorldSim stops being a god game, where you watch and press intervention buttons.
- It becomes a **Dwarf Fortress–style settlement sim**:
  - about 10 founders, growing to 30–200 people, with an architecture that can scale to thousands;
  - the player gives orders (dig, chop, build, zones, work priorities, craft orders), with RimWorld-style drafting;
  - technology is owned by individuals and spreads between people, and eras emerge from how widely the marker technologies have spread;
  - a compressed year: a day is about 2 minutes at 1×, and a year is 4 seasons × 6 days;
  - a dark, earthy top-down view where one person equals one tile.

The repository does not fit that direction. Planner's measurements at `b1ca032f`:

| area | size |
|---|---|
| `rust/crates/sim-test` | 86,529 lines (150 `harness_*` test files plus per-pass probe modules: `probe/pass4.rs` … `pass21.rs`, `ablation.rs`) |
| `sim-systems` / `sim-core` / `sim-bridge` / `sim-engine` | 18,246 / 13,994 / 6,356 / 1,490 lines |
| `scripts/test/**` | 222 GDScript files, 33,947 lines |
| `scripts/ui/**` | 52 files, 8,717 lines, many with pass suffixes (`activity_icons_p11.gd`, `settlement_relations_p12.gd`, `art_walker_p13b.gd` …) |
| `docs/` | `docs/revitalize` 1.5 GB (`probe/` is nearly all of it), `docs/art` 902 MB, `docs/worldmodel` 3.4 MB |
| `tools/` | `revitalize` 165 files, `art_pipeline` 367 files, `harness`, `worldmodel`, a dozen one-off migration scripts |
| repo history | `size-pack` 5.25 GiB |

Other problems:
- `CLAUDE.md` is 994 lines of V7-era rules.
- `AGENTS.md` tells agents to run `git reset --hard origin/lead/main` and to push to `lead/main`. Both are now forbidden.
- Skills are split between `.claude/skills/` (`worldsim-code`, `worldsim-harness`) and `.agents/skills/` (`godot`, `worldsim-code`, `systematic-debugging`, `verification-before-completion`).

The owner asked for four things, and this pass delivers all four:
1. Delete what the new direction does not need.
2. Make the code structure clean.
3. Keep the direction, purpose and history as markdown in the repository, updated every pass.
4. Turn coding problems, better methods and Godot-specific know-how into skills that keep growing.

**V8-0 must not change the simulation.** Every deletion and every move is checked against the baseline captured in Part 0. The probe output on 32 worlds × 120 days and the chronicle must be byte-identical (timing fields excepted). The only intended changes are:
- in the game, the god-hand panel and the overlays listed in 7.3 disappear;
- in the bridge, the intervention methods disappear.

New gameplay starts in M1, not here.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin
git checkout -b v8/wip origin/codex/art-production-2026-10        # b1ca032f
mkdir -p docs/v8
cp ~/Downloads/claude_goal_worldsim_v8_0.md docs/v8/V8_0_GOAL.md
cp ~/Downloads/WorldSim_V8_방향.md docs/v8/V8_DIRECTION.md
claude --dangerously-skip-permissions
# then: /goal <completion condition naming docs/v8/V8_0_GOAL.md>
```
The clone's hooks are neutralized as before:
- the local `.claude/settings.json` holds `{}` and is marked `--skip-worktree`;
- never edit the working file and never clear the flag (7.6 says how its committed content changes);
- leave the local `.gitignore` edit and `.ignore` uncommitted;
- stage explicit paths only, never `git add -A` or `git add .`.

**This document is `docs/v8/V8_0_GOAL.md`.** Commit it and `docs/v8/V8_DIRECTION.md` unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause.

Your final message must contain:
- the section 15 report;
- the deletion table;
- the identity verdict;
- commit hashes;
- the `git ls-remote` heads of `v8/wip` and `v8/main`, and the two tags.

## 0.2 Overrides of CLAUDE.md and AGENTS.md
Both files are V7-era and are **rewritten by this pass** (Part 5). Until then, ignore them where they conflict with this document. In particular:
- never `git reset --hard`;
- never push `lead/main`, `main` or any `codex/*` branch;
- no `install_hooks.sh`;
- no `harness_pipeline.sh`;
- built-in `general-purpose` subagents only;
- if a blocking hook other than `/goal` appears, record it, push and report.

## 0.3 Autonomy
1. **Never ask the owner anything.**
   - Decisions go in `docs/v8/V8_0_DECISIONS.md` (one numbered entry each: context, choice, reason, evidence path).
   - Things only the owner can decide go to the **ASK list** (8.4). Those things are kept, not deleted.
2. **Do not end your turn** between Parts or while a build, probe or gate runs.
3. **Parts:**

   | Part | Section | Content |
   |---|---|---|
   | 0 | 5 | pins and baseline |
   | 1 | 6 | inventory and harvest |
   | 2 | 7 | deletions |
   | 3 | 8 | restructure |
   | 4 | 9 | repository docs |
   | 5 | 10 | agent instructions and skills |
   | 6 | 11 | gate, review, report |

   Push after every Part, and after every deletion batch in Part 2.
4. **Review loops:** two rounds at most.
5. **On failure:** work around it, record it, continue. A bar you miss is reported as missed, never hidden.
6. **Time rule:** after **14 hours** of working time (pauses excluded and recorded), finish the current step, push and go to Part 6.
7. **Drop order:** 8.6 (decision split) → 8.5 (bridge split) → 8.7 (GDScript layout move) → 8.3 (dead-flag removal) → the lesson minimums of 10.3 halved.
   - **Never drop:** Part 0, the inventory and harvest (Part 1), all of Part 2, 8.1, 8.2, 8.4, 8.8, Part 4, the skill skeletons with their seed lessons, the gate, identity and the report.
8. **Mac sleep:** record the pause and continue.

---

# 1. MACHINE
- **Work only in** `/Users/rexxa/work/new-world-pass4` and one base worktree, `../new-world-v8-base` (detached at `b1ca032f`, for Part 0 and for comparisons).
  - Never touch other checkouts, `~/github/worldsim-training`, `~/github/new-world-wt/lead`, ComfyUI, ports 3100/3200/8188, or processes you did not start.
  - Never signal a process you did not start.
- **Parallelism:** `CARGO_BUILD_JOBS=8`; tests `--test-threads=8`; probe `--workers 6`.
- **Data:** `/Users/rexxa/work/v8-0-data/` (outside the repository).
- `GODOT_BIN` is the same as Pass 21 (`tools/revitalize/gate_pass21.sh` shows how it is resolved). Copy that resolution into `tools/v8/`.

# 2. GIT
- **Branches:**
  - Commits go to **`v8/wip`**.
  - **`v8/main`** is created only by the final step of Part 6, at a commit whose gate (11.1) is green. After that it moves only by fast-forward to green commits.
  - Run `git ls-remote origin v8/wip v8/main 'refs/tags/v7-final-*'` after every push.
  - Commit messages are `v8-0(pN): …`.
- **Never:**
  - push to `lead/main`, `main` or any `codex/*` branch;
  - force-push;
  - delete or move a remote branch or tag;
  - rewrite history (no filter-repo, no rebase of pushed commits).
- **May change:** everything in the repository, except:
  - the working file `.claude/settings.json` (only its committed blob may change, through 7.6);
  - `.gitignore` and `.ignore`. Proposed ignore rules go in the report.
- **Tags (Part 0):** these are the only remote refs you create besides `v8/wip` and `v8/main`. Use annotated tags.
  - `v7-final-sim` → `354e4a13b21a20b28c65d84fab103c40b5143fa0`
  - `v7-final-art` → `b1ca032f8fa84cee87d68a185e471b8e2c2bddce`
  - If a tag of that name already exists on the remote, don't overwrite it. Record it and create `-v8pin` variants instead.

---

# 3. CONTEXT (planner's survey at `b1ca032f`, paths under `rust/crates/` unless stated; verify each fact)

## 3.1 The live game
- `sim-bridge/src/ffi/world_node.rs`:
  - `build_game_engine(seed)` (≈ L204) is `build_world_with(&WorldGenConfig::game_v26(seed), WorldRules::game_v26(seed))`;
  - `process` drives 30 TPS.
- `WorldRules::game_v20 … game_v26` are chained `const fn`s in `sim-core/src/rules.rs` (1,821 lines, plus `rules/routine.rs`). `game_v20` draws `variety::WorldVariety::draw_v18(seed)`, so **`variety.rs` is live**.
- Older constructors and scenes (`game_v9 … game_v19`, `legacy_scene`, `reset_world_stress`, the `game_v16/17/18_*_scene(_with)` family in `sim-test/src/scenes.rs`, `probe/ablation.rs`) exist only for past passes and the research track.
- **Intervention (the god hand):**
  - `sim-systems/src/runtime/intervention.rs` (196 lines);
  - `sim-engine/src/intervention.rs` (87);
  - in `world_node.rs`: `apply_intervention` (≈ L340), `parse_intervention` (≈ L2843), `apply_intervention_command` (≈ L2878), `intervention_reply_to_dict` (≈ L2896), and the tests at ≈ L3090/L3103;
  - in GDScript: `scripts/ui/panels/god_hand_panel.gd` and `scenes/ui/god_hand_panel.tscn`.
- **Large files:**
  - `sim-systems/src/runtime/decision/agent_decision.rs` 3,827 lines;
  - `sim-bridge/src/ffi/world_node.rs` 3,500+;
  - `sim-core/src/rules.rs` 1,821;
  - `sim-systems/src/runtime/settlement/settlement_system.rs` 1,378;
  - `sim-core/src/causal/event.rs` 1,055.
- **Art kernels:** `sim-bridge/src/ffi/art_kernels.rs` (declump, walker offsets) serves `scripts/ui/art_declump_p12.gd` and `art_walker_p13b.gd`.

## 3.2 The research track (must stay reproducible, not in V8)
- **World model:**
  - `sim-test/src/worldmodel/**` and `sim-test/src/bin/worldsim_wm.rs`;
  - `tools/worldmodel/**` and `docs/worldmodel/**`;
  - the branches `codex/worldmodel-2026-10` (`58cf8611`) and `-wip`. Their merge-base with the sim line is `f5373ab2`; they live on their own line, and WM-6 runs from that branch on the DGX.
- After the tags exist, the research line and everything at `v7-final-sim` / `v7-final-art` are reproducible without the V8 line.
- **Rule:** V8 owes the research code nothing. It is deleted from V8, and HISTORY records where it lives (9.4).

## 3.3 Tooling to reuse
- **Probe:** `sim-test/src/bin/worldsim_probe.rs`, with flags `--scene --seeds --days --out --chronicle-out --workers --check-determinism --sample-every --clock-samples`.
- **Identity:** `tools/revitalize/identity_p21.sh` (probe + chronicle per scene), `compare_summaries.py` and `validate_identity_p21.sh`.
- **Gate:** `tools/revitalize/gate_pass21.sh` (15 steps: bridge build, Godot import, tests debug and release, clippy, clippy all targets, forbidden-math grep, Godot import, `tools/harness/gdscript_strict_check.sh`, a 900-frame headless smoke, python unit tests, perf quick, allowed paths).
- **Bounded runs:** `tools/revitalize/run_bounded.sh`.
- **Perf:** `sim-test/src/bin/worldsim_perf.rs`. Its `quick` mode runs the `v19` stress scene (`quick_stress_v19_s15100_n500_side64_fed.json`).
- **Localization at runtime:** `scripts/core/locale.gd` (autoload `Locale`) loads `localization/manifest.json`, then `localization/compiled/{lang}.json`. If that fails it falls back to the category JSONs in `localization/{lang}/`. `localization/fluent/` is **not** read by `locale.gd`; verify whether anything else reads it.

## 3.4 Agent configuration
- `.claude/skills/`: `worldsim-code`, `worldsim-harness` (`SKILL.md`, `evaluation_criteria.md`, `test_templates.md`).
- `.claude/agents/**`.
- `.agents/skills/`: `godot` (with `references/`, `assets/`, `scripts/`), `worldsim-code`, `systematic-debugging`, `verification-before-completion`.
- Also `addons/harness/` (only `.uid` files and `worldsim_mapping.md`), `.harness/`, `hooks/pre-commit-harness` and `stats/`.

---

# 4. RULES

## 4.1 Identity (the core safety net)
**Base data** is captured once in Part 0 at `b1ca032f`:
- the probe for scene **`game_v26`**, seeds **0–31**, **120 days**, with `--chronicle-out`;
- `--check-determinism` on seeds 0–3.

**Candidate:** the same probe on the V8 baseline scene (8.1). Probe summaries must be **identical except timing fields**, and chronicle outputs identical **by content**.

**When to run it:**
- after every Part 2 batch that touches `rust/**`;
- after every Part 3 step;
- in the final gate.

**If identity fails:**
- the failing commit is not pushed to `v8/wip` until it is fixed or reverted;
- record the cause in DECISIONS;
- a mismatch is never "explained away".

Write `tools/v8/identity.sh` by copying the `identity_p21.sh` logic: one scene, base data path as an argument, `compare_summaries.py` copied into `tools/v8/`.

## 4.2 Bridge API
Part 0 writes the sorted list of every GDScript-visible method name (`#[func]` and every other exported item) to `docs/v8/bridge_api_v7_final.txt`. The final list must equal it minus the intervention methods (7.2). Each removed name is listed in DECISIONS.

## 4.3 Code rules carried over (they become skill content in Part 5)
- Integers and permille only in `sim-core` / `sim-systems` / `sim-engine`; keep the forbidden-math grep.
- Maps are iterated in key order (BTreeMap or sorted).
- No structural hecs insertion while iterating a hash set.
- Keyed draws consume no stream state.
- GDScript never writes simulation state.
- No `.unwrap()` in production Rust you touch.

## 4.4 Deletion rules
- **Nothing is deleted without an inventory row** (Part 1) giving the reason and the evidence.
- **Harvest before deleting** (6.4). Anything with history, lessons or architecture knowledge is mined first.
- **Delete in batches, one area per commit.** After each `rust/**` batch: build, test, identity. After each GDScript/scene batch: Godot import + smoke.
- **When in doubt, keep it and put it on the ASK list.**

---

# 5. PART 0 — Pins and baseline
1. **Commit and push.** Commit `docs/v8/V8_0_GOAL.md` and `docs/v8/V8_DIRECTION.md` unchanged and push `v8/wip`.
2. **Tags.** Create and push the two annotated tags (section 2). The message of each is one line naming the line it closes.
3. **Base worktree and baseline data.** Create the base worktree `../new-world-v8-base` at `b1ca032f`. Build release `sim-test` there and capture:
   - **Probe:** `game_v26`, seeds 0–31, 120 days, with chronicle → `$DATA/base/probe`, `$DATA/base/chronicle`.
   - **Determinism:** `--check-determinism` on seeds 0–3 → `$DATA/base/determinism.txt`.
   - **Rules dump:** `format!("{:#?}", WorldRules::game_v26(s))` and the `WorldGenConfig::game_v26(s)` Debug string for s ∈ {0, 1, 7, 42}, using a tiny throwaway test or binary that you **don't commit**. Commit the output to `rust/crates/sim-core/tests/data/v7_final_rules_seed{0,1,7,42}.txt` and `…/v7_final_worldgen_seed{…}.txt`.
   - **Bridge API list** (4.2).
   - **Perf:** Pass 21's natural measurement procedure from `docs/revitalize/PERF_BUDGET.md` on `game_v26`, plus `worldsim_perf quick` as it is → `$DATA/base/perf/`. Copy the numbers you will compare against into `docs/v8/V8_0_BASE_NUMBERS.md`.
   - **Headless smoke log** of the base game (900 frames) → `$DATA/base/smoke.log`. Count the `SCRIPT ERROR`, `ERROR:` and missing-locale warnings; the candidate may not exceed them.
   - **Screenshots:** two screens of the base game for the report (as Pass 21 captured, with `tools/revitalize/capture.sh`): one zoomed in, one zoomed out → `$DATA/base/shots/`. Don't commit them.
4. **Push** the Part 0 commit(s).

---

# 6. PART 1 — Inventory and harvest
Output: **`docs/v8/V8_0_INVENTORY.md`**. It is committed before any deletion.

## 6.1 The live graph
- **A. Rust.** Start from `build_game_engine` and the bridge's exported methods. List:
  - every system registered in the live engine: name, priority, interval, file;
  - every `sim-core` module the live engine and bridge reach.

  Write the method (grep plus a compile check) into the inventory. A module is **live** if removing it breaks `cargo build -p sim-bridge` or changes the identity probe.
- **B. GDScript and scenes.** Write `tools/v8/gd_reach.py`. Start from `project.godot` (main scene and autoloads) and follow, transitively:
  - `preload("res://…")`, `load("res://…")`, `extends "res://…"`;
  - `class_name` uses;
  - `[ext_resource path="res://…"]` in `.tscn` / `.tres`;
  - `res://` string literals.

  Output the reachable set. Unit-test the parser on 5 hand-made cases.
- **C. Assets and shaders:** referenced by a reachable script, scene or resource, or by Rust data paths.
- **D. Localization:** keys used by reachable GDScript, plus keys built dynamically.
  - Find every `Locale.ltr(` / `Locale.trf(`-style call whose argument is not a literal. List the prefix families they build (for example `"CHRONICLE_KIND_%s"`) and keep the whole family.
  - Add keys that Rust sends across the bridge.

## 6.2 Verdict table
Give one row per:
- top-level path;
- file in `scripts/**`, `scenes/**`, `tools/**`;
- module in `rust/crates/*/src/**` and `rust/crates/sim-test/tests/*`;
- directory in `docs/**`, `assets/**`, `localization/**`.

Columns: `path | verdict (KEEP / DELETE / MOVE→path / ASK) | reason | evidence`. The final gate checks that every `git ls-files` path is covered by a row, by prefix.

## 6.3 Planner's pre-decided verdicts
Apply these, and verify that each KEEP is really live.

**DELETE**
- **docs:**
  - `docs/revitalize/`, `docs/art/`, `docs/worldmodel/`;
  - `docs/analysis/`, `docs/design/`, `docs/guides/`, `docs/plans/`, `docs/reference/`, `docs/traits/`, `docs/localization/`, `docs/architecture/`;
  - `docs/CHANGELOG.md`, `docs/llm-setup.md`, `docs/sprite_session_handoff_wildlife_a4.md`.

  All after the harvest.
- **Root:** `vlm_interactive_test_results.zip`, `stats/`, `.harness/`, `hooks/`, `addons/harness/` (check `project.godot` `[editor_plugins]` first), `.claude/agents/**`.
- **tools:**
  - `tools/revitalize/`, `tools/art_pipeline/`, `tools/worldmodel/`;
  - `tools/harness/` — but first **MOVE** `tools/harness/gdscript_strict_check.sh` and `tools/revitalize/run_bounded.sh` to `tools/v8/`;
  - the one-off root scripts: `migrate_i18n.py`, `trait_migration.py`, `gen_trait_v3.py`, `fix_trait_centers.py`, `data_localization_extract.py`, `derive_composite_violation_stress.py`, `generate_building_sprites.py`, `migration_verify*.sh`, `rust_shadow_*`, `find_unused_files.py`, `tools/scripts/`, `tools/art/`.

  **Exception:** keep whichever `localization_compile*.py` / `generate_locale_from_json.py` produces `localization/compiled/*.json`. Identify it by running it and diffing the output.
- **Godot test code:** `scripts/test/**` and `scenes/test/**`. The new smoke (11.1) is written fresh in `tools/v8/`.
- **Research and pass scaffolding in sim-test:**
  - `src/worldmodel/**`, `src/bin/worldsim_wm.rs`;
  - `src/probe/ablation.rs`, `src/probe/pass*.rs`, `src/subprofile_report.rs` if only pass code uses it;
  - every `tests/*.rs` whose subject is a deleted feature, a scene other than the baseline, or a pass-specific measurement (6.5).
- **God hand:** see 7.2.
- **Old scenes and constructors:** see 8.1.
- **UI outside the keep set (7.3).**
- **Localization:** `localization/fluent/**` and `key_registry.json` / `key_owners.json` if nothing kept reads them; unused keys (7.4).

**KEEP**
- crates `sim-core`, `sim-systems`, `sim-engine`, `sim-bridge`, `sim-test` (as a test crate: probe, perf, tests);
- `sim-core/src/material/**` (data only, not live — kept for M2);
- `variety.rs`;
- `causal/**` and the chronicle (the "why?" UI);
- `project.godot`, `icon.svg*`, `sim_bridge.gdextension*`, `.editorconfig`, `.gitattributes`;
- `.agents/skills/systematic-debugging` and `verification-before-completion` (generic; moved per 10.1).

**ASK** (keep, list for the owner): any `sim-core` / `sim-systems` module that is neither live nor material. Expected candidates: `influence/**`, `tile/**`, `runtime/influence/**`, `runtime/combat/**`, `runtime/memory/**`, `fission_threshold.rs`. Verify each against 6.1 A; for each, state whether it is registered in the live engine.

## 6.4 Harvest (before Part 2)
Write drafts under `docs/v8/harvest/`. They are consumed by Parts 4 and 5 and deleted at the end of Part 5.
- **`history_raw.md`:** one line per revitalize pass 1–21, art pass 1–13b and WM 1–6: date, branch head, what changed, the measured result.
  - Sources: `docs/revitalize/REPORT_PASS*.md`, `DECISIONS.md`, `docs/art/**/REPORT*.md`, `docs/worldmodel/**/REPORT*.md`, `git log`.
  - Also: the V7 reset (`742dd1c5`, 2026-05-03) and the pre-V7 eras, from `docs/CHANGELOG.md` (one paragraph).
- **`lessons_raw.md`:** every recorded problem, workaround or "better method" in those reports, in `DECISIONS.md`, the `REVIEW_PASS*.md` files and the art DECISIONS. Format: symptom, cause, fix, source path at `v7-final-art`. Tag each Godot/gdext, Rust/ECS/determinism, testing/measurement or process.
- **`architecture_raw.md`:** whatever `docs/architecture/**`, `CLAUDE.md` and `.claude/skills/worldsim-code` say that is still true of the kept code. Verify each claim.
- **`removed_index.md`:** one line per deleted directory or file group: path, what it was, `available at tag v7-final-art` (or `v7-final-sim` / the WM branch).

## 6.5 Test triage (`sim-test/tests`, 150 files, plus unit tests inside deleted modules)
- **KEEP** a test file if both are true:
  - it exercises behavior of a kept system on the live rules (or rule-independent engine behavior);
  - it passes at base.
- **DELETE** a test file if any of these is true:
  - it targets a deleted feature or a scene other than `game_v26`;
  - it is pass-specific scaffolding (evidence writers, report generators, art boards);
  - it duplicates a kept test.
- **Coverage table** in the inventory: every live system → its kept tests. If deletions leave a live system with **no** test, write one minimal test for it in Part 3 (8.4).

---

# 7. PART 2 — Deletions
One area per commit, in this order. After each Rust batch run build, `cargo test --workspace`, clippy and identity. After each GDScript batch run Godot import and smoke.

## 7.1 docs, root junk, stats, `.harness`, hooks
After the harvest is committed. Also write `docs/archive/INDEX.md` from `removed_index.md`. It is permanent and is the only trace of deleted docs.

## 7.2 God hand (the only intended behavior change)
- **Remove:**
  - the GDScript panel, its scene and every reference to them;
  - the bridge methods `apply_intervention` (the `#[func]`), `parse_intervention`, `apply_intervention_command`, `intervention_reply_to_dict` and their tests;
  - `sim-engine/src/intervention.rs`;
  - `sim-systems/src/runtime/intervention.rs`;
  - intervention-only fields and types in `sim-core`.
- **Identity must still pass:** natural runs never intervene.
- V8 will build a storyteller from scratch in M5. The removed code stays reachable at `v7-final-art` (record this in `removed_index.md`).

## 7.3 UI keep set
**Keep exactly these, plus their transitive dependencies from `gd_reach.py`:**
- `scenes/main.tscn`, `scripts/core/locale.gd`;
- `camera_controller.gd`, `world_renderer.gd`, `terrain_renderer.gd`, `building_renderer.gd`, `agent_renderer.gd`, `daylight_controller.gd`;
- `panels/hud_topbar.gd`, `panels/chronicle_panel.gd`, `panels/agent_inspector_panel.gd`, `panels/causal_panel.gd`;
- the shaders those use.

**Delete everything else in `scripts/ui/**`**, including:
- activity badges, tooltips and the village activity summary;
- settlement relation lines and porter presentation;
- seek/death/need-bar/activity-trail overlays;
- restless meter, offscreen markers, founding-group renderer;
- the feed ledger panel, `art_ui.gd` and the art calibration scene.

Rules:
- A deleted file that a kept file needs is not deleted. It is kept as a dependency and listed in the report under "M1 replaces".
- Remove the deleted nodes from `main.tscn` cleanly.
- The game must boot and show: the world, people, buildings, the top bar, the chronicle panel, and the inspector and causal panel on click.

## 7.4 Localization
- Keep the runtime pipeline (`manifest.json`, `compiled/`, the category JSONs, the generator) and only the keys from 6.1 D.
- Regenerate `compiled/*.json` with the kept generator.
- The smoke may show **no more** missing-key warnings than base. Every warning count is logged.

## 7.5 Assets and shaders
Delete what is not referenced (6.1 C). Keep `assets/fonts/galmuri/**` and anything the kept renderers load at runtime (atlases, `*.json` region maps).

## 7.6 `.claude/settings.json` committed blob
The committed file still wires harness hooks to scripts this pass deletes. Make the committed content equal to the local neutralized file **without touching the working file or the skip-worktree flag**:
```bash
sha=$(git hash-object -w .claude/settings.json)      # reads the local neutralized file
git update-index --cacheinfo 100644,"$sha",.claude/settings.json
```
Commit it alone. Then verify both of these:
- `git show HEAD:.claude/settings.json` is byte-equal to the local file;
- `git ls-files -v .claude/settings.json` still shows `S`.

## 7.7 sim-test
Delete per 6.3 and 6.5. Keep:
- `worldsim_probe` and `worldsim_perf` (their `quick` mode is re-pointed in 8.1);
- `perf.rs`, `scenes.rs` (reduced), `probe/{args,runner,summary,metrics,chronicle_out,fingerprint,mod}.rs`;
- any probe module the kept binaries need.

---

# 8. PART 3 — Restructure (identity after every step)

## 8.1 One baseline instead of version chains
- **Rules:**
  - Replace the `game_vNN` chain with `WorldRules::baseline(seed)` and `WorldGenConfig::baseline(seed)`.
  - Their values are written out as flat literals equal to `game_v26`, except values that come from `variety` draws, which stay as calls.
  - Move `rules.rs` into `sim-core/src/rules/`: `mod.rs` holds `WorldRules`, one file per sub-rule struct (`routine.rs` exists; add `relations.rs`, `family.rs` and the rest as they exist), and `baseline.rs`.
- **Tests:**
  - Add `sim-core/tests/baseline_matches_v7_final.rs`. It compares `format!("{:#?}", …)` of the new baseline against the Part 0 dumps for seeds 0, 1, 7, 42.
  - If a later step removes a field (8.3), update the expected dump in the same commit and justify it in DECISIONS.
- **Callers:**
  - Delete every `game_v9 … game_v26`, `legacy`, `reset_world_stress`, `*_nofission`, `*_neutral` and `*_with` constructor and scene that nothing kept uses.
  - The bridge's `build_game_engine` and the sim-test scene `baseline_scene(seed)` call the baseline.
  - The probe scene name becomes `baseline`. Keep `game_v26` as an accepted alias in the probe only, so `identity.sh` can run the same command at base and at candidate.
- **Perf:** re-point `worldsim_perf quick` to a `baseline` stress with 500 founders on a 64×64 map, fed as v19 was. Record the new file name.
- **Identity.**

## 8.2 Names
- Remove pass and version suffixes from kept identifiers and files in Rust and GDScript (`_p10` … `_p13b`, `pass17`, `_v26`, `v18` in function names).
- Keep the meaning in the name: for example `art_walker_p13b.gd` → `walker_offsets.gd`, and `draw_v18` → `draw` (variety).
- The bridge API names that GDScript calls keep their names unless the GDScript side changes in the same commit. Every rename goes in DECISIONS.
- Identity.

## 8.3 Dead flags (droppable)
Remove a rule field when all three hold:
- its baseline value makes one side of every branch on it unreachable;
- no kept test sets it to the other value;
- removing it keeps identity.

Delete the field and the dead branch. Each removal is one line in DECISIONS (field, baseline value, branches removed).

## 8.4 Tests by domain
- Rename the kept `tests/harness_*.rs` into domain files: `tests/agents_*.rs`, `settlement_*.rs`, `family_*.rs`, `relations_*.rs`, `chronicle_*.rs`, `engine_*.rs`, `determinism.rs`. Merge small ones.
- Add a shared helper `sim_test::baseline_engine(seed)`.
- Write the missing minimal tests from 6.5's coverage table. Each one runs the baseline for a fixed seed and days, and asserts a meaningful invariant with a threshold, not `> 0`.

## 8.5 Bridge split (droppable)
- Split `world_node.rs` by concern, as pure moves:
  - `ffi/node.rs`: the node, lifecycle, `process`, speed;
  - `ffi/snapshot.rs`: per-frame arrays;
  - `ffi/reads_agent.rs`, `ffi/reads_settlement.rs`: getters;
  - `ffi/engine_build.rs`: `build_game_engine`;
  - the existing `activity.rs`, `chronicle.rs`, `family.rs`, `relations.rs`, `art_kernels.rs` stay.
- The bridge API list must equal 4.2's minus the interventions.
- Identity and smoke.

## 8.6 Decision split (droppable)
- Split `agent_decision.rs` into `runtime/decision/{system.rs, targets.rs, forage.rs, gather.rs, disputes.rs, retarget.rs, post_pass.rs}` or the closest seams the code actually has. Pure moves; `pub(crate)` where needed.
- Identity.

## 8.7 GDScript layout (droppable)
- Lay out `scripts/core/` (autoload, sim reads), `scripts/world/` (renderers, camera, daylight), `scripts/ui/hud/` and `scripts/ui/panels/`.
- **Move each `.gd` together with its `.uid`.** Rewrite every `res://` path, re-import with Godot, and run the smoke.
- Record in `lessons_raw.md` anything Godot did that surprised you.

## 8.8 Module headers and size
- **Rust:** every kept `.rs` file under `src/` starts with a `//!` header.
  - Always: purpose.
  - For systems, also: priority, interval, components read, components written, events emitted.
- **GDScript:** every kept `.gd` starts with a comment block giving its purpose and the bridge methods it calls.
- **cargo fmt:** run `cargo fmt --all` once, as its own commit, at the **start** of Part 3.
- **Size report:** list every `src` file over 1,200 lines after the splits, as a table in the report. Not a failure.

---

# 9. PART 4 — Repository docs
**Language:**
- owner-facing docs in **Korean**: `README.md` (Korean first, a short English paragraph after), `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`;
- agent-facing docs in **English**: `docs/ARCHITECTURE.md`, `docs/decisions/*`, `CHANGELOG.md`, `AGENTS.md`, `CLAUDE.md`, skills.

## 9.1 `README.md`
- what the game is (from VISION §0–1);
- how to build the bridge and run the game;
- how to run the gate;
- a doc map with one line per doc;
- the branch rules in three lines.

## 9.2 `docs/VISION.md`
- `V8_DIRECTION.md` §0–§7 and §10, wording unchanged; only fix relative paths. §8 goes to ROADMAP, §9 and §11 to AGENTS/skills.
- `docs/v8/V8_DIRECTION.md` stays as the dated original.

## 9.3 `docs/ROADMAP.md`
- the M0, V8-0, M1–M6 table from §8, with a **status** column (M0 done 2026-10-20; V8-0 = this pass with its final commit; the rest pending) and a "next" pointer;
- a rule line: every pass updates its row.

## 9.4 `docs/HISTORY.md`
Dated sections, from `history_raw.md`:
- pre-V7 (one paragraph);
- V7 reset `742dd1c5`;
- revitalize passes 1–21 (one line each, with measured outcome);
- art passes 1–13b (one line each);
- the world-model track WM-1–6 (one line each; it continues on `codex/worldmodel-2026-10`);
- the 2026-10-20 V8 decision;
- V8-0 (what was deleted, with tag pointers).

Under 400 lines. Details stay at the tags.

## 9.5 `docs/ARCHITECTURE.md`
- crate map and folder map;
- data flow (engine tick → systems → snapshot → bridge → renderers);
- **the live system table** (name, priority, interval, file, reads, writes), generated by `tools/v8/system_table.py` from the registration code, so it stays true. `check_docs.sh` verifies the table matches the generator's output;
- the bridge API grouped by concern;
- determinism and integer-math rules (short, linking the skill);
- "how to add a system" and "how to add a bridge read" (step lists);
- the GDScript layout.

Every path named in the file must exist (`check_docs.sh` checks).

## 9.6 `docs/decisions/` (ADR, one file each)
Each file has: Context, Decision, Alternatives considered, Consequences, Date, Source.

| ADR | Topic | Source |
|---|---|---|
| 0001 | V8 settlement sim instead of god game | VISION §1 |
| 0002 | View B, top-down, 1 person = 1 tile | |
| 0003 | 10 founders | |
| 0004 | Compressed year: 2-minute day, 24-day year | §10.1 |
| 0005 | No z-levels; hill digging on the plane; deep layers as mine buildings | `WorldSim_지하자원_4자합성.md` summary in VISION §3 |
| 0006 | Neighbours as the channel of knowledge | §10.2 |
| 0007 | RimWorld-style drafting | |
| 0008 | Research pinned by tags; V8 owes research nothing | |
| 0009 | Repository docs and skills are updated every pass | |
| 0010 | God hand removed; storyteller to be built in M5 | |
| 0011 | `game_vNN` chains replaced by one baseline | |

## 9.7 `CHANGELOG.md` (root)
Keep-a-changelog style. The first entry is `## V8-0 — 2026-10-xx` with Added / Changed / Removed. Removed items name the area and its line or file counts.

---

# 10. PART 5 — Agent instructions and skills

## 10.1 Where skills live
- **Canonical location:** `.claude/skills/<name>/SKILL.md`, with frontmatter `name` and `description` (the description says when to use it).
- **Codex:** for every skill, `.agents/skills/<name>` is a **relative symlink** to `../../.claude/skills/<name>`. Verify that Git stores it as a symlink (`git ls-files -s` mode `120000`) and that it resolves.
- If you find evidence that Codex cannot follow symlinked skill directories, use plain copies plus a `tools/v8/sync_skills.sh` that `check_docs.sh` verifies. Record which you chose and why.
- Move `systematic-debugging` and `verification-before-completion` into `.claude/skills/` unchanged, and symlink them back.

## 10.2 The skill set
| skill | content | absorbs |
|---|---|---|
| `worldsim-architecture` | crate boundaries, ECS rules, determinism and integer math, "add a system", "add a bridge read", file size and header rules | `.claude/skills/worldsim-code`, `.agents/skills/worldsim-code`, `architecture_raw.md` |
| `worldsim-godot` | Godot 4.6 + gdext: FFI patterns, headless import/smoke/capture, `.uid` handling on moves, pixel fonts on the design grid, Container UI rules, tile and bulk rendering notes for M1, GDScript strict check | `.agents/skills/godot` (keep its generic references under `references/`; drop what is wrong for 4.6) |
| `worldsim-pitfalls` | the problem log: one entry per real problem | `lessons_raw.md` |
| `worldsim-testing` | identity, determinism, the gate, perf comparison, test layout by domain, how to write a meaningful assertion | `.claude/skills/worldsim-harness` (drop the pipeline and hook parts) |
| `worldsim-docs-upkeep` | the end-of-pass checklist (10.4) and how to write HISTORY, ADR and CHANGELOG entries | — |

**Format of each SKILL.md:**
- **Rules:** do / don't, each with a one-line reason.
- **Procedures:** numbered steps.
- **Lessons:** a table: `date | symptom | root cause | fix | source`.

Keep SKILL.md ≤ 400 lines; overflow goes to `references/*.md`.

## 10.3 Seed lessons
Each lesson must cite its source (a commit hash or a path at `v7-final-art`). Drop a seed you cannot verify and say so.

**Minimums:** `worldsim-pitfalls` ≥ 15, `worldsim-godot` ≥ 10, `worldsim-architecture` ≥ 8, `worldsim-testing` ≥ 6.

Planner's seeds:
1. **Hash iteration and structural insertion:**
   - symptom: archetype-dependent nondeterminism from iterating a `HashSet` while inserting into hecs;
   - fix (`c03ae2bc`): sorted `(AgentId, Entity)` pairs;
   - rule: BTreeMap for new collections.
2. **Forbidden float math in the sim crates:** integers and permille only, enforced by the gate grep.
3. **Keyed draws:** they consume no stream state, so skipping one is deterministic.
4. **Green structural tests ≠ a working game loop:** a missing tick dispatcher once passed every harness test.
5. **Sprite variant filenames:** they must be 1-indexed and contiguous, or the variant loader caches null and rendering fails.
6. **A filter added to "the" selector must reach every selector** (Pass 20: the avoid list missed the opportunistic Gather arm). Grep every caller of every function that can set the target.
7. **Tuned thresholds that coincide make a branch unreachable** (Pass 20: aid thresholds = exchange thresholds and aid tried first, so 0 exchanges).
8. **Censoring:** metrics on durations must account for runs that end before the event (Pass 21 U3).
9. **Tail percentiles:** p99.9 from 3 repeats is noise; report it as such (Pass 21 U6).
10. **Mac contention:** measurements need a calibrated admission probe (Art 13), with attempts ≥ 30 minutes apart.
11. **Pixel glyphs:** size rules are stated in design pixels on the font's 12-px grid, not in screen pixels (Art 13 → 13b; Life text at 24 px).
12. **Godot `.uid` files** must move with their scripts.
13. **Headless import** must run before a headless smoke, or class names are unresolved (verify against the gate's two import steps).
14. **GDScript reads the bridge only**; it never writes sim state.
15. **Hook neutralization:** skip-worktree on `.claude/settings.json`, never committed.
16. **Identity before claiming "no behavior change":** summaries + chronicle by content.

Add every other lesson `lessons_raw.md` yields that is still true for V8.

## 10.4 End-of-pass checklist (in `worldsim-docs-upkeep` and in AGENTS.md)
1. **CHANGELOG** entry.
2. **ROADMAP** row status.
3. **HISTORY** line: when a milestone ends or something is deleted.
4. **ADR:** when a design decision was made.
5. **ARCHITECTURE:** regenerate the system table; update the maps if structure changed.
6. **Skills:** add a lesson when a problem needed more than one attempt, a better method was found, or Godot/gdext behaved unexpectedly. Do it in the same pass.
7. **`tools/v8/check_docs.sh`** green.

## 10.5 `AGENTS.md` and `CLAUDE.md`
**`AGENTS.md`** is canonical, English, ≤ 150 lines:
- what WorldSim is (two lines, link VISION);
- branch and push rules (work on `v8/wip`; `v8/main` by fast-forward on a green gate; never `lead/main`, `main`, `codex/*`; never force-push; never reset --hard; never delete remote refs);
- stage explicit paths;
- the code rules of 4.3;
- the skill index (name → when to read, with path);
- the gate command;
- the end-of-pass checklist;
- where docs live.

It must contain no `reset --hard` and no instruction to push `lead/main` or `main` (`check_docs.sh` greps).

**`CLAUDE.md`** = `@AGENTS.md` plus at most 30 lines specific to Claude Code (goal files under `docs/<line>/`, subagent use).

Finally, delete `docs/v8/harvest/` (its content now lives in docs and skills), keeping `docs/archive/INDEX.md`.

---

# 11. PART 6 — Gate, review, report

## 11.1 `tools/v8/gate.sh <log_dir>`
Each step is logged with its time:

| # | step | what it checks |
|---|---|---|
| 1 | `fmt` | `cargo fmt --all -- --check` |
| 2 | `bridge_build` | `cargo build -p sim-bridge` |
| 3 | `godot_import` | bounded headless `--import --editor --quit` |
| 4 | `test_debug` | `cargo test --workspace --no-fail-fast -- --test-threads=8` |
| 5 | `test_release` | the same in `--release` |
| 6 | `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` |
| 7 | `forbidden_math` | the Pass 21 regex over `sim-core/src`, `sim-systems/src`, `sim-engine/src` |
| 8 | `gdscript_strict` | `tools/v8/gdscript_strict_check.sh` |
| 9 | `smoke` | `tools/v8/smoke.sh`: bounded headless 900 frames; fails if counts of `SCRIPT ERROR`, `ERROR:` or missing-locale warnings exceed base |
| 10 | `identity` | `tools/v8/identity.sh $DATA/base` |
| 11 | `determinism` | probe `--check-determinism` seeds 0–3 |
| 12 | `perf` | report only: baseline natural measurement and `quick` vs base; > +10 % mean is a WARN in the report |
| 13 | `bridge_api` | 4.2 |
| 14 | `docs` | `tools/v8/check_docs.sh` (9.5, 10.1, 10.5 checks + required files exist + `CHANGELOG.md` has `## V8-0` + no tracked file > 5 MB + no binary under `docs/` except ≤ 12 images in `docs/images/`) |
| 15 | `inventory_closure` | every `git ls-files` path is covered by an inventory row |
| 16 | `python_unittest` | unit tests for `tools/v8/*.py` |

## 11.2 Independent review
Two fresh `general-purpose` subagents that have not seen the work.

**Docs reader.** Gets only `README.md`, `docs/**` and the skills, and must answer 10 questions with file paths:
1. where the relations system is and its priority;
2. how to add a system;
3. how to add a bridge read;
4. which branch to push;
5. what the gate is;
6. where the god hand went;
7. what the time scale is;
8. where research code lives;
9. what to do after a pass;
10. where a Godot lesson is recorded.

Fix the docs for every wrong or "not found" answer, then re-ask that question (two rounds maximum).

**Diff reviewer.** Gets the inventory and `git diff --stat b1ca032f..HEAD`, and looks for:
- deleted files that were live;
- KEEP rows that are dead;
- ASK items that should have been decided.

Address the findings or record them.

## 11.3 Publish
1. Run the final gate on a commit with no edits during the run.
2. If green: create `v8/main` at that commit and push it.
3. Run `git ls-remote`.
4. Write `docs/v8/V8_0_REPORT.md` in the section 15 format and commit it to `v8/wip`.
5. Fast-forward `v8/main` to the report commit after rerunning step 14 only, since the report is docs-only. If any other step would be affected, rerun the full gate.

## 11.4 Screens
Capture the same two screens as Part 0 on the candidate → `$DATA/cand/shots/`. Don't commit. Describe the differences in the report.

---

# 12. Invariants (tests that must exist at the end)
- `baseline_matches_v7_final` (8.1);
- the determinism test on the baseline;
- each live system has at least one kept or new test (6.5 table);
- `gd_reach.py` unit tests;
- `check_docs.sh` passes;
- the bridge API test (4.2) as a Rust or Python test, not only a gate step.

---

# 13. Ask list and proposals the report must carry
- **ASK modules:** the 6.3 ASK modules, with a recommendation each.
- **Remote branches:** every remote branch with its last commit date, whether it is contained in `v7-final-art` or `v7-final-sim`, and a recommendation (delete / keep). Nothing is deleted by you.
- **Repository size:** pack 5.25 GiB. Present the options without doing any:
  - (a) keep the repo, with no more large files going forward;
  - (b) start a fresh repository for V8 from `v8/main`'s tree and archive the old one;
  - (c) history rewrite.
- **Ignore rules:** the proposed `.gitignore` rules (for example `docs/**/*.png` except `docs/images/`).

---

# 14. 인게임 확인사항 (오너용)

`v8/main`에서 게임을 실행했을 때 확인할 것:
1. 에러 없이 게임이 뜨고, 이전과 **같은 세계**가 보인다. 같은 시드라면 사람 수, 마을, 건물이 Art 13b 화면과 같아야 한다.
2. **신의 손 패널이 없다.** 활동 배지, 툴팁 배지, 마을 관계선, 운반꾼 표시, 욕구 막대, 활동 궤적, 죽음·탐색 표시도 없다.
3. 상단 막대, 연대기 패널이 그대로 동작한다. 사람을 클릭하면 상세 정보와 원인('왜?') 패널이 뜬다.
4. 콘솔에 빨간 에러와 "missing key" 경고가 Art 13b보다 늘지 않았다.
5. 2배·3배속에서 체감 FPS가 이전과 같다.
6. GitHub의 `v8/main`에서 확인할 것:
   - `README.md`만 읽고 이 게임이 무엇인지, 어떻게 실행하는지 알 수 있다.
   - `docs/VISION.md`, `ROADMAP.md`, `HISTORY.md`가 한글로 읽힌다.
   - `docs/decisions/`에 결정 11개가 있다.
7. `.claude/skills/`에 스킬 다섯 개가 있다. 각 스킬의 교훈 표에 출처(커밋 또는 문서 경로)가 달려 있다.

---

# 15. 구현 완료 보고 (`docs/v8/V8_0_REPORT.md`와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고 — V8-0

### 구현 의도
(한 문장: 신 게임 시절 코드·문서를 정리하고, 시뮬레이션을 바꾸지 않은 채 V8 개발의 깨끗한 출발점을 만든다)

### 구현 내용
- 태그: v7-final-sim = …, v7-final-art = …
- 브랜치: v8/wip = …, v8/main = …
- 삭제 요약 표:
  | 영역 | 전 (파일/줄 또는 용량) | 후 | 비고 |
  (sim-test, sim-systems, sim-core, sim-bridge, sim-engine, scripts/ui, scripts/test, tools, docs, localization 키 수, assets)
- 구조 변경: (baseline 통합, 분할한 파일, 폴더 배치, 이름 변경 수)
- 문서: (만든 파일 목록)
- 스킬: (스킬별 규칙 수 / 교훈 수)

### 구현 방법
(인벤토리 방식, 도달성 도구, 수확 → 삭제 순서, 매 단계 identity 확인)

### 기능 설명
(게임에서 사라진 것, 남은 것, 사용자 관점 변화)

### 변경된 파일 목록
(커밋별: 해시 — 메시지 — 영역)

### Identity 결과
(32 세계 × 120일 요약·연대기 비교 결과, 결정론 4 시드, 단계별 통과 기록)

### 성능
(base 대비 평균·p99, quick 결과, WARN 여부)

### 문서 검토 결과
(질문 10개: 정답/수정 후 정답/실패)

### 확인된 제한사항
(M1에서 교체할 의존 파일, 드롭한 단계, 1,200줄 초과 파일 표)

### 오너 결정 필요
(ASK 모듈별 권고, 원격 브랜치 정리 제안 표, 저장소 용량 선택지, .gitignore 제안)

### Gate 결과
(16단계 각각 통과/실패, 로그 경로)
```
