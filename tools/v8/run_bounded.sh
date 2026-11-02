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
import os
from pathlib import Path
limit = int(sys.argv[1])
if 'WORLDSIM_STALL_CHILD_PLAN' in os.environ or 'WORLDSIM_STALL_CHILD_PLAN_SHA256' in os.environ:
    sys.path.insert(0, str(Path.cwd() / 'tools/v8'))
    from gate_stall_child import run
    raise SystemExit(run(Path(os.environ['WORLDSIM_STALL_CHILD_PLAN']),
                         os.environ['WORLDSIM_STALL_CHILD_PLAN_SHA256'], tuple(sys.argv[2:]), limit))
try:
    raise SystemExit(subprocess.run(sys.argv[2:], timeout=limit, check=False).returncode)
except subprocess.TimeoutExpired:
    print(f"command exceeded the {limit}-second limit", file=sys.stderr)
    raise SystemExit(124)
PY
