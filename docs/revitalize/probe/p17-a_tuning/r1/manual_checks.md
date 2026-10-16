# Independent arithmetic checks of tuning round 1

These checks read raw JSON without importing the metric evaluator. They do not replace the reserved evaluation cohort.

## R1 pooled counts

| Scene | Idle observations | All nonelder-adult observations | Quotient |
|---|---:|---:|---:|
| game_v21 | 35505 | 260477 | 0.136307620251 |
| game_v20 | 148470 | 260626 | 0.569666878976 |

## Eligible adult duration check

Seed 17100, AgentId 31 has 42 nonelder-adult days, P = 391 and 54 completed activities. It is eligible for R3-P. Duration is `90 + trunc(60 * (391 - 500) / 500) = 77` minutes from arrival, shortened by a valid day-end completion.

Raw adult consuming intervals sum by kind to `[1218, 1191, 649, 1196, 786, 0, 0]` (total 5040). Independently grouped completed episodes sum to 3814 minutes across 54 activities; mean = **70.629629629630 minutes**. All counters match the adult trace record.

| Kind | Chosen | Arrived | End | Interval minutes |
|---|---:|---:|---:|---:|
| Craft | 17722 | 17760 | 17837 | 77 |
| TendFire | 18997 | 19000 | 19077 | 77 |
| Repair | 19210 | 19240 | 19317 | 77 |

All 54 completions have an arrival and meet the duration deadline or 18:00 cutoff. Interrupted, dead and open episodes are excluded from the completed mean. The raw files are included in `raw.sha256`.
