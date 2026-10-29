#!/usr/bin/env bash
# M6a explicit dispatch. Original CORE checks retain their original contract.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
mode="${1:?expected M6a check mode}"
shift
[[ $# == 1 ]] || exit 2
case "$mode" in
    fixture_run) mode=fixtures ;;
    evaluation|food_fixture|data_only|preflight|final) ;;
    *) echo "unknown M6a check: $mode" >&2; exit 2 ;;
esac
exec python3 tools/v8/m6a_gate.py "$mode" "$1"
