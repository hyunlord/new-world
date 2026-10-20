#!/usr/bin/env bash
# usage: smoke.sh <log_dir> [--record]
#
# V8 headless smoke (goal 11.1 step 9): run the game headless for 900 frames
# (bounded), count `SCRIPT ERROR`, `ERROR:` (audio-driver lines excluded) and
# `Locale:` warnings, then run tools/v8/smoke.gd for 900 frames and count
# labels that show a raw localization key. Fails when any count exceeds the
# base counts in tools/v8/smoke_base_counts.txt (captured at v7-final-art).
# `--record` prints the counts in that file's format instead of comparing.
# Requires an imported project (gate step 3) and a built bridge (step 2).
set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG="${1:?usage: smoke.sh <log_dir> [--record]}"
mkdir -p "$LOG"
source "$SCRIPT_DIR/godot_bin.sh"
cd "$ROOT"
bash "$SCRIPT_DIR/run_bounded.sh" 600 "$GODOT_BIN" --headless --path . --quit-after 900 > "$LOG/smoke.log" 2>&1
boot=$?
bash "$SCRIPT_DIR/run_bounded.sh" 600 "$GODOT_BIN" --headless --path . --script res://tools/v8/smoke.gd -- --frames 900 > "$LOG/smoke_labels.log" 2>&1
labels=$?
count() { grep -E "$1" "$2" | grep -viE 'alsa|audio' | wc -l | tr -d ' '; }
script_errors=$(( $(count 'SCRIPT ERROR' "$LOG/smoke.log") + $(count 'SCRIPT ERROR' "$LOG/smoke_labels.log") ))
errors=$(( $(count 'ERROR:' "$LOG/smoke.log") + $(count 'ERROR:' "$LOG/smoke_labels.log") ))
locale=$(( $(count 'Locale:' "$LOG/smoke.log") + $(count 'Locale:' "$LOG/smoke_labels.log") ))
raw_keys=$(grep -oE 'raw_keys=[0-9]+' "$LOG/smoke_labels.log" | cut -d= -f2)
done_line=$(grep -c '^SMOKE_DONE' "$LOG/smoke_labels.log")
{
    echo "script_errors $script_errors"
    echo "errors $errors"
    echo "locale_warnings $locale"
    echo "raw_keys ${raw_keys:-missing}"
} > "$LOG/smoke_counts.txt"
cat "$LOG/smoke_counts.txt"
if [[ "${2:-}" == --record ]]; then exit 0; fi
status=0
(( boot == 0 )) || { echo "boot run exited $boot"; status=1; }
(( labels == 0 && done_line == 1 )) || { echo "label scan exited $labels (SMOKE_DONE lines: $done_line)"; status=1; }
while read -r name base; do
    now=$(awk -v n="$name" '$1 == n {print $2}' "$LOG/smoke_counts.txt")
    if [[ ! "$now" =~ ^[0-9]+$ ]] || (( now > base )); then
        echo "smoke: $name $now exceeds base $base"; status=1
    else
        echo "smoke: $name $now <= base $base"
    fi
done < <(grep -v '^#' "$SCRIPT_DIR/smoke_base_counts.txt")
exit "$status"
