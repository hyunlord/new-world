# Future activity capture scaffolding

Prepared only; no Godot runtime or capture was performed. Use after Part 1 acceptance, Part 2 renderer integration, merged game_v22 debug bridge build/import and normal sharing-protocol preflight.

```sh
P10_DEADLINE_EPOCH=ORIGINAL_TASK_DEADLINE_EPOCH \
python3 tools/art_pipeline/run_activity_captures_p10.py \
  /Users/rexxa/work/new-world-art-prod \
  /Users/rexxa/work/art-production10-evidence/activities-NEW \
  ACTUAL_PROBE_WINNER_SEED \
  /absolute/path/to/actual-probe-selection-evidence.json
```

The winner argument has no default and must belong to the mandated 20261011–20261027 cohort. The nonempty selection-evidence file is copied and hash-bound; the runner does not decide the winner or validate a currently unknown probe-result format. The leader must supply the actual maximum day60 population winner from the bounded four-worker release probe, including any cap/fewer-seed limitation. This is an explicit outstanding integration requirement, not proof supplied by the runner.

The output parent must exist and the output directory must be new. Existing directories are refused atomically; failed runs remain intact. GODOT_BIN has the prescribed Mac default; P10_CAPTURE_LIMIT defaults to1800 and accepts1..3600 seconds. Six independent windowed launches capture day30 10:00/15:00 on seed20261010 and day60 10:00 on the supplied winner, each at Z4/Z1. No stress reset exists. Quiet-lock absence is checked before each launch and again inside with_window_lock/run_bounded. It never takes a quiet hold. Quiet waits are logged at five-minute intervals and truncated at the mandatory future P10_DEADLINE_EPOCH (original task deadline, never a newly extended deadline). An outer process-group timeout also bounds window-lock waiting, the inner quiet recheck and Godot to the lesser of remaining task time and P10_CAPTURE_LIMIT. Timeout sends SIGTERM only to the owned launch group, waits up to10 seconds, then escalates to SIGKILL and records exit124. The window wrapper's lock acquisition is the existing shared protocol; no foreign lock/process is removed. As with the existing identity runner, quiet absence is an observation, not a reservation against a later competing hold.

State changes occur at absolute process frame10; screenshot and metadata at frame130 under actual fixed1/60 deltas. Clock ticks derive from live world-info calendar fields. The camera anchors to the lowest-ID settlement; this intentionally does not guarantee every activity is on screen. Sidecars enumerate EVERY raw activity component, including travelling holders, and join entity bits to agent_id. A missing sprite/live mapping is recorded and fails the capture; it is not silently dropped. Every found sprite reports visibility, viewport intersection/full enclosure, transformed rect, texture/region, flip and all actual metadata. Offscreen/hidden holders remain in the sidecar. Image and JSON hashes, HEAD, working-tree patch/status, source digest, capture-script hash, loaded debug library hash and bridge-source hash are recorded. Source drift during the matrix fails the run. Build-to-source correspondence still requires the leader's build/gate evidence.

## Required future renderer contract

For every sprite corresponding to an activity holder, Part2 must set:

- `p10_activity_schema = 1` only when all fields are implemented.
- `activity_kind`, `activity_place_known` matching raw kind and until_tick>=0.
- Existing `activity`, `pose`, `body`, `facing`, `sprite_id`, `cold_pose`, `cold_base_pose`, `entity_bits`, `art_id`.
- `activity_frame_view` and `activity_frame_activity`: actual chosen atlas frame fields, not requested facing/pose. W mirroring is recorded separately via flip_h.
- `activity_overrides`: ordered list of actually applied pickup/meal/cold/morning-chill overrides, empty if none. Do not infer this afterward from the final pose.

The script fails closed on absent contract keys, wrong schema or mismatched kind/arrival/AgentId. It never writes renderer metadata itself and contains no expected-pose oracle. `pose_oracle_checked=false` always: capture-contract success is **not H7 success**. The Python runner independently checks the requested seed/day/hour/zoom, known calendar constants and computed tick, frame10/130 schedule, observed min/max fixed deltas, raw array lengths and integer types, unique live entity/AgentIds, holder mappings/counts, required metadata types and identity, explicit override-list vocabulary and viewport geometry/flags. Malformed JSON and missing artifacts reject the launch and preserve validation errors. These structural checks still do not decide whether the pose is correct for the activity; integrate that independent expected-pose checker later. Current pre-Part2 rendering should fail this script by design.

## Later blind crops, without fabricated visibility

After real captures and the independent pose/override checker: select two actually visible, fully inside viewport holders per kind from these PNG/sidecars. Include enough nearby context to read the action; use real pixel rectangles with padding, agent centered, 4x nearest-neighbour upscale. Require nonzero body dimensions and check UI/other-sprite occlusion visually: viewport intersection is not proof of an unobscured activity. If any kind lacks two usable real observations, report the shortage and collect additional authorized live evidence; never create a crop from metadata or relabel another pose. Freeze the14 crop IDs, source PNG hash/entity/tick/rect and a separate hidden answer key. Give a fresh reviewer only the crops and seven names; save answers before revealing the key. Report per-kind/overall accuracy without inventing a threshold.

## Validation performed

Ten offline unittest checks cover exact six-case calendar/matrix and seed bounds; window/quiet launch structure; expired-deadline refusal and mocked process-group termination; malformed or mismatched sidecars, deltas, counts, IDs and metadata; honest offscreen/hidden holders; missing CLI refusal; and static no-stress/fail-closed contract. Combined repository discovery passed47 tests. No process was launched by deadline tests (Popen and killpg are mocked). GDScript parse/runtime, actual image output, time/frame determinism and renderer metadata are unverified until the later authorized runtime step.

Calendar cross-check against `baseline/live-day30-1000.json`: ticks_per_day=1440, clock_offset_ticks=480, day_index29 at10:00 gives tick41880. The same capture formula gives day30 15:00 tick42180 and day60 10:00 tick85080. The day60 value is a calendar calculation, not a runtime observation of the future selected seed.
