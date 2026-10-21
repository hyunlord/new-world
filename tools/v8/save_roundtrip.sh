#!/usr/bin/env bash
# usage: save_roundtrip.sh
#
# M1.5 save/load gate step (contract clause C1, goal §7): runs the save tests
# in rust/crates/sim-test/tests/save_load.rs in release — the colony round trip
# on seeds 0-7 (tick 2,000 → save → load → tick 1,000 against a continuous
# run), the baseline round trips (two seeds, and one through deaths and
# births with two chained loads), byte-identical saves, and the rejection of
# another format version. Prints SAVE_ROUNDTRIP PASS or FAIL; exits 0 / 1.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT/rust" || exit 1
out="$(cargo test --release -p sim-test --test save_load -- --test-threads=8 2>&1)"
status=$?
printf '%s\n' "$out"
if [[ $status -eq 0 ]] && printf '%s\n' "$out" | grep -q '^test result: ok\. [1-9][0-9]* passed; 0 failed'; then
  echo "SAVE_ROUNDTRIP PASS"
  exit 0
fi
echo "SAVE_ROUNDTRIP FAIL"
exit 1
