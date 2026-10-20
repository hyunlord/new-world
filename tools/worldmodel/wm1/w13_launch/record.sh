#!/bin/bash
# WM-1.3 Part 1.2: record game_v17 32000-32559 then game_v16 30000-30559 (120 days, every 60, no agents, 8 workers).
B=/home/hyunlord/work/wm1-runs/_build/src_wm13/rust/target/release
D=/home/hyunlord/work/worldsim-wm-data/w13
L=/home/hyunlord/work/wm1-runs/_record_w13
mkdir -p "$D" "$L"
export WORLDSIM_GIT_SHA=8f6b10ed2fdc11809055eb833224f07b00cbbc58
for spec in "game_v17 32000..32560" "game_v16 30000..30560"; do
  set -- $spec
  date +%s > "$L/$1.start"
  "$B/worldsim_wm" --scene "$1" --seeds "$2" --days 120 --every 60 --workers 8 --out "$D/$1" > "$L/$1.log" 2>&1
  echo $? > "$L/$1.exit"
  date +%s > "$L/$1.end"
done
