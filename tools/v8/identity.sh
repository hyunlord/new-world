#!/usr/bin/env bash
# usage: identity.sh <base_data_dir> [<candidate_out_dir>]      capture the candidate and compare
#        identity.sh --capture <bin_dir> <out_dir>               capture only (used once at base)
#        identity.sh --raw-digest <probe_dir>                    print the raw sample digests
#
# V8-0 identity (goal 4.1). One scene, `game_v26` (the V8 probe keeps it as an
# alias of `baseline`, so base and candidate run the same command):
#   probe  --scene game_v26 --seeds 0..32 --days 120 (Pass 21's clock samples
#          and sample interval) --chronicle-out   -> <out>/probe, <out>/chronicle
#   determinism --check-determinism on seeds 0..4  -> <out>/determinism.txt
# Compare: summary.json equal except timing and the git_sha label
# (compare_summaries.py, zero one-sided paths); every JSON file the probe
# writes to the chronicle directory (chronicle_N.json, relations_N.json) equal
# by content except git_sha; every raw per-seed sample file (*.jsonl, which
# holds no timing) equal line by line with the git_sha label removed
# (raw_digest below); DETERMINISM PASS for seeds 0-3. Keys a commit removed
# on purpose (tools/v8/identity_removed_keys.txt, each with its reason) are
# dropped from the base side only where their base value is zero/empty.
# Copied from tools/revitalize/identity_p21.sh + validate_identity_p21.sh,
# reduced to one scene. Raw data must live outside the repository.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
WORKERS="${IDENTITY_WORKERS:-6}"
REMOVED="$SCRIPT_DIR/identity_removed_keys.txt"
SCENE=game_v26
FLAGS=(--seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers "$WORKERS")

capture() {
    local bin="$1" out="$2"
    [[ ! -e "$out/probe" ]] || { echo "refusing to reuse $out/probe" >&2; exit 2; }
    mkdir -p "$out"
    out="$(cd "$out" && pwd -P)"
    case "$out/" in "$ROOT/"*) echo 'identity data must be outside the repository' >&2; exit 2 ;; esac
    "$bin/worldsim_probe" --scene "$SCENE" "${FLAGS[@]}" --out "$out/probe" --chronicle-out "$out/chronicle" > "$out/probe.log" 2>&1
    "$bin/worldsim_probe" --scene "$SCENE" --seeds 0..4 --days 120 --workers 4 --check-determinism \
        --out "$out/determinism_probe" > "$out/determinism.txt" 2>&1
    (cd "$out/chronicle" && find . -type f -name '*.json' | LC_ALL=C sort | while IFS= read -r f; do shasum -a 256 "$f"; done) > "$out/chronicle_sha256.txt"
    raw_digest "$out/probe" > "$out/raw_sha256.txt"
}

# sha256 of each raw *.jsonl file's content with the git_sha label removed
# from every row (rows re-serialised with sorted keys). With a second
# argument (tools/v8/identity_removed_keys.txt) the listed keys are removed
# too, after checking that every such value is zero/false/null/empty.
raw_digest() {
    python3 - "$1" "${2:-}" <<'PY'
import hashlib, json, sys
from pathlib import Path
removed = set()
if sys.argv[2]:
    for line in Path(sys.argv[2]).read_text().splitlines():
        key = line.split('#', 1)[0].strip()
        if key:
            removed.add(key)
for path in sorted(Path(sys.argv[1]).glob('*.jsonl')):
    digest = hashlib.sha256()
    for line in path.read_text().splitlines():
        row = json.loads(line)
        row.pop('git_sha', None)
        for key in removed & set(row):
            if row[key] not in (0, 0.0, False, None, '', [], {}):
                sys.exit(f'{path.name}: removed key {key} has a non-zero value {row[key]!r}')
            row.pop(key)
        digest.update(json.dumps(row, sort_keys=True).encode() + b'\n')
    print(digest.hexdigest(), path.name)
PY
}

