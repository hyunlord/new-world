# WorldSim dataset v1

Recorded worlds for WM-Bench v1 (`BENCH_v1.md`), Pass 8. Recorder schema 2 (`README.md`): the world's weather — `winter_severity`, the night's burn `night_burn_per_hour`, the day's deadwood fall `winter_fall_today` — and the values that encode it (`wood_target`, `latent.npy`) are marked `hidden`; `meta.json` lists `observable_fields`. The files live outside git under `WM_DATA_ROOT` (`/Users/rexxa/work/worldsim-wm-data/v1/<split>/<episode>`); `dataset_v1_manifest.json` lists every episode with the sha256 and size of every file.

Episodes: 432; total size 1383.8 MiB; recorder built from `['4ab0778abc5beeea9fa0bfb42aa3a1b0b678e530']`.

All splits use `--every 60` (a row per simulated hour). Days are one-based display days; the tick of day D at minute m is (D − 1) × 1440 + m − 480.

| Split | Scene | Seeds | Days | Interventions | Roles | `--agents` | Episodes | Size (MiB) |
|---|---|---|---:|---|---|---|---:|---:|
| A_obs | game_v14 | 10000–10127 | 120 | none | train 10000–10095, val 10096–10111, test 10112–10127 | 10000–10031, 10096–10099, 10112–10119 | 128 | 1083.7 |
| B_act | game_v14 | 11000–11063 | 120 | 8 random each (`s<seed>_b.json`, v0's sampler) | train 11000–11047, val 11048–11055, test 11056–11063 | 11056–11063 | 64 | 222.0 |
| C_cf | game_v14 | 12000–12031 | 35 | factual + 5 variants each | val 12000–12007, test 12008–12031 | none | 192 | 38.3 |
| OOD_v13 | game_v13 | 13000–13031 | 120 | none | test 13000–13031 | none | 32 | 20.1 |
| OOD_long | game_v14 | 14000–14015 | 240 | none | test 14000–14015 | none | 16 | 19.7 |

## Schedules

- **B_act**: v0's sampler (`b_act_schedule`, `numpy.random.default_rng(seed)`), eight interventions a seed between day 2 00:00 (tick 960) and day 118 23:00 (tick 169380).
- **C_cf** variants, each against the same seed's factual episode `none`: `gift_wood` gift_wood 60 at tick 29400; `rot_wood` rot_wood 60 at tick 29400; `rain` rain 13 at tick 33720; `cold_snap` cold_snap 2 at tick 35160; `windfall` windfall 20 at tick 31920. `windfall` (new in v1) lands 20 units on the deadwood spots at day 23 12:00, in the first winter; under `game_v14` it lands on the snow (spots fill to their max) instead of stopping at the winter cap.

## Splits and leakage

No seed appears in two splits or two roles (checked when the manifest is written), and v1's seeds (10000–14015) are disjoint from dataset v0 (0–4015) and from the probe's tuning, held-out and fresh seeds (0–191). OOD_v13 is Pass 7's ruleset (no night or day draws, windfall under the snow cap): a controlled distribution shift. OOD_long runs 240 days (eight winters).

## Reproducing

```bash
python3 tools/worldmodel/make_dataset.py v1          # resumable: matching episodes are skipped
python3 tools/worldmodel/bench_v1.py --root $WM_DATA_ROOT --out docs/worldmodel
```
## Notes

- Recorded 18:55–19:08 KST on 2026-10-11 with `worldsim_wm` built from `4ab0778a` (Pass 8 Part 2; `game_v14()` = Part 4 round 1, unchanged through the held-out look of Part 5), `--workers 4`, `WORLDSIM_GIT_SHA` = `4ab0778abc5beeea9fa0bfb42aa3a1b0b678e530` in every `meta.json`; recorder schema 2 in every episode. Every episode verifies against its `meta.json` and this manifest.
- B_act holds 512 interventions; 35 of its 80 windfalls applied 0 (outside winter the deadwood spots were already at their maxima); every other intervention changed the world. In C_cf every `windfall` variant placed all 20 units (32 of 32 seeds): under `game_v14` the windfall lands on the snow.
- **DGX copy:** `rsync` (without `timing.json`) to `hyunlord@100.70.109.50:/home/hyunlord/work/worldsim-wm-data/v1` (1.4 G) with `v1.sha256` built from this manifest; all 5,012 file hashes verified there with `sha256sum -c` on 2026-10-11 19:10 KST. Only the copy and the hashing ran on the DGX.
