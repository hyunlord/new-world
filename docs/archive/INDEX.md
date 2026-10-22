# Archive index — what V8-0 removed and where it lives

Permanent. This is the only trace of the deleted docs in the V8 tree (goal 7.1).
Later passes that delete something add a section below with their own tag.

## V8-0 (2026-10-20)

Every path below was deleted from the V8 line by V8-0 and is available at tag
`v7-final-art` (= `b1ca032f`, the art line, which contains the whole
simulation line). Paths that also exist at `v7-final-sim` (= `354e4a13`) say so.
Read a file with `git show v7-final-art:<path>`; list a directory with
`git ls-tree -r --name-only v7-final-art <dir>`.

| path | what it was | files / size at the tag | available at |
|---|---|---|---|
| `docs/revitalize/` | Revitalize passes 3–21: goals, reports, reviews, DECISIONS, METRICS, PERF_BUDGET, probe/perf evidence (`probe/` is ~1.5 GB) | 3,713 files, 1.5 GB | `v7-final-art`, `v7-final-sim` |
| `docs/art/` | Art production passes 1–13b: plans, reports, decisions, reviews, boards, captures | 13,501 files, 243 MB | `v7-final-art` |
| `docs/worldmodel/` | World-model track datasets/benchmarks (WM-1…) | 41 files, 3.4 MB | `v7-final-art`; the track continues on `codex/worldmodel-2026-10` |
| `docs/analysis/`, `docs/design/`, `docs/guides/`, `docs/plans/`, `docs/reference/`, `docs/traits/`, `docs/localization/`, `docs/architecture/` | pre-V7 / V7 design notes (band analysis, emotion/personality/stress design, controls, temperament UI plan, research references, trait effect tables, i18n rules, V7 architecture) | 21 files | `v7-final-art`, `v7-final-sim` |
| `docs/CHANGELOG.md`, `docs/llm-setup.md`, `docs/sprite_session_handoff_wildlife_a4.md` | pre-V7 changelog (summarised in docs/HISTORY.md), LLM setup notes, a sprite-session handoff | 3 files | `v7-final-art`, `v7-final-sim` |
| `stats/` | pre-V7 stat-definition JSON (body/skills…) | 198 files | `v7-final-art`, `v7-final-sim` |
| `.harness/` | V7 harness pipeline state: prompts, plans, audits, test baselines | 193 files | `v7-final-art`, `v7-final-sim` |
| `hooks/` | V7 git pre-commit harness hook | 1 file | `v7-final-art`, `v7-final-sim` |
| `addons/harness/` | harness MCP Godot addon (never enabled in `project.godot`) | 6 files | `v7-final-art`, `v7-final-sim` |
| `.claude/agents/` | V7 harness pipeline agent prompts (drafter, challenger, evaluator, …) | 8 files | `v7-final-art`, `v7-final-sim` |
| `vlm_interactive_test_results.zip` | V7 VLM test artefact | 1 file | `v7-final-art`, `v7-final-sim` |
| `tools/revitalize/` | per-pass identity/gate/perf/capture scripts (Pass 3–21) | 169 files | `v7-final-art`, `v7-final-sim` |
| `tools/art_pipeline/` | art generation/packing/board pipeline | 579 files | `v7-final-art` |
| `tools/worldmodel/` | world-model research tooling | 12 files | `v7-final-art`; `codex/worldmodel-2026-10` |
| `tools/harness/` | V7 harness pipeline (`harness_pipeline.sh`, hooks, results) | 116 files | `v7-final-art`, `v7-final-sim` |
| `tools/scripts/`, `tools/art/`, one-off `tools/*.py` / `*.sh` migration scripts | pre-V7 migrations (i18n, traits, rust shadow cut-over, sprite generation) | ~17 files | `v7-final-art`, `v7-final-sim` |
| `scripts/test/`, `scenes/test/` | Godot test/capture code of the V7 and art lines (capture framings, art probes, frame-time sessions) | 223 files | `v7-final-art` |
| `scripts/ui/**` outside the keep set, `scripts/rendering/`, `scenes/ui/god_hand_panel.tscn` | V7 overlays: god-hand panel/effects, activity trails, need bars, seek/death viz, status panel, settlement overview/labels/relations, village activity summary, tooltip, restless meter, zoom LOD, nature/world effects | see `docs/v8/V8_0_INVENTORY.md` | `v7-final-art` |
| god hand (Rust) | `sim-engine/src/intervention.rs`, `sim-systems/src/runtime/intervention.rs`, bridge `apply_intervention` / `get_intervention_state` and helpers, intervention-only resource fields | 2 files + edits | `v7-final-art`, `v7-final-sim` |
| `game_v9 … game_v25` rule/world/scene constructors, `legacy` and ablation scenes | the per-pass rule chain (replaced by one `baseline`) | edits | `v7-final-art`, `v7-final-sim` |
| `rust/crates/sim-test/src/worldmodel/`, `src/bin/worldsim_wm.rs`, `src/probe/ablation.rs` | world-model recorder, ablation scenes | 9 files | `v7-final-art`; `codex/worldmodel-2026-10` |
| `rust/crates/sim-test/tests/*` (deleted ones) | per-pass harness tests (frozen-world pins, old scenes, UI text assertions) | see inventory | `v7-final-art` |
| `rust/crates/sim-bridge/tests/fixtures/` | art-kernel bit-equality fixtures (57 MB + 83 MB) | 6 files | `v7-final-art` |
| `localization/fluent/`, `key_registry.json`, `key_owners.json`, unused keys | Fluent sources and key bookkeeping; ~5,000 keys the V8-0 UI never shows | 4 files | `v7-final-art`, `v7-final-sim` |
| `assets/art/production/`, `assets/art/direction/`, `assets/audio/`, `assets/tilesets/`, unused `assets/sprites/*` | atlas source PNGs, direction references, ambience loops, V7 tileset, pre-art-line sprites | ~690 files | `v7-final-art` |
| unused `shaders/*.gdshader` | V7 overlay shaders | 8 files | `v7-final-art`, `v7-final-sim` |
| `rust/**/CLAUDE.md`, `scripts/**/CLAUDE.md` | V7 per-directory agent notes (superseded by docs/ARCHITECTURE.md and skills) | — | `v7-final-art`, `v7-final-sim` |

