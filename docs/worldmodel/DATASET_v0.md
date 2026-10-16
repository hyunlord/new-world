# WorldSim dataset v0

Recorded worlds for the first world-model benchmark (`BENCH_v0.md`). Format: `README.md` (recorder schema 1). The files live outside git under `WM_DATA_ROOT` (`/Users/rexxa/work/worldsim-wm-data/v0/<split>/<episode>`); `dataset_v0_manifest.json` lists every episode with the sha256 and size of every file.

Episodes: 400; total size 1368.3 MiB; recorder built from `['0c46f69c31344f3bde23ce625a84214c5ed72fc0']`.

All splits use `--every 60` (a row per simulated hour). Days are one-based display days; the tick of day D at minute m is (D − 1) × 1440 + m − 480.

| Split | Scene | Seeds | Days | Interventions | Roles | `--agents` | Episodes | Size (MiB) |
|---|---|---|---:|---|---|---|---:|---:|
| A_obs | game_v13 | 0–127 | 120 | none | train 0–95, val 96–111, test 112–127 | 0–31, 96–99, 112–119 | 128 | 1078.3 |
| B_act | game_v13 | 1000–1063 | 120 | 8 random each (`s<seed>_b.json`) | train 1000–1047, val 1048–1055, test 1056–1063 | 1056–1063 | 64 | 221.1 |
| C_cf | game_v13 | 2000–2031 | 35 | factual + 4 variants each | val 2000–2007, test 2008–2031 | none | 160 | 30.6 |
| OOD_v12 | game_v12 | 3000–3031 | 120 | none | test 3000–3031 | none | 32 | 19.3 |
| OOD_long | game_v13 | 4000–4015 | 240 | none | test 4000–4015 | none | 16 | 19.0 |

## Schedules

- **B_act** (`make_dataset.py`, `b_act_schedule`): `numpy.random.default_rng(seed)`; per intervention a tick uniform over the hour boundaries from day 2 00:00 (tick 960) to day 118 23:00 (tick 169380), then a kind uniform over the seven, then an amount uniform on its grid — food 20–120 step 10, wood 10–80 step 10, rain 6–24 hours, cold snap 1–3 nights, windfall 5–40 step 5; settlement `home`; rows sorted by tick, then draw order.
- **C_cf** variants, each against the same seed's factual episode `none`: `gift_wood` gift_wood 60 at tick 29400; `rot_wood` rot_wood 60 at tick 29400; `rain` rain 13 at tick 33720; `cold_snap` cold_snap 2 at tick 35160. `gift_wood`/`rot_wood` act on the last autumn evening (day 21 18:00), `rain` on day 24 18:00, `cold_snap` (2 nights) on day 25 18:00.

## Splits and leakage

No seed appears in two splits or two roles (checked when the manifest is written). OOD_v12 is the Pass 6 ruleset (no winter severity, unbounded claims): a controlled distribution shift. OOD_long runs 240 days (eight winters) where training saw 120.

## Reproducing

```bash
python3 tools/worldmodel/make_dataset.py v0          # resumable: verified episodes are skipped
python3 tools/worldmodel/bench_v0.py --root $WM_DATA_ROOT --out docs/worldmodel
```
## Notes

- Recorded 14:39–14:51 KST on 2026-10-11 with `worldsim_wm` built from `0c46f69c` (round 2 of Pass 7 Part 5, `game_v13()` weights 520/280/200), `--workers 4`. Every episode verifies against its `meta.json` and this manifest.
- B_act holds 512 interventions; 28 windfalls applied 0 (the deadwood spots were already at their ceilings, in winter the snow cap of 1); every other intervention changed the world.
- **DGX copy:** `rsync` to `hyunlord@100.70.109.50:/home/hyunlord/work/worldsim-wm-data/v0` (1.4 G); all 4,660 file hashes of this manifest verified there with `sha256sum -c` on 2026-10-11.
