#!/usr/bin/env bash
# usage: perf_p20.sh OUT FINAL_BIN BASE_BIN FINAL_SHA BASE_SHA NATURAL_SEEDS
# Run inside quiet_lock_p20.sh (timed measurements only). Pass 20 T6 (brief
# 4.5, 7.2): game_v25 against the new baseline game_v24.
#   1. stress: `stress --agents 500 --side 64 --days 1 --feed fed`, FIVE
#      interleaved repeats (v24 then v25), final build (Pass 19's procedure);
#   2. quick on the base and the final build;
#   3. the three-size slopes (100/500/1,000, 2 days) of v25 and v24;
#   4. natural: `natural --days 120` of each seed in NATURAL_SEEDS
#      (comma-separated; the 4 lowest evaluation seeds with >= 2 settlements
#      alive at day 60 in game_v25), three interleaved repeats (v24 then v25).
# Every command, its binary's sha256 before and after, and its exit code go to
# OUT/commands.jsonl; the quiet-hold identity goes to OUT/provenance.json.
set -uo pipefail
(( $# == 6 )) || { echo 'usage: perf_p20.sh OUT FINAL_BIN BASE_BIN FINAL_SHA BASE_SHA NATURAL_SEEDS' >&2; exit 2; }
OUT="$1"; FINAL="$2"; BASE="$3"; FINAL_SHA="$4"; BASE_SHA="$5"; NATURAL="$6"; STAGE=t6
[[ -n "${P20_QUIET_PID:-}" && "${P20_QUIET_OWNER:-}" == 'pass20-sim (Claude Code, new-world-pass4)' ]] || { echo 'timed performance requires quiet_lock_p20.sh' >&2; exit 2; }
[[ ! -e "$OUT/commands.jsonl" ]] || { echo "refusing to reuse $OUT" >&2; exit 2; }
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd -P)"
FINAL="$(cd "$FINAL" && pwd -P)" || exit 2
BASE="$(cd "$BASE" && pwd -P)" || exit 2
python3 - "$OUT" "$STAGE" "$FINAL" "$BASE" "$FINAL_SHA" "$BASE_SHA" <<'PY' || exit 2
import hashlib, json, os, sys
from pathlib import Path
out, stage, final, base, final_sha, base_sha = sys.argv[1:]
info = dict(line.split('=', 1) for line in Path('/Users/rexxa/work/.mac_quiet.lock/info.txt').read_text().splitlines())
if any(info[k] != os.environ[f'P20_QUIET_{k.upper()}'] for k in ('owner', 'pid', 'started')):
    sys.exit('quiet lock ownership changed')
digest = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
Path(out, 'provenance.json').write_text(json.dumps({'stage': stage, 'quiet': info,
    'contended': os.environ.get('PERF_CONTENDED') == '1',
    'builds': {'final': {'dir': final, 'commit': final_sha, 'worldsim_perf_sha256': digest(Path(final, 'worldsim_perf'))},
               'base': {'dir': base, 'commit': base_sha, 'worldsim_perf_sha256': digest(Path(base, 'worldsim_perf'))}}}, indent=2) + '\n')
PY
status=0
run() {
    local role="$1" sha="$FINAL_SHA" bin code=0 before after started
    shift
    [[ "$role" == final ]] || sha="$BASE_SHA"
    bin="$1"
    local info=/Users/rexxa/work/.mac_quiet.lock/info.txt
    grep -Fxq -- "pid=$P20_QUIET_PID" "$info" && grep -Fxq -- "started=$P20_QUIET_STARTED" "$info" || { echo 'owned quiet lock is no longer held' >&2; return 1; }
    before=$(shasum -a 256 "$bin" | cut -d' ' -f1)
    started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    printf '%s %s ' "$started" "$role" >> "$OUT/runs.log"; printf '%q ' "$@" >> "$OUT/runs.log"; printf '\n' >> "$OUT/runs.log"
    WORLDSIM_GIT_SHA="$sha" "$@" >> "$OUT/runs.log" 2>&1 || code=$?
    after=$(shasum -a 256 "$bin" | cut -d' ' -f1)
    python3 -c 'import json,sys; print(json.dumps({"started": sys.argv[1], "role": sys.argv[2], "exit": int(sys.argv[3]), "sha_before": sys.argv[4], "sha_after": sys.argv[5], "argv": sys.argv[6:]}))' \
        "$started" "$role" "$code" "$before" "$after" "$@" >> "$OUT/commands.jsonl"
    (( code == 0 )) && [[ "$before" == "$after" ]] || status=1
    return 0
}
STRESS=(stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed)
for repeat in 1 2 3 4 5; do
    for rules in game_v24 game_v25; do
        run final "$FINAL/worldsim_perf" "${STRESS[@]}" --rules "$rules" --out "$OUT/r$repeat/$rules" || exit 1
    done
done
run base "$BASE/worldsim_perf" quick --out "$OUT/quick_base" || exit 1
run final "$FINAL/worldsim_perf" quick --out "$OUT/quick_final" || exit 1
for rules in game_v25 game_v24; do
    run final "$FINAL/worldsim_perf" stress --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed --rules "$rules" --out "$OUT/slope_$rules" || exit 1
done
IFS=, read -r -a SEEDS <<< "$NATURAL"
for repeat in 1 2 3; do
    for seed in "${SEEDS[@]}"; do
        for scene in game_v24 game_v25; do
            run final "$FINAL/worldsim_perf" natural --scene "$scene" --seeds "$seed..$((seed + 1))" --days 120 --out "$OUT/nat_r$repeat/$scene" || exit 1
        done
    done
done
echo "status $status" >> "$OUT/runs.log"
exit "$status"
