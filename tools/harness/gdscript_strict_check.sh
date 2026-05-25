#!/usr/bin/env bash
# tools/harness/gdscript_strict_check.sh — strict GDScript verification.
#
# Catches three error classes that the Codex-based pipeline cannot:
#   1. GDScript parse errors
#   2. GDScript warnings (INTEGER_DIVISION, UNUSED_PARAMETER, etc.)
#   3. FFI binding mismatches (GDScript calls a Rust #[func] that does
#      not exist)
#
# Mechanism (Godot 4.6):
#   - For (1)+(2): temporarily inject `treat_warnings_as_errors=true`
#     plus per-warning levels into project.godot, then run
#     `--check-only --script` on every changed .gd file. Stderr is
#     grep'd for "SCRIPT ERROR" / "Parse Error" because Godot's exit
#     code is 0 even on parse failure (Godot quirk).
#   - For (3): grep GDScript for `world_sim.METHOD(` and compare to
#     `#[func] fn METHOD` in sim-bridge. Any GDScript call without a
#     matching Rust export is an FFI mismatch.
#
# Usage:
#   bash tools/harness/gdscript_strict_check.sh                # check all .gd in scripts/
#   bash tools/harness/gdscript_strict_check.sh path1.gd path2.gd  # check specific files
#
# Exit codes:
#   0  — all checks pass
#   2  — parse error or warning escalated to error
#   3  — FFI binding mismatch
#   4  — environment problem (Godot not found, project.godot missing)

set -uo pipefail

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
GODOT_BIN="${GODOT_BIN:-/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot}"
PROJECT_GODOT="$PROJECT_ROOT/project.godot"
FFI_FILE="$PROJECT_ROOT/rust/crates/sim-bridge/src/ffi/world_node.rs"

if [[ ! -x "$GODOT_BIN" ]]; then
    echo "[gdcheck] ERROR: Godot binary not found at $GODOT_BIN" >&2
    echo "[gdcheck] Set GODOT_BIN env var to point at your Godot 4.6 binary." >&2
    exit 4
fi
if [[ ! -f "$PROJECT_GODOT" ]]; then
    echo "[gdcheck] ERROR: project.godot not found at $PROJECT_GODOT" >&2
    exit 4
fi

cd "$PROJECT_ROOT"

# ── Argument parsing ────────────────────────────────────────────────────
FILES_FILE="$(mktemp)"
cleanup_files_file() { rm -f "$FILES_FILE"; }

if [[ $# -gt 0 ]]; then
    for f in "$@"; do echo "$f"; done > "$FILES_FILE"
else
    find scripts -name "*.gd" -not -path "*/test/*" 2>/dev/null > "$FILES_FILE"
fi

NUM_FILES=$(wc -l < "$FILES_FILE" | tr -d ' ')
if [[ "$NUM_FILES" -eq 0 ]]; then
    echo "[gdcheck] no .gd files to check"
    cleanup_files_file
    exit 0
fi

# ── 1. Parse + warnings: inject strict project setting, run --check-only ──
BACKUP="$(mktemp)"
cp "$PROJECT_GODOT" "$BACKUP"

cleanup() {
    local rc=$?
    cp "$BACKUP" "$PROJECT_GODOT"
    rm -f "$BACKUP"
    rm -f "$FILES_FILE" 2>/dev/null
    exit "$rc"
}
trap cleanup EXIT

# Append (idempotent) the strict warning block.
cat >> "$PROJECT_GODOT" <<'EOF'

[debug]

gdscript/warnings/integer_division=2
gdscript/warnings/unused_parameter=2
gdscript/warnings/unused_variable=2
gdscript/warnings/unused_local_constant=2
gdscript/warnings/unused_signal=2
gdscript/warnings/treat_warnings_as_errors=true
EOF

OVERALL_RC=0
PARSE_ERRORS=0
FFI_ERRORS=0

while IFS= read -r gd; do
    [[ -z "$gd" ]] && continue
    # Convert absolute path to res:// or keep relative.
    rel="${gd#$PROJECT_ROOT/}"
    OUT="$("$GODOT_BIN" --path "$PROJECT_ROOT" --headless --check-only --script "$rel" 2>&1 || true)"
    if echo "$OUT" | grep -qE "SCRIPT ERROR|Parse Error|Failed to load script"; then
        echo "[gdcheck] FAIL parse/warning: $rel" >&2
        echo "$OUT" | grep -E "SCRIPT ERROR|Parse Error|Failed to load script" | head -10 >&2
        PARSE_ERRORS=$((PARSE_ERRORS + 1))
        OVERALL_RC=2
    fi
done < "$FILES_FILE"

if [[ $PARSE_ERRORS -eq 0 ]]; then
    echo "[gdcheck] parse + warnings: ${NUM_FILES} file(s) clean ✓"
fi

# ── 2. FFI binding check ───────────────────────────────────────────────
if [[ -f "$FFI_FILE" ]]; then
    CALLED_FILE="$(mktemp)"
    EXPORTED_FILE="$(mktemp)"
    cleanup_ffi() { rm -f "$CALLED_FILE" "$EXPORTED_FILE"; }

    # GDScript calls of the form `world_sim.METHOD(` / `_world_sim.METHOD(`.
    grep -rhEo '(world_sim|_world_sim)\.[a-z_][a-z_0-9]*\(' scripts/ 2>/dev/null \
        | sed -E 's/.*\.([a-z_][a-z_0-9]*)\(/\1/' \
        | sort -u > "$CALLED_FILE"

    # Rust #[func] fn declarations in sim-bridge.
    grep -A 1 '^    #\[func\]' "$FFI_FILE" \
        | grep -E '^\s+(pub )?fn ' \
        | sed -E 's/.*fn ([a-z_][a-z_0-9]*).*/\1/' \
        | sort -u > "$EXPORTED_FILE"

    NUM_CALLED=$(wc -l < "$CALLED_FILE" | tr -d ' ')

    # Skip Godot built-in methods.
    SKIP_RE='^(has_method|call|connect|disconnect|get_node|queue_free|set_meta|get_meta|is_class|emit_signal)$'

    while IFS= read -r fn; do
        [[ -z "$fn" ]] && continue
        if echo "$fn" | grep -qE "$SKIP_RE"; then
            continue
        fi
        if ! grep -qx "$fn" "$EXPORTED_FILE"; then
            echo "[gdcheck] FAIL FFI: GDScript calls world_sim.${fn}() but no matching #[func] in sim-bridge" >&2
            FFI_ERRORS=$((FFI_ERRORS + 1))
            OVERALL_RC=3
        fi
    done < "$CALLED_FILE"

    if [[ $FFI_ERRORS -eq 0 ]]; then
        echo "[gdcheck] FFI bindings: ${NUM_CALLED} GDScript call(s) all match Rust #[func] exports ✓"
    fi
    cleanup_ffi
else
    echo "[gdcheck] WARN: $FFI_FILE not found — skipping FFI check" >&2
fi

# ── Summary ────────────────────────────────────────────────────────────
if [[ $OVERALL_RC -eq 0 ]]; then
    echo "[gdcheck] ALL CHECKS PASS"
else
    echo "[gdcheck] FAIL: parse=$PARSE_ERRORS FFI=$FFI_ERRORS"
fi
exit $OVERALL_RC
