# Pass 14 Part 1 — Stage A exact commands (DGX)

`D=/home/hyunlord/work/worldsim-wm-data/p14`; recorder `$D/bin/wm_frozen` (sha256 `ba43b546…`, built at `2ba871c0`); `WORLDSIM_GIT_SHA=2ba871c09819c240904510bbf26b0fb2a59a79b0`; code clone `/home/hyunlord/work/new-world-p14` detached at `4eb829d9` (P14_CODE_COMMIT) for the schedule writer and the scorer. Job script `tools/revitalize/p14_dgx/stage_record.sh A 12000..12032 F W`, launched through `launch.sh` (setsid nohup), each recorder job behind `wait_load.sh`.

```bash
# schedules (code 6e66f6f7), committed in schedules/F and schedules/W with manifests (sha256 of every file) before the run
python3 tools/revitalize/schedules_p14.py --timeline $D/timeline_A_out/timeline.json --arms F --out $D/schedules_A/F
python3 tools/revitalize/schedules_p14.py --timeline $D/timeline_A_out/timeline.json --arms W --out $D/schedules_A/W
# group F with the factual (11:18:12–12:06:02 KST), then group W keeping the verified factual (12:06:02–12:21:45)
nice -n 10 $D/bin/wm_frozen --scene game_v19 --seeds 12000..12032 --days 120 --every 60 --schedule-dir $D/schedules_A/F \
  --include-factual --fork-tick 14280 --replicates 2 --workers 4 --out $D/stageA
nice -n 10 $D/bin/wm_frozen --scene game_v19 --seeds 12000..12032 --days 120 --every 60 --schedule-dir $D/schedules_A/W \
  --include-factual --fork-tick 14280 --replicates 2 --workers 4 --out $D/stageA --resume
# scoring (wm1-venv numpy; 12:22 KST)
nice -n 10 /home/hyunlord/work/wm1-venv/bin/python tools/revitalize/dose_map_p14.py --root $D/stageA --seeds 12000..12032 \
  --manifest $D/schedules_A/F/manifest.json --manifest $D/schedules_A/W/manifest.json \
  --probe-summary $D/timeline_A/summary.json --unforked $D/unforked --label "Stage A" --out $D/score_A
```

Recordings: `$D/stageA/` — 992 episodes (group F 340 schedules + 32 factual, group W 124 schedules; × 2 replicates). The group-F-only scoring (`$D/score_A_F`, run while group W recorded) gives byte-identical per-arm results for the F arms.
