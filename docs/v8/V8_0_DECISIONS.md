# V8-0 decisions

One numbered entry per decision taken without the owner (goal 0.3). Format:
context, choice, reason, evidence. Raw data paths are under
`/Users/rexxa/work/v8-0-data/` (`$DATA`, outside the repository).

## D1 — Push target of `v8/wip`
- Context: the owner's launch command created `v8/wip` from `origin/codex/art-production-2026-10`, so its upstream was that `codex/*` branch; a bare `git push` would have pushed to a forbidden branch.
- Choice: the first push was `git push -u origin v8/wip`, re-pointing the upstream to `origin/v8/wip`; every later push names the branch explicitly.
- Reason: goal section 2 (never push `codex/*`).
- Evidence: `git rev-parse --abbrev-ref @{u}` before/after; `$DATA/PROGRESS.md`.

## D2 — Tags
- Context: no `v7-final-*` tag existed on the remote (`git ls-remote origin 'refs/tags/v7-final-*'` was empty).
- Choice: annotated tags `v7-final-sim` → `354e4a13` and `v7-final-art` → `b1ca032f` created and pushed; no `-v8pin` variant was needed.
- Evidence: `git ls-remote` in the Part 0 log of this file's commit.

## D3 — Identity capture flags
- Context: goal 4.1 asks for the probe on `game_v26`, seeds 0–31, 120 days, with chronicle, and `--check-determinism` on seeds 0–3.
- Choice: `tools/v8/identity.sh` runs the probe with Pass 21's identity flags (`--clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120`, so the clock-sample blocks of the summary are compared too) and `--workers 6`; the determinism check is a second probe run on seeds 0..4 (half-open) whose `DETERMINISM PASS` line is required.
- Reason: the same flags as Pass 21 cover the most summary content; worker count does not change results (Pass 21 identity procedure).
- Evidence: `tools/v8/identity.sh`, `$DATA/base/probe.log`, `$DATA/base/determinism.txt`.

