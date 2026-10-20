# Changelog

All notable changes per pass. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Older history (pre-V7, V7, revitalize passes 1–21, art passes 1–13b, world-model
WM-1–6) is summarised in [docs/HISTORY.md](docs/HISTORY.md); the full record is at
tags `v7-final-sim` and `v7-final-art`.

## V8-0 — 2026-10-20

The clean slate for the V8 settlement sim. The simulation is unchanged
(32 worlds × 120 days: probe summaries, raw samples and chronicles identical
to `v7-final-art`); the only intended behaviour changes are that the god-hand
panel and the V7 overlays are gone from the game and the intervention methods
are gone from the bridge.

### Added
- `WorldRules::baseline(seed)` and `WorldGenConfig::baseline(seed)` (`rust/crates/sim-core/src/rules/baseline.rs`, `rust/crates/sim-systems/src/worldgen/mod.rs`): the live game's rules as one flat literal equal to V7's `game_v26`; `sim_test::baseline_engine(seed)`; probe scene `baseline` (alias `game_v26`).
- Tests: `sim-core/tests/baseline_matches_v7_final.rs` and `sim-systems/tests/baseline_worldgen_matches_v7_final.rs` (the baseline against the V7 dumps, seeds 0, 1, 7, 42); `sim-test/tests/determinism.rs` baseline tick-for-tick test; `tools/v8/test_bridge_api.py`, `tools/v8/test_gd_reach.py`.
- Tooling `tools/v8/`: `gate.sh` (16 steps), `identity.sh` + `compare_summaries.py` (32 worlds × 120 days), `smoke.sh`/`smoke.gd`, `capture.gd`, `perf.sh` + `perf_summary.py`, `bridge_api.py`, `gd_reach.py`, `inventory.py`, `locale_keys.py`, `locale_prune.py`, `system_table.py`, `check_docs.sh`, `worldsim_systems` binary (the live system list).
- Docs: `README.md`, `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`, `docs/ARCHITECTURE.md` (generated system table), ADRs `docs/decisions/0001`–`0011`, `docs/archive/INDEX.md`, this file; `AGENTS.md` rewritten (78 lines), `CLAUDE.md` = `@AGENTS.md` + 8 lines.
- Skills `.claude/skills/`: `worldsim-architecture`, `worldsim-godot`, `worldsim-testing`, `worldsim-pitfalls`, `worldsim-docs-upkeep` (with sourced lesson tables), plus `systematic-debugging` and `verification-before-completion` moved from `.agents/skills/`; `.agents/skills/<name>` are symlinks to them.

### Changed
- The `game_v8 … game_v26` rule chain and its scenes became the one baseline (`rules.rs` → `rules/`; `rules/mod.rs` 1,821 → 819 lines); the perf `quick` world runs under the baseline rules (`quick_stress_baseline_s15100_n500_side64_fed.json`, same fingerprint as V7 `stress --rules game_v26`).
- Seven dead rule fields removed (`capacity_weighted`, two `winter_prep`, `rain_wet_burn`, `reset_on_split`, `avoid_gather`, `exchange_first`; DECISIONS D25).
- Names: pass and version suffixes removed from Rust and GDScript identifiers and files (probe `pass*` modules renamed by content; `draw_v18` → `draw`; `*_p10…_p13b.gd` renamed; D24).
- Structure: `sim-bridge/src/ffi/world_node.rs` (4,364 lines) split into `node.rs`, `engine_build.rs`, `snapshot.rs`, `reads_agent.rs`, `reads_settlement.rs`, `causal.rs`; `sim-systems/.../decision/agent_decision.rs` (4,337) into `system.rs`, `targets.rs`, `needs.rs`, `forage.rs`, `memory_bias.rs`, `wood.rs`; GDScript laid out as `scripts/core`, `scripts/world`, `scripts/ui/hud`, `scripts/ui/panels`.
- Tests: 108 `sim-test/tests/harness_*.rs` files became 68 domain files (`agents_*`, `settlement_*`, `family_*`, `relations_*`, `chronicle_*`, `engine_*`, `determinism.rs`); tests of version deltas and frozen worlds deleted, the rest ported to the baseline.
- Localization source switched from Fluent to the category JSONs (`localization/<lang>/<category>.json`), pruned to the 156 keys the kept UI uses.
- Every `src/**/*.rs` has a `//!` header (systems: priority, interval, reads, writes, events); every `.gd` a purpose + bridge-methods comment block.
- `.claude/settings.json` committed as the neutral `{}` (the V7 harness hooks pointed at deleted scripts).

### Removed
- The god hand: GDScript panel and scene, bridge methods `apply_intervention` and `get_intervention_state`, `sim-engine/src/intervention.rs`, `sim-systems/src/runtime/intervention.rs` and intervention-only fields (ADR 0010).
- V7 overlays and panels outside the kept UI (activity badges and tooltips, village activity summary, settlement relation lines, seek/death/need-bar/activity-trail overlays, restless meter, founding-group renderer, feed ledger, status panel, zoom LOD controller): `scripts/ui` 53 → 8 files (plus 31 moved to `scripts/core`, `scripts/world`, `scripts/ui/hud`); `scripts/test` 222 files (33,947 lines) deleted.
- `docs/`: 17,280 files (2.36 GiB of pass records, evidence and boards) → 27 files (the new docs and the V8-0 records); the index of what was deleted and where it lives is `docs/archive/INDEX.md`.
- `tools/`: 895 → 25 files (V7 harness, revitalize, art pipeline, world-model tooling).
- `sim-test`: 202 → 111 files (world-model recorder, ablation scenes, pass scaffolding, version-delta tests); `sim-bridge` test fixtures of 135 MB (art kernels).
- Assets: 888 → 198 files (15.0 → 12.9 MiB); shaders 11 → 3; scenes 3 → 1; localization 47 → 8 files and 5,287 → 156 keys (the Fluent sources and the key-owner policy).
- Whole tree: 20,299 files / 2,533 MiB → 715 files / 18.5 MiB (at the gate commit). Everything removed is at tag `v7-final-art`.
