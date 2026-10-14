# Part 6 measurement preparation

Status: final capture measurement completed. A1–A33:22PASS/11FAIL; A18 society FAIL; A18b OBSERVED. Full original verdict is losslessly preserved in measurements.json.gz. Earlier preparation notes below remain historical evidence.

## Commands

```sh
python3 tools/art_pipeline/measure_p8.py docs/art/production8/qa_config.json
python3 tools/art_pipeline/artboards_p8.py docs/art/production8/boards_config.json
python3 -m pytest tools/art_pipeline/test_measurement_p8.py -q
```

The first command writes `ingame_qa.md/.json`, `measurements.md/.json` under production8 only. The second writes exactly the seven required PNG boards plus `artboard_sources.json`. Both take explicit configuration so measured v19 dates replace fixed v17 dates without inventing evidence. Existing Pillow/numpy dependencies are reused; no dependencies installed.

## Configuration and inherited checks

`qa_config.json` keys:

- `raw_art`, `raw_long`, `raw_society`, `raw_god`: raw directories with original PNGs, sidecars and `manifest.json`.
- `gate`: current gate directory with `results.tsv`.
- `expected_long_stems`: exact measured capture plan stems, including no extensions.
- `winter_noon_stem`: measured winter midday frame for the unchanged A5 edge test.
- `society_seed`: selected Part0 seed.
- `morning_stems`: measured cold-morning stems; `warm_control_stem`: measured valid warm-control stem or null.
- `departure_feed_stem`: actual departure+3h society feed frame stem.
- `evidence`: normalized JSON described below.

A1–A18 call existing pure Pass7 checks. The obsolete fixed `LONG_CAPTURE_SET` row and fixed winter-noon A5 are replaced with measured equivalents at the same thresholds. ART remains unchanged. A16 still expects the inherited day10/day18/day24/day32/day40 seasonal anchor frames; do not remove those merely because additional cold/night dates change. A19 calls the existing pure cold check on natural frames. A18 society runs independently; it cannot rescue natural A18. Every missing A1–A33 row is explicitly N/A.

A20 now calls `harvest_integrity_p8.audit_all`: actual installed sprite/mask hashes and frozen approval bindings, writing only production8/harvest. A21–A25 call `ingame_qa_observed_p8` with current production8/visual_observations.json and measured morning/control stems. All original numeric formulas remain unchanged. A26–A29 call `ingame_qa_society_p8` with the measured society seed, reading current production8 blind/visual observations. These are implemented checks, not externally supplied verdicts.

- A26 reuses `ingame_qa_deaths_p7.check_death_frame`: raw death reasons/feed/markers.
- A27 reuses `ingame_qa_villages_p7.check_village_frame`: stable numbering/colours/discs, plus current SHA-bound blind review.
- A28 uses `ingame_qa_founding_p8.check_founding_frame` with the exact inherited Pass7 thresholds, including the actual site being on screen. An off-screen site still fails A28 even when the new arrow succeeds. A32 separately verifies the arrow solution, angle and distance. Both results remain visible; no product change or earlier verdict is reverted.
- A29 uses a production8 copy of the targeting check with dynamic society seed, restricted to actual rain-cast frames so new paired-food worlds do not cause unrelated rain failures.
- A21–A25 use a production8 copy of the observer cross-check with current cold-morning and warm-control stems. No old production7 observation file is consumed.
- Missing actual independent observations produce FAIL, not an implementation N/A or metadata-only PASS.

A1–A25 primary scope is natural live seed20261010; effects are reported separately as `A24_INTERVENTION` with intervention provenance. No society/god frame can silently improve natural numbers. All prior A30 results remain untouched.

## Normalized evidence JSON

This adapter input must be generated from actual capture sidecars; it is not a replacement for them. Preserve source paths/hashes in the generation record.

### `performance`

Fields: `run_id`, `seed`, `alive_count`, `first_three_alive_tick`, `framing:"FV"`, `simulation_frozen:true`, `vsync_disabled:true`, `vsync_restored:true`, `process_priorities:[-1000,1000]`, `uptime`, `processes`, `blocks`.

Exactly12 blocks, alternating `layers_enabled`, each with identical `tick` and `camera:[x,y]`, and50 counted `samples`. Each sample has `process_ms`, `render_cpu_ms`, `setup_cpu_ms`, `gpu_ms`; all finite and nonnegative. GPU zero means unavailable in the report, not measured zero cost. Preserve timestamp latency/discarded transition samples beside counted blocks in the raw capture file. Require300 counted samples per state. Primary median = process + render CPU; setup CPU remains separately reported and its subtotal is diagnostic. PASS uses <=1.15x off **or** <=off+1ms. `old_pacing_ms` records historical wall-clock values without calling them CPU time.

### `paired_worlds`

`P` and `G` each have `pressure`, `threshold` at the morning after the third cast, and `departure_tick` (null if none through day120). Booleans `same_seed_and_cast_days` and `both_reached_day120` must be derived from captured reset/cast/snapshot provenance. Meter comparison uses P/theta; tie fails. Only P departing passes the departure component; only G or neither departing fails. Keep raw tick/day conversion and all three cast days/food/scarcity lines in the source artifact.

### `way`

The runner derives rows directly with `way_adapter_p8.py` from `society_presentation.founding_groups` and raw FFI founder/agent arrays. Expected distance uses raw leader position and raw founding-site tile. All target/edge/centre comparisons use logical screen pixels; `edge_framebuffer_position`, `site_screen` and `leader_screen` are deliberately not mixed into this geometry. Missing raw-group presentation, duplicates, out-of-viewport arrows and incorrect on-screen flags fail.

