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
            echo 'usage: probe.sh [--label LABEL] --scene legacy|game --seeds a..b --days N --sample-every N --workers 1..8 [--out .harness/evidence/revitalize/probe/DIR] [--check-determinism]'
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
raw_root=$(realpath -m .harness/evidence/revitalize/probe)
out=$(realpath -m "${out:-$raw_root/$label}")
[[ "$out" == "$raw_root" || "$out" == "$raw_root/"* ]] || { echo '--out must remain under .harness/evidence/revitalize/probe' >&2; exit 1; }
export CARGO_BUILD_JOBS=10
workers_override=()
check_memory() {
    local attempt available
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
df -h /home
available_kb=$(df -Pk /home | awk 'NR==2 {print $4}')
(( available_kb >= 15 * 1024 * 1024 )) || { echo 'Less than 15 GiB disk free; reclaim space only in this dedicated clone before continuing' >&2; exit 1; }
check_memory
(cd rust && cargo build --release -p sim-test --bin worldsim_probe)
check_memory
export WORLDSIM_GIT_SHA
WORLDSIM_GIT_SHA=$(git rev-parse --short HEAD)
mkdir -p "$out"
rust/target/release/worldsim_probe "${probe_args[@]}" "${workers_override[@]}" --out "$out"
mkdir -p "docs/revitalize/probe/$label"
cp "$out/summary.json" "$out/summary.md" "docs/revitalize/probe/$label/"
