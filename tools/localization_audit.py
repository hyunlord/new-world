#!/usr/bin/env python3
"""Localization/data structure audit tool for WorldSim.

Checks:
1) en/ko keyset parity per localization file.
2) duplicate localization keys across en files.
3) inline localized fields in data JSON (*_en, *_ko, *_kr).

Usage:
  python3 tools/localization_audit.py --project-root .
  python3 tools/localization_audit.py --project-root . --strict
  python3 tools/localization_audit.py --project-root . --report-json localization/reports/audit.json
  python3 tools/localization_audit.py --project-root . --strict-duplicate-conflicts
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


def _collect_top_level_entries(locale_dir: Path) -> Dict[str, Dict[str, Any]]:
    result: Dict[str, Dict[str, Any]] = {}
    for file in sorted(locale_dir.glob("*.json")):
        data = _load_json(file)
        if not isinstance(data, dict):
            continue
        result[file.name] = dict(data)
    return result


def _find_duplicates(keys_by_file: Dict[str, Set[str]]) -> Dict[str, List[str]]:
    owners: Dict[str, List[str]] = defaultdict(list)
    for file_name, keyset in keys_by_file.items():
        for key in keyset:
            owners[key].append(file_name)
    return {k: sorted(v) for k, v in owners.items() if len(v) > 1}


def _find_duplicate_details(
    entries_by_file: Dict[str, Dict[str, Any]]
) -> Dict[str, Dict[str, Any]]:
    owners: Dict[str, List[str]] = defaultdict(list)
    values_by_key: Dict[str, Dict[str, Any]] = defaultdict(dict)
    for file_name, entries in entries_by_file.items():
        for key, value in entries.items():
            key_name = str(key)
            owners[key_name].append(file_name)
            values_by_key[key_name][file_name] = value

    result: Dict[str, Dict[str, Any]] = {}
    for key in sorted(owners.keys()):
        files = sorted(owners[key])
        if len(files) <= 1:
            continue
        values = values_by_key[key]
        canonical = json.dumps(values[files[0]], ensure_ascii=False, sort_keys=True)
        value_conflict = False
        for file_name in files[1:]:
            sample = json.dumps(values[file_name], ensure_ascii=False, sort_keys=True)
            if sample != canonical:
                value_conflict = True
                break
        result[key] = {
            "files": files,
            "value_conflict": value_conflict,
            "values_by_file": {file_name: values[file_name] for file_name in files},
        }
    return result


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
        grouped_types: Dict[str, Set[str]] = {}
        for key in node.keys():
            key_str = str(key)
            for suffix in INLINE_SUFFIXES:
                if key_str.endswith(suffix):
                    base_field = key_str[: -len(suffix)]
                    lang = suffix[1:]
                    grouped_langs.setdefault(base_field, set()).add(lang)
                    value = node[key]
                    if isinstance(value, str):
                        value_type = "string"
                    elif value is None:
                        value_type = "null"
                    elif isinstance(value, bool):
                        value_type = "bool"
                    elif isinstance(value, (int, float)):
                        value_type = "number"
                    elif isinstance(value, list):
                        value_type = "array"
                    elif isinstance(value, dict):
                        value_type = "object"
                    else:
                        value_type = type(value).__name__
                    grouped_types.setdefault(base_field, set()).add(value_type)
                    break

        for base_field, langs in grouped_langs.items():
            key_field = f"{base_field}_key"
            value_types = sorted(grouped_types.get(base_field, set()))
            keyable_group = value_types == ["string"]
            groups.append(
                {
                    "file": str(data_file),
                    "path": json_path,
                    "base_field": base_field,
                    "languages": sorted(langs),
                    "value_types": value_types,
                    "keyable_group": keyable_group,
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
    manifest_path = localization_root / "manifest.json"
    supported_locales: List[str] = ["ko", "en"]
    if manifest_path.exists():
        raw_manifest = _load_json(manifest_path)
        if isinstance(raw_manifest, dict):
            raw_locales = raw_manifest.get("supported_locales")
            if isinstance(raw_locales, list):
                normalized: List[str] = []
                for item in raw_locales:
                    locale = str(item)
                    if locale and locale not in normalized:
                        normalized.append(locale)
                if normalized:
                    supported_locales = normalized

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

    duplicate_locale_summary: Dict[str, Dict[str, Any]] = {}
    all_localization_keys: Set[str] = set()
    for locale in supported_locales:
        locale_dir = localization_root / locale
        if not locale_dir.exists():
            continue
        locale_keys = _collect_top_level_keys(locale_dir)
        for keyset in locale_keys.values():
            all_localization_keys.update(keyset)
        locale_duplicates = _find_duplicates(locale_keys)
        locale_duplicate_details = _find_duplicate_details(_collect_top_level_entries(locale_dir))
        locale_conflict_count = sum(
            1
            for item in locale_duplicate_details.values()
            if bool(item.get("value_conflict", False))
        )
        duplicate_locale_summary[locale] = {
            "duplicate_key_count": len(locale_duplicates),
            "duplicate_conflict_count": locale_conflict_count,
            "duplicate_keys": locale_duplicates,
            "duplicate_details": locale_duplicate_details,
        }

    duplicate_report_locale = ""
    if duplicate_locale_summary:
        duplicate_report_locale = sorted(
            duplicate_locale_summary.keys(),
            key=lambda locale: (
                int(duplicate_locale_summary[locale]["duplicate_conflict_count"]),
                int(duplicate_locale_summary[locale]["duplicate_key_count"]),
                locale == "en",
            ),
            reverse=True,
        )[0]
    duplicate_report = duplicate_locale_summary.get(duplicate_report_locale, {})
    duplicate_keys = dict(duplicate_report.get("duplicate_keys", {}))
    duplicate_details = dict(duplicate_report.get("duplicate_details", {}))
    duplicate_report_key_count = int(duplicate_report.get("duplicate_key_count", 0))
    duplicate_report_conflict_count = int(duplicate_report.get("duplicate_conflict_count", 0))
    max_duplicate_key_count = max(
        (
            int(item.get("duplicate_key_count", 0))
            for item in duplicate_locale_summary.values()
        ),
        default=0,
    )
    max_duplicate_conflict_count = max(
        (
            int(item.get("duplicate_conflict_count", 0))
            for item in duplicate_locale_summary.values()
        ),
        default=0,
    )

    inline_localized_fields: List[Dict[str, str]] = []
    inline_localized_groups: List[Dict[str, Any]] = []
    for json_file in sorted(data_dir.rglob("*.json")):
        if json_file.name.startswith("localization_"):
            continue
        inline_localized_fields.extend(_find_inline_localized_fields(json_file))
        inline_localized_groups.extend(_find_inline_localized_groups(json_file))

    keyable_groups = [item for item in inline_localized_groups if bool(item.get("keyable_group", False))]
    non_keyable_groups = [
        item for item in inline_localized_groups if not bool(item.get("keyable_group", False))
    ]
    keyable_group_with_key_count = sum(
        1 for item in keyable_groups if bool(item.get("has_key_field", False))
    )
    keyable_group_without_key_count = len(keyable_groups) - keyable_group_with_key_count
    owner_policy_path = _resolve_manifest_key_owner_policy_path(project_root)
    owner_policy_payload: Any = {}
    if owner_policy_path.exists():
        owner_policy_payload = _load_json(owner_policy_path)
    owner_policy_map = _extract_owner_map(owner_policy_payload)
    duplicate_key_union: Set[str] = set()
    for item in duplicate_locale_summary.values():
        for key in dict(item.get("duplicate_keys", {})).keys():
            duplicate_key_union.add(str(key))
    owner_keys: Set[str] = set(owner_policy_map.keys())
    owner_policy_missing_duplicate_keys = sorted(duplicate_key_union - owner_keys)
    owner_policy_unused_keys = sorted(owner_keys - all_localization_keys)

    return {
        "parity_issues": parity_issues,
        "duplicate_key_count": max_duplicate_key_count,
        "duplicate_keys": duplicate_keys,
        "duplicate_conflict_count": max_duplicate_conflict_count,
        "duplicate_consistent_count": duplicate_report_key_count
        - duplicate_report_conflict_count,
        "duplicate_report_locale": duplicate_report_locale,
        "duplicate_report_key_count": duplicate_report_key_count,
        "duplicate_report_conflict_count": duplicate_report_conflict_count,
        "duplicate_locale_summary": {
            locale: {
                "duplicate_key_count": int(item.get("duplicate_key_count", 0)),
                "duplicate_conflict_count": int(item.get("duplicate_conflict_count", 0)),
            }
            for locale, item in duplicate_locale_summary.items()
        },
        "duplicate_details": duplicate_details,
        "inline_localized_field_count": len(inline_localized_fields),
        "inline_localized_fields": inline_localized_fields,
        "inline_localized_group_count": len(inline_localized_groups),
        "inline_keyable_group_count": len(keyable_groups),
        "inline_non_keyable_group_count": len(non_keyable_groups),
        "inline_keyable_group_with_key_count": keyable_group_with_key_count,
        "inline_keyable_group_without_key_count": keyable_group_without_key_count,
        "owner_policy_path": str(owner_policy_path),
        "owner_policy_entry_count": len(owner_keys),
        "owner_policy_missing_duplicate_count": len(owner_policy_missing_duplicate_keys),
        "owner_policy_unused_count": len(owner_policy_unused_keys),
        "owner_policy_missing_duplicate_keys": owner_policy_missing_duplicate_keys,
        "owner_policy_unused_keys": owner_policy_unused_keys,
        "inline_localized_groups": inline_localized_groups,
        "inline_keyable_groups": keyable_groups,
        "inline_non_keyable_groups": non_keyable_groups,
    }


def _load_manifest_dict(project_root: Path) -> Dict[str, Any]:
    manifest_path = project_root / "localization" / "manifest.json"
    if not manifest_path.exists():
        return {}
    payload = _load_json(manifest_path)
    if not isinstance(payload, dict):
        return {}
    return payload


def _resolve_manifest_key_owner_policy_path(project_root: Path) -> Path:
    manifest = _load_manifest_dict(project_root)
    localization_root = project_root / "localization"
    rel = "key_owners.json"
    if "key_owners_path" in manifest:
        rel = str(manifest.get("key_owners_path") or rel)
    return (localization_root / rel).resolve()


def _print_report(report: Dict[str, Any]) -> None:
    print("== Localization Audit ==")
    print(f"parity_issues: {len(report['parity_issues'])}")
    print(f"duplicate_keys: {report['duplicate_key_count']}")
    print(f"duplicate_conflicts: {report['duplicate_conflict_count']}")
    print(f"duplicate_consistent: {report['duplicate_consistent_count']}")
    print(f"duplicate_report_locale: {report['duplicate_report_locale']}")
    print(f"inline_localized_fields: {report['inline_localized_field_count']}")
    print(f"inline_groups: {report['inline_localized_group_count']}")
    print(f"inline_keyable_groups: {report['inline_keyable_group_count']}")
    print(f"inline_non_keyable_groups: {report['inline_non_keyable_group_count']}")
    print(f"inline_keyable_with_key: {report['inline_keyable_group_with_key_count']}")
    print(f"inline_keyable_without_key: {report['inline_keyable_group_without_key_count']}")
    print(f"owner_policy_entries: {report['owner_policy_entry_count']}")
    print(
        "owner_policy_missing_duplicates: "
        f"{report['owner_policy_missing_duplicate_count']}"
    )
    print(f"owner_policy_unused: {report['owner_policy_unused_count']}")

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

    if report["inline_keyable_groups"]:
        missing = [
            item
            for item in report["inline_keyable_groups"]
            if not bool(item.get("has_key_field", False))
        ]
        if missing:
            print("\n-- keyable inline groups without *_key (first 20) --")
            for item in missing[:20]:
                print(
                    f"* {item['file']} :: {item['path']} :: "
                    f"{item['base_field']} -> expected {item['key_field']}"
                )

    if report["inline_non_keyable_groups"]:
        print("\n-- non-keyable inline groups (first 20) --")
        for item in report["inline_non_keyable_groups"][:20]:
            print(
                f"* {item['file']} :: {item['path']} :: {item['base_field']} "
                f"(types={','.join(item.get('value_types', []))})"
            )

    if report.get("duplicate_locale_summary"):
        print("\n-- duplicate summary by locale --")
        for locale in sorted(report["duplicate_locale_summary"].keys()):
            item = report["duplicate_locale_summary"][locale]
            print(
                f"* {locale}: keys={item['duplicate_key_count']} "
                f"conflicts={item['duplicate_conflict_count']}"
            )

    if report.get("owner_policy_missing_duplicate_keys"):
        print("\n-- owner policy missing duplicate keys (first 20) --")
        for key in report["owner_policy_missing_duplicate_keys"][:20]:
            print(f"* {key}")

    if report.get("owner_policy_unused_keys"):
        print("\n-- owner policy unused keys (first 20) --")
        for key in report["owner_policy_unused_keys"][:20]:
            print(f"* {key}")

    conflict_items = [
        (key, item)
        for key, item in report.get("duplicate_details", {}).items()
        if bool(item.get("value_conflict", False))
    ]
    if conflict_items:
        print(
            "\n-- duplicate keys with value conflicts in "
            f"{report.get('duplicate_report_locale', 'unknown')} (first 20) --"
        )
        for key, item in conflict_items[:20]:
            files = ",".join(item.get("files", []))
            print(f"* {key} :: files={files}")


def _write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fp:
        json.dump(payload, fp, ensure_ascii=False, indent=2, sort_keys=True)
        fp.write("\n")


def _write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _format_value_sample(value: Any, max_len: int = 96) -> str:
    rendered = json.dumps(value, ensure_ascii=False, sort_keys=True)
    if len(rendered) > max_len:
        rendered = rendered[: max_len - 3] + "..."
    return rendered.replace("|", "\\|")


def _suggest_canonical_file(files: List[str]) -> str:
    if not files:
        return ""
    canonical_file = files[0]
    for preferred in ("ui.json", "game.json", "events.json"):
        if preferred in files:
            canonical_file = preferred
            break
    return canonical_file


def _category_from_file_name(file_name: str) -> str:
    if file_name.endswith(".json"):
        return file_name[: -len(".json")]
    return file_name


def _build_key_owner_policy_payload(report: Dict[str, Any]) -> Dict[str, Any]:
    owners: Dict[str, str] = {}
    duplicate_details = report.get("duplicate_details", {})
    if isinstance(duplicate_details, dict):
        for key, item in duplicate_details.items():
            if not isinstance(item, dict):
                continue
            files = [str(x) for x in item.get("files", [])]
            canonical_file = _suggest_canonical_file(files)
            if not canonical_file:
                continue
            owners[str(key)] = _category_from_file_name(canonical_file)

    return {
        "version": 1,
        "duplicate_report_locale": str(report.get("duplicate_report_locale", "")),
        "owner_key_count": len(owners),
        "owners": dict(sorted(owners.items(), key=lambda item: item[0])),
    }


def _extract_owner_map(payload: Any) -> Dict[str, str]:
    if not isinstance(payload, dict):
        return {}
    raw = payload
    if isinstance(payload.get("owners"), dict):
        raw = payload["owners"]
    if not isinstance(raw, dict):
        return {}
    out: Dict[str, str] = {}
    for raw_key, raw_value in raw.items():
        key = str(raw_key)
        value = str(raw_value)
        if not key or not value:
            continue
        out[key] = value
    return out


def _compare_owner_maps(expected: Dict[str, str], actual: Dict[str, str]) -> Dict[str, Any]:
    expected_keys = set(expected.keys())
    actual_keys = set(actual.keys())
    missing_keys = sorted(expected_keys - actual_keys)
    extra_keys = sorted(actual_keys - expected_keys)
    changed_keys = sorted(
        key
        for key in (expected_keys & actual_keys)
        if str(expected.get(key)) != str(actual.get(key))
    )
    return {
        "missing_keys": missing_keys,
        "extra_keys": extra_keys,
        "changed_keys": changed_keys,
        "missing_count": len(missing_keys),
        "extra_count": len(extra_keys),
        "changed_count": len(changed_keys),
    }


def _build_duplicate_conflict_markdown(report: Dict[str, Any]) -> str:
    locale = str(report.get("duplicate_report_locale", "unknown"))
    conflict_items = [
        (key, item)
        for key, item in report.get("duplicate_details", {}).items()
        if bool(item.get("value_conflict", False))
    ]
    lines: List[str] = [
        "# Localization Duplicate Conflict Report",
        "",
        f"- duplicate_report_locale: `{locale}`",
        f"- duplicate_conflicts: `{report.get('duplicate_conflict_count', 0)}`",
        f"- duplicate_keys: `{report.get('duplicate_key_count', 0)}`",
        "",
    ]
    if not conflict_items:
        lines.append("No duplicate conflicts found.")
        lines.append("")
        return "\n".join(lines)

    lines.extend(
        [
            "| Key | Canonical (Suggested) | Files | Sample Values |",
            "| --- | --- | --- | --- |",
        ]
    )
    for key, item in conflict_items:
        files = [str(x) for x in item.get("files", [])]
        values_by_file = item.get("values_by_file", {})
        sample_parts: List[str] = []
        for file_name in files[:3]:
            sample_value = _format_value_sample(values_by_file.get(file_name))
            sample_parts.append(f"{file_name}: {sample_value}")
        canonical_file = _suggest_canonical_file(files)
        files_joined = ", ".join(files).replace("|", "\\|")
        sample_joined = " / ".join(sample_parts)
        lines.append(
            f"| `{key}` | `{canonical_file}` | {files_joined} | {sample_joined} |"
        )
    lines.append("")
    return "\n".join(lines)


def _build_owner_policy_markdown(report: Dict[str, Any]) -> str:
    lines: List[str] = [
        "# Localization Owner Policy Report",
        "",
        f"- owner_policy_path: `{report.get('owner_policy_path', '')}`",
        f"- owner_policy_entries: `{report.get('owner_policy_entry_count', 0)}`",
        f"- missing_for_duplicates: `{report.get('owner_policy_missing_duplicate_count', 0)}`",
        f"- owner_unused: `{report.get('owner_policy_unused_count', 0)}`",
        "",
    ]
    missing_keys = [str(x) for x in report.get("owner_policy_missing_duplicate_keys", [])]
    unused_keys = [str(x) for x in report.get("owner_policy_unused_keys", [])]

    if not missing_keys and not unused_keys:
        lines.append("No owner policy coverage issues found.")
        lines.append("")
        return "\n".join(lines)

    lines.extend(["## Missing Owner For Duplicate Keys", ""])
    if missing_keys:
        lines.extend(["| Key |", "| --- |"])
        for key in missing_keys:
            lines.append(f"| `{key}` |")
    else:
        lines.append("None")
    lines.append("")

    lines.extend(["## Unused Owner Keys", ""])
    if unused_keys:
        lines.extend(["| Key |", "| --- |"])
        for key in unused_keys:
            lines.append(f"| `{key}` |")
    else:
        lines.append("None")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project-root", default=".", help="WorldSim project root")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="return non-zero when any issue is found",
    )
    parser.add_argument(
        "--strict-duplicate-conflicts",
        action="store_true",
        help="return non-zero when duplicate keys contain value conflicts",
    )
    parser.add_argument(
        "--report-json",
        default="",
        help="optional output path for full audit report json",
    )
    parser.add_argument(
        "--duplicate-report-json",
        default="",
        help="optional output path for duplicate key detail json",
    )
    parser.add_argument(
        "--duplicate-conflict-markdown",
        default="",
        help="optional output path for duplicate conflict markdown report",
    )
    parser.add_argument(
        "--key-owner-policy-json",
        default="",
        help="optional output path for canonical key-owner policy json",
    )
    parser.add_argument(
        "--owner-policy-markdown",
        default="",
        help="optional output path for owner policy markdown report",
    )
    parser.add_argument(
        "--compare-key-owner-policy",
        default="",
        help="optional path to existing key-owner policy json to compare against generated owners",
    )
    parser.add_argument(
        "--compare-key-owner-policy-auto",
        action="store_true",
        help="compare against key_owners_path from localization/manifest.json",
    )
    parser.add_argument(
        "--refresh-key-owner-policy-auto",
        action="store_true",
        help="write generated owner policy to key_owners_path from localization/manifest.json",
    )
    args = parser.parse_args()

    project_root = Path(args.project_root).resolve()
    report = run_audit(project_root)
    _print_report(report)
    owner_policy_payload = _build_key_owner_policy_payload(report)

    if args.report_json:
        out = (project_root / args.report_json).resolve()
        _write_json(out, report)
    if args.duplicate_report_json:
        out = (project_root / args.duplicate_report_json).resolve()
        _write_json(
            out,
            {
                "duplicate_key_count": report["duplicate_key_count"],
                "duplicate_conflict_count": report["duplicate_conflict_count"],
                "duplicate_consistent_count": report["duplicate_consistent_count"],
                "duplicate_details": report["duplicate_details"],
            },
        )
    if args.duplicate_conflict_markdown:
        out = (project_root / args.duplicate_conflict_markdown).resolve()
        _write_text(out, _build_duplicate_conflict_markdown(report))
    if args.key_owner_policy_json:
        out = (project_root / args.key_owner_policy_json).resolve()
        _write_json(out, owner_policy_payload)
    if args.owner_policy_markdown:
        out = (project_root / args.owner_policy_markdown).resolve()
        _write_text(out, _build_owner_policy_markdown(report))
    if args.refresh_key_owner_policy_auto:
        out = _resolve_manifest_key_owner_policy_path(project_root)
        _write_json(out, owner_policy_payload)
        print(f"[localization_audit] key-owner-policy refreshed: {out}")

    owner_policy_mismatch = False
    compare_path: Path | None = None
    if args.compare_key_owner_policy:
        compare_path = (project_root / args.compare_key_owner_policy).resolve()
    elif args.compare_key_owner_policy_auto:
        compare_path = _resolve_manifest_key_owner_policy_path(project_root)

    if compare_path is not None:
        compare_payload: Any = {}
        if compare_path.exists():
            compare_payload = _load_json(compare_path)
        generated_owners = _extract_owner_map(owner_policy_payload)
        existing_owners = _extract_owner_map(compare_payload)
        compare_result = _compare_owner_maps(generated_owners, existing_owners)
        print(
            "[localization_audit] key-owner-policy compare: "
            f"missing={compare_result['missing_count']} "
            f"extra={compare_result['extra_count']} "
            f"changed={compare_result['changed_count']}"
        )
        owner_policy_mismatch = (
            int(compare_result["missing_count"]) > 0
            or int(compare_result["extra_count"]) > 0
            or int(compare_result["changed_count"]) > 0
        )
        if owner_policy_mismatch:
            preview_limit = 10
            if compare_result["missing_keys"]:
                print(
                    "[localization_audit] missing owner keys (first 10): "
                    + ", ".join(compare_result["missing_keys"][:preview_limit]),
                    file=sys.stderr,
                )
            if compare_result["extra_keys"]:
                print(
                    "[localization_audit] extra owner keys (first 10): "
                    + ", ".join(compare_result["extra_keys"][:preview_limit]),
                    file=sys.stderr,
                )
            if compare_result["changed_keys"]:
                print(
                    "[localization_audit] changed owner keys (first 10): "
                    + ", ".join(compare_result["changed_keys"][:preview_limit]),
                    file=sys.stderr,
                )

    strict_duplicate_conflicts = int(report["duplicate_conflict_count"]) > 0
    if owner_policy_mismatch:
        return 1
    if args.strict:
        has_issues = bool(report["parity_issues"]) or (
            int(report["inline_keyable_group_without_key_count"]) > 0
        )
        if args.strict_duplicate_conflicts:
            has_issues = has_issues or strict_duplicate_conflicts
        return 1 if has_issues else 0
    if args.strict_duplicate_conflicts and strict_duplicate_conflicts:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
