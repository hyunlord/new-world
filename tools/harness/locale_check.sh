#!/usr/bin/env bash
# Locale key consistency check — detects:
#   1. Keys used in code (Locale.ltr) but missing from fluent source
#   2. Keys in JSON but not in fluent (won't compile into registry)
#   3. Keys in ko/ but not en/ or vice versa (partial translation)
# Usage: bash tools/harness/locale_check.sh [output_dir]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOCALE_DIR="$PROJECT_ROOT/localization"
OUT_DIR="${1:-/tmp/locale_check}"
mkdir -p "$OUT_DIR"

echo "[locale-check] Scanning code and locale files..."

# 1. Extract keys used in code via Locale.ltr("KEY")
grep -rohE 'Locale\.ltr\("[A-Z_0-9]+"\)' "$PROJECT_ROOT/scripts" 2>/dev/null \
    | sed -E 's/Locale\.ltr\("([A-Z_0-9]+)"\)/\1/' \
    | sort -u > "$OUT_DIR/used_keys.txt"

# 2. Extract keys from fluent source (primary — compiler uses these)
for locale in ko en; do
    cat "$LOCALE_DIR/fluent/$locale"/*.ftl 2>/dev/null \
        | grep -E '^[A-Z_][A-Z_0-9]+ =' \
        | sed -E 's/ =.*//' \
        | sort -u > "$OUT_DIR/fluent_${locale}_keys.txt"
done

# 3. Extract keys from JSON source (secondary — may drift from fluent)
for locale in ko en; do
    find "$LOCALE_DIR/$locale" -name "*.json" -exec cat {} + 2>/dev/null \
        | grep -oE '"[A-Z_][A-Z_0-9]+"[[:space:]]*:' \
        | sed -E 's/"([^"]+)".*/\1/' \
        | sort -u > "$OUT_DIR/json_${locale}_keys.txt"
done

# 4. Extract keys from key_registry.json
python3 -c "
import json
with open('$LOCALE_DIR/key_registry.json') as f:
    data = json.load(f)
for k in sorted(data.get('key_to_id', {}).keys()):
    print(k)
" > "$OUT_DIR/registry_keys.txt" 2>/dev/null

# 5. Extract keys from compiled JSON
python3 -c "
import json
with open('$LOCALE_DIR/compiled/ko.json') as f:
    data = json.load(f)
for k in sorted(data.get('strings', {}).keys()):
    print(k)
" > "$OUT_DIR/compiled_keys.txt" 2>/dev/null

# Union of fluent ko+en as the canonical source
sort -u "$OUT_DIR/fluent_ko_keys.txt" "$OUT_DIR/fluent_en_keys.txt" > "$OUT_DIR/fluent_all_keys.txt"

# ── Counts ──
USED=$(wc -l < "$OUT_DIR/used_keys.txt" | tr -d ' ')
FLUENT_KO=$(wc -l < "$OUT_DIR/fluent_ko_keys.txt" | tr -d ' ')
FLUENT_EN=$(wc -l < "$OUT_DIR/fluent_en_keys.txt" | tr -d ' ')
JSON_KO=$(wc -l < "$OUT_DIR/json_ko_keys.txt" | tr -d ' ')
JSON_EN=$(wc -l < "$OUT_DIR/json_en_keys.txt" | tr -d ' ')
REG=$(wc -l < "$OUT_DIR/registry_keys.txt" | tr -d ' ')
COMPILED=$(wc -l < "$OUT_DIR/compiled_keys.txt" | tr -d ' ')

echo ""
echo "=== LOCALE KEY COUNTS ==="
echo "Code (Locale.ltr):    $USED"
echo "Fluent ko/en:         $FLUENT_KO / $FLUENT_EN"
echo "JSON ko/en:           $JSON_KO / $JSON_EN"
echo "Registry:             $REG"
echo "Compiled:             $COMPILED"

# ── Issue detection ──
ISSUES=0

# P1: Keys used in code but not in fluent (will show as raw key in-game)
comm -23 "$OUT_DIR/used_keys.txt" "$OUT_DIR/fluent_all_keys.txt" > "$OUT_DIR/code_not_fluent.txt"
P1=$(wc -l < "$OUT_DIR/code_not_fluent.txt" | tr -d ' ')

# P2: Keys in JSON but not in fluent (added to JSON but forgot fluent)
comm -23 "$OUT_DIR/json_ko_keys.txt" "$OUT_DIR/fluent_all_keys.txt" > "$OUT_DIR/json_not_fluent.txt"
P2=$(wc -l < "$OUT_DIR/json_not_fluent.txt" | tr -d ' ')

# P3: Fluent ko/en mismatch (partial translation)
comm -23 "$OUT_DIR/fluent_ko_keys.txt" "$OUT_DIR/fluent_en_keys.txt" > "$OUT_DIR/ko_only.txt"
comm -13 "$OUT_DIR/fluent_ko_keys.txt" "$OUT_DIR/fluent_en_keys.txt" > "$OUT_DIR/en_only.txt"
P3_KO=$(wc -l < "$OUT_DIR/ko_only.txt" | tr -d ' ')
P3_EN=$(wc -l < "$OUT_DIR/en_only.txt" | tr -d ' ')

echo ""
echo "=== ISSUES ==="

if [[ $P1 -gt 0 ]]; then
    echo "❌ P1: $P1 key(s) used in code but MISSING from fluent (raw key in-game):"
    head -10 "$OUT_DIR/code_not_fluent.txt" | sed 's/^/   - /'
    [[ $P1 -gt 10 ]] && echo "   ... and $((P1 - 10)) more"
    echo "   → Add to localization/fluent/{ko,en}/messages.ftl then recompile"
    echo ""
    ISSUES=$((ISSUES + P1))
fi

if [[ $P2 -gt 0 ]]; then
    echo "⚠️  P2: $P2 key(s) in JSON but NOT in fluent (won't compile):"
    head -10 "$OUT_DIR/json_not_fluent.txt" | sed 's/^/   - /'
    [[ $P2 -gt 10 ]] && echo "   ... and $((P2 - 10)) more"
    echo "   → Run: bash tools/harness/locale_sync.sh"
    echo ""
    ISSUES=$((ISSUES + P2))
fi

if [[ $P3_KO -gt 0 ]]; then
    echo "⚠️  P3: $P3_KO key(s) only in ko fluent (missing en translation)"
    [[ $P3_KO -le 5 ]] && head -5 "$OUT_DIR/ko_only.txt" | sed 's/^/   - /'
    echo ""
fi

if [[ $P3_EN -gt 0 ]]; then
    echo "⚠️  P3: $P3_EN key(s) only in en fluent (missing ko translation)"
    [[ $P3_EN -le 5 ]] && head -5 "$OUT_DIR/en_only.txt" | sed 's/^/   - /'
    echo ""
fi

echo "=== SUMMARY ==="
if [[ $ISSUES -eq 0 ]]; then
    echo "✅ No critical locale issues found"
    echo "LOCALE_CHECK_PASS"
    exit 0
else
    echo "❌ $ISSUES critical issue(s) — keys will display as raw text in-game"
    echo "LOCALE_CHECK_FAIL"
    exit 1
fi
