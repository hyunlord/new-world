#!/usr/bin/env bash
# M-UX B3: fresh-process seven-family simulation and real selected-scene UI proof.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
OUT="${1:?usage: data_only_ext.sh OUT}"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd -P)"
cd "$ROOT"
source tools/v8/godot_bin.sh
(
    cd rust
    cargo test -p sim-test --test data_only_ext -- --test-threads=1
) > "$OUT/simulation.log" 2>&1
cat "$OUT/simulation.log"
WORLDSIM_CONTENT_PACK="$ROOT/rust/crates/sim-test/data/ext_pack/pack.ron" \
    bash tools/v8/run_bounded.sh 600 "$GODOT_BIN" --headless --path "$ROOT" \
    --script res://tools/v8/data_only_ext_test.gd -- "$OUT/ui" > "$OUT/ui.log" 2>&1
cat "$OUT/ui.log"
if grep -Eq 'SCRIPT ERROR|Parse Error|DATA_ONLY_EXT_FAIL' "$OUT/ui.log"; then
    exit 1
fi
python3 - "$OUT/ui/data-only-ext-results.json" <<'PY'
import json
import sys
from pathlib import Path
report = json.loads(Path(sys.argv[1]).read_text())
assert report['failures'] == 0, report
checks = [row for row in report['checks'] if 'passed' in row]
assert len(checks) >= 20 and all(row['passed'] for row in checks), report
print('DATA_ONLY_EXT PASS: seven-family simulation and selected-scene UI; rendered capture is separate')
PY
