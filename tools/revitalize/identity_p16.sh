#!/usr/bin/env bash
# usage: identity_p16.sh <release_bin_dir> <probe_out_dir> <data_dir> <git_sha>
# Pass 16 identity checks I1–I3 (PASS16_GOAL.md 4.1, as Pass 15):
#   I1 game_v19 and I3 game_v16 probe summaries, --seeds 0..32 (32 worlds), 120 days,
#      Pass 13 round 3's flags, 4 workers each (run together);
#   I2 the recorder on game_v19, --seeds 12000..12002 (half-open: 12000, 12001),
#      every array's sha256 (meta.json / timing.json left out: git sha, wall times).
# Probe outputs land in <probe_out_dir>/{i1_v19,i3_v16}; everything but summary.json
# is moved to <data_dir>/probe/ (outside the repository). Recorder episodes go to
# <data_dir>/i2/, their sha256 list to <probe_out_dir>/i2_sha256.txt.
set -euo pipefail
BIN="${1:?bin dir}"; OUT="${2:?probe out}"; DATA="${3:?data dir}"; SHA="${4:?git sha}"
export WORLDSIM_GIT_SHA="$SHA"
mkdir -p "$OUT" "$DATA/probe" "$DATA/i2"
FLAGS=(--seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism)
echo "I1/I3 start $(date '+%F %T')"
"$BIN/worldsim_probe" --scene game_v19 "${FLAGS[@]}" --out "$OUT/i1_v19" > "$DATA/i1.log" 2>&1 &
p1=$!
"$BIN/worldsim_probe" --scene game_v16 "${FLAGS[@]}" --out "$OUT/i3_v16" > "$DATA/i3.log" 2>&1 &
p3=$!
wait $p1; wait $p3
echo "I1/I3 end $(date '+%F %T')"
for d in i1_v19 i3_v16; do
    mkdir -p "$DATA/probe/$d"
    find "$OUT/$d" -mindepth 1 -maxdepth 1 ! -name summary.json -exec mv {} "$DATA/probe/$d/" \;
done
echo "I2 start $(date '+%F %T')"
rm -rf "$DATA/i2"; mkdir -p "$DATA/i2"
"$BIN/worldsim_wm" --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --workers 2 --out "$DATA/i2" > "$DATA/i2.log" 2>&1
echo "I2 end $(date '+%F %T')"
( cd "$DATA/i2" && find . -type f \( -name '*.npy' -o -name interventions.json \) | LC_ALL=C sort | xargs shasum -a 256 ) > "$OUT/i2_sha256.txt"
wc -l < "$OUT/i2_sha256.txt"
