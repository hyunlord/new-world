"""M-ART2 grep tests (goal §0, §8, §11; ADR 0033): the renderer names no content and knows no 16-px tile.

1. No string literal in `scripts/world/**/*.gd` (comments excluded) equals a content id: a manifest file id (and its
   look / species / pose parts), a logical look id, a material key, a construction style id, a ground or structure
   id, a sim kind name listed in `assets/v8art/bindings.json`, or (M-ART2.1) a flora species or map canopy id. Content reaches the renderer only through
   manifest.json, bindings.json and the material documents.
2. No `16` (the M1 tile size) in a tile-size position — a factor or divisor (`* 16`, `16 *`, `/ 16`), `16.0`, or an
   argument of Vector2/Vector2i/Rect2/Rect2i — and no `TILE_SIZE` constant in `scripts/world/**/*.gd`; the world scale
   is one constant, `View.TILE` = 64. (Colour components, bit shifts and the wall's named 16-px cap are not tile sizes.)
   Texture roles (`face`, `top_ew`, `junction`, ...) and data-file handles are schema names, not content.
Both checks are shown to fail on a planted line (test_*_catches_*).
"""
from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORLD = ROOT / "scripts/world"
ASSETS = ROOT / "assets/v8art"
STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')
# A 16 in a tile-size position: a factor or divisor, a float 16.0, or inside a Vector2/Vector2i/Rect2/Rect2i.
SIXTEEN = re.compile(r"\*\s*16\b|\b16\s*\*|/\s*16\b|\b16\.0\b|(?:Vector2i?|Rect2i?)\([^)]*\b16\b")
# Schema names, not content: texture roles of the material documents and the people/material data files.
SCHEMA_ROLES = {"material_texture", "people_data", "material_data", "material_schema", "flora_data"}


def content_ids() -> set[str]:
    ids: set[str] = set()
    manifest = json.loads((ASSETS / "manifest.json").read_text())
    for row in manifest["files"]:
        if row["role"] in SCHEMA_ROLES:
            continue
        ident = row["id"]
        ids.add(ident)
        ids.update(part for part in ident.split("/") if len(part) > 1 and part not in {"S", "E"})
    for look in manifest["looks"]:
        ids.add(look["id"])
    ids -= set(manifest["people"]["poses"]) | set(manifest["people"]["variants"])
    for pose_list in manifest["animals"]["poses"].values():
        ids -= set(pose_list)
    materials = json.loads((ASSETS / "materials/material_looks.json").read_text())
    ids.update(m["id"] for m in materials["materials"])
    styles = json.loads((ASSETS / "materials/construction_styles.json").read_text())
    for style in styles["styles"]:
        ids.add(style["id"])
        ids.update(style.get("sim_blueprints", []))

    def sims(node: object) -> None:
        if isinstance(node, dict):
            if isinstance(node.get("sim"), str):
                ids.add(node["sim"])
            for v in node.values():
                sims(v)
        elif isinstance(node, list):
            for v in node:
                sims(v)

    sims(json.loads((ASSETS / "bindings.json").read_text()))
    # M-ART2.1: flora species ids and map canopy ids are content (the renderer resolves them through the catalogue).
    flora = json.loads((ASSETS / "flora/species.json").read_text())
    ids.update(flora["species"])
    ids.update(flora["map_canopies"])
    return {i for i in ids if i}


def code_lines(path: Path) -> list[tuple[int, str]]:
    out = []
    for n, line in enumerate(path.read_text().splitlines(), 1):
        stripped = line.split("#", 1)[0] if '"' not in line.split("#", 1)[0] or line.lstrip().startswith("#") else line
        if line.lstrip().startswith("#") or line.lstrip().startswith("##"):
            continue
        out.append((n, stripped))
    return out


def literal_hits(files: list[Path], ids: set[str]) -> list[str]:
    hits = []
    for f in files:
        for n, line in code_lines(f):
            for m in STRING.finditer(line):
                if m.group(1) in ids:
                    hits.append(f"{f.relative_to(ROOT)}:{n}: \"{m.group(1)}\"")
    return hits


def sixteen_hits(files: list[Path]) -> list[str]:
    hits = []
    for f in files:
        for n, line in code_lines(f):
            code = STRING.sub('""', line)
            if SIXTEEN.search(code) or "TILE_SIZE" in code:
                hits.append(f"{f.relative_to(ROOT)}:{n}: {line.strip()}")
    return hits


class ExtensibilityGrepTest(unittest.TestCase):
    def test_no_content_literals_in_the_renderer(self) -> None:
        ids = content_ids()
        self.assertGreater(len(ids), 150)  # observed ~330: 105 materials, 36 looks, items, structures, styles, sims
        files = sorted(WORLD.rglob("*.gd"))
        self.assertGreater(len(files), 10)
        self.assertEqual(literal_hits(files, ids), [])

    def test_no_sixteen_pixel_tile_literal(self) -> None:
        files = sorted(WORLD.rglob("*.gd"))
        self.assertEqual(sixteen_hits(files), [])
        view = (WORLD / "colony/view.gd").read_text()
        self.assertRegex(view, r"const TILE: int = 64\b")

    def test_literal_check_catches_a_planted_name(self) -> None:
        ids = content_ids()
        planted = ROOT / "tools/v8/_planted_test.gd"
        try:
            planted.write_text('extends Node\nvar x: String = "oak"\nvar y: String = "hearth_1"\n')
            self.assertEqual(len(literal_hits([planted], ids)), 2)
        finally:
            planted.unlink()

    def test_sixteen_check_catches_a_planted_tile_size(self) -> None:
        planted = ROOT / "tools/v8/_planted_test.gd"
        try:
            planted.write_text("extends Node\nvar p: Vector2 = Vector2(3, 4) * 16\nvar q: int = 160  # fine\n"
                               "var r: Rect2 = Rect2(0, 0, 16, 16)\nvar s: int = x >> 16\n")
            self.assertEqual(len(sixteen_hits([planted])), 2)
        finally:
            planted.unlink()


if __name__ == "__main__":
    unittest.main()
