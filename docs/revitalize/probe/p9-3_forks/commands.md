# Pass 9 Part 3 — exact commands

Recorder `wm_p2` = `worldsim_wm` built from `874dbf54` (the Part 2 Rust; frozen copy `/Users/rexxa/work/p9bin/wm_p2`), `WORLDSIM_GIT_SHA=874dbf542886963a67b56c2d57e4a8c5d5e6ea42`. Data under `WM_DATA_ROOT/p9/` (outside git, 8.8 GiB).

```bash
# schedules (dataset v1's five C_cf variants unchanged, the one-unit gift, seven amount-0 placebos)
python3 tools/revitalize/pass9_fork_schedules.py /Users/rexxa/work/worldsim-wm-data/p9/schedules
# recordings: see p3_forks.sh in this directory (branch families R = 8 forked at 28,800; placebos; unforked runs)
bash /Users/rexxa/work/p9bin/p3_forks.sh
# 8.3 prerequisite: re-record two v1 C_cf seeds under game_v14 (Pass 8 recorder wm_p0 and Pass 9 recorder wm_p2)
D=/Users/rexxa/work/worldsim-wm-data
cp $D/v1/schedules/C_cf/s12000_rain.json $D/v1/schedules/C_cf/s12001_gift_wood.json $D/p9/v14_check/sched/
WORLDSIM_GIT_SHA=4ab0778abc5beeea9fa0bfb42aa3a1b0b678e530 /Users/rexxa/work/p9bin/wm_p0 --scene game_v14 --seeds 12000..12002 --days 35 --every 60 --schedule-dir $D/p9/v14_check/sched --include-factual --workers 2 --out $D/p9/v14_check/wm_p0
WORLDSIM_GIT_SHA=4ab0778abc5beeea9fa0bfb42aa3a1b0b678e530 /Users/rexxa/work/p9bin/wm_p2 --scene game_v14 --seeds 12000..12002 --days 35 --every 60 --schedule-dir $D/p9/v14_check/sched --include-factual --workers 2 --out $D/p9/v14_check/wm_p2
# metrics
python3 tools/revitalize/fork_metrics.py --root $D/p9/forks --scene game_v15 --seeds 12000..12032 --replicates 8 --fork-tick 28800 \
  --variants gift_wood,rot_wood,rain,cold_snap,windfall --gift1 gift1 \
  --placebos p0_cold_snap,p0_gift_food,p0_gift_wood,p0_rain,p0_rot_wood,p0_spoil_food,p0_windfall --placebo-seeds 12000..12008 \
  --unforked $D/p9/unforked --v14 $D/v1/C_cf \
  --intervention-ticks gift_wood=29400,rot_wood=29400,rain=33720,cold_snap=35160,windfall=31920,gift1=29400 \
  --out docs/revitalize/probe/p9-3_forks
```

Wall time (recorder log): branch families 1,792 episodes 1,360.8 s (11:28:26–11:51:07, concurrent with the Part 2 gate), placebos 512 episodes listed / 448 recorded (factual members kept by `--resume`) 302.4 s, unforked 224 + 64 episodes 151.4 + 41.1 s; total 31 min for 2,520 recorded episodes (≈ 2.8 s each, 4 workers). Metrics: 13 s.

v14 re-record (8.3): with the Pass 8 recorder every file of the four episodes (`s12000_none`, `s12000_rain`, `s12001_none`, `s12001_gift_wood`) is byte-identical to dataset v1; with the Pass 9 recorder every `.npy` is identical and `meta.json` differs only by `schema_version` (2 → 3) and the new `keyed_draws` / `noise_fork` keys.
