# Pass 11 Part 5 — exact commands

Recorder `wm_frozen` (frozen commit `c5b200d7`, sha256 `47f70a9e6bbd4f91f7e320296f106e08220cab2662db11f3eb26b311a08f7575`), `WORLDSIM_GIT_SHA=c5b200d7ee618b465d6aa0fdf054fb82d820eed6`. Data under `/Users/rexxa/work/worldsim-wm-data/p11/pilot/` (outside git). Seeds 22000–22111 were used for nothing before (fresh: no test, probe or scratch run touched them); roles 22000–22095 train, 22096–22111 test.

```bash
# recording (Part 5, 00:02–00:23 KST, scenes one after the other, 4 workers each)
bash /Users/rexxa/work/p11bin/p5_pilot.sh
#   = for scene in game_v16 game_v17_nofission game_v17:
#       wm_frozen --scene $scene --seeds 22000..22112 --days 120 --every 60 --workers 4 --out $D/$scene
# implementation check (WM-1.1's dataset-v1 A_test values, +-0.002)
python3 tools/revitalize/diversity_p11.py check-v1 --root /Users/rexxa/work/worldsim-wm-data --out docs/revitalize/probe/p11-5_pilot
# metrics (Pass 10's pilot recordings re-scored read-only as the reference)
python3 tools/revitalize/diversity_p11.py pilot --root /Users/rexxa/work/worldsim-wm-data/p11/pilot \
  --scenes game_v16,game_v17_nofission,game_v17 --train 22000..22096 --test 22096..22112 \
  --reference-root /Users/rexxa/work/worldsim-wm-data/p10/pilot --reference-scenes game_v15,game_v16 \
  --reference-train 20000..20096 --reference-test 20096..20112 --out docs/revitalize/probe/p11-5_pilot
```
