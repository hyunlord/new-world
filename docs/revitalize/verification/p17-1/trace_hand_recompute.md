# Part 1 raw activity recomputation

Producer: `8de4073adbd152ecdf91ab28a1a348085e87a20f`. Inputs: `/Users/rexxa/work/p17-data/part1-smoke-r2/game_v21_trace`, seeds 42 and 43, three days each. The complete input inventory is bound by `smoke_raw.sha256`.

An independent observer audit recomputed every person's starts, redraws, failed choices, activity minutes by kind, completed count and completed duration from `_routine.jsonl`. Consuming intervals are half-open; adult counters include only adult-stage minutes. All **53 people records and 43 adult records match**, with **zero observation misses**.

Every completed episode has an arrival and ends at its duration deadline or at the configured day end. Seed 42 has 70 completions (50 deadline, 20 day-end); seed 43 has 35 (28 deadline, 7 day-end). All 105 satisfy the completion definition.

## One adult recomputed

Seed 42, AgentId 0 has two completed activities:

| End tick | End condition | Consuming minutes |
|---:|---|---:|
| 2040 | 18:00 day end, before deadline 2135 | 8 |
| 3159 | Duration deadline | 103 |

Completed minutes = 8 + 103 = 111. Mean completed duration = 111 / 2 = **55.5 minutes**. Interrupted and still-open activities are excluded.

## Retained failed check

The coordinator's first manual check guessed `clock_offset_ticks = 360`. That incorrectly reads tick 2040 as 16:00 and rejects the valid day-end completion. The actual hour-zero trace records establish an offset of **480**. With `(tick + 480) % 1440`, tick 2040 is 18:00 and every completion agrees. This was a verification-script assumption error; no production code or trace data was changed.
