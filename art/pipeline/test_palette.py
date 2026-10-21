"""Palette colour science and reproducible source-provenance tests."""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import numpy as np
from PIL import Image

from .build_palette import build_palette, hex_rgb
from .color import rgb_to_lab, vector_length
from .models import decode_json, json_array, json_integer, json_object, json_string

ROOT = Path(__file__).resolve().parents[2]


class PaletteTests(unittest.TestCase):
    def test_lab_matches_d65_reference_values(self) -> None:
        # Given standard sRGB primaries and neutral endpoints.
        rgb = np.array([[0, 0, 0], [255, 255, 255], [255, 0, 0],
                        [0, 255, 0], [0, 0, 255]], dtype=np.uint8)
        # When converted to D65 Lab.
        lab = rgb_to_lab(rgb)
        # Then published standard values agree within matrix rounding.
        expected = [[0, 0, 0], [100, 0, 0], [53.2408, 80.0925, 67.2032],
                    [87.7347, -86.1827, 83.1793], [32.2970, 79.1875, -107.8602]]
        np.testing.assert_allclose(lab, expected, atol=0.001)
        self.assertEqual(lab.dtype, np.float64)

    def test_lab_preserves_image_shape(self) -> None:
        # Given a two-dimensional image.
        rgb = np.zeros((2, 4, 3), dtype=np.uint8)
        # When converted.
        lab = rgb_to_lab(rgb)
        # Then spatial dimensions are unchanged.
        self.assertEqual(lab.shape, (2, 4, 3))

    def test_generated_palette_is_reproducible_and_semantically_complete(self) -> None:
        # Given the real committed sources and isolated output directories.
        with tempfile.TemporaryDirectory() as directory:
            first, second = Path(directory) / "a", Path(directory) / "b"
            # When the complete builder runs twice.
            build_palette(ROOT, first)
            build_palette(ROOT, second)
            # Then JSON and PNG are byte-identical, with reserved people channels.
            for name in ("worldsim_b.json", "worldsim_b.png"):
                self.assertEqual((first / name).read_bytes(), (second / name).read_bytes())
                self.assertEqual((first / name).read_bytes(), (ROOT / "art/palette" / name).read_bytes())
            payload = json_object(decode_json((first / "worldsim_b.json").read_text()))
            entries = [json_object(entry) for entry in json_array(payload["colors"])]
            people = json_object(payload["people"])
            method = json_object(payload["method"])
            colors = {json_string(entry["hex"]) for entry in entries}
            self.assertLessEqual(len(entries), 48)
            self.assertEqual(len(colors), len(entries))
            self.assertTrue({"#141210", "#9A9A9A", "#6E6E6E", "#B0B0B0", "#C0C0C0"} <= colors)
            self.assertEqual(json_array(people["skins"])[0], "#D8A884")
            self.assertEqual(len(json_array(people["skins"])), 4)
            self.assertEqual(len(json_array(people["hairs"])), 4)
            self.assertEqual(len(json_array(people["tunics"])), 6)
            self.assertEqual(method["clusters_requested"], 40)
            self.assertTrue(method["converged"])
            source_people = json_object(json_object(decode_json((ROOT / "assets/v8/tiles.json").read_text()))["palettes"])
            for family in ("hairs", "tunics"):
                self.assertEqual(people[family],
                                 ["#" + json_string(value).upper() for value in json_array(source_people[family])])
            lab = rgb_to_lab(np.array([hex_rgb(json_string(item["hex"])) for item in entries]))
            for index, item in enumerate(entries):
                if json_string(item["name"]).startswith("source_cluster_"):
                    self.assertTrue(np.all(vector_length(lab[:index] - lab[index, :], axis=1) >= 4))

    def test_cluster_swatches_are_existing_opaque_source_pixels(self) -> None:
        # Given the generated palette and both real source images.
        with tempfile.TemporaryDirectory() as directory:
            # When building using the real image distributions.
            build_palette(ROOT, Path(directory))
            # Then every cluster is snapped to a real opaque source pixel.
            payload = json_object(decode_json((Path(directory) / "worldsim_b.json").read_text()))
            method = json_object(payload["method"])
            source_colors: set[str] = set()
            for name in ("assets/v8/tiles.png", "docs/images/v8_target_view_b.png"):
                with Image.open(ROOT / name) as image:
                    pixels = np.asarray(image.convert("RGBA"), dtype=np.uint8).reshape(-1, 4)
                unique = np.unique(pixels[pixels[:, 3] == 255, :3], axis=0)
                source_colors.update(f"#{int(unique.item((index, 0))):02X}{int(unique.item((index, 1))):02X}{int(unique.item((index, 2))):02X}"
                                     for index in range(len(unique)))
            clusters = [json_object(item) for item in json_array(method["clusters"])]
            self.assertEqual(len(clusters), 40)
            self.assertTrue(all(json_string(item["hex"]) in source_colors for item in clusters))
            self.assertEqual(sum(json_integer(item["population"]) for item in clusters),
                             sum(json_integer(json_object(item)["opaque_pixels"]) for item in json_array(method["sources"])))


if __name__ == "__main__":
    _ = unittest.main()
