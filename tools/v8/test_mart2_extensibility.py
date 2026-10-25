"""M-ART2 grep tests (goal §0, §8, §11; ADR 0033): the renderer names no content and knows no 16-px tile.

1. No string literal in world/UI GDScript or shaders (comments excluded) equals a content id: a manifest file id (and its
   look / species / pose parts), a logical look id, a material key, a construction style id, a ground or structure
   id, a sim kind name listed in `assets/v8art/bindings.json`, or (M-ART2.1) a flora species or map canopy id. Content reaches the renderer only through
   manifest.json, bindings.json and the material documents.
2. No `16` (the M1 tile size) in a tile-size position — a factor or divisor (`* 16`, `16 *`, `/ 16`), `16.0`, or an
   argument of Vector2/Vector2i/Rect2/Rect2i — and no `TILE_SIZE` constant in `scripts/world/**/*.gd`; the world scale
   is one constant, `View.TILE` = 64. (Colour components, bit shifts and the wall's named 16-px cap are not tile sizes.)
   Texture roles (`face`, `top_ew`, `junction`, ...) and data-file handles are schema names, not content.
The only UI schema exceptions are firewood DTO reads on the story receiver in
story_text, learning_facts and ui_facts_adapter (including raw["story"] reads and
row.merge DTO output keys in the adapter), and the meat locale parameter key
in chronicle_text's StoryText.content_phrase call. Comparisons remain forbidden.
Both checks are shown to fail on a planted line (test_*_catches_*).
"""
from __future__ import annotations

import ast
import json
import re
import unittest
from pathlib import Path
from typing import NamedTuple
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[2]
WORLD = ROOT / "scripts/world"
ASSETS = ROOT / "assets/v8art"
# Strings precede comments so comment markers inside strings remain literal.
STRING = "(?P<string>" + "|".join((
    '"""(?:[^\\\\]|\\\\[\\s\\S])*?"""',
    "'''(?:[^\\\\]|\\\\[\\s\\S])*?'''",
    '"(?:[^"\\\\\\n]|\\\\[\\s\\S])*"',
    "'(?:[^'\\\\\\n]|\\\\[\\s\\S])*'",
)) + ")"
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


class Token(NamedTuple):
    kind: str
    value: str
    line: int


def scan(source: str, shader: bool = False) -> tuple[list[Token], str]:
    """Tokenize strings and operators, masking comments/strings without moving lines."""
    comments = r"//[^\n]*|/\*[\s\S]*?\*/" if shader else r"#[^\n]*"
    pattern = re.compile(STRING + r"|(?P<comment>" + comments
                         + r")|(?P<space>\s+|\\\r?\n)|(?P<other>[A-Za-z_][A-Za-z_0-9]*|.)")
    tokens: list[Token] = []
    masked: list[str] = []
    line = 1
    for match in pattern.finditer(source):
        kind = match.lastgroup or "other"
        value = match.group()
        if kind not in {"comment", "space"}:
            tokens.append(Token(kind, value, line))
        masked.append(re.sub(r"[^\n]", " ", value) if kind in {"comment", "string"} else value)
        line += value.count("\n")
    return tokens, "".join(masked)


def constant(tokens: list[Token], start: int) -> tuple[str, int] | None:
    """Read a literal or parenthesized sum of literals; never evaluate source code."""
    if start >= len(tokens):
        return None
    token = tokens[start]
    if token.kind == "string":
        try:
            value = ast.literal_eval(token.value)
        except (SyntaxError, ValueError):
            return None
        if not isinstance(value, str):
            return None
        end = start + 1
    elif token.value == "(":
        inner = constant(tokens, start + 1)
        if inner is None:
            return None
        value, end = inner
        if end >= len(tokens) or tokens[end].value != ")":
            return None
        end += 1
    else:
        return None
    while end < len(tokens) and tokens[end].value == "+":
        right = constant(tokens, end + 1)
        if right is None:
            break
        suffix, end = right
        value += suffix
    return value, end


