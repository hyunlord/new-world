# Part 3 fill checkpoint

Six calls, 96 distinct requested cells. Generation stopped: **15/96 newly accepted generated targets (15.625%)**, below 48/96. No quota failure.

| Sheet | Visual ok | Guard + review accepted |
|---|---:|---:|
| P2_fill_01 | 14 | 4 |
| P2_fill_02 | 13 | 4 |
| P2_fill_03 | 0 | 0 |
| P2_fill_04 | 13 | 4 |
| P2_fill_05 | 7 | 4 |
| P2_fill_06 | 0 | 0 |

Accepted cells and newly filled targets differ: the checkpoint excludes previously approved targets and all derived output. The separate installation selection may replace whole incomplete activity groups, dropping old partial frames to preserve one physical sheet/attempt per animation group.

Final visual reasons: `{"": 47, "artefact": 14, "region_map_wrong": 34, "wrong_pose": 1}`.

First-round and final-round sheets/verdicts are retained under `fills/review/`. Source color sprites and frozen candidate ledgers were not repainted. The one mask correction preserved all147 Part2 contextual mask hashes and all34 old overrides; proof is `fills/region_preservation.json`.
