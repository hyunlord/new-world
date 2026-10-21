"""Unit tests for tools/v8/contact_sheet.py: output size, layout and the size limit."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import contact_sheet  # noqa: E402


class ContactSheetTest(unittest.TestCase):
    def _shots(self, folder: Path, count: int) -> list[dict]:
        entries = []
        for i in range(count):
            name = f"shot_{i}.png"
            Image.new("RGB", (1920, 1080), (40 * i % 256, 80, 120)).save(folder / name)
            entries.append({"file": name, "caption": f"한 줄 설명 {i}"})
        return entries

    def test_output_size_two_per_row(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            (folder / "m.json").write_text(json.dumps(self._shots(folder, 3)), encoding="utf-8")
            code = contact_sheet.main(["--manifest", str(folder / "m.json"), "--out", str(folder / "sheet.png")])
            self.assertEqual(code, 0)
            with Image.open(folder / "sheet.png") as sheet:
                # 3 shots -> 2 rows; width 2 x 960 + 3 gaps, height 2 x (540 + 64) + 3 gaps.
                self.assertEqual(sheet.size, (1944, 1232))
                self.assertEqual(sheet.size, contact_sheet.sheet_size(3))

    def test_each_shot_is_scaled_to_960x540(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            Image.new("RGB", (1920, 1080), (255, 0, 0)).save(folder / "red.png")
            sheet = contact_sheet.build([(folder / "red.png", "빨강")])
            gap = contact_sheet.GAP
            # Inside the tile: red at both far corners; just outside it: the background.
            self.assertEqual(sheet.getpixel((gap, gap)), (255, 0, 0))
            self.assertEqual(sheet.getpixel((gap + 959, gap + 539)), (255, 0, 0))
            self.assertEqual(sheet.getpixel((gap + 960, gap)), contact_sheet.BACKGROUND)
            self.assertNotEqual(sheet.getpixel((gap + 959, gap + 540)), (255, 0, 0))

    def test_three_columns_and_blank_cells(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            entries = self._shots(folder, 2)
            entries.insert(1, {"file": "", "caption": ""})
            (folder / "m.json").write_text(json.dumps(entries), encoding="utf-8")
            out = folder / "sheet.png"
            code = contact_sheet.main(["--manifest", str(folder / "m.json"), "--out", str(out), "--columns", "3"])
            self.assertEqual(code, 0)
            with Image.open(out) as sheet:
                self.assertEqual(sheet.size, (3 * 960 + 4 * contact_sheet.GAP, 540 + 64 + 2 * contact_sheet.GAP))
                rgb = sheet.convert("RGB")
                gap = contact_sheet.GAP
                # the blank middle cell stays background; the third cell holds the second shot
                self.assertEqual(rgb.getpixel((gap + 960 + gap + 100, gap + 100)), contact_sheet.BACKGROUND)
                self.assertNotEqual(rgb.getpixel((gap + 2 * (960 + gap) + 100, gap + 100)), contact_sheet.BACKGROUND)

    def test_missing_shot_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            (folder / "m.json").write_text(json.dumps([{"file": "nope.png", "caption": "x"}]), encoding="utf-8")
            code = contact_sheet.main(["--manifest", str(folder / "m.json"), "--out", str(folder / "sheet.png")])
            self.assertEqual(code, 1)
            self.assertFalse((folder / "sheet.png").exists())

    def test_empty_manifest_fails(self):
        with self.assertRaises(ValueError):
            contact_sheet.build([])


if __name__ == "__main__":
    unittest.main()
