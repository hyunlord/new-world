# Part 0 source world selection

Status: SOURCE_VERIFIED; population and occupied-village counts NOT_VERIFIED until the new bridge census. No Godot or cargo was run for this audit.

Selected W-multi: **game_v26, seed 19603, tick 90840**. Departure day is display day 64 (zero-based day index 63). The clock has 1440 ticks/day and offset 480; 10:00 is `63 * 1440 + 600 - 480 = 90840`. Source assertions and the runtime harness must verify those clock constants before use.

Raw input: `/Users/rexxa/work/p21-data/p2/A/chronicle_game_v26/chronicle_19603.json`, SHA256 `d5bbd19a1454165f3adf8d917a243be0f578019059c81297412b9a4969a8e6c4`. Metadata scene `game_v26`, git_sha `8f681805`; `/Users/rexxa/work/p21-data/p2/eval_sha` agrees. This is the Pass 21 evaluation binary's revision, not a claim it is the final merged art bridge. The new runtime census/identity is still required.

Entries are `[tick, kind, subject, other, third, settlement, settlement2, cause, value]`:

- First successful delivery: `[90840, 18, 62, null, null, 2, 3, 0, 10]`.
- Matched preceding departure (same person, reversed villages): `[90720, 17, 62, null, null, 3, 2, 0, 10]`.
- Departure at 08:00; capture/start at 10:00. The first porter has delivered exactly at start tick, so do not assume an outbound pose at that tick. Source selection does not guarantee a particular runtime pose.

The query scans all seeds 19600–19631, accepts kind18 with cause<10 before display day90, selects the lowest seed, then its earliest successful delivery and matching latest preceding kind17. Seeds19600–19602 have no qualifying entry, proving minimality. Even interpreting before-day90 by raw simulation day would not change the chosen seed/trip.

| Seed | Qualifying delivered porter entries |
|---|---:|
| 19600 | 0 |
| 19601 | 0 |
| 19602 | 0 |
| 19603 | 8 |
| 19604 | 0 |
| 19605 | 0 |
| 19606 | 1 |
| 19607 | 20 |
| 19608 | 0 |
| 19609 | 0 |
| 19610 | 28 |
| 19611 | 0 |
| 19612 | 4 |
| 19613 | 0 |
| 19614 | 0 |
| 19615 | 2 |
| 19616 | 0 |
| 19617 | 0 |
| 19618 | 0 |
| 19619 | 0 |
| 19620 | 0 |
| 19621 | 4 |
| 19622 | 0 |
| 19623 | 25 |
| 19624 | 2 |
| 19625 | 0 |
| 19626 | 0 |
| 19627 | 0 |
| 19628 | 0 |
| 19629 | 0 |
| 19630 | 0 |
| 19631 | 0 |

Reproduction (read-only):

```python
import json
from pathlib import Path
root = Path('/Users/rexxa/work/p21-data/p2/A/chronicle_game_v26')
for seed in range(19600, 19632):
    doc = json.loads((root / f'chronicle_{seed}.json').read_text())
    entries = doc['entries']
    delivered = [e for e in entries if e[1] == 18 and e[7] < 10
                 and (e[0] + 480) // 1440 + 1 < 90]
    print(seed, len(delivered))
    if seed == 19603:
        event = min(delivered, key=lambda e: e[0])
        departure = max((e for e in entries if e[1] == 17
                         and e[2] == event[2] and e[0] <= event[0]
                         and e[5] == event[6] and e[6] == event[5]),
                        key=lambda e: e[0])
        print(event, departure, (departure[0] + 480) // 1440 * 1440 + 120)
```

Cross-check: `docs/revitalize/probe/p21-2_eval/eval.json` U1 lists 19 multi-village worlds, 13 delivering worlds, 89 delivered trips, 170 porters. U3 lists 21 estrangements, 8 reconciliations and 13 estranged pairs of45 at end. Its 120-day trip counts are not the before-day90 porter-entry counts above. `chronicle.json` confirms the same 170 successful delivery entries and 8 kind21 entries over the full cohort.

Retained recipes from `docs/art/production12/perf/worlds.md` and `scripts/test/art/perf_frames_p12.gd`: W0 live seed20261010 tick40560; W1 live seed20261013 tick126994; W200 stress seed15100 tick1440 count200; W500 stress seed15100 tick1440 count500. Live worlds now use game_v26; stress worlds stay game_v19. Prior pass population/village counts are historical and must not be reused as this pass's measurements. Six framings must be re-derived by the p13 harness.
