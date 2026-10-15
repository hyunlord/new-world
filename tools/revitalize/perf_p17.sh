#!/usr/bin/env bash
# usage: perf_p17.sh OUT FINAL_BIN BASE_BIN a|b FINAL_SHA BASE_SHA QUIET_LOG
# Run inside quiet_lock_p17.sh. Each stage compares three interleaved same-build
# repeats (a: v21/v20; b: v22/v21), then before/after quick and three-size slopes.
set -uo pipefail
(( $# == 7 )) || { echo 'usage: perf_p17.sh OUT FINAL_BIN BASE_BIN a|b FINAL_SHA BASE_SHA QUIET_LOG' >&2; exit 2; }
OUT="$1"; FINAL="$2"; BASE="$3"; STAGE="$4"; FINAL_SHA="$5"; BASE_SHA="$6"; QUIET_LOG="$7"
case "$STAGE" in a) OLD=game_v20; NEW=game_v21 ;; b) OLD=game_v21; NEW=game_v22 ;; *) echo 'stage must be a or b' >&2; exit 2 ;; esac
[[ -n "${P17_QUIET_PID:-}" && "${P17_QUIET_OWNER:-}" == 'pass17-sim (Codex, new-world-pass4)' ]] || { echo 'timed performance requires quiet_lock_p17.sh' >&2; exit 2; }
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd -P)"
FINAL="$(cd "$FINAL" && pwd -P)" || exit 2
BASE="$(cd "$BASE" && pwd -P)" || exit 2
case "$OUT/" in "$ROOT/docs/revitalize/"*) ;; "$ROOT/"*) echo 'performance outputs must be outside the repository or in docs/revitalize' >&2; exit 2 ;; esac
provenance() {
    python3 - "$OUT" "$STAGE" "$FINAL/worldsim_perf" "$BASE/worldsim_perf" "$FINAL_SHA" "$BASE_SHA" "$QUIET_LOG" "$@" <<'PY'
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import time

root, stage, final, base, final_sha, base_sha, quiet_path, operation, *args = sys.argv[1:]
root = Path(root)
manifest_path = root / 'provenance.json'
ledger = root / 'commands.jsonl'
pending = root / 'command_pending.json'


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


if operation == 'init':
    if any((root / name).exists() for name in ('provenance.json', 'commands.jsonl', 'command_pending.json', 'runs.log', 'a', 'quick_base', 'quick_final', 'b_game_v20', 'b_game_v21', 'b_game_v22')):
        sys.exit('refusing to reuse performance evidence directory')
    if not all(re.fullmatch('[0-9a-f]{40}', sha) for sha in (final_sha, base_sha)):
        sys.exit('full final and baseline commit SHAs are required')
    info = dict(line.split('=', 1) for line in Path('/Users/rexxa/work/.mac_quiet.lock/info.txt').read_text().splitlines())
    if any(info[key] != os.environ[f'P17_QUIET_{key.upper()}'] for key in ('owner', 'pid', 'started')):
        sys.exit('quiet lock ownership changed')
    quiet_log = Path(quiet_path).resolve()
    prefix = quiet_log.read_bytes()
    contention = os.environ.get('PERF_CONTENDED')
    if contention not in ('0', '1') or f'HOLD start: {info["purpose"]};' not in prefix.decode():
        sys.exit('quiet hold evidence missing')
    manifest = {'schema_version': 1, 'stage': stage, 'status': None,
                'builds': {role: {'path': path, 'commit': sha, 'sha256': digest(path)} for role, path, sha in (('final', final, final_sha), ('baseline', base, base_sha))},
                'quiet': info | {'log': str(quiet_log), 'prefix_bytes': len(prefix), 'prefix_sha256': hashlib.sha256(prefix).hexdigest(), 'contended': contention == '1'}}
    ledger.touch()
elif operation == 'start':
    manifest = json.loads(manifest_path.read_text())
    role, *argv = args
    before = digest(argv[0])
    if before != manifest['builds'][role]['sha256']:
        sys.exit('binary changed before command')
    row = {'index': len(ledger.read_text().splitlines()), 'build': role, 'argv': argv, 'started': time.time(), 'binary_sha256_before': before}
    pending.write_text(json.dumps(row) + '\n')
    sys.exit(0)
elif operation == 'end':
    row = json.loads(pending.read_text())
    argv = row['argv']
    mode = argv[1]
    version = argv[argv.index('--rules') + 1].removeprefix('game_v') if mode == 'stress' else '19'
    sizes = argv[argv.index('--agents') + 1].split(',') if mode == 'stress' else ['500']
    directory = Path(argv[argv.index('--out') + 1])
    paths = [directory / f'{mode}_stress_v{version}_s15100_n{size}_side64_fed.json' for size in sizes]
    row.update({'finished': time.time(), 'exit_code': int(args[0]), 'binary_sha256_after': digest(argv[0]), 'outputs': {str(path.relative_to(root)): digest(path) for path in paths if path.exists()}})
    with ledger.open('a') as stream:
        stream.write(json.dumps(row) + '\n')
    pending.unlink()
    sys.exit(0 if row['binary_sha256_before'] == row['binary_sha256_after'] else 1)
elif operation == 'finish':
    manifest = json.loads(manifest_path.read_text())
    manifest.update({'status': int(args[0]), 'commands': {'file': ledger.name, 'sha256': digest(ledger)}, 'runlog': {'file': 'RUNLOG.txt', 'sha256': digest(root / 'RUNLOG.txt')}, 'indexes': {str(path.relative_to(root)): digest(path) for path in root.glob('**/runs.json')}})
else:
    sys.exit('unknown provenance operation')
manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
PY
}
provenance init || exit 2
status=0
run() {
    local role="$1" code=0 sha="$FINAL_SHA"
    shift
    [[ "$role" != baseline ]] || sha="$BASE_SHA"
    bash "$SCRIPT_DIR/wait_mac_p17.sh" "$OUT/RUNLOG.txt" "stage $STAGE performance" || return 1
    local info=/Users/rexxa/work/.mac_quiet.lock/info.txt
    [[ "${P17_QUIET_PID:-}" =~ ^[0-9]+$ ]] && kill -0 "$P17_QUIET_PID" 2>/dev/null &&
        grep -Fxq -- "owner=$P17_QUIET_OWNER" "$info" && grep -Fxq -- "pid=$P17_QUIET_PID" "$info" &&
        grep -Fxq -- "started=${P17_QUIET_STARTED:-}" "$info" || { echo 'owned quiet lock is no longer held' >&2; return 1; }
    printf '%s contended=%s ' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "${PERF_CONTENDED:-0}" >> "$OUT/runs.log"
    printf '%q ' "$@" >> "$OUT/runs.log"; printf '\n' >> "$OUT/runs.log"
    provenance start "$role" "$@" || return 1
    WORLDSIM_GIT_SHA="$sha" "$@" >> "$OUT/runs.log" 2>&1 || code=$?
    provenance end "$code" || return 1
    (( code == 0 )) || status=1
    return 0
}
for repeat in 1 2 3; do
    for rules in "$OLD" "$NEW"; do
        run final "$FINAL/worldsim_perf" stress --seeds 15100..15101 --agents 500 --side 64 --days 1 --feed fed --rules "$rules" --out "$OUT/a/r$repeat" || exit 1
    done
done
run baseline "$BASE/worldsim_perf" quick --out "$OUT/quick_base" || exit 1
run final "$FINAL/worldsim_perf" quick --out "$OUT/quick_final" || exit 1
for rules in "$NEW" "$OLD"; do
    run final "$FINAL/worldsim_perf" stress --seeds 15100..15101 --agents 100,500,1000 --side 64 --days 2 --feed fed --rules "$rules" --out "$OUT/b_$rules" || exit 1
done
echo "status $status" >> "$OUT/runs.log"
provenance finish "$status" || exit 1
exit "$status"
