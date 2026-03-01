#!/usr/bin/env python3
"""Extract inline localized data fields into localization key-value JSON files.

Scans `data/**/*.json` for keys with `_en`, `_ko`, `_kr` suffixes and generates:
- localization/en/data_generated.json
- localization/ko/data_generated.json
- data/localization_extraction_map.json

Default mode is non-destructive (data JSON files are not modified).
When `--apply-key-fields` is set, the script injects `*_key` references while
keeping existing inline localized text for backward compatibility.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Tuple


INLINE_SUFFIXES: Tuple[str, ...] = ("_en", "_ko", "_kr")


def _load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as fp:
        return json.load(fp)


def _write_json(path: Path, data: Any, indent: int = 2, sort_keys: bool = True) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fp:
        json.dump(data, fp, ensure_ascii=False, indent=indent, sort_keys=sort_keys)
        fp.write("\n")


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


def _sanitize_token(raw: str) -> str:
    token = re.sub(r"[^A-Z0-9]+", "_", raw.upper()).strip("_")
    return token or "X"


def _identity_token(node: Dict[str, Any], rel_file: Path, json_path: str) -> str:
    for key_name in ("id", "key", "display_key"):
        value = node.get(key_name)
        if isinstance(value, str) and value.strip():
            return _sanitize_token(value)

    file_token = _sanitize_token(str(rel_file.with_suffix("")))
    path_token = _sanitize_token(json_path.replace("$", "ROOT"))
    return f"{file_token}_{path_token}"


def _build_unique_key(
    preferred: str,
    en_text: str,
    ko_text: str,
    used_keys: Dict[str, Tuple[str, str]],
) -> str:
    if preferred not in used_keys:
        used_keys[preferred] = (en_text, ko_text)
        return preferred

    if used_keys[preferred] == (en_text, ko_text):
        return preferred

    index = 2
    while True:
        candidate = f"{preferred}_{index}"
        if candidate not in used_keys:
            used_keys[candidate] = (en_text, ko_text)
            return candidate
        if used_keys[candidate] == (en_text, ko_text):
            return candidate
        index += 1


def run(project_root: Path, apply_key_fields: bool) -> int:
    data_root = project_root / "data"
    localization_root = project_root / "localization"

    out_en = localization_root / "en" / "data_generated.json"
    out_ko = localization_root / "ko" / "data_generated.json"
    out_map = data_root / "localization_extraction_map.json"

    en_map: Dict[str, str] = {}
    ko_map: Dict[str, str] = {}
    entries: List[Dict[str, Any]] = []
    used_keys: Dict[str, Tuple[str, str]] = {}

    changed_files: List[Path] = []
    for json_file in sorted(data_root.rglob("*.json")):
        if json_file.name.startswith("localization_"):
            continue
        rel_file = json_file.relative_to(project_root)
        data = _load_json(json_file)
        file_changed = False

        for json_path, node in _walk_json_paths(data):
            if not isinstance(node, dict):
                continue

            grouped: Dict[str, Dict[str, str]] = {}
            for raw_key, raw_value in node.items():
                if not isinstance(raw_key, str):
                    continue
                if not isinstance(raw_value, str):
                    continue

                matched_suffix = ""
                for suffix in INLINE_SUFFIXES:
                    if raw_key.endswith(suffix):
                        matched_suffix = suffix
                        break
                if not matched_suffix:
                    continue

                base_field = raw_key[: -len(matched_suffix)]
                lang = matched_suffix[1:]
                grouped.setdefault(base_field, {})[lang] = raw_value

            if not grouped:
                continue

            for base_field, lang_values in grouped.items():
                en_text = lang_values.get("en", "")
                ko_text = lang_values.get("ko", lang_values.get("kr", ""))

                if not en_text and not ko_text:
                    continue

                identity = _identity_token(node, rel_file, json_path)
                field_token = _sanitize_token(base_field)
                preferred_key = f"DATA_{identity}_{field_token}"
                key = _build_unique_key(preferred_key, en_text, ko_text, used_keys)

                if key not in en_map:
                    en_map[key] = en_text
                if key not in ko_map:
                    ko_map[key] = ko_text

                if apply_key_fields:
                    key_field = f"{base_field}_key"
                    if key_field not in node or str(node[key_field]) != key:
                        node[key_field] = key
                        file_changed = True

                entries.append(
                    {
                        "file": str(rel_file),
                        "json_path": json_path,
                        "field": base_field,
                        "generated_key": key,
                        "has_en_value": bool(lang_values.get("en")),
                        "has_ko_value": bool(lang_values.get("ko")),
                        "has_kr_value": bool(lang_values.get("kr")),
                    }
                )

        if apply_key_fields and file_changed:
            _write_json(json_file, data, indent=4, sort_keys=False)
            changed_files.append(rel_file)

    report = {
        "summary": {
            "entry_count": len(entries),
            "generated_key_count": len(en_map),
            "ko_empty_count": sum(1 for value in ko_map.values() if value == ""),
            "en_empty_count": sum(1 for value in en_map.values() if value == ""),
        },
        "entries": entries,
    }

    _write_json(out_en, en_map)
    _write_json(out_ko, ko_map)
    _write_json(out_map, report)

    print(
        "[data_localization_extract] "
        f"entries={len(entries)} keys={len(en_map)} "
        f"en={out_en} ko={out_ko} map={out_map}"
    )
    if apply_key_fields:
        print(f"[data_localization_extract] apply_key_fields: changed_files={len(changed_files)}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project-root", default=".", help="WorldSim project root")
    parser.add_argument(
        "--apply-key-fields",
        action="store_true",
        help="write *_key fields into source data JSON files (inline localized text is kept)",
    )
    args = parser.parse_args()

    project_root = Path(args.project_root).resolve()
    return run(project_root=project_root, apply_key_fields=args.apply_key_fields)


if __name__ == "__main__":
    sys.exit(main())