compare() {
    local base="$1" cand="$2" status=0 out diffs ref_only cand_only line
    out=$(python3 "$SCRIPT_DIR/compare_summaries.py" "$base/probe/summary.json" "$cand/probe/summary.json" --removed-keys "$REMOVED" 2>&1) || true
    diffs=$(printf '%s\n' "$out" | awk '/^differences:/ {print $2}')
    ref_only=$(printf '%s\n' "$out" | awk '/^reference-only paths:/ {print $3}')
    cand_only=$(printf '%s\n' "$out" | awk '/^candidate-only paths:/ {print $3}')
    shared=$(printf '%s\n' "$out" | awk '/^shared simulation values:/ {print $4}')
    if [[ "$diffs" == 0 && "$ref_only" == 0 && "$cand_only" == 0 ]]; then
        echo "summary: PASS: $shared shared values, 0 differences, 0 one-sided paths"
    else
        echo "summary: FAIL: ${diffs:-?} differences, ${ref_only:-?} base-only, ${cand_only:-?} candidate-only paths"
        printf '%s\n' "$out" | sed 's/^/  /' | head -60
        status=1
    fi
    if line=$(python3 - "$base/chronicle" "$cand/chronicle" 2>&1 <<'PY'
import json, sys
from pathlib import Path
ref, cand = Path(sys.argv[1]), Path(sys.argv[2])
names = sorted(p.name for p in ref.glob('*.json'))
if not names or names != sorted(p.name for p in cand.glob('*.json')):
    sys.exit(f'file sets differ ({len(names)} base files)')
entries = 0
for name in names:
    a, b = json.loads((ref / name).read_text()), json.loads((cand / name).read_text())
    for key in sorted(set(a) | set(b)):
        if key != 'git_sha' and a.get(key) != b.get(key):
            sys.exit(f'{name}: {key} differs')
    entries += len(a.get('entries', []))
print(f'{len(names)} files equal by content (every field but git_sha; {entries} entries)')
PY
    ); then
        echo "chronicle: PASS: $line"
    else
        echo "chronicle: FAIL: $line"; status=1
    fi
    if raw_digest "$base/probe" "$REMOVED" > "$cand/raw_sha256_base_reduced.txt" && raw_digest "$cand/probe" "$REMOVED" > "$cand/raw_sha256_reduced.txt" \
        && [[ -s "$cand/raw_sha256_reduced.txt" ]] && cmp -s "$cand/raw_sha256_base_reduced.txt" "$cand/raw_sha256_reduced.txt"; then
        echo "raw samples: PASS: $(wc -l < "$cand/raw_sha256_reduced.txt" | tr -d ' ') per-seed sample files equal (git_sha and $(grep -cv '^#' "$REMOVED" | tr -d ' ') removed zero-valued keys stripped)"
    else
        echo "raw samples: FAIL (raw_sha256.txt differs or is missing)"; status=1
    fi
    if grep -q 'DETERMINISM PASS' "$cand/determinism.txt" && ! grep -q 'DETERMINISM FAIL' "$cand/determinism.txt"; then
        echo "determinism: PASS: $(grep -c 'DETERMINISM PASS' "$cand/determinism.txt") DETERMINISM PASS line(s), seeds 0-3"
    else
        echo "determinism: FAIL (see $cand/determinism.txt)"; status=1
    fi
    return "$status"
}

if [[ "${1:-}" == --raw-digest ]]; then
    raw_digest "$2"
    exit 0
fi
if [[ "${1:-}" == --capture ]]; then
    (( $# == 3 )) || { echo 'usage: identity.sh --capture <bin_dir> <out_dir>' >&2; exit 2; }
    export WORLDSIM_GIT_SHA="${WORLDSIM_GIT_SHA:-unknown}"
    capture "$2" "$3"
    echo "captured $3"
    exit 0
fi
(( $# >= 1 && $# <= 2 )) || { echo 'usage: identity.sh <base_data_dir> [<candidate_out_dir>]' >&2; exit 2; }
BASE="$(cd "$1" && pwd -P)"
SHA="$(git -C "$ROOT" rev-parse HEAD)"
CAND="${2:-$(dirname "$BASE")/cand/identity-${SHA:0:8}-$(date +%Y%m%d-%H%M%S)}"
(cd "$ROOT/rust" && cargo build --release -p sim-test --bin worldsim_probe)
export WORLDSIM_GIT_SHA="$SHA"
capture "$ROOT/rust/target/release" "$CAND"
echo "candidate $SHA -> $CAND"
compare "$BASE" "$CAND"
