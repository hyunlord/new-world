# Dataset v2x (WM-6): five times v2's counterfactual training worlds

`docs/worldmodel/wm6/v2x_manifest.json` (sha256 `34ddda513c7b70bf203523f50b279e0fdfcf4a773bdb09d93e2d7ad15fed90b5`);
data on the DGX at `/home/hyunlord/work/worldsim-wm-data/v2x/` (`flat/` the episodes, `fam/` one symlink directory per
world, `timeline/`, `timeline_out/`, `schedules/`). Numbers below: `runs/describe/describe.json` (`data6.py describe`,
rendered by `data6.dataset_md`), `runs/record/` (the recorder's logs and the prefix check).

**What it is.** 1,600 new `game_v19` worlds, seeds 50000–51599 (unused before this run, DECISIONS L0.4), recorded
exactly as WM-2's v2 train worlds — the frozen recorder `_build/src_wm2` (sha256 `52b36541…692db`, `WORLDSIM_GIT_SHA`
`f5373ab2…`), WM-2's timeline flags and clock list, `timeline_p14.py` / `schedules_p14.py` with the arms `sp240L`,
`sp120L`, `gf240E`, `rw120E`, the fork at tick 14,280, factual included — except **one replicate** (R = 1, replicate
0, the no-op fork): WM-6 trains on replicate 0 only (GOAL.md 4.1). Role: train. One recorder call per world
(DECISIONS L1.1) reproduces WM-2's members bit for bit (checked on one world: seed 40000, all five members, sha256
of `global.npy` and `settlements.npy`). The metadata commit string is `f5373ab2` (GOAL.md 1); the recorder binary was
built from `26b9af1f`, a merge of `f5373ab2`, whose string WM-2's v2 episodes carry.

**Checks.** The timeline probe ran in 16 chunks of 100 seeds, each with `--check-determinism` (DETERMINISM PASS in all
16); `timeline_p14.py`'s line check reproduces every logged scarcity value (109,197 / 109,197). Every episode is
verified (`wm_data.verify_episode`) in the manifest; the per-family prefix check (`families_v2.check`: every member
equals `none_r0` on every row up to the fork row 238, every arm equals `none_r0` up to its cast row, `none` carries no
intervention) passes for **1,600 / 1,600** families (7,576 members). Every schedule file has its episode and no arm
episode lacks one.

**Counts and arms per world.**

Worlds 1,600, episodes 7,576 (every one verified); members per arm `gf240E` 1,600, `none` 1,600, `rw120E` 1,600, `sp120L` 1,388, `sp240L` 1,388.

| members of a world | worlds |
|---|---|
| gf240E,none,rw120E,sp120L,sp240L | 1,388 |
| gf240E,none,rw120E | 212 |

Timeline: T_low for 1,388 of 1,600 worlds (86.8 %), T_high 1,260, never departing 588; line check 109,197 / 109,197.

Cases (replicate 0): v2x 5,976, WM-2's 400 train worlds 1,490.

