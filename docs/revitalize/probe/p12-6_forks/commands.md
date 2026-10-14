# Pass 12 Part 6 — exact commands

Recorder `p12bin/wm_frozen` (frozen commit `8142cc3a`, sha256 `4d3210528af5828f173180ca4de6a31db2e30000aeae23d4dde065226d660594`), `WORLDSIM_GIT_SHA=8142cc3a66b459a658258daee8c4b1eb6d393f0b`. Data under `/Users/rexxa/work/worldsim-wm-data/p12/` (outside git); `p11/` read only.

```bash
# schedules: Pass 9's generator (byte-identical to p11/schedules: diff -r gives no difference)
python3 tools/revitalize/pass9_fork_schedules.py /Users/rexxa/work/worldsim-wm-data/p12/schedules
# recordings (12.1): Pass 11's p11_forks.sh with --scene game_v18, output under p12/
bash docs/revitalize/probe/p12-6_forks/p12_forks.sh 8142cc3a66b459a658258daee8c4b1eb6d393f0b
# metrics
D=/Users/rexxa/work/worldsim-wm-data
python3 tools/revitalize/fork_metrics.py --root $D/p12/forks --scene game_v18 --seeds 12000..12032 --replicates 4 --fork-tick 28800 \
  --variants gift_wood,rot_wood,rain,cold_snap,windfall --gift1 gift1 \
  --placebos p0_cold_snap,p0_gift_food,p0_gift_wood,p0_rain,p0_rot_wood,p0_spoil_food,p0_windfall --placebo-seeds 12000..12008 \
  --unforked $D/p12/unforked --deaths-with-exposure --home-slot-fire --deaths-by-stage \
  --intervention-ticks gift_wood=29400,rot_wood=29400,rain=33720,cold_snap=35160,windfall=31920,gift1=29400 \
  --out docs/revitalize/probe/p12-6_forks
python3 tools/revitalize/fork_extras_p12.py --root $D/p12/forks --scene game_v18 --seeds 12000..12032 --replicates 4 \
  --variants gift_wood,rot_wood,rain,cold_snap,windfall,gift1 --out docs/revitalize/probe/p12-6_forks
```

Wall time (recorder log): branch families 896 episodes 784 s (14:04–14:17 KST), placebos 256 episodes 274 s, unforked runs to 14:26; concurrently with the pilot recording.
