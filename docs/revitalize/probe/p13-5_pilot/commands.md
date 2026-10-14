# Pass 13 Part 5 — exact commands

Recorder `p13bin/wm_frozen` (frozen `9f0d38ca`, sha256 `1844ebc3…`), `WORLDSIM_GIT_SHA=9f0d38ca…`; recording 23:39–00:55 KST (concurrently with Parts 4 and 6).

```bash
bash p5_pilot.sh <sha> game_v19 game_v18 game_v16 game_v17   # worlds 26000..26063, 120 d, --every 60, fork tick 1, R = 4, empty schedule dir
D=/Users/rexxa/work/worldsim-wm-data/p13
# replicate-0 view for the Pass 11/12 tools: $D/ubs_view/pilot/<scene>/<scene>_s<seed>_none -> pilot/<scene>/<scene>_s<seed>_none_r0
python3 tools/revitalize/diversity_p11.py check-v1 --root /Users/rexxa/work/worldsim-wm-data --out docs/revitalize/probe/p13-5_pilot/check_v1
python3 tools/revitalize/paths_p13.py --scene game_v16=$D/pilot/game_v16 --scene game_v17=$D/pilot/game_v17 --scene game_v18=$D/pilot/game_v18 \
  --scene game_v19=$D/pilot/game_v19 --diff game_v19:game_v18 --diff game_v19:game_v17 --diff game_v19:game_v16 --diff game_v18:game_v17 --out docs/revitalize/probe/p13-5_pilot
python3 tools/revitalize/diversity_p11.py pilot --root $D/ubs_view/pilot --scenes game_v16,game_v17,game_v18,game_v19 --train 26000..26048 --test 26048..26064 --out docs/revitalize/probe/p13-5_pilot
python3 tools/revitalize/diversity_p12.py score --root $D/ubs_view/pilot --scenes game_v16,game_v17,game_v18,game_v19 --seeds 26000..26064 \
  --bootstrap game_v17:game_v19 game_v18:game_v19 --out docs/revitalize/probe/p13-5_pilot/ubs_r0
python3 tools/revitalize/pilot_p13.py --root $D/pilot --diversity docs/revitalize/probe/p13-5_pilot/diversity.json --paths docs/revitalize/probe/p13-5_pilot/paths.json --out docs/revitalize/probe/p13-5_pilot
```
(`diversity.md`'s "J7" lines are Pass 11's definition, printed by the reused tool; Pass 13's J7 is `pilot.md`.)
