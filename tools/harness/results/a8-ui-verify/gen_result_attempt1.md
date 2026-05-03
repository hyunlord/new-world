---
feature: a8-ui-verify
code_attempt: 1
---

## Files Changed

No new code written in this attempt. The hardening that plan_final.md demands
was already applied in prior commits on `lead/main` and still matches the plan's
locked thresholds. Verified files on the current tree:

- `tools/harness/interactive_controller.py` (committed `035313d1`, extended
  `38744ec3`): dispatcher raises `RuntimeError` on any unrecognised step (no
  silent skip); `_perform_agent_click()` queries live `get_agents`, enforces
  `must_be_different` via `_selection_history`, retries up to 4x, rejects
  building-adjacent candidates, logs post-click `get_selected_entity` (id,
  name, panel_visible, TCI 4-axis, temperament_label_key);
  `_compute_cross_scenario_tci_delta()` emits live-measured
  `max_axis_delta_pp`; writes both `interactive_results.txt` and
  `interactive_results.json`; recognises every step listed under
  "## Interactive Scenarios" of the feature prompt.
- `scripts/test/harness_visual_verify.gd` (committed `a2218695`, extended
  `035313d1`): `_handle_interactive_command()` handles `screenshot`, `click`
  (synchronous `_handle_click` invocation to avoid tick-queue race), `zoom`,
  `wait_frames`, `wait_ticks`, `get_state`, `get_agents` (decoder-synced
  pixel coords matching the renderer's own transform), `get_selected_entity`
  (reports `entity_id`, `selected_building_id`, `selected_settlement_id`,
  `panel_visible`, TCI 4-axis + label), `get_buildings` (footprints +
  settlement centers for click-steal avoidance), `click_tab`,
  `get_panel_state`, `quit`. No simulation state is mutated.
- `tools/harness/test_interactive_controller.py`: 13 stdlib-only unit tests
  covering the three original defects plus the building-adjacency filter.

Zero changes to `sim-core`, `sim-systems`, `sim-engine`, `sim-bridge`,
`sim-data`, `rust/crates/sim-test/src/main.rs`, or any scene/RON/locale file.
No new Rust harness test added — the feature prompt explicitly states
"Already existing from A-8 Part 2 (no new tests required)" and plan
Assertion 10 locks the 7 existing TCI tests as the regression guard.

## Observed Values (seed 42, 20 agents)

### Rust-evaluable (Assertion 10 — regression guard)
`cargo test -p sim-test harness_bridge_tci` at seed 42 with 20 bootstrap
agents + 3 settlement adults = 23 identities:

- `harness_bridge_tci_keys_present_on_all_agents`: **PASS** — 23 identities,
  0 missing temperament, 23 helper_ok.
- `harness_bridge_tci_axes_within_unit_interval`: **PASS** — 0 violations.
- `harness_bridge_tci_at_least_two_distinct_labels`: **PASS** — 3 distinct
  labels observed: `{TEMPERAMENT_PHLEGMATIC, TEMPERAMENT_CHOLERIC,
  TEMPERAMENT_SANGUINE}`.
- `harness_bridge_tci_meaningful_variance`: **PASS** — σ values: ns=0.1705,
  ha=0.2210, rd=0.1292, p=0.2818; 4 / 4 axes with σ ≥ 0.05.
- `harness_bridge_tci_label_consistent_with_axes`: **PASS**.
- `harness_bridge_tci_matches_ecs_values`: **PASS** — 0 ECS↔bridge mismatches.
- `harness_bridge_tci_valid_locale_key`: **PASS** — every key matches the
  `TEMPERAMENT_*` enum.
- Run time: **0.30s** (7 passed, 0 failed, 0 ignored, 217 filtered out).

### Controller-logic evaluable (regression guard for Assertions 1, 2, 3, 9)
`python3 tools/harness/test_interactive_controller.py`:

- `test_defect1_unrecognized_step_raises`: **PASS** — unknown step raises
  `RuntimeError` ("unrecognized step: ..."), scenario would FAIL.
- `test_defect2_selects_nearest_to_center`: **PASS** — picks agent closest
  to viewport center, not fallback.
- `test_defect2_skips_offscreen_agents`: **PASS** — agents outside the
  `x ∈ [40, vp*0.70]`, `y ∈ [40, vp-80]` window are rejected.
- `test_defect3_avoid_ids_excludes_previous`: **PASS** — `avoid_ids`
  excludes previously-clicked agents.
- `test_defect3_no_valid_agent_returns_none`: **PASS** — empty candidate
  pool → `None` → caller fails the scenario.
- `test_negative_id_skipped`: **PASS**.
- `test_empty_agents_returns_none`: **PASS**.
- `test_near_building_detects_3x3_footprint`: **PASS**.
- `test_near_building_empty_list_false`: **PASS**.
- `test_near_building_multi_tile_footprint`: **PASS**.
- `test_choose_agent_skips_building_adjacent`: **PASS** — agent standing on
  a building tile is rejected (the 3×3 tile search in
  `entity_renderer._handle_click` would steal the selection).
- `test_choose_agent_loose_also_skips_building_adjacent`: **PASS**.
- `test_empty_space_avoids_building_and_agents`: **PASS** — "close the
  panel" click coordinate is clear of buildings and agents.
- Totals: **13 / 13 passed**.

## Threshold Compliance

| # | Assertion                                                           | Plan threshold                                        | Observed                                 | Status     |
|---|---------------------------------------------------------------------|-------------------------------------------------------|------------------------------------------|------------|
| 1 | Scenario 1 selects a real agent                                     | `entity_id ≥ 0` AND in snapshot AND sprite pixel      | enforced by `_perform_agent_click`       | CONTROLLER-READY |
| 2 | Scenario 3 selects a different agent                                | `id_s1 ≠ id_s3`, both ≥ 0, both in snapshot           | `must_be_different=True` + retry         | CONTROLLER-READY |
| 3 | TCI pairwise delta ≥ 10 pp on some axis                             | `max_axis_delta_pp ≥ 10.0`                            | computed live from `SimBridge.get_entity_detail`; seed-42 σ ≥ 0.1292 / axis | CONTROLLER-READY |
| 4 | VLM confirms 4 TCI axes present                                     | 4 CONFIRM, each value ∈ [0, 100]                      | requires VLM run on personality_tab PNG  | VLM-STAGE  |
| 5 | VLM confirms temperament label is localized                         | no regex match on `^(UI_|TEMPERAMENT_|TCI_|PERSONALITY_|AGE_)[A-Z0-9_]+$` | requires VLM run           | VLM-STAGE  |
| 6 | No raw locale keys in any scenario screenshot                       | 0 matches                                             | requires VLM run                         | VLM-STAGE  |
| 7 | Agent names differ between the two selected agents                  | `name_s1 ≠ name_s3`, neither empty or raw key         | requires VLM run on two panel PNGs       | VLM-STAGE  |
| 8 | Zero script / ERR_ / ERROR lines in `console_log.txt`               | count = 0                                             | requires interactive run                 | RUN-STAGE  |
| 9 | Every listed step is recognised                                     | `unrecognized == 0`                                   | enforced by `execute_step` default raise | CONTROLLER-READY |
| 10| 7 A-8 Part 2 TCI harness tests pass                                 | 7 / 7 PASS, 0 failures, 0 ignored                     | **7 / 7 PASS, 0.30s**                    | **PASS**   |
| 11| VLM verdict aggregation = VISUAL_OK                                 | VISUAL_OK                                             | requires VLM run                         | VLM-STAGE  |

Assertions marked **CONTROLLER-READY** cannot regress silently: an unknown
step raises, a missing distinct agent raises, and `max_axis_delta_pp` is
computed from `SimBridge.get_entity_detail` (not anything the controller can
fabricate). Assertions marked **VLM-STAGE** are measured downstream by
`run_vlm_interactive` against the evidence PNGs captured by this controller;
they cannot be evaluated from this attempt's scope but the inputs they
require are produced correctly.

## Gate Result

- `cargo test -p sim-test harness_bridge_tci` (Assertion 10): **PASS** —
  7 / 7 passed, 0 failed, 0 ignored, finished in 0.30s.
- `cargo clippy --workspace -- -D warnings`: **PASS** — exit code 0, zero
  warnings, profile `dev [optimized + debuginfo]`.
- `python3 tools/harness/test_interactive_controller.py`: **PASS** —
  13 / 13 passed.
- `python3 -m py_compile tools/harness/interactive_controller.py`: **PASS**.
- `harness`: **CONTROLLER-READY** — Rust / Python gates green; VLM-stage
  assertions (4, 5, 6, 7, 8, 11) are measured downstream by the pipeline's
  `run_vlm_interactive` + `validate_interactive_evidence` steps using
  evidence produced by this controller.

## Notes

- **Why no new Rust test.** The feature prompt is explicit: "Already
  existing from A-8 Part 2 (no new tests required)". The plan encodes this
  as Assertion 10 — the 7 TCI tests as a regression guard. The TDD RED-GREEN
  discipline applies to Rust simulation features; this feature is a
  Python / GDScript harness-controller hardening whose assertions live in the
  interactive-scenario layer. Adding a Rust test here would be test theatre —
  there is no simulation state to assert on.
- **Why this attempt changes no files.** The generator's brief is "implement
  the feature per the plan". The plan's implementation target (controller
  hardening) was already delivered by commits `035313d1` (three defect fixes +
  unit tests) and `38744ec3` (building-adjacency filter + pipeline gate
  wiring). The current controller and GDScript TCP server already satisfy
  the plan's locked thresholds for every assertion in scope. Re-implementing
  them would violate scope discipline ("Don't improve adjacent code. Don't
  refactor things that aren't broken.").
- **Thresholds locked and honoured.** The following plan thresholds are
  honoured exactly as received — none were adjusted in this attempt: `id ≥ 0`,
  `id_s1 ≠ id_s3`, `max_axis_delta_pp ≥ 10.0`, 4 CONFIRMs on 4 axes,
  `(UI_|TEMPERAMENT_|TCI_|PERSONALITY_|AGE_|SEX_)[A-Z0-9_]+` regex = 0
  matches, `name_s1 ≠ name_s2`, ERROR-line count = 0, unrecognised = 0,
  7 / 7 TCI tests pass, VLM verdict VISUAL_OK.
- **One observation worth recording (not a threshold change).** Assertion 3
  is plan-type B (measurable threshold), and it depends on the controller
  finding *any* pair of distinct agents with Δ ≥ 10pp on at least one axis.
  The observed seed-42 population has σ of 0.1705 / 0.2210 / 0.1292 / 0.2818
  across NS / HA / RD / P — every axis comfortably above the 10pp threshold
  in expectation. The plan's rationale already anticipated this
  ("`harness_bridge_tci_meaningful_variance` establishes the population
  spans more than this"). No concern; flagged only to document why the
  controller can satisfy a type-B threshold without randomness in selection.
- **Scope discipline.** No changes to `sim-core`, `sim-systems`, `sim-engine`,
  `sim-bridge`, `sim-data`, `rust/crates/sim-test/src/main.rs`, any scene
  (`.tscn`), any RON, any locale file, or `scripts/ui/panels/*`. The feature
  prompt pins the boundary: "Changes Required: None to simulation / panel
  code." This attempt respects that boundary exactly.
- **No `unwrap()` introduced.** No Rust code was added in this attempt, so
  the production-code `unwrap()` prohibition is vacuously satisfied. (The
  existing Rust code path — SimBridge TCI helpers — already uses
  `unwrap_or`/`match` per commit `4589f324` and is covered by the 7 passing
  tests.)
- **No hardcoded UI strings introduced.** No GDScript UI code was touched.
  The `harness_visual_verify.gd` file is `scripts/test/`, a test harness
  script — it is not user-facing and is correctly exempt from the
  `Locale.ltr` rule per project convention (debug/log strings are exempt,
  and this file only produces harness logs and evidence files).
