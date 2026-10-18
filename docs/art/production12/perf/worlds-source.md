# World source selection (Part 0)

Source audit only. Runtime snapshots must populate `worlds.md` with seed, tick, population and living-settlement count. No cargo or Godot was launched for this selection.

| world | seed | start tick | generator | population / living settlements |
|---|---:|---:|---|---|
| W0 | 20261010 | 40560 | game_v25 | NOT_MEASURED |
| W1 | 20261013 | 126994 | game_v25 | NOT_MEASURED |
| W200 | 15100 | 1440 | stress_v19, requested 200 | NOT_MEASURED |
| W500 | 15100 | 1440 | stress_v19, requested 500 | NOT_MEASURED |
| W-multi | 19400 | 121080 | game_v25 | NOT_MEASURED |

W0/W1/stress parameters are copied from `scripts/test/art/perf_frames_p11.gd:9`, not from a new probe. Live and stress builders are `rust/crates/sim-bridge/src/ffi/world_node.rs:192` and `:202`.

## W-multi proof

Read-only raw source: `/Users/rexxa/work/p20-data/p2/A/chronicle_game_v25/chronicle_19400.json`.

- SHA256: `e134cbc4ee5a449518aa033024fbc60eb2d66d2d258ea9cb5534f0bb37d90ef6`.
- Header: scene game_v25, seed19400, ticks172800, producer git_sha3562a9d4, entry_count571.
- Entry field order: tick, kind, subject, other, third, settlement, settlement2, cause, value.
- Zero-based entries389/390: `[120960,17,69,null,null,2,1,0,10]`, `[120960,17,73,null,null,2,1,0,10]`.
- Entries391/392: `[121088,18,69,null,null,1,2,0,10]`, `[121088,18,73,null,null,1,2,0,10]`.
- Cause0 is successful aid food, quantity20 total, from village2 to village1. Seed19400 is the lower bound of the allowed interval, so proving its eligibility proves minimality without inspecting larger seeds.

The source clock is offset480 (`rust/crates/sim-core/src/rules.rs:1354`) and ticks_per_day1440 (`rust/crates/sim-engine/src/lib.rs:701`). Departure display day index=floor((120960+480)/1440)=84, minute480 (08:00). The prescribed 10:00 start is **121080**, because (121080+480)%1440=600; delivery at121088 is at10:08 on that day, before day90. Day index84 is recorded explicitly to avoid ambiguity with any one-based UI day label. At the chosen start, raw events predict both porters still outbound for eight ticks. Fresh bridge getters must verify this and return their entity bits before captures.

Collectors must read `get_world_info().clock_offset_ticks` and `.ticks_per_day` (world_node.rs:3578) at runtime and assert this arithmetic, rather than silently trusting these source-derived constants. Reach the world by reset_world(19400) then advance_ticks(121080). Do not substitute the 120-day end snapshot for the requested start population.

## Estrangement capture source

Read-only `/Users/rexxa/work/p20-data/p2/A/chronicle_game_v25/chronicle_19414.json`, SHA256 `79178939e9c070201467b92211bbfbf5148ad02373e117a5a9bff2e61a8237f8`, same producer short SHA. First kind20 is entry189: `[34560,20,null,null,null,1,2,0,-311]`; day_index24,08:00. A same-day10:00 capture is tick34680. Another pair1/3 estranges at95040 (entry509, value-416). This proves a source exists, not that fresh in-game lines have passed visual QA.

## Evidence limits

The raw producer SHA differs from the merged head. Committed `docs/revitalize/probe/p20-2_eval/chronicle.json` independently records seed19400 with two departures/two deliveries and32-file repeat identity, but does not prove this new bridge's runtime state. Runtime reproduction remains mandatory. No fallback is warranted: qualifying raw evidence and both getters exist. W0/W1 populations must be remeasured under game_v25; Art Pass11's v24 populations cannot be reused.
