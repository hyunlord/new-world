"""Tests of the M-ART2 art import (tools/v8/art_import.py, goal §3.3 and §6).

Each test can fail: a changed byte, a missing file, an unlisted file, a broken binding or a second import that
differs all turn one of them red (see test_verify_catches_* for the negative cases).
"""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/v8"))
import art_import  # noqa: E402

CONFIG = json.loads((ROOT / "tools/v8/art_import.json").read_text())
OUT = ROOT / CONFIG["out"]


class ArtImportTest(unittest.TestCase):
    def test_committed_folder_matches_its_manifest(self) -> None:
        self.assertEqual(art_import.verify(OUT, CONFIG), [])

    def test_size_budget(self) -> None:
        manifest = json.loads((OUT / "manifest.json").read_text())
        total = sum(row["bytes"] for row in manifest["files"])
        self.assertLessEqual(total, 40 * 1024 * 1024)
        # observed 13.9 MB at c8ed8608; a re-import that doubles it should be looked at, not waved through
        self.assertGreater(total, 5 * 1024 * 1024)

    def test_import_is_idempotent_and_equals_the_committed_folder(self) -> None:
        first = art_import.render(CONFIG, CONFIG["source_commit"])
        second = art_import.render(CONFIG, CONFIG["source_commit"])
        self.assertEqual(first, second)
        for path, data in first.items():
            self.assertEqual((OUT / path).read_bytes(), data, path)

    def test_counts_by_role(self) -> None:
        manifest = json.loads((OUT / "manifest.json").read_text())
        roles: dict[str, int] = {}
        for row in manifest["files"]:
            roles[row["role"]] = roles.get(row["role"], 0) + 1
        self.assertEqual(roles["people_frame"], 34 * 2 * 5)
        self.assertEqual(roles["portrait"], 34 * 6)
        self.assertEqual(roles["animal_frame"], 2 * 2 * 5)
        self.assertEqual(roles["material_texture"], 29)

    def test_the_two_fallback_looks_resolve_to_the_bun(self) -> None:
        manifest = json.loads((OUT / "manifest.json").read_text())
        looks = {row["id"]: row for row in manifest["looks"]}
        self.assertEqual(len(looks), 36)
        for missing in ("adult_M_fur_cloak_long_tied", "adult_M_hide_wrap_long_tied"):
            self.assertTrue(looks[missing]["fallback"])
            self.assertEqual(looks[missing]["frames"], missing.replace("long_tied", "bun"))
            for pose in ("idle", "sleep"):
                self.assertTrue((OUT / f"people/frames/{looks[missing]['frames']}/S/{pose}.png").is_file())
        self.assertEqual(sum(1 for row in looks.values() if row["fallback"]), 2)
        # the grid order is the appearance index: age, sex, top, hair
        self.assertEqual(manifest["looks"][0]["id"], "child_M_loincloth_short")
        self.assertEqual(manifest["looks"][35]["id"], "adult_F_fur_cloak_bun")

    def test_frames_carry_the_template_anchor(self) -> None:
        manifest = json.loads((OUT / "manifest.json").read_text())
        for row in manifest["files"]:
            if row["role"] != "people_frame":
                continue
            sleep = row["id"].endswith("/sleep")
            self.assertEqual(row["size"], [112, 80] if sleep else [80, 112], row["id"])
            self.assertEqual(row["meta"]["anchor"], [56, 72] if sleep else [40, 104], row["id"])

    def test_verify_catches_a_changed_byte_and_an_unlisted_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / "v8art"
            shutil.copytree(OUT, copy, ignore=shutil.ignore_patterns("*.import"))
            self.assertEqual(art_import.verify(copy, CONFIG), [])
            target = copy / "items/meat.png"
            data = bytearray(target.read_bytes())
            data[-20] ^= 0xFF
            target.write_bytes(bytes(data))
            (copy / "items/extra.png").write_bytes(b"x")
            problems = art_import.verify(copy, CONFIG)
            self.assertTrue(any("items/meat.png" in p for p in problems), problems)
            self.assertTrue(any("items/extra.png" in p for p in problems), problems)

    def test_verify_catches_a_binding_without_an_asset(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / "v8art"
            shutil.copytree(OUT, copy, ignore=shutil.ignore_patterns("*.import"))
            bindings = json.loads((copy / "bindings.json").read_text())
            bindings["items"]["codes"][0]["assets"][0] = "no_such_asset"
            (copy / "bindings.json").write_text(json.dumps(bindings))
            self.assertTrue(any("no_such_asset" in p for p in art_import.verify(copy, CONFIG)))

    def test_check_mode_reports_clean(self) -> None:
        result = subprocess.run([sys.executable, str(ROOT / "tools/v8/art_import.py"), "--check"],
                                capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