Rows per walking group/frame: `frame`, `walking`, `site_on_screen`, `stake_visible`, `arrows` (list of drawn screen xy positions), `screen_center`, `target_screen`, `distance_tiles`, `expected_distance_tiles`. Expected distance is independently derived from leader/site tiles. Geometry requires exactlyone arrow off-screen, no stake, angle<=10degrees and distance error<=1tile. On-screen requires stake and zero arrows. A32 geometry is separate from human recognition.

`blind_way`: rows `{source,sha256,expected,answers:[observer1,observer2]}`. Directions use the same eight-direction alphabet in frozen answers/results. Freeze source/answer hash before fresh observers run; both answers must match. Feed only images and the neutral question to blind observers, never this document or the answer file.

### `news`

The runner derives this from actual `feed_audit` plus independent death/fission/cast histories using `feed_adapter_p8.py`; normalized rows need not be manually supplied. An append-only event-ID cursor includes same-tick casts. `event_oracle_p8.py` consumes the independently sampled `event_truth_trace` for births/construction/formation/abandonment/fission/casts; missing or incomplete census fails A31. Stable source IDs match without predicting notification sequence IDs.

Rows: `frame`, `previous_tick`, `tick`, `expected_event_ids`, `events`, `visible_event_ids`, `overflow_event_ids`, `more_count`, `summaries`, `movements`, `departure_feed_frame`, `departure_event_id`.

- `events`: `{event_id,tick,key,event_class,visible}`. IDs stable across repeated snapshots.
- Expected events derive from raw event histories/snapshot deltas, not the emitted feed itself.
- `movements`: `{tick,village_id,delta,excluded}`. Mark actual god food and fission transfers excluded; retain them in audit. Sum every ordinary delta, not just >=3 routine notifications.
- `summaries`: `{village_id,start_tick,end_tick,stored,eaten,net}`. Intervals are `(start,end]`, 20:00-to20:00 boundaries from time snapshots.
- Overflow identities must represent actual omitted protected events; `more_count` equals unique overflow IDs, disjoint from visible IDs.
- Require a departure+3h feed frame with actual departure ID visible; require summary evidence. No routine visibility, no duplicate identities, every expected event exactlyonce, and every protected event since previous frame visible or counted.

## Boards

`boards_config.json` contains exactly `news`, `way`, `restless`, `crowds`, `effects`, `before-after-live`, `before-after-society`. Each value is `{caption,paths:[absolute PNG paths...]}`. No placeholder image is generated for missing captures. Every source is pasted at1x with captions outside original pixels, and recorded by SHA256 plus board rectangle.

Before/after sources:

- Live baseline: `/Users/rexxa/work/art-production8-evidence/baseline-live/shots/final`.
- Society baseline: `/Users/rexxa/work/art-production8-evidence/baseline-society/shots/art_society`.
- Final comparison must use exactly the same seed/time/framing as baseline.
- `crowds` may use historical Pass7 raw frames for the explicitly labelled historical comparison; do not pretend they are the same v19 simulation state.

## Verification performed

Thirteen regression fixtures passed: tied departures fail, only-P departure passes, old pacing-only A30 fails, additive1ms A30 branch passes, backwards arrow fails, simultaneous on-screen stake/arrow fails, gift-contaminated summary fails, board pixels and source hash remain exact, proven initial tick0 root formation is accepted, later unobserved formation fails, logical geometry ignores framebuffer decoys, missing walking-group marker fails, inherited A28 rejects an off-screen site despite an arrow while A32 permits valid arrow geometry. Python compile passed. Programming no-excuse checker: no violations in eleven files at initial preparation; four changed adapter files rechecked after integration. `ruff` and `basedpyright` are not installed; no new dependency was added. Final real-data CLI and image inspection remain pending final captures.

Interim A20 execution at HEAD `792696a00100c29de9951f1de2453a191b4a0080`: PASS,74 bound frames,61 changed villager frames,20 animationgroups; atlas SHA256 `72a01917bc9cdd151da3a2e8074bfa2e1963627aadd693680f5a1eb143e8c274`. Rerun at final capture HEAD.

A18b input `blind_crowds`: exactly12 `{source,sha256,answers:[answer1,answer2]}` rows with yes/roughly/no. Output is OBSERVED with all answers, never a replacement PASS for A18.

Part1 runtime audit (`measurements/p1-news-audit.json`): A31 PASS for one actual departure+3h frame only. 904 feed records,28 independent source events,1462 movement rows,32 summaries,3 visible notifications,8 overflow identities. Exact1tick source census0..46140. Screenshot and sidecar SHA-bound. This is not the full Part5 capture-set verdict.

Final A11 identity guard is enforced by `blind_identity_p8.py`; absent/stale final4x/2x lineup hashes force A11FAIL. P8 effect observations use `observation_rules_p8.py`: dust visible until1.3s (1s hold +.3s fade), drop ends.8s; no other thresholds changed. Combined P8/P7 measurement regression suite:26testsPASS.

## Final execution

Actual169captures,120natural QA frames,17society frames,28intervention frames. First parser attempt failed on a preparation TSV header (retained review/measurement-attempt1.md); corrected to the unchanged gate.sh headerless contract. FinalCLI exit0, no threshold changes. Thirty focused tests PASS; cached uvx Ruff and basedpyright used without installing project dependencies. Whole P8 tool check initially0errors/1303warnings; finalchecks are recorded in gates/p6-tools. Compressed JSON reader instructions: EVIDENCE_FORMAT.md.
