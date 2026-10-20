#!/usr/bin/env python3
"""Prune localization to the keys the kept game uses (V8-0 goal 7.4).

Input: a key list (`locale_keys.py --json` output, field `used`).
For every supported locale:
- the value of each kept key is taken from the CURRENT runtime table
  `localization/compiled/<locale>.json` (what the game shows today), so the
  pruned game shows exactly the same text;
- the key is written to the category JSON (`localization/<locale>/<cat>.json`)
  that holds it today (first in `categories_order`, then any other category
  file), or to `ui.json` when no category file holds it;
- category files left empty are deleted.
The manifest switches to `source_format: "json"` (the category JSONs become
the source), drops the Fluent settings, sets `preserve_key_ids: false` and
lists only the remaining categories. Then run `tools/localization_compile.py`
and `locale_prune.py --verify KEYS.json`, which checks that every compiled
table holds exactly the kept keys, equals the category JSONs, and shows the
same value as `v7-final-art` for every kept key.

usage: locale_prune.py KEYS.json [ROOT]
       locale_prune.py --verify KEYS.json [ROOT]
"""
import json
import subprocess
import sys
from pathlib import Path


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def dump(path, data):
    Path(path).write_text(json.dumps(data, ensure_ascii=False, indent="\t") + "\n", encoding="utf-8")


def plan(root, keys):
    loc = Path(root, "localization")
    manifest = load(loc / "manifest.json")
    locales = manifest.get("supported_locales", ["ko", "en"])
    order = list(manifest.get("categories_order", []))
    result = {}
    for locale in locales:
        compiled = load(loc / "compiled" / f"{locale}.json")["strings"]
        files = {p.stem: load(p) for p in sorted((loc / locale).glob("*.json"))}
        search = [c for c in order if c in files] + [c for c in sorted(files) if c not in order]
        out = {}
        for key in keys:
            if key not in compiled:
                raise SystemExit(f"{locale}: kept key {key} missing from compiled table")
            category = next((c for c in search if key in files[c]), "ui")
            out.setdefault(category, {})[key] = compiled[key]
        result[locale] = (out, files)
    return manifest, order, result


def apply(root, keys):
    loc = Path(root, "localization")
    manifest, order, result = plan(root, keys)
    kept_categories = set()
    for locale, (out, files) in result.items():
        for category in sorted(set(files) | set(out)):
            path = loc / locale / f"{category}.json"
            if category in out:
                ordered = {k: out[category][k] for k in files.get(category, {}) if k in out[category]}
                ordered.update({k: v for k, v in sorted(out[category].items()) if k not in ordered})
                dump(path, ordered)
                kept_categories.add(category)
            elif path.exists():
                path.unlink()
    manifest["source_format"] = "json"
    manifest["categories_order"] = [c for c in order if c in kept_categories] + sorted(kept_categories - set(order))
    manifest["preserve_key_ids"] = False
    for stale in ("fluent_dir", "use_fluent_runtime", "key_owners_path"):
        manifest.pop(stale, None)
    for limit in ("max_duplicate_key_count", "max_duplicate_conflict_count"):
        if limit in manifest:
            manifest[limit] = 0
    dump(loc / "manifest.json", manifest)
    print(f"pruned to {len(keys)} keys in categories {manifest['categories_order']}")


def verify(root, keys):
    loc = Path(root, "localization")
    manifest = load(loc / "manifest.json")
    status = 0
    for locale in manifest.get("supported_locales", ["ko", "en"]):
        strings = load(loc / "compiled" / f"{locale}.json")["strings"]
        merged = {}
        for path in sorted((loc / locale).glob("*.json")):
            merged.update(load(path))
        if set(strings) != set(keys) or strings != merged:
            extra, missing = set(strings) - set(keys), set(keys) - set(strings)
            print(f"{locale}: FAIL compiled {len(strings)} keys (extra {sorted(extra)[:5]}, missing {sorted(missing)[:5]}); "
                  f"equals category JSONs: {strings == merged}")
            status = 1
        else:
            print(f"{locale}: compiled table = category JSONs = {len(keys)} kept keys")
        base = subprocess.run(["git", "show", f"v7-final-art:localization/compiled/{locale}.json"], cwd=root,
                              capture_output=True, text=True)
        if base.returncode == 0:
            base_strings = json.loads(base.stdout)["strings"]
            changed = [k for k in keys if strings.get(k) != base_strings.get(k)]
            print(f"{locale}: {len(keys) - len(changed)}/{len(keys)} values equal to v7-final-art" + (f"; differ: {changed[:5]}" if changed else ""))
            status |= 1 if changed else 0
    return status


def main(argv):
    if len(argv) >= 3 and argv[1] == "--verify":
        keys = load(argv[2])["used"]
        return verify(argv[3] if len(argv) > 3 else ".", keys)
    if len(argv) >= 2:
        keys = load(argv[1])["used"]
        apply(argv[2] if len(argv) > 2 else ".", keys)
        return 0
    print(__doc__.strip(), file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
