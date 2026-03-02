#!/usr/bin/env python3
"""Compile flat compiled JSON locale strings into Fluent-style .ftl files.

Input:
  localization/compiled/<locale>.json
Output:
  localization/fluent/<locale>/messages.ftl
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict


def _read_json(path: Path) -> Dict:
    with path.open("r", encoding="utf-8") as fp:
        data = json.load(fp)
    if isinstance(data, dict):
        return data
    return {}


def _escape_fluent_value(text: str) -> str:
    return text.replace("\\", "\\\\").replace("\n", "\\n")


def _render_ftl(strings: Dict[str, str]) -> str:
    lines = [
        "# Auto-generated from localization/compiled/*.json",
        "# Do not hand-edit generated blocks directly.",
        "",
    ]
    for key in sorted(strings.keys()):
        value = str(strings[key])
        lines.append(f"{key} = {_escape_fluent_value(value)}")
    lines.append("")
    return "\n".join(lines)


def compile_locale(project_root: Path, locale: str) -> bool:
    compiled_path = project_root / "localization" / "compiled" / f"{locale}.json"
    if not compiled_path.exists():
        print(f"[localization_compile_fluent] missing compiled locale: {compiled_path}")
        return False

    root = _read_json(compiled_path)
    strings = root.get("strings", {})
    if not isinstance(strings, dict):
        print(f"[localization_compile_fluent] invalid strings object: {compiled_path}")
        return False

    out_path = (
        project_root / "localization" / "fluent" / locale / "messages.ftl"
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    rendered = _render_ftl(strings)
    existing = out_path.read_text(encoding="utf-8") if out_path.exists() else None
    if existing == rendered:
        print(f"[localization_compile_fluent] unchanged: {out_path}")
        return True
    out_path.write_text(rendered, encoding="utf-8")
    print(f"[localization_compile_fluent] wrote: {out_path}")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--project-root", default=".", help="WorldSim project root")
    parser.add_argument(
        "--locales",
        nargs="*",
        default=["ko", "en"],
        help="locales to compile",
    )
    args = parser.parse_args()
    project_root = Path(args.project_root).resolve()

    success = True
    for locale in args.locales:
        success = compile_locale(project_root, locale) and success
    return 0 if success else 1


if __name__ == "__main__":
    raise SystemExit(main())
