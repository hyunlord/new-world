#!/usr/bin/env bash
# usage: validate_identity_p20.sh <reference_summary_dir> <candidate_summary_dir> <candidate_raw_dir>
# Pass 20 (brief 4.1): every identity label's summary equal except timing
# (`compare_summaries.py`), the I2 array hashes equal, the chronicle files'
# hashes equal (brief: I7/I8; every label is compared), and DETERMINISM PASS
# in every probe log of the candidate's raw directory. Prints one line per
# check; exits non-zero on any difference.
set -uo pipefail
(( $# == 3 )) || { echo 'usage: validate_identity_p20.sh <reference> <candidate> <candidate_raw>' >&2; exit 2; }
REF="$1"; CAND="$2"; RAW="$3"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
status=0
for label in i1_v19 i3_v16 i4_v20 i5_v21 i6_v22 i7_v23 i8_v24; do
    out=$(python3 "$SCRIPT_DIR/compare_summaries.py" "$REF/$label/summary.json" "$CAND/$label/summary.json" 2>&1)
    diffs=$(printf '%s\n' "$out" | awk '/^differences:/ {print $2}')
    only=$(printf '%s\n' "$out" | awk '/-only paths:/ {s+=$3} END {print s+0}')
    if [[ "$diffs" == 0 && "$only" == 0 ]]; then
        echo "$label: PASS: 0 differences (timing keys excluded by compare_summaries.py)"
    else
        echo "$label: FAIL: $diffs differences, $only one-sided paths"; status=1
    fi
    if cmp -s "$REF/$label/chronicle_sha256.txt" "$CAND/$label/chronicle_sha256.txt"; then
        echo "$label chronicles: PASS: $(wc -l < "$CAND/$label/chronicle_sha256.txt" | tr -d ' ') file hashes equal"
    else
        echo "$label chronicles: FAIL"; status=1
    fi
done
if cmp -s "$REF/i2_sha256.txt" "$CAND/i2_sha256.txt"; then
    echo "i2: PASS: $(wc -l < "$CAND/i2_sha256.txt" | tr -d ' ') array hashes equal"
else
    echo "i2: FAIL"; status=1
fi
passes=$(grep -l 'DETERMINISM PASS' "$RAW"/i[1345678]_v*.log 2>/dev/null | wc -l | tr -d ' ')
echo "determinism: $passes of 7 probes print DETERMINISM PASS"
[[ "$passes" == 7 ]] || status=1
exit "$status"
