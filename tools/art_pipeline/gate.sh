#!/usr/bin/env bash
# Full production gate; no early exit so every failure remains visible.
set -uo pipefail
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
export GODOT_BIN="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
root="$(git rev-parse --show-toplevel)"
cd "$root"
label="${1:-final}"
out="${GATE_OUTPUT_DIR:-docs/art/production/gates/$label}"
mkdir -p "$out"
status=0
run_check() {
 local name="$1"; shift
 local start="$(date +%s)"
 bash tools/art_pipeline/run_bounded.sh 1800 "$@" > "$out/$name.log" 2>&1
 local code=$?
 printf '%s\t%s\t%s\n' "$name" "$code" "$(( $(date +%s)-start ))" >> "$out/results.tsv"
 if (( code != 0 )); then status=1; fi
}
: > "$out/results.tsv"
run_check cargo_test cargo test --manifest-path rust/Cargo.toml --workspace -- --test-threads="${CARGO_BUILD_JOBS}"
run_check clippy cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets -- -D warnings
run_check strict bash tools/harness/gdscript_strict_check.sh
run_check bridge cargo build --manifest-path rust/Cargo.toml -p sim-bridge
run_check smoke bash tools/art_pipeline/run_bounded.sh 600 "$GODOT_BIN" --headless --path . --quit-after 900
if grep -E 'SCRIPT ERROR|ERROR:' "$out/smoke.log" | grep -viE 'alsa|audio' > "$out/smoke_errors.log"; then
 printf 'smoke_errors\t1\t0\n' >> "$out/results.tsv"; status=1
else printf 'smoke_errors\t0\t0\n' >> "$out/results.tsv"; fi
run_check python python3 -m unittest discover -s tools/art_pipeline -p 'test_*.py'
shopt -s nullglob
for scene_test in scripts/test/art/test_*.gd; do
 if [[ "$scene_test" == "scripts/test/art/test_real_input_p12.gd" || "$scene_test" == "scripts/test/art/test_real_input_p13.gd" ]]; then
  real_input_parent="$(mktemp -d "$(cd "$out" && pwd)/real-input.XXXXXX")"
  run_check "$(basename "$scene_test" .gd)" env P12_REAL_INPUT_OUT="$real_input_parent/capture" P13_REAL_INPUT_OUT="$real_input_parent/capture" bash tools/art_pipeline/with_window_lock.sh 600 "$GODOT_BIN" --path . --script "res://$scene_test"
 elif [[ "$scene_test" == "scripts/test/art/test_glyph_distinct_p13.gd" ]]; then
  glyph_output="${P13_GLYPH_OUT:-$(mktemp -d "${TMPDIR:-/tmp}/worldsim-p13-glyph.XXXXXX")}"
  run_check "$(basename "$scene_test" .gd)" env P13_GLYPH_OUT="$glyph_output" bash tools/art_pipeline/run_bounded.sh 300 "$GODOT_BIN" --headless --path . --script "res://$scene_test"
 else
  run_check "$(basename "$scene_test" .gd)" bash tools/art_pipeline/run_bounded.sh 300 "$GODOT_BIN" --headless --path . --script "res://$scene_test"
 fi
done
cat "$out/results.tsv"
exit "$status"
