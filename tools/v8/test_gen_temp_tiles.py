"""Unit tests for tools/v8/gen_temp_tiles.py: atlas shape, name contract, layer rules,
palette hierarchy and byte-identical determinism."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_temp_tiles  # noqa: E402

TERRAIN = ["grass_0", "grass_1", "grass_2", "grass_3", "grass_dark_0", "grass_dark_1",
           "dirt_0", "dirt_1", "water_0", "water_1", "shallow", "hill", "mountain",
           "forest_floor_0", "forest_floor_1"]
NATURE = ["tree_0", "tree_1", "tree_2", "bush_full", "bush_bare", "deadwood"]
ITEMS = ["logs_s", "logs_m", "logs_l"]
SMALL_BUILDINGS = ["hearth", "shelter"]
PERSON = ["person_skin", "person_hair", "person_tunic"]
CHILD = ["child_skin", "child_hair", "child_tunic"]
ICONS = ["carry_log", "tool_axe", "tool_hammer"]
OVERLAYS = ["mark_chop", "zone_tint", "zone_edge", "select"]

EXPECTED_SIZE = {n: (16, 16) for n in TERRAIN + NATURE + ITEMS + SMALL_BUILDINGS + PERSON + CHILD + ICONS + OVERLAYS}
EXPECTED_SIZE.update({"hut": (48, 48), "hut_ghost": (48, 48), "shadow": (16, 5)})


def luminance(p):
    return (299 * p[0] + 587 * p[1] + 114 * p[2]) / 1000


def visible(img):
    return [p for p in img.get_flattened_data() if p[3] > 0]


def stdev(values):
    m = sum(values) / len(values)
    return (sum((v - m) ** 2 for v in values) / len(values)) ** 0.5


class GenTempTilesTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.dir_a = Path(cls.tmp.name) / "a"
        cls.dir_b = Path(cls.tmp.name) / "b"
        gen_temp_tiles.write(cls.dir_a)
        gen_temp_tiles.write(cls.dir_b)
        cls.atlas = Image.open(cls.dir_a / "tiles.png")
        cls.atlas.load()
        cls.meta = json.loads((cls.dir_a / "tiles.json").read_text())
        cls.rects = cls.meta["tiles"]

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def tile(self, name):
        x, y, w, h = self.rects[name]
        return self.atlas.crop((x, y, x + w, y + h))

    def test_atlas_size_mode_and_header(self):
        self.assertEqual(self.atlas.size, (256, 128))
        self.assertEqual(self.atlas.mode, "RGBA")
        self.assertEqual(self.meta["tile_size"], 16)
        self.assertEqual(self.meta["size"], [256, 128])
        self.assertEqual(self.meta["atlas"], "tiles.png")

    def test_every_required_name_has_the_right_size(self):
        for name, (w, h) in EXPECTED_SIZE.items():
            with self.subTest(name=name):
                self.assertIn(name, self.rects)
                self.assertEqual(self.rects[name][2:], [w, h])
        self.assertEqual(set(self.rects) - set(EXPECTED_SIZE), set(), "unexpected extra tiles")

    def test_rects_on_grid_inside_and_disjoint(self):
        items = sorted(self.rects.items())
        for name, (x, y, w, h) in items:
            with self.subTest(name=name):
                self.assertEqual((x % 16, y % 16), (0, 0))
                self.assertTrue(0 <= x and 0 <= y and x + w <= 256 and y + h <= 128)
        for i, (na, (ax, ay, aw, ah)) in enumerate(items):
            for nb, (bx, by, bw, bh) in items[i + 1:]:
                overlap = ax < bx + bw and bx < ax + aw and ay < by + bh and by < ay + ah
                self.assertFalse(overlap, f"{na} overlaps {nb}")

    def test_terrain_tiles_are_fully_opaque(self):
        for name in TERRAIN:
            with self.subTest(name=name):
                self.assertEqual(min(p[3] for p in self.tile(name).get_flattened_data()), 255)

    def test_sprite_tiles_have_transparent_background(self):
        for name in NATURE + ITEMS + SMALL_BUILDINGS + PERSON + CHILD + ICONS + ["shadow"]:
            with self.subTest(name=name):
                alphas = [p[3] for p in self.tile(name).get_flattened_data()]
                self.assertIn(0, alphas)
                self.assertTrue(any(alphas), "tile is empty")

    def test_person_child_and_ghost_layers_are_greyscale(self):
        for name in PERSON + CHILD + ["hut_ghost", "zone_tint", "zone_edge"]:
            with self.subTest(name=name):
                px = visible(self.tile(name))
                self.assertTrue(px)
                self.assertTrue(all(r == g == b for r, g, b, _ in px))

    def test_child_content_fits_the_centred_12x12_box(self):
        for name in CHILD:
            with self.subTest(name=name):
                x0, y0, x1, y1 = self.tile(name).getbbox()
                self.assertTrue(x0 >= 2 and y0 >= 2 and x1 <= 14 and y1 <= 14, (x0, y0, x1, y1))

    def test_tool_icons_stay_in_the_top_right_quarter(self):
        for name in ("tool_axe", "tool_hammer"):
            with self.subTest(name=name):
                x0, y0, x1, y1 = self.tile(name).getbbox()
                self.assertTrue(x0 >= 8 and y1 <= 8, (x0, y0, x1, y1))

    def test_translucent_alphas(self):
        self.assertEqual({p[3] for p in self.tile("zone_tint").get_flattened_data()}, {89})
        ghost_alphas = {p[3] for p in visible(self.tile("hut_ghost"))}
        self.assertTrue(all(a < 255 for a in ghost_alphas))
        self.assertIn(128, ghost_alphas)
        self.assertLessEqual(max(p[3] for p in self.tile("shadow").get_flattened_data()), 110)
        edge = self.tile("zone_edge")
        self.assertEqual(edge.getpixel((8, 8))[3], 0)
        self.assertEqual(edge.getpixel((0, 8))[3], 255)

    def test_ground_is_the_calmest_layer(self):
        # luminance spread inside each ground tile is far below that of any object, item or icon
        ground = max(stdev([luminance(p) for p in visible(self.tile(n))]) for n in TERRAIN)
        objects = min(stdev([luminance(p) for p in visible(self.tile(n))])
                      for n in NATURE + ITEMS + SMALL_BUILDINGS + ICONS + ["hut"])
        self.assertLess(ground, 6.0)
        self.assertGreater(objects, 3 * ground)

    def test_ground_is_darker_than_people_and_events(self):
        brightest_ground = max(luminance(p) for n in TERRAIN for p in visible(self.tile(n)))
        self.assertLess(brightest_ground, 110)
        for name in PERSON + CHILD:
            with self.subTest(name=name):
                values = sorted(luminance(p) for p in visible(self.tile(name)))
                lit = values[len(values) * 3 // 4]  # upper quartile = the layer's base tone
                self.assertGreater(lit, brightest_ground + 60)
        for name in ("mark_chop", "select"):
            with self.subTest(name=name):
                peak = max(luminance(p) for p in visible(self.tile(name)))
                self.assertGreater(peak, brightest_ground + 80)

    def test_variants_differ(self):
        for group in (["grass_0", "grass_1", "grass_2", "grass_3"], ["water_0", "water_1"],
                      ["tree_0", "tree_1", "tree_2"], ["logs_s", "logs_m", "logs_l"]):
            with self.subTest(group=group):
                self.assertEqual(len({self.tile(n).tobytes() for n in group}), len(group))

    def test_two_runs_are_byte_identical(self):
        for fname in ("tiles.png", "tiles.json"):
            with self.subTest(file=fname):
                self.assertEqual((self.dir_a / fname).read_bytes(), (self.dir_b / fname).read_bytes())

    def test_json_keys_sorted(self):
        self.assertEqual(list(self.rects), sorted(self.rects))
        self.assertEqual(list(self.meta), sorted(self.meta))


if __name__ == "__main__":
    unittest.main()
