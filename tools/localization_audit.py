#!/usr/bin/env python3
"""Localization/data structure audit tool for WorldSim.

Checks:
1) en/ko keyset parity per localization file.
2) duplicate localization keys across en files.
3) inline localized fields in data JSON (*_en, *_ko, *_kr).

Usage:
  python3 tools/localization_audit.py --project-root .
  python3 tools/localization_audit.py --project-root . --strict
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Dict, Iterable, List, Set, Tuple


INLINE_SUFFIXES: Tuple[str, ...] = ("_en", "_ko", "_kr")


def _load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as fp:
        return json.load(fp)


def _collect_top_level_keys(locale_dir: Path) -> Dict[str, Set[str]]:
    result: Dict[str, Set[str]] = {}
    for file in sorted(locale_dir.glob("*.json")):
        data = _load_json(file)
        if not isinstance(data, dict):
            continue
        result[file.name] = set(str(k) for k in data.keys())
    return result


def _find_duplicates(keys_by_file: Dict[str, Set[str]]) -> Dict[str, List[str]]:
    owners: Dict[str, List[str]] = defaultdict(list)
    for file_name, keyset in keys_by_file.items():
        for key in keyset:
            owners[key].append(file_name)
    return {k: sorted(v) for k, v in owners.items() if len(v) > 1}


def _walk_json_paths(obj: Any, path: str = "$") -> Iterable[Tuple[str, Any]]:
    yield path, obj
    if isinstance(obj, dict):
        for key, value in obj.items():
            next_path = f"{path}.{key}"
            yield from _walk_json_paths(value, next_path)
    elif isinstance(obj, list):
        for idx, value in enumerate(obj):
            next_path = f"{path}[{idx}]"
            yield from _walk_json_paths(value, next_path)


def _find_inline_localized_fields(data_file: Path) -> List[Dict[str, str]]:
    data = _load_json(data_file)
    matches: List[Dict[str, str]] = []
    for json_path, node in _walk_json_paths(data):
        if not isinstance(node, dict):
            continue
        for key in node.keys():
            if any(str(key).endswith(suffix) for suffix in INLINE_SUFFIXES):
                matches.append(
                    {
                        "file": str(data_file),
                        "path": json_path,
                        "key": str(key),
                    }
                )
    return matches


def _find_inline_localized_groups(data_file: Path) -> List[Dict[str, Any]]:
    data = _load_json(data_file)
    groups: List[Dict[str, Any]] = []
    for json_path, node in _walk_json_paths(data):
        if not isinstance(node, dict):
            continue

        grouped_langs: Dict[str, Set[str]] = {}
        for key in node.keys():
            key_str = str(key)
            for suffix in INLINE_SUFFIXES:
                if key_str.endswith(suffix):
                    base_field = key_str[: -len(suffix)]
                    lang = suffix[1:]
                    grouped_langs.setdefault(base_field, set()).add(lang)
                    break

        for base_field, langs in grouped_langs.items():
            key_field = f"{base_field}_key"
            groups.append(
                {
                    "file": str(data_file),
                    "path": json_path,
                    "base_field": base_field,
                    "languages": sorted(langs),
                    "has_key_field": key_field in node,
                    "key_field": key_field,
                }
            )
    return groups


def run_audit(project_root: Path) -> Dict[str, Any]:
    localization_root = project_root / "localization"
    en_dir = localization_root / "en"
    ko_dir = localization_root / "ko"
    data_dir = project_root / "data"

    en_keys = _collect_top_level_keys(en_dir)
    ko_keys = _collect_top_level_keys(ko_dir)

    parity_issues: List[Dict[str, Any]] = []
    all_files = sorted(set(en_keys.keys()) | set(ko_keys.keys()))
    for file_name in all_files:
        en_set = en_keys.get(file_name, set())
        ko_set = ko_keys.get(file_name, set())
        missing_in_ko = sorted(en_set - ko_set)
        missing_in_en = sorted(ko_set - en_set)
        if missing_in_ko or missing_in_en:
            parity_issues.append(
                {
                    "file": file_name,
                    "missing_in_ko": missing_in_ko,
                    "missing_in_en": missing_in_en,
                }
            )

    duplicate_keys = _find_duplicates(en_keys)

    inline_localized_fields: List[Dict[str, str]] = []
    inline_localized_groups: List[Dict[str, Any]] = []
    for json_file in sorted(data_dir.rglob("*.json")):
        if json_file.name.startswith("localization_"):
            continue
        inline_localized_fields.extend(_find_inline_localized_fields(json_file))
        inline_localized_groups.extend(_find_inline_localized_groups(json_file))

    inline_group_with_key_count = sum(
        1 for item in inline_localized_groups if bool(item.get("has_key_field", False))
    )
    inline_group_without_key_count = len(inline_localized_groups) - inline_group_with_key_count

    return {
        "parity_issues": parity_issues,
        "duplicate_key_count": len(duplicate_keys),
        "duplicate_keys": duplicate_keys,
        "inline_localized_field_count": len(inline_localized_fields),
        "inline_localized_fields": inline_localized_fields,
        "inline_localized_group_count": len(inline_localized_groups),
        "inline_group_with_key_count": inline_group_with_key_count,
        "inline_group_without_key_count": inline_group_without_key_count,
        "inline_localized_groups": inline_localized_groups,
    }


def _print_report(report: Dict[str, Any]) -> None:
    print("== Localization Audit ==")
    print(f"parity_issues: {len(report['parity_issues'])}")
    print(f"duplicate_keys: {report['duplicate_key_count']}")
    print(f"inline_localized_fields: {report['inline_localized_field_count']}")
    print(f"inline_groups: {report['inline_localized_group_count']}")
    print(f"inline_groups_with_key: {report['inline_group_with_key_count']}")
    print(f"inline_groups_without_key: {report['inline_group_without_key_count']}")

    if report["parity_issues"]:
        print("\n-- parity issues --")
        for issue in report["parity_issues"]:
            print(f"* {issue['file']}")
            if issue["missing_in_ko"]:
                print(f"  missing_in_ko: {len(issue['missing_in_ko'])}")
            if issue["missing_in_en"]:
                print(f"  missing_in_en: {len(issue['missing_in_en'])}")

    if report["inline_localized_fields"]:
        print("\n-- inline localized fields (first 20) --")
        for item in report["inline_localized_fields"][:20]:
            print(f"* {item['file']} :: {item['path']} :: {item['key']}")

    if report["inline_localized_groups"]:
        missing = [
            item
            for item in report["inline_localized_groups"]
            if not bool(item.get("has_key_field", False))
        ]
        if missing:
            print("\n-- inline groups without *_key (first 20) --")
            for item in missing[:20]:
                print(
                    f"* {item['file']} :: {item['path']} :: "
                    f"{item['base_field']} -> expected {item['key_field']}"
                )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project-root", default=".", help="WorldSim project root")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="return non-zero when any issue is found",
    )
    args = parser.parse_args()

    project_root = Path(args.project_root).resolve()
    report = run_audit(project_root)
    _print_report(report)

    if args.strict:
        has_issues = bool(report["parity_issues"]) or bool(report["inline_localized_fields"])
        return 1 if has_issues else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
