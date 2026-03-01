#!/usr/bin/env python3
"""Compile category-based localization JSON files into per-locale flat maps.

This keeps authoring ergonomics (split files by category) while giving runtime
fast O(1) lookup from a single loaded JSON payload.

Usage:
  python3 tools/localization_compile.py --project-root .
  python3 tools/localization_compile.py --project-root . --strict-duplicates
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple


DEFAULT_MANIFEST: Dict[str, Any] = {
    "default_locale": "ko",
    "supported_locales": ["ko", "en"],
    "categories_order": [
        "ui",
        "game",
        "traits",
        "emotions",
        "events",
        "deaths",
        "buildings",
        "tutorial",
        "debug",
        "coping",
        "childhood",
        "reputation",
        "economy",
        "tech",
        "data_generated",
    ],
    "compiled_dir": "compiled",
    "include_sources": False,
    "key_registry_path": "key_registry.json",
    "key_owners_path": "key_owners.json",
    "preserve_key_ids": True,
    "embed_keys": False,
    "max_duplicate_key_count": None,
    "max_duplicate_conflict_count": None,
    "max_missing_key_fill_count": None,
    "max_owner_rule_miss_count": None,
    "max_owner_unused_count": None,
    "max_duplicate_owner_missing_count": None,
}


def _load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as fp:
        return json.load(fp)


def _write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fp:
        json.dump(data, fp, ensure_ascii=False, indent=2, sort_keys=True)
        fp.write("\n")


def _write_json_if_changed(path: Path, data: Any) -> bool:
    path.parent.mkdir(parents=True, exist_ok=True)
    rendered = json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if path.exists():
        existing = path.read_text(encoding="utf-8")
        if existing == rendered:
            return False
    path.write_text(rendered, encoding="utf-8")
    return True


def _load_manifest(manifest_path: Path) -> Dict[str, Any]:
    if not manifest_path.exists():
        return dict(DEFAULT_MANIFEST)
    data = _load_json(manifest_path)
    if not isinstance(data, dict):
        return dict(DEFAULT_MANIFEST)

    merged = dict(DEFAULT_MANIFEST)
    merged.update(data)
    return merged


def _load_category_data(
    localization_root: Path,
    locale: str,
    category: str,
    fallback_locale: str,
) -> Tuple[Dict[str, Any], str]:
    locale_file = localization_root / locale / f"{category}.json"
    if locale_file.exists():
        data = _load_json(locale_file)
        if isinstance(data, dict):
            return data, locale
        return {}, locale

    fallback_file = localization_root / fallback_locale / f"{category}.json"
    if fallback_file.exists():
        data = _load_json(fallback_file)
        if isinstance(data, dict):
            return data, fallback_locale
        return {}, fallback_locale

    return {}, ""


def _compile_locale(
    localization_root: Path,
    locale: str,
    fallback_locale: str,
    categories: List[str],
    key_owners: Dict[str, str],
) -> Dict[str, Any]:
    flat: Dict[str, str] = {}
    key_sources: Dict[str, str] = {}
    duplicate_keys: Dict[str, List[str]] = {}
    duplicate_conflict_keys: Dict[str, List[str]] = {}
    entries_by_key: Dict[str, List[Tuple[str, str, str]]] = {}
    owner_rule_seen_count = 0
    owner_rule_hit_count = 0
    owner_rule_miss_count = 0
    owner_rule_override_count = 0

    for category in categories:
        category_data, loaded_from = _load_category_data(
            localization_root=localization_root,
            locale=locale,
            category=category,
            fallback_locale=fallback_locale,
        )

        for raw_key, raw_value in category_data.items():
            key = str(raw_key)
            value = str(raw_value)
            source_label = f"{loaded_from}/{category}" if loaded_from else f"missing/{category}"
            entries_by_key.setdefault(key, []).append((category, source_label, value))

    for key, entries in entries_by_key.items():
        if len(entries) > 1:
            duplicate_sources = [item[1] for item in entries]
            duplicate_keys[key] = duplicate_sources
            distinct_values = {item[2] for item in entries}
            if len(distinct_values) > 1:
                duplicate_conflict_keys[key] = list(duplicate_sources)

        _, first_source, first_value = entries[0]
        selected_source = first_source
        selected_value = first_value

        owner_category = key_owners.get(key, "")
        if owner_category:
            owner_rule_seen_count += 1
            owner_match: Tuple[str, str, str] | None = None
            for entry in entries:
                if entry[0] == owner_category:
                    owner_match = entry
                    break
            if owner_match is None:
                owner_rule_miss_count += 1
            else:
                owner_rule_hit_count += 1
                _, selected_source, selected_value = owner_match
                if selected_source != first_source:
                    owner_rule_override_count += 1

        flat[key] = selected_value
        key_sources[key] = selected_source

    return {
        "strings": flat,
        "sources": key_sources,
        "duplicate_keys": duplicate_keys,
        "duplicate_conflict_keys": duplicate_conflict_keys,
        "owner_rule_seen_count": owner_rule_seen_count,
        "owner_rule_hit_count": owner_rule_hit_count,
        "owner_rule_miss_count": owner_rule_miss_count,
        "owner_rule_override_count": owner_rule_override_count,
        "keys": sorted(flat.keys()),
    }


def _load_key_registry(path: Path) -> List[str]:
    if not path.exists():
        return []
    data = _load_json(path)
    if not isinstance(data, dict):
        return []
    keys = data.get("keys")
    if not isinstance(keys, list):
        return []
    deduped: List[str] = []
    seen: set[str] = set()
    for item in keys:
        key = str(item)
        if key in seen:
            continue
        seen.add(key)
        deduped.append(key)
    return deduped


def _normalize_owner_category(raw: Any) -> str:
    category = str(raw).strip()
    if category.endswith(".json"):
        category = category[: -len(".json")]
    return category


def _load_key_owners(path: Path) -> Dict[str, str]:
    if not path.exists():
        return {}
    data = _load_json(path)
    if not isinstance(data, dict):
        return {}
    raw_owners: Any = data
    if "owners" in data and isinstance(data["owners"], dict):
        raw_owners = data["owners"]
    if not isinstance(raw_owners, dict):
        return {}

    owners: Dict[str, str] = {}
    for raw_key, raw_category in raw_owners.items():
        key = str(raw_key)
        category = _normalize_owner_category(raw_category)
        if not key or not category:
            continue
        owners[key] = category
    return owners


def _build_key_registry(
    canonical_keys: List[str],
    existing_registry_keys: List[str],
    preserve_key_ids: bool,
) -> List[str]:
    if not preserve_key_ids:
        return list(canonical_keys)

    merged: List[str] = []
    seen: set[str] = set()
    for key in existing_registry_keys:
        if key in seen:
            continue
        seen.add(key)
        merged.append(key)
    for key in canonical_keys:
        if key in seen:
            continue
        seen.add(key)
        merged.append(key)
    return merged


def _write_key_registry(path: Path, keys: List[str], active_keys: List[str]) -> None:
    key_to_id: Dict[str, int] = {}
    for idx, key in enumerate(keys):
        key_to_id[key] = idx
    active_set = set(active_keys)
    removed_keys: List[str] = []
    for key in keys:
        if key not in active_set:
            removed_keys.append(key)
    output: Dict[str, Any] = {
        "version": 1,
        "key_count": len(keys),
        "active_key_count": len(active_keys),
        "removed_key_count": len(removed_keys),
        "keys": keys,
        "key_to_id": key_to_id,
        "removed_keys": removed_keys,
    }
    _write_json_if_changed(path, output)


def run(project_root: Path, strict_duplicates: bool) -> int:
    localization_root = project_root / "localization"
    manifest_path = localization_root / "manifest.json"
    manifest = _load_manifest(manifest_path)

    default_locale = str(manifest.get("default_locale", "ko"))
    supported_locales = [str(x) for x in manifest.get("supported_locales", ["ko", "en"])]
    categories = [str(x) for x in manifest.get("categories_order", [])]
    compiled_dir_name = str(manifest.get("compiled_dir", "compiled"))
    include_sources = bool(manifest.get("include_sources", False))
    key_registry_rel = str(manifest.get("key_registry_path", "key_registry.json"))
    key_owners_rel = str(manifest.get("key_owners_path", "key_owners.json"))
    preserve_key_ids = bool(manifest.get("preserve_key_ids", True))
    embed_keys = bool(manifest.get("embed_keys", False))
    max_duplicate_key_count_raw = manifest.get("max_duplicate_key_count")
    max_duplicate_key_count: int | None = None
    if max_duplicate_key_count_raw is not None:
        try:
            max_duplicate_key_count = int(max_duplicate_key_count_raw)
        except (TypeError, ValueError):
            print(
                "[localization_compile] invalid max_duplicate_key_count in manifest",
                file=sys.stderr,
            )
            return 1
    max_duplicate_conflict_count_raw = manifest.get("max_duplicate_conflict_count")
    max_duplicate_conflict_count: int | None = None
    if max_duplicate_conflict_count_raw is not None:
        try:
            max_duplicate_conflict_count = int(max_duplicate_conflict_count_raw)
        except (TypeError, ValueError):
            print(
                "[localization_compile] invalid max_duplicate_conflict_count in manifest",
                file=sys.stderr,
            )
            return 1
    max_missing_key_fill_count_raw = manifest.get("max_missing_key_fill_count")
    max_missing_key_fill_count: int | None = None
    if max_missing_key_fill_count_raw is not None:
        try:
            max_missing_key_fill_count = int(max_missing_key_fill_count_raw)
        except (TypeError, ValueError):
            print(
                "[localization_compile] invalid max_missing_key_fill_count in manifest",
                file=sys.stderr,
            )
            return 1
    max_owner_rule_miss_count_raw = manifest.get("max_owner_rule_miss_count")
    max_owner_rule_miss_count: int | None = None
    if max_owner_rule_miss_count_raw is not None:
        try:
            max_owner_rule_miss_count = int(max_owner_rule_miss_count_raw)
        except (TypeError, ValueError):
            print(
                "[localization_compile] invalid max_owner_rule_miss_count in manifest",
                file=sys.stderr,
            )
            return 1
    max_owner_unused_count_raw = manifest.get("max_owner_unused_count")
    max_owner_unused_count: int | None = None
    if max_owner_unused_count_raw is not None:
        try:
            max_owner_unused_count = int(max_owner_unused_count_raw)
        except (TypeError, ValueError):
            print(
                "[localization_compile] invalid max_owner_unused_count in manifest",
                file=sys.stderr,
            )
            return 1
    max_duplicate_owner_missing_count_raw = manifest.get("max_duplicate_owner_missing_count")
    max_duplicate_owner_missing_count: int | None = None
    if max_duplicate_owner_missing_count_raw is not None:
        try:
            max_duplicate_owner_missing_count = int(max_duplicate_owner_missing_count_raw)
        except (TypeError, ValueError):
            print(
                "[localization_compile] invalid max_duplicate_owner_missing_count in manifest",
                file=sys.stderr,
            )
            return 1

    if not categories:
        print("[localization_compile] categories_order is empty", file=sys.stderr)
        return 1

    compiled_root = localization_root / compiled_dir_name
    compiled_root.mkdir(parents=True, exist_ok=True)
    key_owners_path = localization_root / key_owners_rel
    key_owners = _load_key_owners(key_owners_path)

    compiled_by_locale: Dict[str, Dict[str, Any]] = {}
    total_duplicates = 0
    max_locale_duplicates = 0
    max_locale_duplicate_conflicts = 0
    max_locale_owner_rule_misses = 0
    for locale in supported_locales:
        compiled = _compile_locale(
            localization_root=localization_root,
            locale=locale,
            fallback_locale="en",
            categories=categories,
            key_owners=key_owners,
        )
        compiled_by_locale[locale] = compiled
        duplicate_count = len(compiled["duplicate_keys"])
        duplicate_conflict_count = len(compiled["duplicate_conflict_keys"])
        total_duplicates += duplicate_count
        max_locale_duplicates = max(max_locale_duplicates, duplicate_count)
        max_locale_duplicate_conflicts = max(
            max_locale_duplicate_conflicts, duplicate_conflict_count
        )
        max_locale_owner_rule_misses = max(
            max_locale_owner_rule_misses, int(compiled.get("owner_rule_miss_count", 0))
        )

    canonical_key_set: set[str] = set()
    for compiled in compiled_by_locale.values():
        canonical_key_set.update(compiled["strings"].keys())
    canonical_keys: List[str] = sorted(canonical_key_set)
    duplicate_key_union: set[str] = set()
    for compiled in compiled_by_locale.values():
        duplicate_key_union.update(compiled["duplicate_keys"].keys())
    owner_keys = set(key_owners.keys())
    duplicate_owner_missing_keys = sorted(duplicate_key_union - owner_keys)
    owner_unused_keys = sorted(owner_keys - canonical_key_set)
    duplicate_owner_missing_count = len(duplicate_owner_missing_keys)
    owner_unused_count = len(owner_unused_keys)
    print(
        "[localization_compile] owner-policy: "
        f"entries={len(key_owners)} duplicate_keys={len(duplicate_key_union)} "
        f"missing_for_duplicates={duplicate_owner_missing_count} unused={owner_unused_count}"
    )
    key_registry_path = localization_root / key_registry_rel
    existing_registry_keys = _load_key_registry(key_registry_path)
    registry_keys = _build_key_registry(
        canonical_keys=canonical_keys,
        existing_registry_keys=existing_registry_keys,
        preserve_key_ids=preserve_key_ids,
    )
    _write_key_registry(
        path=key_registry_path,
        keys=registry_keys,
        active_keys=canonical_keys,
    )
    fallback_strings: Dict[str, str] = {}
    if "en" in compiled_by_locale:
        fallback_strings = dict(compiled_by_locale["en"]["strings"])
    max_locale_missing_filled = 0

    for locale in supported_locales:
        compiled = compiled_by_locale[locale]
        duplicate_count = len(compiled["duplicate_keys"])
        duplicate_conflict_count = len(compiled["duplicate_conflict_keys"])
        owner_rule_seen_count = int(compiled.get("owner_rule_seen_count", 0))
        owner_rule_hit_count = int(compiled.get("owner_rule_hit_count", 0))
        owner_rule_miss_count = int(compiled.get("owner_rule_miss_count", 0))
        owner_rule_override_count = int(compiled.get("owner_rule_override_count", 0))
        locale_strings: Dict[str, str] = dict(compiled["strings"])
        locale_sources: Dict[str, str] = dict(compiled["sources"])
        missing_filled_count = 0
        for key in registry_keys:
            if key in locale_strings:
                continue
            missing_filled_count += 1
            if key in fallback_strings:
                locale_strings[key] = fallback_strings[key]
                locale_sources[key] = "fallback/en"
            else:
                locale_strings[key] = key
                locale_sources[key] = "fallback/key"
        max_locale_missing_filled = max(max_locale_missing_filled, missing_filled_count)

        output = {
            "meta": {
                "locale": locale,
                "default_locale": default_locale,
                "categories_order": categories,
                "fallback_locale": "en",
                "duplicate_key_count": duplicate_count,
                "duplicate_conflict_count": duplicate_conflict_count,
                "key_count": len(registry_keys),
                "active_key_count": len(canonical_keys),
                "missing_key_fill_count": missing_filled_count,
                "include_sources": include_sources,
                "key_registry_path": key_registry_rel,
                "key_owners_path": key_owners_rel,
                "preserve_key_ids": preserve_key_ids,
                "embed_keys": embed_keys,
                "owner_rule_seen_count": owner_rule_seen_count,
                "owner_rule_hit_count": owner_rule_hit_count,
                "owner_rule_miss_count": owner_rule_miss_count,
                "owner_rule_override_count": owner_rule_override_count,
                "owner_policy_entry_count": len(key_owners),
                "owner_policy_missing_duplicate_count": duplicate_owner_missing_count,
                "owner_policy_unused_count": owner_unused_count,
            },
            "strings": locale_strings,
        }
        if embed_keys:
            output["keys"] = registry_keys
        if include_sources:
            output["sources"] = locale_sources
        out_path = compiled_root / f"{locale}.json"
        updated = _write_json_if_changed(out_path, output)

        print(
            f"[localization_compile] {locale}: "
            f"strings={len(locale_strings)} duplicates={duplicate_count} "
            f"duplicate_conflicts={duplicate_conflict_count} "
            f"owner_seen={owner_rule_seen_count} owner_hits={owner_rule_hit_count} "
            f"owner_misses={owner_rule_miss_count} owner_overrides={owner_rule_override_count} "
            f"filled={missing_filled_count} updated={1 if updated else 0} -> {out_path}"
        )

    if strict_duplicates and total_duplicates > 0:
        print(
            f"[localization_compile] strict mode failed: duplicate_keys={total_duplicates}",
            file=sys.stderr,
        )
        return 1

    if max_duplicate_key_count is not None and max_locale_duplicates > max_duplicate_key_count:
        print(
            "[localization_compile] duplicate regression: "
            f"max_locale_duplicates={max_locale_duplicates} max_allowed={max_duplicate_key_count}",
            file=sys.stderr,
        )
        return 1
    if (
        max_duplicate_conflict_count is not None
        and max_locale_duplicate_conflicts > max_duplicate_conflict_count
    ):
        print(
            "[localization_compile] duplicate-conflict regression: "
            f"max_locale_duplicate_conflicts={max_locale_duplicate_conflicts} "
            f"max_allowed={max_duplicate_conflict_count}",
            file=sys.stderr,
        )
        return 1
    if (
        max_missing_key_fill_count is not None
        and max_locale_missing_filled > max_missing_key_fill_count
    ):
        print(
            "[localization_compile] missing-fill regression: "
            f"max_locale_missing_filled={max_locale_missing_filled} "
            f"max_allowed={max_missing_key_fill_count}",
            file=sys.stderr,
        )
        return 1
    if (
        max_owner_rule_miss_count is not None
        and max_locale_owner_rule_misses > max_owner_rule_miss_count
    ):
        print(
            "[localization_compile] owner-rule miss regression: "
            f"max_locale_owner_rule_misses={max_locale_owner_rule_misses} "
            f"max_allowed={max_owner_rule_miss_count}",
            file=sys.stderr,
        )
        return 1
    if max_owner_unused_count is not None and owner_unused_count > max_owner_unused_count:
        print(
            "[localization_compile] owner-policy unused regression: "
            f"owner_unused_count={owner_unused_count} max_allowed={max_owner_unused_count}",
            file=sys.stderr,
        )
        return 1
    if (
        max_duplicate_owner_missing_count is not None
        and duplicate_owner_missing_count > max_duplicate_owner_missing_count
    ):
        print(
            "[localization_compile] owner-policy duplicate coverage regression: "
            f"missing_for_duplicates={duplicate_owner_missing_count} "
            f"max_allowed={max_duplicate_owner_missing_count}",
            file=sys.stderr,
        )
        return 1

    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project-root", default=".", help="WorldSim project root")
    parser.add_argument(
        "--strict-duplicates",
        action="store_true",
        help="return non-zero when duplicate localization keys exist",
    )
    args = parser.parse_args()

    project_root = Path(args.project_root).resolve()
    return run(project_root=project_root, strict_duplicates=args.strict_duplicates)


if __name__ == "__main__":
    sys.exit(main())
