# Pass 14 Part 3 — Stage B exact commands (DGX)

`D=/home/hyunlord/work/worldsim-wm-data/p14`; frozen binaries as Stage A; `WORLDSIM_GIT_SHA=2ba871c0…`; code clone detached at `062e67e5` (PREREG commit; timeline and schedules), `11271117` (recording; adds only docs and schedules) and `0cd073a8` (scoring). PREREG committed 12:23:27 KST and pushed 12:23:29; the timeline probe started 12:23:36.

```bash
# timeline probe (tools/revitalize/p14_dgx/p3_timelineB.sh; 12:23:36–12:36:09 KST)
nice -n 10 $D/bin/probe_frozen --scene game_v19 --seeds 14000..14064 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 \
  --sample-every 60 --workers 4 --check-determinism --out $D/timeline_B
python3 tools/revitalize/timeline_p14.py --summary $D/timeline_B/summary.json --out $D/timeline_B_out
python3 tools/revitalize/schedules_p14.py --timeline $D/timeline_B_out/timeline.json --arms sp240L,gf240L,sp240E,gf240E --out $D/schedules_B/main
# schedules committed (11271117, 12:36:59) before the recording (stage_record.sh B 14000..14064 main; 12:37:05–13:15:06)
nice -n 10 $D/bin/wm_frozen --scene game_v19 --seeds 14000..14064 --days 120 --every 60 --schedule-dir $D/schedules_B/main \
  --include-factual --fork-tick 14280 --replicates 2 --workers 4 --out $D/stageB
# scoring (13:15)
nice -n 10 /home/hyunlord/work/wm1-venv/bin/python tools/revitalize/dose_map_p14.py --root $D/stageB --seeds 14000..14064 \
  --manifest $D/schedules_B/main/manifest.json --prereg sp240L,gf240L --probe-summary $D/timeline_B/summary.json --label "Stage B" --out $D/score_B
```

Recordings: `$D/stageB/` — 588 episodes (230 schedules + 64 factual, × 2 replicates). All 64 seeds recorded (the drop clause was not needed).
