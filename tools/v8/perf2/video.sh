#!/bin/bash
# video.sh <play-root worktree> <out-dir> <scene> [zoom ...]: PERF2-0 capture of <scene> at each zoom with the
# on-screen frame-time graph (perf2_frame.gd --overlay 1), Godot movie maker at 30 fps, 900 frames (30 s) after a
# 5 s warm-up; trimmed to the measured part and joined into <out>/<scene>-zooms.mp4. Takes the mux2 lock itself.
# Frame times in the graph are wall times under movie capture (capture overhead included).
set -uo pipefail
export PATH=/opt/homebrew/bin:$PATH
ROOT=${1:?worktree}; OUT=${2:?out}; SCENE=${3:?scene}; shift 3
ZOOMS=("${@:-0.5 1 3 4}"); [ $# -eq 0 ] && ZOOMS=(0.5 1 3 4)
SCENES=${SCENES:-/Users/rexxa/work/art-img-data/perf2/data/scenes}
GODOT=${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}
H="$(cd "$(dirname "$0")/../../.." && pwd)"
mkdir -p "$OUT"
cat > "$OUT/capture.sh" <<EOF
set -u
for z in ${ZOOMS[*]}; do
  WORLDSIM_MUX2_PROFILE=1 bash "$H/tools/v8/run_bounded.sh" 900 "$GODOT" --disable-vsync --write-movie "$OUT/$SCENE-z\$z.avi" --fixed-fps 30 \
    --path "$ROOT" -s res://tools/v8/perf2_frame.gd -- --load "$SCENES/$SCENE.wsave" --zoom \$z --overlay 1 --warmup 5 \
    --frames 900 --time-save 0 --out "$OUT/$SCENE-z\$z.json" > "$OUT/$SCENE-z\$z.log" 2>&1
  echo "zoom \$z exit \$?"
done
EOF
python3 "$H/tools/v8/mux2_gate_lock.py" -- bash "$OUT/capture.sh"
list="$OUT/concat.txt"; : > "$list"
for z in "${ZOOMS[@]}"; do
  start=$(sed -n 's/^PERF2_MOVIE_START //p' "$OUT/$SCENE-z$z.log" | head -1)
  ss=$(python3 -c "print(max(0, int('${start:-0}') ) / 30)")
  ffmpeg -y -loglevel error -ss "$ss" -i "$OUT/$SCENE-z$z.avi" -t 30 -vf scale=1280:-2 -c:v libx264 -pix_fmt yuv420p -crf 23 "$OUT/$SCENE-z$z.mp4" \
    && rm -f "$OUT/$SCENE-z$z.avi"
  echo "file '$SCENE-z$z.mp4'" >> "$list"
done
ffmpeg -y -loglevel error -f concat -safe 0 -i "$list" -c copy "$OUT/$SCENE-zooms.mp4"
ls -la "$OUT"
