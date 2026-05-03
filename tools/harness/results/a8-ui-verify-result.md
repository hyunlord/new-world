---
feature: a8-ui-verify
code_attempt: 1
---

## Files Changed
- `tools/harness/interactive_controller.py`: full rewrite of step dispatcher — **unrecognized steps now raise RuntimeError → scenario FAIL** (was silent-skip PASS); new `_perform_agent_click()` queries live agent snapshots via `get_agents`, avoids any id in `_selection_history` when `must_be_different=True`, logs post-click `get_selected_entity` (id, name, panel_visible, TCI 4-axis + label); new `_compute_cross_scenario_tci_delta()` computes max |axis_A − axis_B| in percentage points across scenarios; emits both `interactive_results.txt` (human-readable) and `interactive_results.json` (machine-readable with `scenarios[*].tci_samples`, `cross_scenario_tci_delta`, `overall_pass`); recognizes `Set zoom to Z<N>`, `Wait N ticks/frames`, `Screenshot: "label"`, `Click on an agent`, `Click a different agent`, `Click empty space`, `Click the personality tab` / `Navigate to personality tab`, `Review all screenshots`.
- `scripts/test/harness_visual_verify.gd`: four new TCP command handlers in `_handle_interactive_command()` — `get_agents` (returns alive agents with world + screen pixel coords using current canvas transform so zoom is applied), `get_selected_entity` (returns HUD `_selected_entity_id`, panel visibility, name, tci_ns/ha/rd/p, temperament_label_key via `SimulationEngine.get_entity_detail`), `click_tab` (sets `_tab_bar.current_tab` on the active `_entity_detail_panel` and re-emits `tab_changed` signal), `get_panel_state` (panel_visible / tab_index / tab_count). No simulation state is modified; all handlers are read-only reflections of existing HUD + SimBridge state.

Summary: zero changes to `sim-core`, `sim-systems`, `sim-engine`, `sim-bridge`, `sim-data`, `rust/crates/sim-test/src/main.rs`, or any scene/RON/locale file. The existing 7 TCI harness tests (commit `4589f324`) remain the Rust-evaluable regression guard; per the feature prompt, no new Rust harness tests are required.

## Observed Values (seed 42, 20 agents)

### Rust-evaluable (runtime measured)
- `harness_bridge_tci_keys_present_on_all_agents`: **PASS** (23 / 23 identities report present TCI keys; 20 requested + 3 settlement bootstrap)
- `harness_bridge_tci_axes_within_unit_interval`: **PASS** (0 NaN/inf/out-of-[0,1])
- `harness_bridge_tci_matches_ecs_values`: **PASS** (0 bridge↔ECS mismatches at ε = 1e-12)
- `harness_bridge_tci_label_consistent_with_axes`: **PASS**
- `harness_bridge_tci_at_least_two_distinct_labels`: **PASS** (3 distinct labels under seed 42)
- `harness_bridge_tci_meaningful_variance`: **PASS** (4 / 4 axes with σ ≥ 0.05)
- `harness_bridge_tci_valid_locale_key`: **PASS** (every returned key is one of `TEMPERAMENT_*` enum strings)
- Run time: 0.30s (7 passed, 0 failed, 0 ignored, 217 filtered out)

### Controller logic (verified with mocked socket)
- Scenario parsing: **PASS** — extracts 4 scenarios from `.harness/prompts/a8-ui-verify.md`, all 6 Scenario 1 step forms recognized (Set zoom to Z2, Wait 200 ticks, Screenshot: "state_before_click", Click on an agent..., Wait 10 frames, Screenshot: "panel_opened").
- Step dispatch: **PASS** — `Set zoom to Z2` → `{"action":"zoom","level":3.0}`; `Wait 200 ticks` → `{"action":"wait_ticks","count":200}`; `Wait 10 frames` → `{"action":"wait_frames","count":10}`; `Screenshot: "state_before_click"` → `{"action":"screenshot","label":"state_before_click"}`; `Click the personality tab` AND `Navigate to personality tab` → `{"action":"click_tab","tab_index":3}`; unknown step → `RuntimeError` → scenario FAIL.
- `_perform_agent_click(must_be_different=True)` with `_selection_history=[7]` and live agents `[7, 12]` → picks agent 12 (avoids 7), records `{target_entity_id:12, selected_entity_id:12, panel_visible:True}`. With only agent 7 available → FAIL with detail `"no valid agent within viewport click region (avoided=[7], total_alive=1)"`.
- `_compute_cross_scenario_tci_delta()` on `(NS 0.30→0.70, HA 0.60→0.30, RD 0.50→0.55, P 0.45→0.40)` → `max_axis_delta_pp = 40.0`, `per_axis_pp = {NS:40.0, HA:30.0, RD:5.0, P:5.0}`, `threshold_met = True`. On identical samples → `max_axis_delta_pp = 0.0`, `threshold_met = False` (regression guard verified).
- JSON evidence format satisfies plan assertions A1 (`overall_pass==true` and every `scenarios[*].result=="PASS"`), A3 (`tci_samples[0].{selected_entity_id≥0, panel_visible==true, name!=""}`), A5 (`steps_log` contains `click_tab personality`), A9 (`scenarios[2].tci_samples[0].selected_entity_id != scenarios[0].tci_samples[0].selected_entity_id`), A10 (`cross_scenario_tci_delta.max_axis_delta_pp ≥ 10.0`).

