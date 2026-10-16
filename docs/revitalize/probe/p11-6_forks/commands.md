# Pass 11 Part 6 — exact commands

Recorder `wm_frozen` (frozen commit `c5b200d7`, sha256 `47f70a9e6bbd4f91f7e320296f106e08220cab2662db11f3eb26b311a08f7575`), `WORLDSIM_GIT_SHA=c5b200d7ee618b465d6aa0fdf054fb82d820eed6`. Data under `/Users/rexxa/work/worldsim-wm-data/p11/` (outside git). Pass 9–10 data are read only.

```bash
# schedules: Pass 9's generator, byte-identical to p10/schedules (diff -r: no difference)
python3 tools/revitalize/pass9_fork_schedules.py /Users/rexxa/work/worldsim-wm-data/p11/schedules
# recordings: p11_forks.sh (Pass 10's p6_forks.sh with --scene game_v17 --replicates 4, output under p11/)
bash docs/revitalize/probe/p11-6_forks/p11_forks.sh c5b200d7ee618b465d6aa0fdf054fb82d820eed6
# metrics (dark nights from the home settlement's slot; exposure deaths counted; deaths by stage)
D=/Users/rexxa/work/worldsim-wm-data
python3 tools/revitalize/fork_metrics.py --root $D/p11/forks --scene game_v17 --seeds 12000..12032 --replicates 4 --fork-tick 28800 \
  --variants gift_wood,rot_wood,rain,cold_snap,windfall --gift1 gift1 \
  --placebos p0_cold_snap,p0_gift_food,p0_gift_wood,p0_rain,p0_rot_wood,p0_spoil_food,p0_windfall --placebo-seeds 12000..12008 \
  --unforked $D/p11/unforked --deaths-with-exposure --home-slot-fire --deaths-by-stage \
  --intervention-ticks gift_wood=29400,rot_wood=29400,rain=33720,cold_snap=35160,windfall=31920,gift1=29400 \
  --out docs/revitalize/probe/p11-6_forks
```

Wall time (recorder log, 00:28–00:45 KST): branch families 896 episodes 632 s, placebos 256 listed 182 s, unforked runs of both schedule sets afterwards; metrics a few seconds.
