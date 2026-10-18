#!/usr/bin/env bash
# usage: validate_identity_p21.sh <reference_summary_dir> <candidate_summary_dir> <candidate_raw_dir> <reference_raw_dir>
# Pass 21 (brief 4.1): validate_identity_p20.sh with I9 (game_v25) added.
# Pass 20 (brief 4.1): every identity label's summary equal except timing
# (`compare_summaries.py`), the I2 array hashes equal, the chronicles equal
# (brief: I7/I8; every label is compared) and DETERMINISM PASS in every probe
# log of the candidate's raw directory. Chronicle files are compared by
# content: every field equal except the run label `git_sha` and the `kinds`
# vocabulary, of which the reference's list must be a prefix (Pass 20 appends
# kinds 17-21, so a candidate file names 22 kinds where the base names 17).
# Prints one line per check; exits non-zero on any difference.
set -uo pipefail
(( $# == 4 )) || { echo 'usage: validate_identity_p21.sh <reference> <candidate> <candidate_raw> <reference_raw>' >&2; exit 2; }
REF="$1"; CAND="$2"; RAW="$3"; REF_RAW="$4"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
status=0
for label in i1_v19 i3_v16 i4_v20 i5_v21 i6_v22 i7_v23 i8_v24 i9_v25; do
    out=$(python3 "$SCRIPT_DIR/compare_summaries.py" "$REF/$label/summary.json" "$CAND/$label/summary.json" 2>&1)
    diffs=$(printf '%s\n' "$out" | awk '/^differences:/ {print $2}')
    only=$(printf '%s\n' "$out" | awk '/-only paths:/ {s+=$3} END {print s+0}')
    if [[ "$diffs" == 0 && "$only" == 0 ]]; then
        echo "$label: PASS: 0 differences (timing keys excluded by compare_summaries.py)"
    else
        echo "$label: FAIL: $diffs differences, $only one-sided paths"; status=1
    fi
    if line=$(python3 - "$REF_RAW/chronicle/$label" "$RAW/chronicle/$label" 2>&1 <<'PY'
import json, sys
from pathlib import Path
ref, cand = Path(sys.argv[1]), Path(sys.argv[2])
names = sorted(p.name for p in ref.glob('chronicle_*.json'))
if names != sorted(p.name for p in cand.glob('chronicle_*.json')) or not names:
    sys.exit(f'file sets differ ({len(names)} reference files)')
for name in names:
    a, b = json.loads((ref / name).read_text()), json.loads((cand / name).read_text())
    if b['kinds'][:len(a['kinds'])] != a['kinds']:
        sys.exit(f'{name}: kind names are not an extension')
    for key in sorted(set(a) | set(b)):
        if key not in ('git_sha', 'kinds') and a.get(key) != b.get(key):
            sys.exit(f'{name}: {key} differs')
print(f'{len(names)} files equal (entries, people and every field but git_sha; kinds {len(a["kinds"])} -> {len(b["kinds"])} names)')
PY
); then
        echo "$label chronicles: PASS: $line"
    else
        echo "$label chronicles: FAIL: $line"; status=1
    fi
done
if cmp -s "$REF/i2_sha256.txt" "$CAND/i2_sha256.txt"; then
    echo "i2: PASS: $(wc -l < "$CAND/i2_sha256.txt" | tr -d ' ') array hashes equal"
else
    echo "i2: FAIL"; status=1
fi
passes=$(grep -l 'DETERMINISM PASS' "$RAW"/i[13456789]_v*.log 2>/dev/null | wc -l | tr -d ' ')
echo "determinism: $passes of 8 probes print DETERMINISM PASS"
[[ "$passes" == 8 ]] || status=1
exit "$status"
