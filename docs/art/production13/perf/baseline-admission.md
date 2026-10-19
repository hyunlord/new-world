# Baseline admission record

Tooling6d493fa8; release/debug SHA b4d29cf526676adc3fa77f21454d2bee79c7865a08b32f398c975e7d8bf6a9f6. R=2.0089961669873446s; probe limit2.109445975336712s; load must be <8 at both ends.

First attempts: no Godot session was launched because pre-admission failed. No frame time or cost ranking is measured. Retry after the recorded completion plus1800s, up to6 actual attempts/item. This is an interim record, not final contention exhaustion.

| Evidence | P before s | P after s | Load before | Load after | Verdict |
|---|---:|---:|---:|---:|---|
| ablation-W200-attempt1.json.validity.json | 2.076126125 | 1.996885958 | 27.58838 | 27.58838 | NOT_ADMITTED |
| matrix/baseline-W-multi-attempt1.json.validity.json | 1.993373917 | 2.032287750 | 24.18359 | 24.18359 | NOT_ADMITTED |
| matrix/baseline-W0-attempt1.json.validity.json | 2.045171833 | 2.000379750 | 22.86670 | 23.83789 | NOT_ADMITTED |
| matrix/baseline-W1-attempt1.json.validity.json | 2.020485500 | 2.010546667 | 25.07275 | 25.07275 | NOT_ADMITTED |
| matrix/baseline-W200-attempt1.json.validity.json | 2.056259209 | 2.009438875 | 24.02588 | 24.02588 | NOT_ADMITTED |
| matrix/baseline-W500-attempt1.json.validity.json | 1.967913750 | 2.024051958 | 23.83789 | 26.73340 | NOT_ADMITTED |
| sections-W-multi-attempt1/frames.json.validity.json | 2.071241292 | 2.137553250 | 26.85645 | 26.85645 | NOT_ADMITTED |
| sections-W200-attempt1/frames.json.validity.json | 1.969317041 | 2.023593708 | 26.06055 | 26.06055 | NOT_ADMITTED |
| sections-W500-attempt1/frames.json.validity.json | 2.096809458 | 2.023052791 | 26.28809 | 25.54395 | NOT_ADMITTED |

All18 probe snapshots and process lists remain in the linked external JSONs; holds.md and attempts.json record timing. W-multi section post-probe also exceeds the CPU limit. All other first-round CPU probes meet the limit, but every load fails.

Next common retry boundary:2026-10-19T02:59:25Z (matrix alone eligible from02:58:20Z). No admission waiting has yet been charged as of this record.