### Files that remain in place from prior attempt (unchanged in this attempt)
- `scripts/ui/panels/entity_detail_panel_v4.gd` lines 352–371: `_format_personality_tab_text` formats `tci_ns/ha/rd/p` as `int(round(v * 100.0))%` with label via `Locale.ltr("UI_TCI_{NS,HA,RD,P}")` and temperament via `Locale.ltr(temperament_key)`.
- `localization/{en,ko}/ui.json` lines 212–215, 267–271: all `UI_TCI_*` and `TEMPERAMENT_*` keys present in both locales.

## Threshold Compliance

Plan assertion summary (from `.harness/plans/a8-ui-verify/plan_final.md`, which lists the Evaluator-driven hardening targets):

| # | Assertion | Plan threshold | Status in this run | Evidence |
|---|-----------|---------------|---------------------|----------|
| 1 | `overall_pass==true` AND every scenario PASS | all 4 scenarios PASS, overall=true | **CONTROLLER-READY** — silent-skip bug eliminated; regression unit tests confirm unknown step → FAIL | controller diff + mock-socket dispatch test |
| 2 | `state_before_click.png` AND `panel_opened.png` present & > 1 KB | both files > 1 KB | **CONTROLLER-READY** — Scenario 1 step 3 and step 6 now map to `screenshot` action; bytes produced on live run | controller `_parse_screenshot_label` + prompt scenarios |
| 3 | `tci_samples[0]`: `entity_id≥0`, `panel_visible==true`, `name!=""` | all three | **CONTROLLER-READY** — `_perform_agent_click` fails scenario if `entity_id<0` OR `panel_visible==false` | `_perform_agent_click` lines 199–218 |
| 5 | `steps_log` contains `click_tab personality` with `ok=true` | present | **CONTROLLER-READY** — tab handler raises `RuntimeError` if `ok==false` | controller lines 266–276 |
| 9 | Scenario 3 `selected_entity_id` ≠ any prior Scenario 1/2 selection | differs | **CONTROLLER-READY** — `must_be_different=True` via `_selection_history` set; mock test confirms FAIL when forced to reuse | controller lines 206–212 + `_selection_history` |
| 10 | `cross_scenario_tci_delta.max_axis_delta_pp ≥ 10.0` | ≥ 10.0 pp | **CONTROLLER-READY** — `_compute_cross_scenario_tci_delta` emits the field; with seed 42 σ range 0.13–0.28 across 4 axes, 10pp delta on at least one axis between two distinct agents is expected (A-8 Part 2 harness: 4 / 4 axes above σ 0.05) | controller lines 378–453 + existing `harness_bridge_tci_meaningful_variance` |
| 11 | VLM cross-screenshot diff: agent name OR TCI axis ≥10pp OR label differs | differs | **CONTROLLER-READY** — distinct agents with distinct TCI samples are emitted; VLM gate runs after pipeline captures both personality_tab and personality_tab_agent2 PNGs | controller tci_samples + two screenshot labels |
| 13 | No fallback text `"—"`, `"N/A"`, `"없음"` in TCI numeric positions | none | **PANEL CODE UNCHANGED AND ALREADY CONFORMING** — `entity_detail_panel_v4.gd` formats axes as `int(round(v * 100.0))%` from `_safe_float` (defaults only on malformed FFI payload; A-8 harness proves payload is always valid) | panel lines 352–371 + `harness_bridge_tci_keys_present_on_all_agents` |
| — | 7 existing TCI harness tests still PASS | 7 / 7 | **PASS** | `cargo test -p sim-test harness_bridge_tci` — 7 passed, 0 failed, 0 ignored, 0.30s |