## D4 — `git_sha` excluded from the summary comparison
- Context: the probe writes the run label `WORLDSIM_GIT_SHA` into `summary.json` (`probe/metrics.rs`) and into every chronicle file. Base and candidate are different commits.
- Choice: `tools/v8/compare_summaries.py` (a copy of Pass 21's) excludes `git_sha` as a label key in addition to the timing keys, and now also fails on one-sided paths; the chronicle comparison excludes only `git_sha` (Pass 21 also tolerated an extended `kinds` list; V8-0 does not).
- Reason: `git_sha` is provenance, not simulation output; everything else must be identical.
- Evidence: `tools/v8/compare_summaries.py`, `tools/v8/identity.sh`.

## D5 — Worldgen dumps live beside the rules dumps
- Context: goal 5.3 puts both dumps under `rust/crates/sim-core/tests/data/`, but `WorldGenConfig` is defined in `sim-systems` (`worldgen/mod.rs`), which `sim-core` cannot depend on.
- Choice: both dump families are committed under `sim-core/tests/data/` as the goal says; the rules comparison test lives in `sim-core/tests/`, the worldgen comparison test in `sim-systems/tests/` and reads the same files by relative path.
- Evidence: `rust/crates/sim-core/tests/data/v7_final_*`.

## D6 — Screenshots
- Context: `tools/revitalize/capture.sh` exits 77 ("Tier A requires an Aqua session") because this run's shell is in a `Background` launchd session; its GDScript (`scripts/test/capture_framings.gd`) is deleted by this pass anyway.
- Choice: a fresh, small `tools/v8/capture.gd` (main scene, `advance_ticks` to tick 1440, a zoomed-in shot at the first village tile at zoom 4 and a zoomed-out shot at the map centre at zoom 1, the framings of Pass 21's F2/F1) runs windowed with `--rendering-driver opengl3 --resolution 1920x1080`, which works from this session. The same script captures base (copied untracked into the base worktree) and candidate.
- Evidence: `$DATA/base/shots/{near,far}_t1440.png`, `$DATA/base/shots.log`.

## D7 — Smoke counts at base
- Context: goal 5.3 asks for the counts of `SCRIPT ERROR`, `ERROR:` and missing-locale warnings in a 900-frame headless smoke at base.
- Choice: counted after two headless imports (the first import of a fresh worktree logs font-import errors before the fonts are imported; the gate also imports twice). `Locale.ltr` returns the key itself for a missing key and logs nothing; the only locale warnings `locale.gd` can print start with `Locale:`, so that prefix is what is counted.
- Evidence: `$DATA/base/smoke.log` (5 lines: 0 / 0 / 0), `$DATA/base/import.log` (13 font-import errors, first import), `$DATA/base/import2.log` (0).

## D8 — The V8 smoke also scans labels for raw keys
- Context: `Locale.ltr` returns the key itself for a missing key and logs nothing (`scripts/core/locale.gd`), so a `Locale:` warning count cannot catch a key pruned by mistake (7.4).
- Choice: `tools/v8/smoke.sh` runs the plain 900-frame boot and `tools/v8/smoke.gd`, which boots the main scene for 900 frames and counts Label/RichTextLabel/Button texts that are key-shaped (`^[A-Z][A-Z0-9]*(_[A-Z0-9]+)+$`). Base counts were recorded with the same scripts in the base worktree: 0 script errors, 0 errors, 0 locale warnings, 0 raw keys (75 labels).
- Evidence: `tools/v8/smoke_base_counts.txt`, `$DATA/base/smoke_v8/`.

## D9 — `probe/pass*.rs` are kept (and renamed), not deleted
- Context: goal 6.3 lists `src/probe/pass*.rs` for deletion, but those modules compute blocks of the probe's `summary.json` and raw rows (`probe/runner.rs` calls `pass10::seed_info`, `pass11::seed_block`, `pass12::seed_block`, `pass13::seed_block`, `pass20::seed_block`, …; `summary.rs` calls `pass4_sections`, `pass5_sections`, `pass8::rules_block`). Deleting them would change the summary that identity (4.1) requires to be identical.
- Choice: keep them as "probe modules the kept binaries need" (7.7) and rename them by content in 8.2.
- Evidence: `rust/crates/sim-test/src/probe/{runner,summary}.rs`.

## D10 — `reset_world_stress` stays in the bridge API, re-pointed to the baseline
- Context: 8.1 deletes `reset_world_stress` constructors that nothing kept uses; 4.2 requires the final bridge API to equal the base list minus the intervention methods only. The bridge method `reset_world_stress` builds `WorldGenConfig::stress_v19` under `WorldRules::game_v19`.
- Choice: keep the `#[func]` name; in 8.1 it builds the same stress world under the baseline rules (the world generator inputs of `stress_v19` equal the baseline's apart from founders and size). Benchmark-only, never called by the kept UI.
- Evidence: `docs/v8/bridge_api_v7_final.txt`; `sim-bridge/src/ffi/world_node.rs` `build_stress_engine`.

## D11 — `nature_renderer.gd` is kept (ASK), not deleted
- Context: 7.3 keeps an explicit list of renderers and deletes "everything else in `scripts/ui/**`", with examples that are all overlays, badges and HUD panels. `nature_renderer.gd` (+ `nature_canopy.gd`) draws the world's trees, berry bushes and deadwood — the resources people walk to — not an overlay. Without it the map shows bare terrain and people walk to invisible food.
- Choice: when in doubt, keep (4.4): the node and its two scripts stay, the item goes on the ASK list with the recommendation to replace it with the M1 tile layer.
- Evidence: `scripts/ui/nature_renderer.gd` header; base shots `$DATA/base/shots/*.png`.

## D12 — Art-kernel fixture tests and their 135 MB fixtures are deleted
- Context: `sim-bridge/src/ffi/art_kernels.rs` has three unit tests that read `tests/fixtures/art_p12/declump_v1.bin` (57 MB) and `art_p13b/walker_v1.bin` (83 MB), recorded by the deleted `scripts/test/art` capture tools. The docs gate (11.1 step 14) forbids tracked files over 5 MB, and VISION §5 reduces the art measurement rigs to one perf regression check.
- Choice: delete the fixtures and the fixture-reading tests; the other `art_kernels` unit tests stay.
- Evidence: `art_kernels.rs` lines ~372, ~1101, ~1245 at `v7-final-art`.

## D13 — 7.7 (sim-test research code) runs before 7.2 (god hand)
- Context: the world-model recorder (`sim-test/src/worldmodel/**`), `perf.rs`'s intervention mode and several harness tests call the intervention API; deleting the god hand first would leave them uncompilable.
- Choice: the sim-test batch (worldmodel, ablation, triaged tests) is committed before the god-hand batch. Each batch is still one area per commit, with build + tests + clippy + identity after it.

## D14 — Localization source becomes the category JSONs
- Context: `localization/compiled/*.json` (what `locale.gd` loads) is generated by `tools/localization_compile.py` from `localization/fluent/*/messages.ftl` (manifest `source_format: fluent`); the category JSONs disagree with it on 1,457 keys (Part 1 scratch run). `tools/localization_compile.py` reproduces the committed compiled files byte-identically.
- Choice: keep `tools/localization_compile.py` as the generator; `tools/v8/locale_prune.py` writes the kept keys into the category JSONs with the values of the current compiled tables, switches the manifest to `source_format: json`, `preserve_key_ids: false`; Fluent and `key_owners.json` are deleted; `key_registry.json` stays because the generator rewrites it on every run. `locale_prune.py --verify` checks compiled = category JSONs = kept keys, each with the `v7-final-art` value.
- Evidence: scratch dry run (157 keys, categories ui + buildings), `tools/v8/locale_prune.py`.

## D15 — Tests on `WorldRules::default()` count as "setting" a flag (8.3)
- Context: the test triage found that 51 KEEP files and most adapted fixtures run on `WorldRules::default()`, which takes the non-baseline side of many flags (`strict_consume=false`, `keyed_draws=false`, `night_rest=None`, `BirthRule::Legacy`, every `SocietyRules` option `None`, …). 8.3 removes a field only when "no kept test sets it to the other value".
- Choice: the literal reading — a kept test that runs on `default()` does reach the other side, so those branches stay. 8.3 then applies only to fields whose other side no kept test reaches (checked per field when 8.3 runs).
- Reason: deleting a branch that kept tests execute would turn rule-independent tests into tests of nothing; 8.3 is the most droppable restructure step.
- Evidence: `docs/v8/harvest/test_triage.md` §1 "A ruling DECISIONS needs".

## D16 — `game_v8`, `game_default`, `game_scene` and `build_world` go with the chain
- Context: the chain's root (`WorldRules::game_v8`, `WorldGenConfig::game_default`, `scenes::game_scene`, `worldgen::build_world`) is not named in 8.1 but nothing live uses it.
- Choice: delete them in 8.1 with the chain; tests that used them are adapted to the baseline (triage recipe R1/R3) or deleted. The bridge's pure helpers used by kept tests (`init_production_engine`, `bootstrap_spawn_agents`, `seed_finite_resource_scarcity`) stay.
- Evidence: `docs/v8/harvest/test_triage.md` §1.
