#!/usr/bin/env bash
# Sync JSON locale keys into fluent source, then recompile.
# Finds keys in ko/en JSON that are missing from fluent and appends them.
# Then runs localization_compile.py to rebuild registry + compiled JSON.
# Usage: bash tools/harness/locale_sync.sh [--dry-run]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOCALE_DIR="$PROJECT_ROOT/localization"
DRY_RUN="${1:-}"

echo "[locale-sync] Scanning for JSON keys missing from fluent..."

python3 - "$LOCALE_DIR" "$DRY_RUN" << 'PYEOF'
import json, sys
from pathlib import Path

LOCALE_DIR = Path(sys.argv[1])
DRY_RUN = sys.argv[2] == "--dry-run" if len(sys.argv) > 2 else False

def load_fluent_keys(ftl_path):
    keys = set()
    if not ftl_path.exists():
        return keys
    for line in ftl_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if "=" in line and not line.startswith("#"):
            key = line.split("=")[0].strip()
            if key and (key[0].isupper() or "_" in key):
                keys.add(key)
    return keys

def load_json_translations(locale):
    translations = {}
    locale_dir = LOCALE_DIR / locale
    for json_file in sorted(locale_dir.glob("*.json")):
        with open(json_file, encoding="utf-8") as f:
            data = json.load(f)
        for k, v in data.items():
            if k == k.upper() or "_" in k:
                translations[k] = str(v)
    return translations

# Load existing fluent keys
fluent_ko_keys = load_fluent_keys(LOCALE_DIR / "fluent" / "ko" / "messages.ftl")
fluent_en_keys = load_fluent_keys(LOCALE_DIR / "fluent" / "en" / "messages.ftl")
fluent_all = fluent_ko_keys | fluent_en_keys

# Load JSON translations
json_ko = load_json_translations("ko")
json_en = load_json_translations("en")
json_all_keys = set(json_ko.keys()) | set(json_en.keys())

# Find missing keys
missing = sorted(json_all_keys - fluent_all)
if not missing:
    print("[locale-sync] All JSON keys already in fluent. Nothing to do.")
    sys.exit(0)

print(f"[locale-sync] {len(missing)} key(s) in JSON but not in fluent:")
for k in missing[:20]:
    print(f"  {k}")
if len(missing) > 20:
    print(f"  ... and {len(missing) - 20} more")

if DRY_RUN:
    print("[locale-sync] --dry-run: no changes written.")
    sys.exit(0)

# Append to fluent files
for locale, json_data, ftl_path in [
    ("ko", json_ko, LOCALE_DIR / "fluent" / "ko" / "messages.ftl"),
    ("en", json_en, LOCALE_DIR / "fluent" / "en" / "messages.ftl"),
]:
    additions = []
    for key in missing:
        value = json_data.get(key, key)
        additions.append(f"{key} = {value}")
    if additions:
        with open(ftl_path, "a", encoding="utf-8") as f:
            f.write(f"\n# --- auto-synced from JSON ({len(additions)} keys) ---\n")
            for line in additions:
                f.write(line + "\n")
        print(f"[locale-sync] Appended {len(additions)} keys to {ftl_path.name} ({locale})")

PYEOF

if [[ "$DRY_RUN" == "--dry-run" ]]; then
    exit 0
fi

# Recompile
echo "[locale-sync] Recompiling localization..."
python3 "$PROJECT_ROOT/tools/localization_compile.py"
echo "[locale-sync] Done."
