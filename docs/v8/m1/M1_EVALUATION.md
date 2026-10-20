# M1 evaluation (Part 5, goal section 10)

Fresh seeds **100–131** (never used for calibration: calibration used 0–7, D23). Nothing measured here fed back into
any value. **Official run (Part 6):** binaries frozen in `$DATA/eval2/` at `8626fdc9`, whose simulation code is the
final M1 code (the Part 6 review fixes, D29, changed colony trajectories: L1 ends a finished builder's job one tick
differently). The first run (Part 5, `$DATA/eval/`, simulation code of `7e712860`) passed the same bars with the
same counts; the hut ticks are the same on every seed, and 20 of 32 seeds differ only slightly in jobs taken
or finished. Raw outputs: `/Users/rexxa/work/m1-data/eval2/` (and `eval/`), outside the repository.

## The U-script (10.1)

`tools/v8/u_script.json` is the committed template. The probe resolves it per seed at tick 360 (06:00, day 0)
(`sim_test::probe::colony`):
- `designate_chop_tiles` / `resolve: nearest_trees`: the 15 trees nearest the hearth by Chebyshev distance,
  ties `(y, x)` (`nearest_trees`).
- `add_stockpile` / `resolve: stockpile_east`: a 4×4 block extending east and south from its corner. The corner
  starts 3 tiles east of the hearth; when a tile is not zoneable (water, mountain, under a building or a site, or
  already zoned), the corner moves through the 7.4 ring order around that start (Chebyshev rings, `(y, x)`
  within a ring) until all 16 tiles are zoneable (`stockpile_block`).
- `place_blueprint` / `resolve: south_of_hearth`: the valid hut top-left whose footprint centre lies south of
  the hearth (centre y > hearth y), nearest the hearth by the centre's Chebyshev distance, ties `(y, x)`. It
  avoids the stockpile block resolved at the same tick (`hut_spot`).

All three orders were accepted on all 32 seeds (`commands_accepted` = 3, `trees_designated` = 15).

## Bars (10.2)

| # | bar | pass condition | result | verdict |
|---|---|---|---|---|
| U1 | hut completed within 3 game days of placement | ≥ 28 / 32 | **32 / 32**; 369–1,220 ticks after placement (median 456; limit 4,320) | **PASS** |
| U2 | all 15 designated trees felled within 4 days | ≥ 28 / 32 | **32 / 32**; 345–1,473 ticks after marking (limit 5,760) | **PASS** |
| U3 | at day 5 every log is in a stockpile cell, carried or delivered | ≥ 28 / 32 | **32 / 32**; 0 loose logs outside zones at tick 7,200 on every seed | **PASS** |
| U4 | log ledger exact at every tick | 32 / 32 | **32 / 32**; 0 violating ticks in 460,800 checked ticks | **PASS** |
| U5 | reservation invariants at every tick; 0 stuck cancels on ≥ 30 / 32 | 32 / 32 invariants | **32 / 32** invariants (0 violating ticks); **32 / 32** with 0 stuck cancels | **PASS** |
| U6 | no death from hunger, thirst or cold in 10 days, with and without the script | 32 / 32 and 32 / 32 | **32 / 32** with the script, **32 / 32** without (0 deaths of any kind) | **PASS** |
| U7 | colony tick mean ≤ 0.6 ms and p99 ≤ 4 ms (perf.sh method, seeds 100–103, 10 days, script); Godot ≥ 60 FPS at 2× zoom, 120 s at 4× (1× zoom: report) | ms bars pass; FPS at 2× passes | mean **0.0727 ms**, p99 **0.107 ms** (medians of 3 repeats); FPS **119.99** at 2× zoom, **119.94** at 1× zoom (Part 5 view; rechecked in Part 6, see U7 details) | **PASS** |
| U8 | baseline identity unchanged | pass | PASS at every Part that touched `rust/**` (`$DATA/identity/p1_try1`, `p1_try2`, `p5`, `p6` at `8626fdc9`); final gate step 10 | **PASS** (final: see the report's gate section) |
| U9 | GUI test (9.8) end to end | pass | `tools/v8/m1_gui_test.gd` 15 / 15 checks; shots in `$DATA/gui/eval/` | **PASS** |

Determinism: both evaluation runs ran with `--check-determinism` (a lockstep twin compared every 240 ticks,
including the command results): `DETERMINISM PASS` on all 32 seeds, with and without the script.

### U7 details

- Sim: `worldsim_perf natural --scene colony --seeds S..S+1 --days 10 --commands tools/v8/u_script.json` for S in
  100–103, three repeats, reduced with `tools/v8/perf_summary.py`'s natural method. Per repeat (mean / p99 /
  p99.9 ms): 0.0722 / 0.106 / 0.141; 0.0727 / 0.107 / 0.149; 0.0730 / 0.109 / 0.155. The 1-minute load was
  5.1–6.0, so the run counts as contended. (Part 5 run: 0.0723 / 0.108 median.) The colony holds 10–29 people, against 18–40 in the baseline
  natural worlds (0.251 ms, `M1_BASE_NUMBERS.md`).