| arm | outcome | lag | v2x mean (std) | v2x median [p10, p90] | WM-2 train mean (std) | KS D | p | n v2x / WM-2 |
|---|---|---|---|---|---|---|---|---|
| gf240E | alive | 30d | -2.42 (8.67) | 0.00 [0.00, 0.00] | -2.73 (9.03) | 0.014 | 1.000 | 1600 / 400 |
| gf240E | alive | 7d | -0.00 (0.08) | 0.00 [0.00, 0.00] | 0.01 (0.12) | 0.004 | 1.000 | 1600 / 400 |
| gf240E | cast_row | — | | | | 0.000 | 1.000 | 1600 / 400 |
| gf240E | deaths | 30d | -0.02 (0.27) | 0.00 [0.00, 0.00] | 0.00 (0.15) | 0.014 | 1.000 | 1600 / 400 |
| gf240E | deaths | 7d | -0.00 (0.10) | 0.00 [0.00, 0.00] | -0.01 (0.07) | 0.002 | 1.000 | 1600 / 400 |
| gf240E | food_stock | 30d | -24.93 (77.42) | -2.00 [-66.10, 15.00] | -29.43 (86.36) | 0.043 | 0.601 | 1600 / 400 |
| gf240E | food_stock | 7d | 45.15 (24.44) | 46.50 [10.00, 75.00] | 44.62 (26.12) | 0.040 | 0.677 | 1600 / 400 |
| gf240E | settlements_alive | 30d | -0.07 (0.26) | 0.00 [0.00, 0.00] | -0.08 (0.28) | 0.011 | 1.000 | 1600 / 400 |
| gf240E | settlements_alive | 7d | 0.00 (0.00) | 0.00 [0.00, 0.00] | 0.00 (0.00) | 0.000 | 1.000 | 1600 / 400 |
| gf240E | shift | — | | | | 0.019 | 1.000 | 1600 / 400 |
| rw120E | alive | 30d | 1.48 (7.38) | 0.00 [0.00, 0.00] | 1.35 (7.41) | 0.012 | 1.000 | 1600 / 400 |
| rw120E | alive | 7d | -0.56 (1.21) | 0.00 [-2.00, 0.00] | -0.56 (1.20) | 0.015 | 1.000 | 1600 / 400 |
| rw120E | cast_row | — | | | | 0.000 | 1.000 | 1600 / 400 |
| rw120E | deaths | 30d | 1.94 (2.35) | 1.00 [0.00, 5.00] | 1.94 (2.28) | 0.021 | 0.999 | 1600 / 400 |
| rw120E | deaths | 7d | 1.24 (1.79) | 1.00 [0.00, 4.00] | 1.26 (1.78) | 0.021 | 0.999 | 1600 / 400 |
| rw120E | food_stock | 30d | 18.94 (69.44) | 4.00 [-17.00, 53.00] | 18.54 (68.82) | 0.032 | 0.883 | 1600 / 400 |
| rw120E | food_stock | 7d | -32.31 (20.36) | -31.00 [-58.00, -5.00] | -33.57 (20.96) | 0.060 | 0.193 | 1600 / 400 |
| rw120E | settlements_alive | 30d | 0.05 (0.24) | 0.00 [0.00, 0.00] | 0.04 (0.22) | 0.007 | 1.000 | 1600 / 400 |
| rw120E | settlements_alive | 7d | 0.00 (0.00) | 0.00 [0.00, 0.00] | 0.00 (0.00) | 0.000 | 1.000 | 1600 / 400 |
| rw120E | shift | — | | | | 0.020 | 0.999 | 1600 / 400 |
| sp120L | alive | 30d | 4.71 (10.16) | 0.00 [0.00, 23.00] | 5.02 (10.90) | 0.027 | 0.989 | 1388 / 345 |
| sp120L | alive | 7d | 2.26 (4.89) | 0.00 [0.00, 9.00] | 1.78 (4.14) | 0.042 | 0.716 | 1388 / 345 |
| sp120L | cast_row | — | | | | 0.021 | 1.000 | 1388 / 345 |
| sp120L | deaths | 30d | 0.05 (0.88) | 0.00 [0.00, 0.00] | 0.05 (0.66) | 0.011 | 1.000 | 1388 / 345 |
| sp120L | deaths | 7d | 0.00 (0.00) | 0.00 [0.00, 0.00] | 0.00 (0.00) | 0.000 | 1.000 | 1388 / 345 |
| sp120L | food_stock | 30d | 47.16 (89.11) | 7.50 [-14.00, 190.30] | 44.03 (88.39) | 0.066 | 0.172 | 1388 / 345 |
| sp120L | food_stock | 7d | -45.15 (52.37) | -43.00 [-116.00, 17.00] | -47.91 (51.78) | 0.054 | 0.384 | 1388 / 345 |
| sp120L | settlements_alive | 30d | 0.14 (0.34) | 0.00 [0.00, 1.00] | 0.13 (0.33) | 0.009 | 1.000 | 1388 / 345 |
| sp120L | settlements_alive | 7d | 0.16 (0.37) | 0.00 [0.00, 1.00] | 0.15 (0.35) | 0.013 | 1.000 | 1388 / 345 |
| sp120L | shift | — | | | | 0.078 | 0.066 | 1388 / 345 |
| sp240L | alive | 30d | 9.86 (12.89) | 0.00 [0.00, 30.00] | 9.31 (13.11) | 0.040 | 0.765 | 1388 / 345 |
| sp240L | alive | 7d | 4.67 (6.80) | 0.00 [0.00, 16.00] | 4.02 (6.27) | 0.045 | 0.626 | 1388 / 345 |
| sp240L | cast_row | — | | | | 0.021 | 1.000 | 1388 / 345 |
| sp240L | deaths | 30d | 0.20 (1.17) | 0.00 [0.00, 0.00] | 0.13 (1.20) | 0.021 | 1.000 | 1388 / 345 |
| sp240L | deaths | 7d | 0.00 (0.00) | 0.00 [0.00, 0.00] | 0.00 (0.00) | 0.000 | 1.000 | 1388 / 345 |
| sp240L | food_stock | 30d | 103.46 (130.13) | 40.50 [-16.00, 287.00] | 95.49 (129.96) | 0.063 | 0.216 | 1388 / 345 |
| sp240L | food_stock | 7d | -101.62 (79.46) | -106.00 [-208.00, 5.00] | -109.16 (76.82) | 0.051 | 0.472 | 1388 / 345 |
| sp240L | settlements_alive | 30d | 0.29 (0.46) | 0.00 [0.00, 1.00] | 0.27 (0.44) | 0.024 | 0.996 | 1388 / 345 |
| sp240L | settlements_alive | 7d | 0.33 (0.47) | 0.00 [0.00, 1.00] | 0.31 (0.46) | 0.020 | 1.000 | 1388 / 345 |
| sp240L | shift | — | | | | 0.066 | 0.168 | 1388 / 345 |

KS summary: 40 tests, smallest p 0.0656, 0 below 0.05 (≈ 2.0 expected by chance), 0 below the Bonferroni level 0.00125; largest D 0.078.

**Consistent with the same population.** The two-sample KS statistic per arm × outcome × lag (and for the cast rows and split shifts)
between the new worlds' true effects and WM-2's 400 train worlds' (both replicate 0) is at most 0.078, with no p below
0.05 in 40 tests (about 2 expected by chance; none below the Bonferroni level). T_low exists for 86.8 % of the new worlds
(WM-2: 489 / 560 = 87.3 %), so the `sp*L` arms are present in 1,388 worlds. The integer outcomes (alive, deaths,
settlements) have many ties, which makes the KS p-values conservative; their means and spreads agree as well (table).
The cast rows of the `E` arms are the same row (490) in both sets; the `sp*L` cast rows (T_low) agree (D 0.021).
No KS test rejects; at these sizes a difference of about D = 0.08 would be detectable at α = 0.05, so the test shows
consistency, not identity. With the same recorder and design generating both sets, adding the new worlds changes only the
number of training worlds.

**Recording time** (DGX, other users' load present throughout, load average 7–61): timeline 16 chunks 00:44–04:01
(15 at 8 workers, 565–1,183 s each; one, `probe_50200`, at 4 workers beside the N = 400 trainings, 1,401 s); families 04:01–07:53 (13,910 s,
8 per-world calls in parallel, ≈ 1.8 s per episode); link, prefix check, manifest, features and description 07:53–07:54.
