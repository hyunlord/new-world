# Pass 10 Part 6 — exact commands

Recorder `wm_frozen` (frozen commit `58119582`), `WORLDSIM_GIT_SHA=581195820e8e6e79924f0583339da077bfc8d9f7`. Data under `/Users/rexxa/work/worldsim-wm-data/p10/` (outside git; forks 3.9 GiB, unforked 0.99 GiB). Pass 9's data (`p9/`) is read only.

```bash
# schedules: Pass 9's generator, byte-identical to p9/schedules (diff -r: no difference)
python3 tools/revitalize/pass9_fork_schedules.py /Users/rexxa/work/worldsim-wm-data/p10/schedules
# recordings: p6_forks.sh in this directory (Pass 9's p6_forks.sh with --scene game_v16 --replicates 4, output under p10/)
bash docs/revitalize/probe/p10-6_forks/p6_forks.sh
# metrics
D=/Users/rexxa/work/worldsim-wm-data
python3 tools/revitalize/fork_metrics.py --root $D/p10/forks --scene game_v16 --seeds 12000..12032 --replicates 4 --fork-tick 28800 \
  --variants gift_wood,rot_wood,rain,cold_snap,windfall --gift1 gift1 \
  --placebos p0_cold_snap,p0_gift_food,p0_gift_wood,p0_rain,p0_rot_wood,p0_spoil_food,p0_windfall --placebo-seeds 12000..12008 \
  --unforked $D/p10/unforked --deaths-with-exposure --v15 $D/p9/forks --v15-replicates 8 \
  --intervention-ticks gift_wood=29400,rot_wood=29400,rain=33720,cold_snap=35160,windfall=31920,gift1=29400 \
  --out docs/revitalize/probe/p10-6_forks
```

Wall time (recorder log, 17:41:35–17:57:32 KST): branch families 896 episodes 591 s, placebos 256 listed (224 recorded) 171 s, unforked 224 + 64 episodes 151 + 45 s; 1,400 recorded episodes. Metrics: 13 s.

Provenance (review round 1, L10): `shasum -a 256` of `wm_frozen` = `282037ec5fb41650712cf1cb25863f8611a902949a620d2a02cc5b27379cf6ff`, of `probe_frozen` = `c10bd3762e3f38ed6f59fabbb44d2d8bf01d1177fc287face9406683fb0ab74e`. Rebuilt in Part 7 from the review-round-1 tree (whose only non-test Rust change after `58119582` is a doc comment), both binaries are byte-identical to these frozen copies.