- Godot: the release bridge library was copied to the debug path (SHA checked; restored after) and the GUI
  test's FPS phase ran 120 s per zoom at 4× (48 ticks per second, 5,760 ticks each). 120 FPS is the display's
  refresh cap, so the bar is met with headroom that this method cannot measure. The 9.5 tree impostor was not
  needed.

### What the runs also show

- Jobs: on average ~150 jobs taken and ~32 finished per seed in 10 days; the rest were released to survival
  work (`CancelReason::Needs`, 36–178 per seed) and re-taken later. Jobs yield to every need, as ADR 0013
  requires. The churn costs walking time but never left a reservation behind (U5).
- Births: every seed, with or without orders, reaches 29 people at day 10 (19 births). These are V7's
  provisioned births at full food on the V7 calendar; M1.5 recalibrates fertility to years.

## Per-seed values (appendix)

| seed | hut placed → done (ticks) | U1 | 15 trees felled (ticks after marking) | U2 | loose logs day 5 | U3 | ledger / reservation violation ticks | stuck | deaths (script / none) | logs created / consumed | jobs taken / done | people day 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 100 | 360 → 810 (450) | ✓ | 1176 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 62 / 12 | 163 / 32 | 29 |
| 101 | 360 → 774 (414) | ✓ | 1317 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 60 / 12 | 146 / 32 | 29 |
| 102 | 360 → 795 (435) | ✓ | 1204 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 59 / 12 | 152 / 32 | 29 |
| 103 | 360 → 1356 (996) | ✓ | 1344 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 63 / 12 | 176 / 31 | 29 |
| 104 | 360 → 816 (456) | ✓ | 1395 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 68 / 12 | 204 / 35 | 29 |
| 105 | 360 → 732 (372) | ✓ | 345 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 63 / 12 | 68 / 32 | 29 |
| 106 | 360 → 888 (528) | ✓ | 1473 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 60 / 12 | 190 / 27 | 29 |
| 107 | 360 → 774 (414) | ✓ | 1332 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 63 / 12 | 124 / 31 | 29 |
| 108 | 360 → 783 (423) | ✓ | 1263 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 59 / 12 | 210 / 32 | 29 |
| 109 | 360 → 735 (375) | ✓ | 1242 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 63 / 12 | 109 / 31 | 29 |
| 110 | 360 → 789 (429) | ✓ | 1236 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 65 / 12 | 157 / 32 | 29 |
| 111 | 360 → 786 (426) | ✓ | 1296 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 57 / 12 | 196 / 31 | 29 |
| 112 | 360 → 729 (369) | ✓ | 1266 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 62 / 12 | 80 / 31 | 29 |
| 113 | 360 → 963 (603) | ✓ | 1140 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 57 / 12 | 149 / 32 | 29 |
| 114 | 360 → 1369 (1009) | ✓ | 1245 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 57 / 12 | 151 / 33 | 29 |
| 115 | 360 → 903 (543) | ✓ | 1326 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 63 / 12 | 155 / 33 | 29 |
| 116 | 360 → 783 (423) | ✓ | 1338 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 59 / 12 | 167 / 32 | 29 |
| 117 | 360 → 744 (384) | ✓ | 363 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 56 / 12 | 75 / 32 | 29 |
| 118 | 360 → 795 (435) | ✓ | 1230 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 58 / 12 | 115 / 31 | 29 |
| 119 | 360 → 1570 (1210) | ✓ | 1266 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 66 / 12 | 177 / 33 | 29 |
| 120 | 360 → 1334 (974) | ✓ | 1266 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 62 / 12 | 161 / 29 | 29 |
| 121 | 360 → 1411 (1051) | ✓ | 1263 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 59 / 12 | 148 / 32 | 29 |
| 122 | 360 → 778 (418) | ✓ | 1203 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 58 / 12 | 158 / 33 | 29 |
| 123 | 360 → 819 (459) | ✓ | 1278 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 59 / 12 | 145 / 32 | 29 |
| 124 | 360 → 894 (534) | ✓ | 1308 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 57 / 12 | 194 / 33 | 29 |
| 125 | 360 → 847 (487) | ✓ | 1119 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 59 / 12 | 152 / 29 | 29 |
| 126 | 360 → 738 (378) | ✓ | 1137 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 64 / 12 | 118 / 32 | 29 |
| 127 | 360 → 828 (468) | ✓ | 1307 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 54 / 12 | 136 / 31 | 29 |
| 128 | 360 → 1580 (1220) | ✓ | 1380 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 55 / 12 | 208 / 32 | 29 |
| 129 | 360 → 816 (456) | ✓ | 378 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 65 / 12 | 72 / 32 | 29 |
| 130 | 360 → 842 (482) | ✓ | 1086 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 60 / 12 | 153 / 31 | 29 |
| 131 | 360 → 840 (480) | ✓ | 1310 | ✓ | 0 | ✓ | 0 / 0 | 0 | 0 / 0 | 63 / 12 | 177 / 31 | 29 |
