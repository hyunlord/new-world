#!/usr/bin/env bash
# usage: export.sh [--no-build] <worldsim_wm arguments>   (see `worldsim_wm --help`)
# Builds the release recorder (unless --no-build or WM_BIN is set) and runs it.
# --out must lie under $WM_DATA_ROOT (default /Users/rexxa/work/worldsim-wm-data)
# or under .harness/evidence/worldmodel/ of this clone.
set -euo pipefail
cd "$(dirname "$0")/../.."
build=1
args=()
out=""
while (( $# )); do
    case "$1" in
        --help|-h) echo 'usage: export.sh [--no-build] --scene S --seeds a..b --days N --every TICKS [--agents] [--schedule-dir DIR [--include-factual]] [--resume] --workers K --out DIR'; exit 0 ;;
        --no-build) build=0; shift ;;
        --out) out="${2:?missing output directory}"; args+=("$1" "$2"); shift 2 ;;
        *) args+=("$1"); shift ;;
    esac
done
[[ -n "$out" ]] || { echo '--out is required' >&2; exit 1; }
resolve_path() { python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$1"; }
data_root=$(resolve_path "${WM_DATA_ROOT:-/Users/rexxa/work/worldsim-wm-data}")
evidence_root=$(resolve_path .harness/evidence/worldmodel)
mkdir -p "$out"
real_out=$(resolve_path "$out")
[[ "$real_out" == "$data_root"/* || "$real_out" == "$data_root" || "$real_out" == "$evidence_root"/* || "$real_out" == "$evidence_root" ]] \
    || { echo "--out must lie under $data_root or $evidence_root" >&2; exit 1; }
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-6}"
bin="${WM_BIN:-rust/target/release/worldsim_wm}"
if [[ -z "${WM_BIN:-}" && "$build" == 1 ]]; then
    (cd rust && cargo build --release -p sim-test --bin worldsim_wm)
fi
export WORLDSIM_GIT_SHA="${WORLDSIM_GIT_SHA:-$(git rev-parse HEAD)}"
"$bin" "${args[@]}"
