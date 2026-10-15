#!/usr/bin/env bash
# usage (on the DGX): dgx_probe_p16.sh <bin> <out_root> <seeds a..b> <days> <workers> <scene> [<scene> ...]
# Pass 16 probe batch (PASS16_GOAL.md 1.3, 8.2, 8.4): one scene after another (never
# two probe jobs at once), each under `nice -n 10` with Pass 13 round 3's flags plus
# --agent-trace, into <out_root>/<scene>/; per-scene logs and exit codes next to it.
# WORLDSIM_GIT_SHA must be set by the caller to the code commit the binary was built from.
set -uo pipefail
BIN="${1:?bin}"; OUT="${2:?out root}"; SEEDS="${3:?seeds}"; DAYS="${4:?days}"; WORKERS="${5:?workers}"; shift 5
: "${WORLDSIM_GIT_SHA:?set WORLDSIM_GIT_SHA}"
mkdir -p "$OUT"
status=0
for scene in "$@"; do
    echo "$scene start $(date '+%F %T')" >> "$OUT/batch.log"
    nice -n 10 "$BIN" --scene "$scene" --seeds "$SEEDS" --days "$DAYS" \
        --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 \
        --workers "$WORKERS" --check-determinism --agent-trace --out "$OUT/$scene" > "$OUT/$scene.log" 2>&1
    code=$?
    echo "$code" > "$OUT/$scene.exit"
    echo "$scene end $(date '+%F %T') exit $code" >> "$OUT/batch.log"
    (( code == 0 )) || status=1
done
echo "$status" > "$OUT/batch.exit"
exit "$status"
