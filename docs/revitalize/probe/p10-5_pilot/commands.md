# Pass 10 Part 5 — exact commands

Recorder `wm_frozen` = `worldsim_wm` built from the frozen commit `58119582` (`/Users/rexxa/work/p10bin/wm_frozen`), `WORLDSIM_GIT_SHA=581195820e8e6e79924f0583339da077bfc8d9f7` (every pilot `meta.json` carries it). Data under `/Users/rexxa/work/worldsim-wm-data/p10/pilot/` (outside git, 230 MiB).

```bash
# 11.2 implementation check on dataset v1 (A_train fit, A_test scored), 16:54 KST
python3 tools/revitalize/diversity_p10.py check-v1 --root /Users/rexxa/work/worldsim-wm-data --out <dir>   # -> check_v1.json
# 11.1 recordings (three concurrent processes, 17:22:11 - 17:30:22 KST)
for sc in game_v15 game_v16_neutral game_v16; do
  WORLDSIM_GIT_SHA=581195820e8e6e79924f0583339da077bfc8d9f7 /Users/rexxa/work/p10bin/wm_frozen \
    --scene $sc --seeds 20000..20112 --days 120 --every 60 --workers 4 --out /Users/rexxa/work/worldsim-wm-data/p10/pilot/$sc
done
# 11.2-11.4 metrics (6 s)
python3 tools/revitalize/diversity_p10.py pilot --root /Users/rexxa/work/worldsim-wm-data/p10/pilot \
  --scenes game_v15,game_v16_neutral,game_v16 --train 20000..20096 --test 20096..20112 --out docs/revitalize/probe/p10-5_pilot
```

Wall time per scene: game_v15 7 min 25 s, game_v16 8 min 5 s, game_v16_neutral 8 min 11 s (112 episodes each, ≈ 16 s CPU per episode, 4 workers per scene, the three scenes concurrently with the last minutes of the gate).

Provenance (review round 1, L10): `shasum -a 256` of `wm_frozen` = `282037ec5fb41650712cf1cb25863f8611a902949a620d2a02cc5b27379cf6ff`, of `probe_frozen` = `c10bd3762e3f38ed6f59fabbb44d2d8bf01d1177fc287face9406683fb0ab74e`. Rebuilt in Part 7 from the review-round-1 tree (whose only non-test Rust change after `58119582` is a doc comment), both binaries are byte-identical to these frozen copies.
