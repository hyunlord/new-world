# Day60 activity capture seed selection

Validation: PASS. Build exit 0; probe exit 0; 16/16 completed seeds.

Ranking uses day60 endpoint tick86400; capture uses calendar day60 10:00 tick85080. They are separate population measurements.

Winner: **20261018**; endpoint population **92**, capture-time population **92**, capture-time settlements **2**.

| Rank | Seed | Endpoint population (86400) | Capture population (85080) |
|---|---|---|---|
| 1 | 20261018 | 92 | 92 |
| 2 | 20261022 | 75 | 75 |
| 3 | 20261024 | 68 | 68 |
| 4 | 20261013 | 66 | 66 |
| 5 | 20261017 | 64 | 64 |
| 6 | 20261021 | 56 | 54 |
| 7 | 20261025 | 44 | 44 |
| 8 | 20261023 | 40 | 41 |
| 9 | 20261016 | 37 | 37 |
| 10 | 20261015 | 36 | 37 |
| 11 | 20261026 | 33 | 33 |
| 12 | 20261014 | 30 | 30 |
| 13 | 20261011 | 27 | 27 |
| 14 | 20261019 | 26 | 26 |
| 15 | 20261012 | 24 | 24 |
| 16 | 20261020 | 21 | 21 |

Binary SHA256 `4bb3be7af45dc718d073f0421412d0bc2b502441381df3c60d9015f742ae1607` matches pre-run record. Rust source manifest checked: 422 files, 0 mismatches.
Summary SHA256 `5442b7c068133731f9f19711e56190509fd0ec4aaa0d0a4bdd155fd0eded03ac`. All output hashes and exact selected JSONL line numbers are in selection.json.
Git source identity `eb7251ed1cd24f3bbf9b7366f8af30572ec1638c`. All parsed per-seed rows checked against this identity.

No probe was relaunched. This selects the seed from native headless simulation; Godot capture and visual/pose validation are not established by this result.

Validator correction: first inspection assumed summary.seeds was an array and flagged a schema mismatch. `probe/summary.rs:177` establishes the actual `{start,end_exclusive}` object. The first validator output is preserved as `selection-validator-attempt1.*`; the corrected validator checks both that range and all sixteen summary run IDs. This was an inspection-script error, not a failed probe run.
