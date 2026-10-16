#!/usr/bin/env bash
set -euo pipefail
python3 - "$@" <<'PY'
import subprocess, sys
try:
    raise SystemExit(subprocess.run(sys.argv[2:], timeout=int(sys.argv[1]), check=False).returncode)
except subprocess.TimeoutExpired:
    print(f"command exceeded {sys.argv[1]} s", file=sys.stderr)
    raise SystemExit(124)
PY
