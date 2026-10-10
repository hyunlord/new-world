#!/usr/bin/env bash
# usage: with_window_lock.sh <timeout_seconds> <command...>
set -euo pipefail
LOCK=/Users/rexxa/work/.worldsim-godot-window.lock
limit="$1"; shift
until mkdir "$LOCK" 2>/dev/null; do
  age=$(( $(date +%s) - $(stat -f %m "$LOCK") ))
  if (( age > 1800 )); then rmdir "$LOCK" 2>/dev/null || true; echo "stale window lock removed (age ${age}s)" >&2
  else sleep 60; fi
done
trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT
python3 - "$limit" "$@" <<'PY'
import subprocess, sys
limit = int(sys.argv[1])
try:
    raise SystemExit(subprocess.run(sys.argv[2:], timeout=limit, check=False).returncode)
except subprocess.TimeoutExpired:
    print(f"command exceeded {limit} s", file=sys.stderr)
    raise SystemExit(124)
PY