## M1 (2026-10-20)

Deleted by M1 (goal 9.7). Every path is available at `24e16677` (`v8/main` before M1, V8-0) and at
`v7-final-art`. Read a file with `git show 24e16677:<path>`.

| path | what it was | files | available at |
|---|---|---|---|
| `scripts/world/{world_renderer,terrain_renderer,nature_renderer,nature_canopy,nature_layout,building_renderer,agent_renderer,agent_activity,agent_frame_choices,agent_pose_moments,agent_selection_index,activity_icons,art_season_tint,camp_seats,carry_load_layer,cold_effects,declump_offsets,morning_chill,offscreen_marker,shelter_cold,walker_offsets}.gd` | the V7 world renderers and their helpers (replaced by `scripts/world/colony/`) | 21 | `24e16677`, `v7-final-art` |
| `scripts/world/cold_audit.json`, `shaders/settlement_outline.gdshader` | data and shader used only by those renderers | 2 | `24e16677`, `v7-final-art` |
| `rust/crates/sim-bridge/src/ffi/art_kernels.rs` | the bridge methods `art_declump`, `art_walker_offsets` (render kernels of the V7 agent renderer) | 1 | `24e16677`, `v7-final-art` |

## Living art branches (2026-10-22)

Not deleted material: the art track's work lives on side branches that are never merged into the V8 line
(ADR 0018). Read a file with `git show origin/<branch>:<path>`.

| branch | head | what it holds | status |
|---|---|---|---|
| `v8/art-trial` | `c3dafb07` | ART-TRIAL: generated 32-px people and environment for the M1 scene; `docs/v8/arttrial/` (goal, decisions, decision brief, blind results, cost, report), the trial's asset sheets and boards; raw originals in `/Users/rexxa/work/arttrial-data/` | done, failed (3/10, 10/24) |
| `v8/art-concept` | `a5406866` | ART-CONCEPT: eight visual languages × eight subjects; `art/concepts/<slug>/` (concept, generation log), `art/concepts/boards/` (12 boards), `docs/v8/artconcept/` (goal, decisions, checks, blind review, scores, book, cost, report); raw originals in `/Users/rexxa/work/artconcept-data/` | done; owner chose 05 |
| `v8/art-trial2` | `034c38ab` (ART-TRIAL2), `2433a612` (ART-TRIAL2b) | ART-TRIAL2: concept 05 at production scale (anchors and judge, environment families, autotiles, seasons, cut-out rigs, portraits, cost); `art/style05/**`, `docs/v8/arttrial2/` (report `TRIAL2_REPORT.md`); raw data in `/Users/rexxa/work/arttrial2-data/`. ART-TRIAL2b repeats it with corrected judge, camera and ground rules (report `TRIAL2B_REPORT.md`; raw data in `/Users/rexxa/work/arttrial2-data/trial2b/`) | ART-TRIAL2 done, sustainability FAIL; ART-TRIAL2b done: 02 fallback withdrawn, environment 35/38, people C5/C7 FAIL |
| `v8/art-trial3` | `352f4101` (ART-TRIAL3), `b2a06115` (ART-TRIAL3b) | ART-TRIAL3: character template, layer contract and structural checker; route A generated 2D layers, route B (3D) preflight; `art/character/**` (template, contract, guides, boards, GIFs, `PRODUCTION.md`), `art/pipeline3/`, `art/proto/` character lab, `docs/v8/arttrial3/` (report `TRIAL3_REPORT.md`, `VERDICT.json`; ART-TRIAL3b `TRIAL3B_REPORT.md`); raw data in `/Users/rexxa/work/arttrial3-data/` | ART-TRIAL3 done, PARTIAL (87/87 layers, 603 combinations; route B not run); ART-TRIAL3b stopped at its tool check (no workspace) |
| `v8/art-trial4` | `9f9c6d6b` | ART-TRIAL4: code-drawn pawns (`route_p`: head, capsule body, hair, top and item layers from a data spec, 4× raster + ink pass), template v2, PawnLab; `docs/v8/arttrial4/` (report `TRIAL4_REPORT.md`, `TRIAL4_DECISIONS.md`, `EVIDENCE_INDEX.md`); raw data in `/Users/rexxa/work/arttrial4-data/` | done, not adopted (S1, D1, P1 PASS; R1 readability and S2 revision limit FAIL); owner rejected the look and parked people art (2026-10-22); `route_p` is the placeholder |

