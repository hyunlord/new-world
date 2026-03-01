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
    "preserve_key_ids": True,
    "embed_keys": False,
}


def _load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as fp:
        return json.load(fp)


def _write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fp:
        json.dump(data, fp, ensure_ascii=False, indent=2, sort_keys=True)
        fp.write("\n")


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
) -> Dict[str, Any]:
    flat: Dict[str, str] = {}
    key_sources: Dict[str, str] = {}
    duplicate_keys: Dict[str, List[str]] = {}

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

            if key in flat:
                duplicate_keys.setdefault(key, [key_sources[key]]).append(source_label)
                # Preserve first-wins semantics to match existing Locale loader behavior.
                continue

            flat[key] = value
            key_sources[key] = source_label

    return {
        "strings": flat,
        "sources": key_sources,
        "duplicate_keys": duplicate_keys,
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
    _write_json(path, output)


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
    preserve_key_ids = bool(manifest.get("preserve_key_ids", True))
    embed_keys = bool(manifest.get("embed_keys", False))

    if not categories:
        print("[localization_compile] categories_order is empty", file=sys.stderr)
        return 1

    compiled_root = localization_root / compiled_dir_name
    compiled_root.mkdir(parents=True, exist_ok=True)

    compiled_by_locale: Dict[str, Dict[str, Any]] = {}
    total_duplicates = 0
    for locale in supported_locales:
        compiled = _compile_locale(
            localization_root=localization_root,
            locale=locale,
            fallback_locale="en",
            categories=categories,
        )
        compiled_by_locale[locale] = compiled
        duplicate_count = len(compiled["duplicate_keys"])
        total_duplicates += duplicate_count

    canonical_key_set: set[str] = set()
    for compiled in compiled_by_locale.values():
        canonical_key_set.update(compiled["strings"].keys())
    canonical_keys: List[str] = sorted(canonical_key_set)
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

    for locale in supported_locales:
        compiled = compiled_by_locale[locale]
        duplicate_count = len(compiled["duplicate_keys"])
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

        output = {
            "meta": {
                "locale": locale,
                "default_locale": default_locale,
                "categories_order": categories,
                "fallback_locale": "en",
                "duplicate_key_count": duplicate_count,
                "key_count": len(registry_keys),
                "active_key_count": len(canonical_keys),
                "missing_key_fill_count": missing_filled_count,
                "include_sources": include_sources,
                "key_registry_path": key_registry_rel,
                "preserve_key_ids": preserve_key_ids,
                "embed_keys": embed_keys,
            },
            "strings": locale_strings,
        }
        if embed_keys:
            output["keys"] = registry_keys
        if include_sources:
            output["sources"] = locale_sources
        out_path = compiled_root / f"{locale}.json"
        _write_json(out_path, output)

        print(
            f"[localization_compile] {locale}: "
            f"strings={len(locale_strings)} duplicates={duplicate_count} "
            f"filled={missing_filled_count} -> {out_path}"
        )

    if strict_duplicates and total_duplicates > 0:
        print(
            f"[localization_compile] strict mode failed: duplicate_keys={total_duplicates}",
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