def schema_key(path: Path, tokens: list[Token], index: int, value: str) -> bool:
    """Exempt only documented DTO reads and this locale parameter, never comparisons."""
    if tokens[index].kind != "string":
        return False
    before = [token.value for token in tokens[max(0, index - 4):index]]
    after = [token.value for token in tokens[index + 1:index + 2]]
    relative = path.relative_to(ROOT).as_posix()
    story_readers = {
        "scripts/ui/story_text.gd", "scripts/ui/mux/learning_facts.gd",
        "scripts/ui/mux/ui_facts_adapter.gd",
    }
    if value == "firewood" and relative == "scripts/ui/mux/ui_facts_adapter.gd":
        preceding = [token.value for token in tokens[max(0, index - 7):index]]
        for story_key in ('"story"', "'story'"):
            if (preceding[-5:] == ["raw", "[", story_key, "]", "["] and after == ["]"]
                    or preceding == ["raw", "[", story_key, "]", ".", "has", "("]
                    and after == [")"]):
                return True
        line = [token.value for token in tokens[:index] if token.line == tokens[index].line]
        if line[:5] == ["row", ".", "merge", "(", "{"] and after == [":"]:
            return True
    if value == "firewood" and relative in story_readers:
        return (before[-2:] == ["story", "["] and after == ["]"]
                or before in (["story", ".", "get", "("], ["story", ".", "has", "("])
                and after in ([","], [")"]))
    if value == "meat" and relative == "scripts/ui/chronicle_text.gd" and after == [":"]:
        # This dictionary is the parameter argument of the animal locale formatter.
        line = [token.value for token in tokens[:index] if token.line == tokens[index].line]
        return (line[:5] == ["return", "StoryText", ".", "content_phrase", "("]
                and before[-1:] in (["{"], [","]))
    return False


def literal_hits(files: list[Path], ids: set[str]) -> list[str]:
    hits: list[str] = []
    for path in files:
        tokens, _ = scan(path.read_text(), path.suffix != ".gd")
        seen: set[tuple[int, str]] = set()
        for index, token in enumerate(tokens):
            result = constant(tokens, index)
            if (token.kind != "string" and result is not None
                    and sum(part.kind == "string" for part in tokens[index:result[1]]) == 1):
                continue
            # Check individual literals too, even when a surrounding expression is dynamic.
            values = [result[0]] if result is not None else []
            if token.kind == "string":
                single = constant([token], 0)
                if single is not None:
                    values.append(single[0])
            for value in values:
                if (value in ids and (token.line, value) not in seen
                        and not schema_key(path, tokens, index, value)):
                    seen.add((token.line, value))
                    hits.append(f'{path.relative_to(ROOT)}:{token.line}: "{value}"')
    return hits


def sixteen_hits(files: list[Path]) -> list[str]:
    hits: list[str] = []
    for path in files:
        _, masked = scan(path.read_text())
        for number, line in enumerate(masked.splitlines(), 1):
            if SIXTEEN.search(line) or "TILE_SIZE" in line:
                hits.append(f"{path.relative_to(ROOT)}:{number}: {line.strip()}")
    return hits


def renderer_files() -> list[Path]:
    """Include UI and all project shaders while keeping tile checks world-only."""
    return sorted(set(WORLD.rglob("*.gd")) | set((ROOT / "scripts/ui").rglob("*.gd"))
                  | set(ROOT.rglob("*.gdshader")) | set(ROOT.rglob("*.gdshaderinc")))


