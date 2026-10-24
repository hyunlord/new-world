"""Tests of the fixed people skin/hair masks (tools/v8/art_masks.py, M-ART2 goal §3.5 and §9).

All 340 route I frames. Each check is a function that returns its violations; every test also feeds that function a
perturbed copy (a clothing-coloured pixel, a polygon pixel, a pixel outside the regions, a grown mask, a gutted mask,
a changed or missing file) and asserts that it reports it, so none of them can pass vacuously.
"""
from __future__ import annotations

import io
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/v8"))
import art_masks as M  # noqa: E402

REGIONS = M.REGIONS_PATH
MASKS_JSON = M.OUT / "MASKS.json"


def check_files(root: Path, frames: dict) -> list[str]:
    """Every frame has its mask file: RGBA, the frame's size, R/G in {0, 255}, B = 0, A = 255."""
    problems = []
    for (look, facing, pose), rec in frames.items():
        p = root / f"assets/v8art/people/masks/{look}/{facing}/{pose}.png"
        if not p.is_file():
            problems.append(f"missing {p}")
            continue
        with Image.open(p) as im:
            mode, arr = im.mode, np.array(im)
        h, w = rec["frame"].shape[:2]
        if mode != "RGBA" or arr.shape != (h, w, 4):
            problems.append(f"{p}: {mode} {arr.shape} != RGBA {(h, w, 4)}")
            continue
        if not (np.isin(arr[..., :2], (0, 255)).all() and (arr[..., 2] == 0).all() and (arr[..., 3] == 255).all()):
            problems.append(f"{p}: channel values outside the contract")
    return problems


def clothing_in_skin(skin: np.ndarray, lab: np.ndarray, centres: dict) -> int:
    return int((skin & (M.nearest_kind(lab, centres) == 0)).sum())


def polygon_pixels(mask: np.ndarray, env: dict) -> int:
    return int((mask & M.raster(mask.shape, M.polygons(env, "garment_exclusions", "prop_exclusions"))).sum())


def outside_regions(skin: np.ndarray, regions: dict) -> int:
    return int((skin & ~M.box_mask(skin.shape, M.region_boxes(regions))).sum())


def grown(after: np.ndarray, before: np.ndarray) -> int:
    return int((after & ~before).sum())


def coverage_ok(after: int, before: int) -> bool:
    return after * 100 >= before * 80


class ArtMasksTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.src = M.load_sources()
        cls.frames = cls.src["frames"]
        cls.files, cls.doc, _ = M.build(cls.src)
        cls.labs = {k: M.frame_lab(r["frame"]) for k, r in cls.frames.items()}
        cls.centres = {look: M.look_centres(look, cls.src, cls.labs) for look in sorted({k[0] for k in cls.frames})}
        cls.regions = json.loads(REGIONS.read_text())["frames"]
        cls.masks = {}
        for (look, facing, pose) in cls.frames:
            with Image.open(M.OUT / look / facing / f"{pose}.png") as im:
                arr = np.array(im.convert("RGBA"))
            cls.masks[(look, facing, pose)] = (arr[..., 0] > 127, arr[..., 1] > 127)

    def test_340_mask_files_with_the_frame_sizes(self) -> None:
        self.assertEqual(len(self.frames), 340)
        self.assertEqual(len([p for p in self.files if p.endswith(".png")]), 340)
        self.assertEqual(check_files(ROOT, self.frames), [])
        with tempfile.TemporaryDirectory() as tmp:  # negative: a wrong size and a missing file are reported
            k0, k1 = sorted(self.frames)[:2]
            d = Path(tmp) / f"assets/v8art/people/masks/{k0[0]}/{k0[1]}"
            d.mkdir(parents=True)
            Image.new("RGBA", (3, 3), (0, 0, 0, 255)).save(d / f"{k0[2]}.png")
            problems = check_files(Path(tmp), {k: self.frames[k] for k in (k0, k1)})
            self.assertEqual(len(problems), 2, problems)
            self.assertTrue(problems[1].startswith("missing"), problems)

    def test_check_is_clean_and_the_build_is_idempotent(self) -> None:
        self.assertEqual(M.differences(self.files), ([], []))
        again, _, _ = M.build(self.src)
        self.assertEqual(again, self.files)
        with tempfile.TemporaryDirectory() as tmp:  # negative: one changed byte, one missing file, one stray file
            out = Path(tmp) / "assets/v8art/people"
            shutil.copytree(M.OUT, out / "masks")
            shutil.copy(REGIONS, out / "skin_regions.json")
            self.assertEqual(M.differences(self.files, Path(tmp)), ([], []))
            first = sorted(p for p in self.files if p.endswith(".png"))[0]
            data = bytearray((Path(tmp) / first).read_bytes())
            data[-20] ^= 1
            (Path(tmp) / first).write_bytes(bytes(data))
            (Path(tmp) / "assets/v8art/people/masks/skin_regions.json").unlink()
            (out / "masks/stray.png").write_bytes(b"x")
            bad, stale = M.differences(self.files, Path(tmp))
            self.assertEqual(sorted(bad), sorted([first, "assets/v8art/people/masks/skin_regions.json"]))
            self.assertEqual(stale, ["assets/v8art/people/masks/stray.png"])

    def test_no_clothing_cluster_pixel_in_any_skin_mask(self) -> None:
        bad = {k: n for k, (skin, _) in self.masks.items()
               if (n := clothing_in_skin(skin, self.labs[k], self.centres[k[0]]))}
        self.assertEqual(bad, {})
        # negative: a fur pixel (the colour nearest the look's darkest clothing centre) added to a skin mask
        key = ("adult_F_fur_cloak_bun", "S", "idle")
        skin = self.masks[key][0].copy()
        kinds = M.nearest_kind(self.labs[key], self.centres[key[0]])
        ys, xs = np.nonzero((kinds == 0) & ~skin)
        skin[ys[0], xs[0]] = True
        self.assertEqual(clothing_in_skin(skin, self.labs[key], self.centres[key[0]]), 1)

    def test_no_garment_or_prop_polygon_pixel_in_any_skin_or_hair_mask(self) -> None:
        bad = {}
        for k, (skin, hair) in self.masks.items():
            env = self.frames[k]["env"]
            n = polygon_pixels(skin, env) + polygon_pixels(hair, env)
            if n:
                bad[k] = n
        self.assertEqual(bad, {})
        key = ("adult_M_fur_cloak_bun", "E", "carry")  # negative: a cloak pixel in the hair mask
        env = self.frames[key]["env"]
        hair = self.masks[key][1].copy()
        ys, xs = np.nonzero(M.raster(hair.shape, M.polygons(env, "garment_exclusions")))
        hair[ys[0], xs[0]] = True
        self.assertEqual(polygon_pixels(hair, env), 1)

    def test_box_rule_every_skin_pixel_inside_its_frame_regions(self) -> None:
        self.assertEqual(len(self.regions), 340)
        bad = {k: n for k, (skin, _) in self.masks.items()
               if (n := outside_regions(skin, self.regions["/".join(k)]))}
        self.assertEqual(bad, {})
        key = ("child_M_fur_cloak_short", "S", "idle")  # negative: a pixel on the cloak's torso, no torso region
        regions = self.regions["/".join(key)]
        self.assertNotIn("torso", regions)
        skin = self.masks[key][0].copy()
        ys, xs = np.nonzero(~M.box_mask(skin.shape, M.region_boxes(regions)) & (self.frames[key]["frame"][..., 3] > 0))
        skin[ys[0], xs[0]] = True
        self.assertEqual(outside_regions(skin, regions), 1)

    def test_regions_come_from_the_source_geometry(self) -> None:
        """The visible pixels of the S/idle anatomy face/forearm boxes and of the owner skin supports lie inside the
        regions (the boxes also hold transparent canvas, which no mask can select); only the bare tops (loincloth,
        hide wrap) get a torso region."""
        def missed(k: tuple, regions: dict) -> int:
            rec = self.frames[k]
            inside = M.box_mask(rec["frame"].shape[:2], M.region_boxes(regions)) | (rec["frame"][..., 3] < 16)
            need = M.raster(inside.shape, M.polygons(rec["env"], "skin_supports"))
            if k[1:] == ("S", "idle"):
                an = self.src["anatomy"][k[0]]
                need |= M.box_mask(inside.shape, [an["face"]] + an["forearms"])
            return int((need & ~inside).sum())

        for k in self.frames:
            regions = self.regions["/".join(k)]
            self.assertEqual("torso" in regions, M.top_of(k[0]) in ("loincloth", "hide_wrap"), k)
            self.assertEqual(missed(k, regions), 0, k)
        key = ("adult_F_fur_cloak_bun", "S", "idle")  # negative: without the arm boxes the owner arms fall outside
        regions = dict(self.regions["/".join(key)], arms=[])
        self.assertGreater(missed(key, regions), 0)

    def test_masks_only_shrink(self) -> None:
        bad = {}
        for k, (skin, hair) in self.masks.items():
            n = grown(skin, self.frames[k]["skin0"]) + grown(hair, self.frames[k]["hair0"])
            if n:
                bad[k] = n
        self.assertEqual(bad, {})
        key = ("adult_F_loincloth_short", "E", "walk_a")  # negative: one pixel the original mask did not select
        skin = self.masks[key][0].copy()
        ys, xs = np.nonzero(~self.frames[key]["skin0"])
        skin[ys[0], xs[0]] = True
        self.assertEqual(grown(skin, self.frames[key]["skin0"]), 1)

    def test_skin_coverage_floor(self) -> None:
        # observed at c8ed8608: skin 63,672 -> 63,492 (99.7 %), hair 20,754 -> 20,700; 103 of 340 frames lose a
        # pixel, the largest loss in one frame is 5.7 % (child_M_fur_cloak_bun E sleep, 6 of 106)
        before = sum(int(r["skin0"].sum()) for r in self.frames.values())
        after = sum(int(s.sum()) for s, _ in self.masks.values())
        self.assertTrue(coverage_ok(after, before), (after, before))
        self.assertEqual((after, before), (self.doc["totals"]["skin_after"], self.doc["totals"]["skin_before"]))
        self.assertFalse(coverage_ok(before * 79 // 100, before))  # negative: a gutted mask set fails

    def test_masks_json_records_every_frame(self) -> None:
        doc = json.loads(MASKS_JSON.read_text())
        self.assertEqual(doc["schema_version"], 1)
        self.assertEqual(doc["source_commit"], M.SOURCE_COMMIT)
        self.assertEqual(len(doc["frames"]), 340)
        for row in doc["frames"]:
            data = (ROOT / row["output"]).read_bytes()
            self.assertEqual(M.sha256(data), row["output_sha256"])
            removed = sum(row["skin_removed"].values())
            self.assertEqual(row["skin_before"] - removed, row["skin_after"])
            self.assertEqual(row["hair_before"] - sum(row["hair_removed"].values()), row["hair_after"])
        json.loads(io.BytesIO(self.files["assets/v8art/people/masks/MASKS.json"]).read())


if __name__ == "__main__":
    unittest.main()
