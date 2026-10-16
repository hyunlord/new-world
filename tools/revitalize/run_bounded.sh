#!/usr/bin/env bash
# usage: run_bounded.sh <timeout_seconds> <command...>
# Portable replacement for GNU `timeout` (macOS has none): exits with the
# command's status, or 124 when the limit is exceeded.
set -euo pipefail
(( $# >= 2 )) || { echo 'usage: run_bounded.sh <timeout_seconds> <command...>' >&2; exit 2; }
limit="$1"; shift
[[ "$limit" =~ ^[0-9]+$ ]] || { echo 'timeout must be whole seconds' >&2; exit 2; }
exec python3 - "$limit" "$@" <<'PY'
import subprocess
import sys
limit = int(sys.argv[1])
try:
    raise SystemExit(subprocess.run(sys.argv[2:], timeout=limit, check=False).returncode)
except subprocess.TimeoutExpired:
    print(f"command exceeded the {limit}-second limit", file=sys.stderr)
    raise SystemExit(124)
PY