All assertions 1/2/3/5/9/10/11 depend on the hardened controller producing live evidence when the interactive pipeline stage runs. The controller now cannot regress to silent skips (unrecognized steps raise), cannot click the same agent twice in Scenario 3 (`must_be_different` is enforced and will FAIL the scenario rather than silently reusing), and cannot emit PASS without a `>=10pp` delta being a live-measured fact (the JSON field is always computed, and the plan's assertion reads the JSON directly — not a controller self-report).

## Gate Result
- cargo test -p sim-test harness_bridge_tci (seed 42, 20 agents): **PASS** — 7 / 7 passed, 0 failed, 0 ignored, 0.30s
- cargo clippy --workspace -- -D warnings: **PASS** — 0 errors, 0 warnings
- cargo test --workspace: **PASS** — exit code 0. sim-test crate: 223 passed, 0 failed, 1 ignored, 259.38s (long-horizon tests: `harness_territory_dispute_detected`, `harness_territory_hardness_scales_with_settlement`, `harness_multi_settlement_emerges` each > 60s). All other crates (sim-bridge, sim-core, sim-data, sim-engine, sim-systems) passed with zero failures in the upstream mechanical-gate step at 12:01:51 (pipeline step 0 `MECHANICAL GATE: PASS`), on the same tree state this attempt targets. Doc-tests: 0 passed, 0 failed, 2 ignored.
- python3 -m py_compile `tools/harness/interactive_controller.py`: **PASS**
- Godot --check-only `scripts/test/harness_visual_verify.gd`: **PASS** (no parse errors reported)

## Notes

- **This is a verification-harness hardening change, not a simulation/panel change.** Per the feature prompt ("Changes Required: None to simulation / panel code") and per the plan's explicit exclusion of new Rust harness tests, no test was added to `rust/crates/sim-test/src/main.rs`. The test-first directive from the system prompt is satisfied by the pre-existing 7 TCI harness tests that serve as regression guards for the data layer; the new failure detection work is in the Python/GDScript interactive harness, where no Rust test can encode it.
- **Why this attempt is different from the prior run.** The prior attempt produced a PASS-labelled `interactive_results.txt` while the controller had silently skipped `Set zoom to Z2 (agents clearly visible)` (Scenario 1) and `Navigate to personality tab` (Scenario 3) and had reused viewport-center coordinates for every agent click. The Evaluator (verdict RE-CODE) explicitly required: (a) unknown steps must FAIL, (b) Scenario 3 must select a genuinely different agent, (c) `max(|axis_A − axis_B|) ≥ 10pp` must be live-measured. All three are now enforced at the controller layer — a broken UI can no longer pass this harness because (a) the controller refuses to report PASS when an expected action wasn't taken, (b) `_perform_agent_click` hard-fails the scenario if no distinct agent is found, and (c) the JSON field `cross_scenario_tci_delta.max_axis_delta_pp` is computed from real `SimBridge.get_entity_detail` return values, not from anything the controller itself can fabricate.
- **No threshold changes.** Every threshold (Scenario 1/2/3/4 PASS, ≥1 KB screenshot bytes, `entity_id≥0`, `panel_visible==true`, name non-empty, `click_tab ok=true`, distinct entity ids, `max_axis_delta_pp ≥ 10.0`, no raw `(UI_|TEMPERAMENT_|TCI_)[A-Z_]+` keys, no `"—"/"N/A"/"없음"` fallback) is honoured exactly as received from `plan_final.md`.
- **Scope discipline.** The controller's `_empty_space_click_coords` was chosen (12% from left, 30% down) to stay well clear of both the right-side HUD sidebar (~30% of viewport width) and the bottom bar; the Scenario 3 "close panel" step depends on this click not landing on an agent or a building. If a future map layout changes this, the hard-fail on same-agent selection in the next click will catch the regression — we do not need to guard the empty-click coordinate against every possible future map.
- **Concern worth recording (not a threshold change):** the pipeline's `sed -n '/^## Interactive Scenarios/,/^## [^I]/p'` extractor will include "## Harness Tests" through "## Verification" if the prompt keeps those level-2 headings starting with letters other than `I`. The current prompt is fine (next section is "## Harness Tests"), but reordering sections could inject `harness_bridge_*` test-name lines into the scenarios file. Out of scope for this attempt; noted for later hardening.
- **Panel code path is UNCHANGED** and already conformant per the prior verification pass (commit `4589f324`). Assertion 13's "no fallback text" is structurally guaranteed by `_safe_float` defaulting to `0.5` only if a field is missing — and the 7 TCI bridge tests prove fields are never missing.
