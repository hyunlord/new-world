# Part 1 independent raw-trace audit

**PASS: zero counter discrepancies across every person and adult record in both three-day smoke worlds.** This verifies trace accounting on these two worlds; it is not the Stage A evaluation or a long-run identity claim.

Source commit: `8de4073adbd152ecdf91ab28a1a348085e87a20f`, also recorded in both trace metadata files. Inputs: `/Users/rexxa/work/p17-data/part1-smoke-r2/game_v21_trace/`, seeds 42 and 43, 4,320 completed ticks each. This audit made no Rust or simulation changes and ran no probe or build.

## Complete recomputation

The audit rebuilt starts, redraws and no-place failures from raw lifecycle events; rebuilt kind minutes from half-open consuming intervals; grouped those intervals by `(id, chosen_tick)` to verify every ended episode; then rebuilt completed counts and duration sums. Adult calculations use only `life_stage == "adult"` intervals/events. All keys of each `_people.jsonl` and `_agents.jsonl` `routine` object matched, including zero-activity records. Both metadata files report zero observation misses.

| Seed | People checked | Adult records checked | Starts | All-stage minutes | Adult minutes | Completed, all stages | Completed adult count / minutes | Discrepancies / misses |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 42 | 24 | 19 | 239 | 6,884 | 4,413 | 70 | 45 / 3,308 | 0 / 0 |
| 43 | 29 | 24 | 303 | 3,378 | 1,511 | 35 | 6 / 271 | 0 / 0 |

All 105 completed episodes had an arrival and ended either at/after their duration deadline or at the display-day cutoff. Seed 42 had 50 deadline completions and 20 earlier day-end completions; seed 43 had 28 and 7. All-stage completed-minute sums were 5,591 and 2,055, respectively. Walking and open episodes contributed no completed duration.

Reproducible external helper:

```sh
python3 /Users/rexxa/work/p17-data/part1-smoke-r2/trace_recompute.py
```

Helper SHA-256: `8c24ca7177597fd3517a29d375d98716cde31dc59a0f17438427baf1320eb336`.

Observed terminal output:

```text
PASS seed=42 people=24 adults=19 offset=480 completed=70 invalid_360_rejections=20 discrepancies=0 misses=0
PASS seed=43 people=29 adults=24 offset=480 completed=35 invalid_360_rejections=7 discrepancies=0 misses=0
```

## Invalid-check history and clock correction

The root's preceding recomputation failed an assertion while using a guessed `+360` clock offset. Its exact assertion code was not retained, so this audit does not claim to recover that original traceback. The guessed offset is independently contradicted by both `_hours.jsonl` files: their first row is `tick: 0, day: 1, minute: 480`. The correct clock is `(system_tick + 480) % 1440`.

Reapplying the guessed `+360` cutoff check falsely rejects **27 valid day-end completions**, exactly 20 in seed 42 and 7 in seed 43. With `+480`, all 105 completed episodes satisfy the specified completion condition. No implementation discrepancy was found. The earlier assertion failure remains an invalid-check failure; it is not reclassified as an earlier passing check.

## One adult recomputed by hand

Seed 42, AgentId 0: temperament P = 611, adult exposure = 4,320 ticks / 1,440 = 3 days. The configured duration after arrival is `90 + trunc(60 × 111 / 500) = 103` minutes. Its complete episode inventory is:

| Kind | Chosen tick | Arrived tick | Deadline | End tick | Consuming minutes | Completed? |
|---|---:|---:|---:|---:|---:|---|
| Repair | 426 | absent | absent | 448 | 0 | No |
| Craft | 451 | absent | absent | 481 | 0 | No |
| Craft | 546 | absent | absent | 553 | 0 | No |
| Explore | 555 | absent | absent | 569 | 0 | No |
| TendFire | 571 | absent | absent | 600 | 0 | No, never arrived |
| TendFire | 1714 | absent | absent | 1763 | 0 | No |
| HearthCircle | 1882 | 1888 | 1991 | 1896 | 8 | No |
| Craft | 1901 | absent | absent | 1921 | 0 | No |
| Craft | 1978 | 2032 | 2135 | 2040 | 8 | Yes, 18:00 cutoff |
| TendFire | 2922 | absent | absent | 2940 | 0 | No |
| Explore | 2991 | absent | absent | 3000 | 0 | No |
| Craft | 3031 | 3056 | 3159 | 3159 | 103 | Yes, duration deadline |
| HearthCircle | 3215 | 3216 | 3319 | 3263 | 47 | No |
| Explore | 3274 | 3368 | 3471 | 3441 | 73 | No |
| Repair | 4291 | 4312 | 4415 | open at 4320 | 8 | No, still open |

Starts in stable kind order are `[3, 5, 2, 2, 3, 0, 0]`. Consuming minutes are `[0, 111, 8, 55, 73, 0, 0]`, totaling 247. The six raw intervals are `[1888,1896)`, `[2032,2040)`, `[3056,3159)`, `[3216,3263)`, `[3368,3441)` and the open `[4312,4320)`.

Only the two completed Craft episodes enter the mean: `(8 + 103) / 2 = 55.5` minutes, matching `completed_minutes = 111` and `completed_activities = 2`. The 8-minute completion is valid because `(2040 + 480) % 1440 = 1080`, or 18:00. The rejected `+360` assumption calls that same tick 16:00. The open Repair's 8 minutes enter its kind total but not the completed mean. This adult has fewer than ten adult days and is therefore a hand-audit example, not an eligible R3 evaluation observation.

## Input fingerprints

| File | SHA-256 |
|---|---|
| `game_v21_s42_routine.jsonl` | `03cfd218c7fabf92c7584866ede1da7cd9b7fbdb562dc5eca428673b586b0b8a` |
| `game_v21_s42_people.jsonl` | `506e446ad2e98f8f2d752583b25f54986c652e0a6ac7f0d0b82f0f9fc73589fb` |
| `game_v21_s42_agents.jsonl` | `d610738796675a19ee231e9179b1829ede3ee12a1215d3ba7c60ab6040d22117` |
| `game_v21_s42_hours.jsonl` | `4e091d4c82e682935d23d51ce40836bb4de1f9452318a7185c52442772e2ba50` |
| `game_v21_s42_trace_meta.json` | `5581e164723489daea2b8729be0ad39af6feea42b3feb5ad44343587f3a08587` |
| `game_v21_s43_routine.jsonl` | `936f30d5ac0b2951ffebe42269a5efd66e371d48490c8495dd6d9f6bb90d31f2` |
| `game_v21_s43_people.jsonl` | `1fe6b964fa3f7e3c1275d465dd4944316bceb5e92e89daa3278425c43429949e` |
| `game_v21_s43_agents.jsonl` | `895c0b35e699b76c469adeb3f935eceb180675c232562f000f1bf7a52f0872a8` |
| `game_v21_s43_hours.jsonl` | `e062e8069e3a04ac0583ca431e646adef7f0f742ad601eed4bce8fe94adb2ff1` |
| `game_v21_s43_trace_meta.json` | `4d0f61664b07c49e2440864056509b9603bdd53174a5f4b78bda5dd0b1f4994e` |