class ExtensibilityGrepTest(unittest.TestCase):
    def test_no_content_literals_in_the_renderer(self) -> None:
        ids = content_ids()
        self.assertGreater(len(ids), 150)  # observed ~330: 105 materials, 36 looks, items, structures, styles, sims
        files = renderer_files()
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

    def planted_hits(self, source: str, suffix: str = ".gd",
                     ids: set[str] | None = None) -> list[str]:
        with TemporaryDirectory(dir=ROOT / "tools/v8") as directory:
            path = Path(directory) / ("planted" + suffix)
            path.write_text(source)
            return literal_hits([path], content_ids() if ids is None else ids)

    def test_quote_and_concatenation_forms(self) -> None:
        sources = [
            "'oak'", '"oak"', '"o" + \'ak\'', '("o" + ("a" + "k"))',
            '("o" + # comment "ignored"\n "ak")',
            '"o" + \\\n "ak"',
            '"""oak"""', "'''oak'''",
            '"o\\u0061k"',
        ]
        for source in sources:
            with self.subTest(source=source):
                self.assertTrue(self.planted_hits("var value = " + source))

    def test_comments_schema_and_dynamic_controls(self) -> None:
        source = (
            '# "oak"\nvar x = "face" # \'oak\'\n'
            'var y = "top_ew"\nvar z = "material_data"\n'
            'var dynamic = "o" + variable + "ak"\n'
            'var separate = ["o", "ak"]\n'
        )
        self.assertEqual(self.planted_hits(source), [])
        self.assertTrue(self.planted_hits('var x = "#" + "oak"', ids={"#oak"}))
        self.assertTrue(self.planted_hits('var x = "https://oak"', ids={"https://oak"}))
        self.assertTrue(self.planted_hits('var x = "oak" # trailing comment'))
        self.assertEqual(self.planted_hits('var x = "safe" # "oak"'), [])

    def test_shader_comments_and_multiline_concatenation(self) -> None:
        source = '// "oak"\n/* \'oak\'\n "oak" */\n#define NAME ("o" + /* split */\n "ak")'
        hits = self.planted_hits(source, ".gdshader")
        self.assertTrue(hits)
        self.assertTrue(all(':4:' in hit for hit in hits))
        self.assertEqual(self.planted_hits('// "oak"\n/* "oak" */', ".gdshaderinc"), [])
        self.assertTrue(self.planted_hits('"//" + "oak"', ".gdshader", {"//oak"}))

    def test_schema_exceptions_require_the_specific_read_context(self) -> None:
        cases = [
            ("scripts/ui/story_text.gd", 'story.get("firewood", [])', True),
            ("scripts/ui/mux/learning_facts.gd", "story.has('firewood')", True),
            ("scripts/ui/mux/ui_facts_adapter.gd", 'story["firewood"]', True),
            ("scripts/ui/mux/ui_facts_adapter.gd", 'raw["story"].has("firewood")', True),
            ("scripts/ui/mux/ui_facts_adapter.gd", 'row.merge({"firewood": amount})', True),
            ("scripts/ui/mux/ui_facts_adapter.gd", 'kind == "firewood"', False),
            ("scripts/ui/mux/ui_facts_adapter.gd", 'var data = {"firewood": amount}', False),
            ("scripts/ui/story_text.gd", 'kind == "firewood"', False),
            ("scripts/ui/story_text.gd", 'other.get("firewood")', False),
            ("scripts/world/view.gd", 'story.get("firewood")', False),
            ("scripts/ui/chronicle_text.gd",
             'return StoryText.content_phrase(locale, family, cause, event, {"meat": value})', True),
            ("scripts/ui/chronicle_text.gd", 'kind == "meat"', False),
            ("scripts/ui/chronicle_text.gd", 'var content = {"meat": value}', False),
        ]
        for relative, source, expected in cases:
            with self.subTest(source=source, path=relative):
                tokens, _ = scan(source)
                index = next(i for i, token in enumerate(tokens) if token.kind == "string"
                             and ast.literal_eval(token.value) in {"firewood", "meat"})
                value = ast.literal_eval(tokens[index].value)
                self.assertEqual(schema_key(ROOT / relative, tokens, index, value), expected)

    def test_renderer_scope_includes_ui_and_shaders(self) -> None:
        files = renderer_files()
        self.assertTrue(any(path.is_relative_to(ROOT / "scripts/ui") for path in files))
        self.assertTrue(any(path.suffix == ".gdshader" for path in files))

    def test_tile_scan_ignores_both_quotes_and_comments(self) -> None:
        with TemporaryDirectory(dir=ROOT / "tools/v8") as directory:
            path = Path(directory) / "planted.gd"
            path.write_text('var x = \'* 16\' # Vector2(16, 16)\nvar y = "TILE_SIZE"\n')
            self.assertEqual(sixteen_hits([path]), [])

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
