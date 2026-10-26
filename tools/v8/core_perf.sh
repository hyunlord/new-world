#!/usr/bin/env bash
# Source-only controller: never builds. Freeze inputs after building, then run.
# usage: core_perf.sh CONFIG.json NEW_SESSION_DIR
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec python3 "$ROOT/tools/v8/core_perf_check.py" run "${1:?config required}" "${2:?new session directory required}"
