#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
label=""
scene=legacy
out=""
probe_args=()
while (( $# )); do
    case "$1" in
        --help|-h)
            echo 'usage: probe.sh [--label LABEL] --scene legacy|game|game_v9|game_v10|game_v11|game_v12|game_v13|game_v14|game_v15|game_v16_neutral|game_v16|game_v17_nofission|game_v17|game_v18_nofission|game_v18|game_v19_nofission|game_v19 --seeds a..b --days N --sample-every N --workers 1..8 [--out .harness/evidence/revitalize/probe/DIR] [--check-determinism]'
            echo 'Default output and summary label: scene name. --out must remain under the raw ignored probe directory.'
            exit 0 ;;
        --label) label="${2:?missing label}"; shift 2 ;;
        --out) out="${2:?missing output directory}"; shift 2 ;;
        --scene) scene="${2:?missing scene}"; probe_args+=("$1" "$2"); shift 2 ;;
        *) probe_args+=("$1"); shift ;;
    esac
done
label="${label:-$scene}"
[[ "$label" =~ ^[a-zA-Z0-9_-]+$ ]] || { echo 'invalid output label' >&2; exit 1; }
resolve_path() { python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$1"; }
raw_root=$(resolve_path .harness/evidence/revitalize/probe)
out=$(resolve_path "${out:-$raw_root/$label}")
[[ "$out" == "$raw_root" || "$out" == "$raw_root/"* ]] || { echo '--out must remain under .harness/evidence/revitalize/probe' >&2; exit 1; }
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-10}"
workers_override=()
host_os=$(uname -s)
check_memory() {
    local attempt available
    # macOS has no `free`; the shared Mac is managed by the caller's job limits.
    [[ "$host_os" == Darwin ]] && return
    for attempt in {0..10}; do
        free -g
        available=$(free -g | awk '/^Mem:/ {print $7}')
        if (( available >= 24 )); then return; fi
        if (( attempt < 10 )); then sleep 60; fi
    done
    export CARGO_BUILD_JOBS=4
    workers_override=(--workers 2)
    echo 'Low memory after ten waits: using four build jobs and two probe workers' >&2
}
if [[ "$host_os" == Darwin ]]; then disk_path=.; else disk_path=/home; fi
df -h "$disk_path"
available_kb=$(df -Pk "$disk_path" | awk 'NR==2 {print $4}')
(( available_kb >= 15 * 1024 * 1024 )) || { echo 'Less than 15 GiB disk free; reclaim space only in this dedicated clone before continuing' >&2; exit 1; }
check_memory
(cd rust && cargo build --release -p sim-test --bin worldsim_probe)
check_memory
export WORLDSIM_GIT_SHA
WORLDSIM_GIT_SHA=$(git rev-parse HEAD)
mkdir -p "$out"
rust/target/release/worldsim_probe "${probe_args[@]}" ${workers_override[@]+"${workers_override[@]}"} --out "$out"
mkdir -p "docs/revitalize/probe/$label"
cp "$out/summary.json" "$out/summary.md" "docs/revitalize/probe/$label/"
