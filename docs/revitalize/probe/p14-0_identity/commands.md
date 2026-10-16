# Pass 14 Part 0 — exact commands (DGX; `tools/revitalize/p14_dgx/p0_probes.sh`)

Binaries: `p14/bin/probe_frozen` (sha256 `a44733e6…`), `p14/bin/wm_frozen` (sha256 `ba43b546…`), built at `2ba871c0` in `/home/hyunlord/work/p14-build`. `WORLDSIM_GIT_SHA=2ba871c09819c240904510bbf26b0fb2a59a79b0`; every job `nice -n 10` behind `wait_load.sh`. `D=/home/hyunlord/work/worldsim-wm-data/p14`.

```bash
# K1 identity (10:59:15–11:06:22 KST)
probe_frozen --scene game_v19 --seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 \
  --sample-every 120 --workers 4 --check-determinism --out $D/identity_v19_120day
python3 tools/revitalize/compare_summaries.py docs/revitalize/probe/p13-3_round3_v19_120day/summary.json $D/identity_v19_120day/summary.json
# replicate 0 (11:06:22–11:06:34)
wm_frozen --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --workers 2 --out $D/unforked
# factual timeline of Stage A (11:06:34–11:13:24)
probe_frozen --scene game_v19 --seeds 12000..12032 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 \
  --sample-every 60 --workers 4 --check-determinism --out $D/timeline_A
python3 tools/revitalize/timeline_p14.py --summary $D/timeline_A/summary.json --out $D/timeline_A_out      # code 6e66f6f7
python3 tools/revitalize/schedules_p14.py --timeline $D/timeline_A_out/timeline.json --arms F --out $D/schedules_A/F
python3 tools/revitalize/schedules_p14.py --timeline $D/timeline_A_out/timeline.json --arms W --out $D/schedules_A/W
```
