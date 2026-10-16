#!/usr/bin/env bash
# usage: scale_p19.sh OUT BIN SUBPROFILE_BIN SHA [repeats]
# Pass 19 Part 3 (brief 8): game_v24 stress worlds of 500, 1,000 and 2,000
# founders on a 64 x 64 map and 2,000 on 128 x 128, two days, fed; ONE process
# per (size, repeat), repeats interleaved (default 1 2 3), each under
# /usr/bin/time -l (macOS BSD time: maximum resident set size in bytes); then
# the 2,000 @ 64 run once with the `subprofile` build. Run inside
# quiet_lock_p19.sh; every command goes to OUT/commands.jsonl.
set -uo pipefail
(( $# >= 4 )) || { echo 'usage: scale_p19.sh OUT BIN SUBPROFILE_BIN SHA [repeats...]' >&2; exit 2; }
OUT="$1"; BIN="$2"; SUB="$3"; SHA="$4"; shift 4
REPEATS=("$@"); (( ${#REPEATS[@]} )) || REPEATS=(1 2 3)
[[ -n "${P19_QUIET_PID:-}" && "${P19_QUIET_OWNER:-}" == 'pass19-sim (Claude Code, new-world-pass4)' ]] || { echo 'timed performance requires quiet_lock_p19.sh' >&2; exit 2; }
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd -P)"
status=0
run() {
    local label="$1" bin="$2"; shift 2
    local dir="$OUT/$label" code=0 before after started
    [[ ! -e "$dir" ]] || { echo "refusing to reuse $dir" >&2; return 1; }
    mkdir -p "$dir"
    grep -Fxq -- "pid=$P19_QUIET_PID" /Users/rexxa/work/.mac_quiet.lock/info.txt || { echo 'owned quiet lock is no longer held' >&2; return 1; }
    before=$(shasum -a 256 "$bin" | cut -d' ' -f1)
    started=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    printf '%s %s ' "$started" "$label" >> "$OUT/runs.log"; printf '%q ' "$bin" "$@" >> "$OUT/runs.log"; printf '\n' >> "$OUT/runs.log"
    WORLDSIM_GIT_SHA="$SHA" /usr/bin/time -l "$bin" "$@" --out "$dir" > "$dir/stdout.log" 2> "$dir/time.log" || code=$?
    after=$(shasum -a 256 "$bin" | cut -d' ' -f1)
    python3 -c 'import json,sys; print(json.dumps({"started": sys.argv[1], "label": sys.argv[2], "exit": int(sys.argv[3]), "sha_before": sys.argv[4], "sha_after": sys.argv[5], "argv": sys.argv[6:]}))' \
        "$started" "$label" "$code" "$before" "$after" "$bin" "$@" >> "$OUT/commands.jsonl"
    (( code == 0 )) && [[ "$before" == "$after" ]] || status=1
    return 0
}
COMMON=(stress --seeds 15100..15101 --days 2 --feed fed --rules game_v24)
for repeat in "${REPEATS[@]}"; do
    for spec in 500:64 1000:64 2000:64 2000:128; do
        n="${spec%%:*}"; side="${spec#*:}"
        run "r$repeat/n${n}_side$side" "$BIN/worldsim_perf" "${COMMON[@]}" --agents "$n" --side "$side" || exit 1
    done
done
if [[ -n "$SUB" && "$SUB" != - ]]; then
    run "subprofile/n2000_side64" "$SUB/worldsim_perf" "${COMMON[@]}" --agents 2000 --side 64 || exit 1
fi
echo "status $status" >> "$OUT/runs.log"
exit "$status"
