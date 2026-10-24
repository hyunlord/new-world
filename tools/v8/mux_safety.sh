#!/usr/bin/env bash
# Capture/compare the MUX refactor safety net without building or touching source.
# capture BIN_DIR INPUT_DIR OUT_DIR [--steward-off]: freeze probe/inputs, capture
# baseline identity and nine colony fixtures. OUT_DIR must be new and external.
# compare BASE_DIR CAND_DIR: exact colony bytes and existing identity comparators.
# Baseline probes predate the steward flag; candidate probes must explicitly opt off.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
VALIDATOR="$ROOT/tools/v8/mux_safety_validate.py"
SCRIPTS=(soak_script.json m5_u_script.json m55_u_script.json m4_f10_script.json m55_u_nodraft_script.json)

fail() { printf 'MUX_SAFETY FAIL %s\n' "$*" >&2; exit 1; }
usage() {
    printf 'usage: mux_safety.sh capture BIN_DIR INPUT_DIR OUT_DIR [--steward-off]\n' >&2
    printf '       mux_safety.sh compare BASE_DIR CAND_DIR\n' >&2
    exit 2
}

external_directory() {
    local directory
    directory="$(cd "$1" && pwd -P)" || fail "missing directory: $1"
    case "$directory/" in "$ROOT/"*) fail "raw data must be outside checkout: $directory" ;; esac
    printf '%s\n' "$directory"
}

capture() {
    [[ $# == 3 || $# == 4 ]] || usage
    local bin inputs requested parent out flag implicit script
    bin="$(cd "$1" && pwd -P)" || fail "missing binary directory: $1"
    inputs="$(cd "$2" && pwd -P)" || fail "missing input directory: $2"
    requested="$3"
    flag="${4:-}"
    [[ -z "$flag" || "$flag" == --steward-off ]] || usage
    [[ -x "$bin/worldsim_probe" ]] || fail "missing executable worldsim_probe"
    [[ "$requested" == /* ]] || fail 'OUT_DIR must be absolute'
    [[ ! -e "$requested" && ! -L "$requested" ]] || fail "refusing existing output: $requested"
    parent="$(external_directory "$(dirname "$requested")")"
    out="$parent/$(basename "$requested")"
    [[ "$(basename "$requested")" != . && "$(basename "$requested")" != .. ]] || fail 'invalid output name'
    mkdir "$out" || fail "cannot create output: $out"
    mkdir "$out/bin" "$out/inputs"
    cp "$bin/worldsim_probe" "$out/bin/worldsim_probe"
    for script in "${SCRIPTS[@]}"; do
        [[ -f "$inputs/$script" ]] || fail "missing fixture input: $script"
        cp "$inputs/$script" "$out/inputs/$script"
    done
    (cd "$out/inputs" && shasum -a 256 "${SCRIPTS[@]}") > "$out/input_sha256.txt"
    (cd "$out/bin" && shasum -a 256 worldsim_probe) > "$out/probe_sha256.txt"
    git -C "$ROOT" rev-parse HEAD > "$out/capture_checkout_sha.txt"
    implicit='off (legacy probe, no steward argument)'
    [[ "$flag" != --steward-off ]] || implicit='off (explicit --steward off)'
    printf '%s\n' "$implicit" > "$out/steward_mode.txt"
    python3 "$VALIDATOR" list > "$out/fixtures.tsv"
    WORLDSIM_GIT_SHA="$(cat "$out/capture_checkout_sha.txt")" \
        bash "$ROOT/tools/v8/identity.sh" --capture "$out/bin" "$out/identity" > "$out/identity.log" 2>&1
    local name count days script story neighbours twin
    local args=()
    while IFS=$'\t' read -r name count days script story neighbours twin; do
        args=(--scene colony --seeds "0..$count" --days "$days" --workers 4
            --commands "$out/inputs/$script" --story "$story" --neighbours "$neighbours"
            --out "$out/colony/$name")
        [[ "$twin" != yes ]] || args+=(--check-determinism)
        [[ "$flag" != --steward-off ]] || args+=(--steward off)
        printf 'MUX_SAFETY capture %s\n' "$name"
        "$out/bin/worldsim_probe" "${args[@]}" > "$out/$name.log" 2>&1
    done < "$out/fixtures.tsv"
    python3 "$VALIDATOR" validate "$out"
    grep -q '^DETERMINISM PASS' "$out/soak.log" || fail 'soak determinism marker missing'
    printf 'MUX_SAFETY capture PASS %s\n' "$out" | tee "$out/capture_complete.txt"
}

compare() {
    [[ $# == 2 ]] || usage
    local base candidate name count days script story neighbours twin directory
    base="$(external_directory "$1")"
    candidate="$(external_directory "$2")"
    [[ "$base" != "$candidate" ]] || fail 'cannot compare a capture with itself'
    for directory in "$base" "$candidate"; do
        [[ -s "$directory/capture_complete.txt" ]] || fail "incomplete capture: $directory"
        python3 "$VALIDATOR" validate "$directory"
        (cd "$directory/inputs" && shasum -a 256 -c ../input_sha256.txt)
        (cd "$directory/bin" && shasum -a 256 -c ../probe_sha256.txt)
        grep -q '^DETERMINISM PASS' "$directory/soak.log" || fail 'missing soak determinism'
        grep -q '^DETERMINISM PASS' "$directory/identity/determinism.txt" || fail 'missing identity determinism'
        if grep -q 'DETERMINISM FAIL' "$directory/identity/determinism.txt" "$directory/soak.log"; then
            fail "determinism failure: $directory"
        fi
    done
    for script in "${SCRIPTS[@]}"; do
        cmp "$base/inputs/$script" "$candidate/inputs/$script" || fail "fixture changed: $script"
    done
    while IFS=$'\t' read -r name count days script story neighbours twin; do
        cmp "$base/colony/$name/summary.json" "$candidate/colony/$name/summary.json" || fail "colony differs: $name"
        printf 'MUX_SAFETY identical %s\n' "$name"
    done < <(python3 "$VALIDATOR" list)
    python3 "$ROOT/tools/v8/compare_summaries.py" "$base/identity/probe/summary.json" \
        "$candidate/identity/probe/summary.json" --removed-keys "$ROOT/tools/v8/identity_removed_keys.txt"
    python3 "$ROOT/tools/v8/compare_chronicles.py" "$base/identity/chronicle" "$candidate/identity/chronicle"
    local base_digest candidate_digest
    base_digest="$(bash "$ROOT/tools/v8/identity.sh" --raw-digest "$base/identity/probe")"
    candidate_digest="$(bash "$ROOT/tools/v8/identity.sh" --raw-digest "$candidate/identity/probe")"
    [[ -n "$base_digest" && "$base_digest" == "$candidate_digest" ]] || fail 'baseline raw sample digests differ or are empty'
    printf 'MUX_SAFETY compare PASS: baseline identity and 9 colony summaries\n'
}

[[ $# -ge 1 ]] || usage
mode="$1"; shift
case "$mode" in
    capture) capture "$@" ;;
    compare) compare "$@" ;;
    *) usage ;;
esac
