#!/usr/bin/env bash
# M-UX steward gate: two no-player-order worlds, 24 days, replay determinism and exact ledgers.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
OUT="${1:?usage: mux_steward.sh OUT}"
mkdir -p "$OUT"
"$ROOT/rust/target/release/worldsim_probe" --scene colony --seeds 0..2 --days 24 \
  --workers 2 --steward on --check-determinism --out "$OUT/probe" > "$OUT/probe.log" 2>&1
cat "$OUT/probe.log"
grep -q '^DETERMINISM PASS' "$OUT/probe.log"
python3 - "$OUT/probe/summary.json" <<'PY'
import json
import sys
from pathlib import Path
report = json.loads(Path(sys.argv[1]).read_text())
assert report['scene'] == 'colony' and report['days'] == 24
rows = report['seeds']
assert len(rows) == 2 and {row['seed'] for row in rows} == {0, 1}
for row in rows:
    assert row['ledger_violation_ticks'] == 0, row['seed']
    assert row['reservation_violation_ticks'] == 0, row['seed']
print('MUX_STEWARD PASS: 2 worlds x 24 days; deterministic, ledger and reservations exact')
PY
