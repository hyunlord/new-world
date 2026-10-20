# Removed in V8-0 — index (draft; becomes docs/archive/INDEX.md)

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
