"""Unit tests for tools/v8/gen_tiles.py: sheet size and name contract, every sprite inside its rect, the outline
rule, greyscale person layers, value hierarchy (ground calm, people brightest, tunics lighter than the forest
floor), the M3 sprites (wall autotiles follow their neighbour mask, doors join their wall, floors calm, ghosts
translucent), the committed sheet equal to a fresh run, and byte-identical determinism."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_tiles  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
OUTLINE = gen_tiles.OUTLINE

GROUND = (["grass_%d" % i for i in range(3)] + ["grass_dark_%d" % i for i in range(3)]
          + ["forest_floor_%d" % i for i in range(4)] + ["dirt_%d" % i for i in range(4)]
          + ["dirt_edge_0", "dirt_edge_1"] + ["hill_%d" % i for i in range(3)]
          + ["mountain_%d" % i for i in range(3)] + ["mountain_top_%d" % i for i in range(3)] + ["mountain_cliff"]
          + ["water_%d" % i for i in range(4)] + ["shallow_%d" % i for i in range(4)]
          + ["bank_%d" % m for m in range(1, 16)] + ["bank_in_" + c for c in ("ne", "nw", "se", "sw")]
          + ["stock_floor"])
ZONE = ["stock_edge_" + s for s in "nesw"] + ["stock_stake_" + c for c in ("nw", "ne", "sw", "se")]
CANOPY = ["tree_%s_%d" % (s, z) for s in ("oak", "birch", "pine") for z in range(3)]
NATURE = ["stump", "bush_full", "bush_bare", "deadwood"]
ITEMS = ["logs_s", "logs_m", "logs_l"]
BUILDINGS = ["hut", "shelter", "hearth_0", "hearth_1"]
FRAMES = ["0", "1", "sleep"]
LAYERS = ["skin", "hair", "tunic", "outline", "halo"]
PEOPLE = ["%s_%s_%s" % (p, l, f) for p in ("person", "child") for l in LAYERS for f in FRAMES]
CARRY = ["carry_back", "carry_front", "tool_axe", "tool_hammer"]
MARKS = ["mark_chop", "select", "hover"]
ICONS = ["icon_" + k for k in ("chop", "stockpile", "hut", "remove", "cancel")]
GHOSTS = ["hut_ghost", "hut_ghost_place"]
# M2 temporaries: resource nodes, workshops, item kinds, the quarry mark and two order icons.
M2_OBJECTS = (["boulder", "flint_outcrop", "knapping_spot"]
              + ["item_" + k for k in ("firewood", "stone", "flint", "blade", "axe", "hide")])
M2_WIDE = ["wood_bench", "drying_rack"]
M2_ICONS = ["icon_build", "icon_quarry"]
# M3 temporaries: wall autotiles (16 masks each), doors, floors, bedroll, blueprint ghosts, marks, order icons.
M3_WALLS = ["%s_wall_%d" % (m, k) for m in ("log", "stone") for k in range(16)]
M3_DOORS = ["door_h", "door_v"]
M3_FLOORS = ["floor_packed", "floor_rock"]
M3_GHOSTS = ["ghost_wall", "ghost_door", "ghost_floor", "ghost_bedroll", "ghost_refused"]
M3_MARKS = ["mark_dig", "mark_deconstruct"]
M3_ICONS = ["icon_dig", "icon_deconstruct"]
M3_16 = M3_WALLS + M3_DOORS + M3_FLOORS + ["bedroll"] + M3_GHOSTS + M3_MARKS

SIZE = {n: (16, 16) for n in GROUND + ZONE + NATURE + ITEMS + CARRY + MARKS + ["hearth_0", "hearth_1"]
        + M2_OBJECTS + ["mark_quarry"] + M3_16}
SIZE.update({n: (32, 16) for n in M2_WIDE})
SIZE.update({n: (12, 12) for n in M3_ICONS})
SIZE.update({n: (12, 12) for n in M2_ICONS})
SIZE.update({n: (24, 24) for n in CANOPY})
SIZE.update({n: (12, 12) for n in ICONS})
SIZE.update({"hut": (48, 48), "hut_ghost": (48, 48), "hut_ghost_place": (48, 48), "shelter": (16, 20),
             "tree_shadow": (20, 8), "shadow": (10, 4)})
SIZE.update({n: ((16, 8) if n.endswith("_sleep") else (16, 16)) for n in PEOPLE})
# Object sprites must carry the outline colour (craft rule 1); people carry it in their outline layer.
OUTLINED = (CANOPY + NATURE + ITEMS + BUILDINGS + CARRY + ["mark_chop"] + ICONS
            + M2_OBJECTS + M2_WIDE + M2_ICONS + ["mark_quarry"]
            + M3_WALLS + M3_DOORS + ["bedroll"] + M3_MARKS + M3_ICONS
            + [n for n in PEOPLE if "_outline_" in n] + ["stock_stake_" + c for c in ("nw", "ne", "sw", "se")])


def luminance(p):
    return (299 * p[0] + 587 * p[1] + 114 * p[2]) / 1000


def visible(img):
    return [p for p in img.get_flattened_data() if p[3] > 0]


def stdev(values):
    m = sum(values) / len(values)
    return (sum((v - m) ** 2 for v in values) / len(values)) ** 0.5


def img_edge(img, side):
    """Whether any opaque pixel lies on that edge of a 16x16 image."""
    pts = {"n": [(x, 0) for x in range(16)], "s": [(x, 15) for x in range(16)],
           "w": [(0, y) for y in range(16)], "e": [(15, y) for y in range(16)]}[side]
    return any(img.getpixel(p)[3] for p in pts)


def hex_rgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


class GenTilesTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.dir_a = Path(cls.tmp.name) / "a"
        cls.dir_b = Path(cls.tmp.name) / "b"
        gen_tiles.write(cls.dir_a)
        gen_tiles.write(cls.dir_b)
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

    def test_sheet_size_mode_and_header(self):
        w, h = self.atlas.size
        self.assertEqual(self.atlas.mode, "RGBA")
        self.assertEqual(w, gen_tiles.SHEET_W)
        self.assertEqual(h % 16, 0)
        self.assertLessEqual(h, 512)
        self.assertEqual(self.meta["size"], [w, h])
        self.assertEqual(self.meta["tile_size"], 16)
        self.assertEqual(self.meta["atlas"], "tiles.png")

    def test_every_name_present_with_its_size(self):
        for name, (w, h) in SIZE.items():
            with self.subTest(name=name):
                self.assertIn(name, self.rects)
                self.assertEqual(self.rects[name][2:], [w, h])
        self.assertEqual(set(self.rects) - set(SIZE), set(), "unexpected extra sprites")

    def test_rects_on_grid_inside_and_disjoint(self):
        items = sorted(self.rects.items())
        W, H = self.atlas.size
        for name, (x, y, w, h) in items:
            with self.subTest(name=name):
                self.assertEqual((x % 16, y % 16), (0, 0))
                self.assertTrue(0 <= x and 0 <= y and x + w <= W and y + h <= H)
        for i, (na, (ax, ay, aw, ah)) in enumerate(items):
            for nb, (bx, by, bw, bh) in items[i + 1:]:
                overlap = ax < bx + bw and bx < ax + aw and ay < by + bh and by < ay + ah
                self.assertFalse(overlap, f"{na} overlaps {nb}")

    def test_no_sprite_exceeds_its_rect(self):
        # every opaque pixel of the sheet lies inside some sprite's rect
        inside = set()
        for x, y, w, h in self.rects.values():
            for yy in range(y, y + h):
                for xx in range(x, x + w):
                    inside.add((xx, yy))
        px = self.atlas.load()
        W, H = self.atlas.size
        stray = [(x, y) for y in range(H) for x in range(W) if px[x, y][3] and (x, y) not in inside]
        self.assertEqual(stray, [])
        for name in SIZE:
            with self.subTest(name=name):
                self.assertIsNotNone(self.tile(name).getbbox(), "empty sprite")

    def test_ground_tiles_opaque_without_outline(self):
        for name in GROUND:
            with self.subTest(name=name):
                px = self.tile(name).get_flattened_data()
                self.assertEqual(min(p[3] for p in px), 255)
                self.assertNotIn(OUTLINE, {p[:3] for p in px})

    def test_outline_colour_on_every_object_sprite(self):
        for name in OUTLINED:
            with self.subTest(name=name):
                self.assertIn(OUTLINE, {p[:3] for p in visible(self.tile(name))})

    def test_object_sprites_have_transparent_background(self):
        for name in (CANOPY + NATURE + ITEMS + ["shelter", "hearth_0", "hearth_1"] + CARRY
                     + ["mark_chop", "shadow", "tree_shadow"] + M2_OBJECTS + M2_WIDE + ["mark_quarry"]
                     + M3_WALLS + M3_DOORS + ["bedroll"] + M3_MARKS):
            with self.subTest(name=name):
                alphas = [p[3] for p in self.tile(name).get_flattened_data()]
                self.assertIn(0, alphas)

    def test_person_layers_greyscale_and_partition_the_figure(self):
        for prefix in ("person", "child"):
            for frame in FRAMES:
                with self.subTest(prefix=prefix, frame=frame):
                    owner = {}
                    for layer in ("skin", "hair", "tunic", "halo"):
                        px = visible(self.tile(f"{prefix}_{layer}_{frame}"))
                        self.assertTrue(px, layer)
                        self.assertTrue(all(r == g == b for r, g, b, _ in px), layer)
                    for layer in ("skin", "hair", "tunic", "outline"):
                        img = self.tile(f"{prefix}_{layer}_{frame}")
                        for y in range(img.height):
                            for x in range(img.width):
                                if img.getpixel((x, y))[3]:
                                    self.assertNotIn((x, y), owner, f"{layer} overlaps {owner.get((x, y))}")
                                    owner[(x, y)] = layer
                    halo = self.tile(f"{prefix}_halo_{frame}")
                    for (x, y) in owner:
                        self.assertEqual(halo.getpixel((x, y))[3], 0, "the halo lies outside the figure")

    def test_figure_sizes_and_feet(self):
        x0, y0, x1, y1 = self.tile("person_outline_0").getbbox()
        self.assertEqual((y0, y1), (0, 16), "adult: head on row 0, feet on row 15")
        x0, y0, x1, y1 = self.tile("child_outline_0").getbbox()
        self.assertLessEqual(x1 - x0, 12)
        self.assertLessEqual(y1 - y0, 10, "a child is clearly shorter than an adult (16)")
        self.assertEqual(y1, 16, "child feet on row 15")
        self.assertEqual(self.tile("person_outline_sleep").getbbox()[3] - self.tile("person_outline_sleep").getbbox()[1], 6)

    def test_walk_frames_differ_only_below_the_belt(self):
        for prefix in ("person", "child"):
            with self.subTest(prefix=prefix):
                a, b = self.tile(f"{prefix}_outline_0"), self.tile(f"{prefix}_outline_1")
                self.assertNotEqual(a.tobytes(), b.tobytes())
                self.assertEqual(a.crop((0, 0, 16, 14)).tobytes(), b.crop((0, 0, 16, 14)).tobytes())

    def test_tunics_at_least_25_percent_lighter_than_forest_floor(self):
        floor = [luminance(p) for n in GROUND if n.startswith("forest_floor") for p in visible(self.tile(n))]
        floor_mean = sum(floor) / len(floor)
        tunic_grey = sorted(p[0] for p in visible(self.tile("person_tunic_0")))
        base = tunic_grey[len(tunic_grey) // 2] / 255.0  # the layer's median grey = the drawn base tone
        for h in self.meta["palettes"]["tunics"]:
            with self.subTest(tunic=h):
                drawn = tuple(c * base for c in hex_rgb(h))
                self.assertGreaterEqual(luminance(drawn), 1.25 * floor_mean)

    def test_ground_is_the_calmest_layer(self):
        ground = max(stdev([luminance(p) for p in visible(self.tile(n))])
                     for n in GROUND if not n.startswith(("bank", "mountain", "dirt_edge")))
        objects = min(stdev([luminance(p) for p in visible(self.tile(n))])
                      for n in CANOPY + NATURE + ITEMS + BUILDINGS)
        self.assertLess(ground, 6.0)
        self.assertGreater(objects, 2 * ground)

    def test_people_are_brighter_than_the_ground(self):
        brightest_ground = max(luminance(p) for n in GROUND for p in visible(self.tile(n)))
        self.assertLess(brightest_ground, 140)
        skin = hex_rgb(self.meta["palettes"]["skin"])
        self.assertGreater(luminance(tuple(c * 232 / 255 for c in skin)), brightest_ground + 30)
        for name in ("mark_chop", "select"):
            with self.subTest(name=name):
                self.assertGreater(max(luminance(p) for p in visible(self.tile(name))), brightest_ground + 80)

    def test_grass_families_differ_by_about_ten(self):
        def mean(n):
            v = [luminance(p) for p in visible(self.tile(n))]
            return sum(v) / len(v)
        g = sum(mean("grass_%d" % i) for i in range(3)) / 3
        dark = sum(mean("grass_dark_%d" % i) for i in range(3)) / 3
        self.assertTrue(6 <= g - dark <= 14, (g, dark))

    def test_canopy_overhangs_its_tile_and_sizes_grow(self):
        for name in CANOPY:
            with self.subTest(name=name):
                x0, y0, x1, y1 = self.tile(name).getbbox()
                self.assertGreater(x1 - x0, 16, "the canopy is wider than its tile")
        self.assertGreater(self.tile("tree_oak_2").getbbox()[2] - self.tile("tree_oak_2").getbbox()[0],
                           self.tile("tree_oak_0").getbbox()[2] - self.tile("tree_oak_0").getbbox()[0])

    def test_translucent_alphas(self):
        self.assertEqual({p[3] for p in visible(self.tile("tree_shadow"))}, {90})
        self.assertEqual({p[3] for p in visible(self.tile("shadow"))}, {110})
        self.assertEqual({p[3] for p in visible(self.tile("hover"))}, {120})
        ghost = {p[3] for p in visible(self.tile("hut_ghost"))}
        self.assertIn(80, ghost)
        self.assertTrue(all(a < 255 for a in ghost))

    def test_log_piles_grow(self):
        sizes = [len(visible(self.tile(n))) for n in ITEMS]
        self.assertEqual(sizes, sorted(sizes))
        self.assertGreater(sizes[2], 180, "the large pile fills its tile")

    def test_hut_floor_is_opaque(self):
        # the walls must not punch the floor out: every interior pixel of the hut is opaque
        hut = self.tile("hut")
        k = gen_tiles.HUT_WALL
        holes = [(x, y) for y in range(k, 48 - k) for x in range(k, 48 - k) if hut.getpixel((x, y))[3] < 255]
        self.assertEqual(holes, [])

    def test_hearth_frames_differ(self):
        self.assertNotEqual(self.tile("hearth_0").tobytes(), self.tile("hearth_1").tobytes())

    def test_palettes(self):
        pal = self.meta["palettes"]
        self.assertEqual(len(pal["tunics"]), 6)
        self.assertEqual(len(pal["hairs"]), 4)
        for key in ("skin", "elder_hair", "halo"):
            self.assertEqual(len(pal[key]), 6)

    def test_two_runs_are_byte_identical(self):
        for fname in ("tiles.png", "tiles.json"):
            with self.subTest(file=fname):
                self.assertEqual((self.dir_a / fname).read_bytes(), (self.dir_b / fname).read_bytes())

    def test_committed_sheet_is_current(self):
        for fname in ("tiles.png", "tiles.json"):
            with self.subTest(file=fname):
                self.assertEqual((REPO / "assets" / "v8" / fname).read_bytes(), (self.dir_a / fname).read_bytes(),
                                 "run python3 tools/v8/gen_tiles.py")

    def test_m3_sprites_in_json_16x16_and_not_empty(self):
        js = json.loads((REPO / "assets" / "v8" / "tiles.json").read_text())["tiles"]
        for name in M3_16:
            with self.subTest(name=name):
                self.assertIn(name, js, "committed tiles.json")
                self.assertEqual(js[name][2:], [16, 16])
                self.assertTrue(any(p[3] for p in self.tile(name).get_flattened_data()), "fully transparent")

    def test_wall_autotile_follows_the_mask(self):
        # bit 1 N, 2 E, 4 S, 8 W: a connected side reaches that tile edge, an open side does not; straight runs
        # (5, 10) have no post, every other mask has the 12x12 post (its corner pixel (2, 2) is opaque)
        edges = {1: [(x, 0) for x in range(16)], 2: [(15, y) for y in range(16)],
                 4: [(x, 15) for x in range(16)], 8: [(0, y) for y in range(16)]}
        for kind in ("log", "stone"):
            for mask in range(16):
                img = self.tile("%s_wall_%d" % (kind, mask))
                with self.subTest(kind=kind, mask=mask):
                    for bit, pts in edges.items():
                        touches = any(img.getpixel(p)[3] for p in pts)
                        self.assertEqual(touches, bool(mask & bit), "side %d" % bit)
                    self.assertEqual(img.getpixel((2, 2))[3] > 0, mask not in (5, 10))

    def test_doors_join_their_wall(self):
        h, v = self.tile("door_h"), self.tile("door_v")
        self.assertTrue(img_edge(h, "w") and img_edge(h, "e") and not img_edge(h, "n") and not img_edge(h, "s"))
        self.assertTrue(img_edge(v, "n") and img_edge(v, "s") and not img_edge(v, "w") and not img_edge(v, "e"))

    def test_m3_floors_opaque_calm_without_outline(self):
        for name in M3_FLOORS:
            with self.subTest(name=name):
                px = self.tile(name).get_flattened_data()
                self.assertEqual(min(p[3] for p in px), 255)
                self.assertNotIn(OUTLINE, {p[:3] for p in px})
                self.assertLess(stdev([luminance(p) for p in px]), 6.0)
                self.assertLess(max(luminance(p) for p in px), 140)
        dirt = sum(luminance(p) for p in visible(self.tile("dirt_0"))) / 256
        packed = sum(luminance(p) for p in visible(self.tile("floor_packed"))) / 256
        self.assertLess(packed, dirt, "packed earth is a little darker than worn earth")

    def test_m3_ghosts_translucent_and_refused_is_red(self):
        for name in M3_GHOSTS:
            with self.subTest(name=name):
                self.assertTrue(all(p[3] < 255 for p in visible(self.tile(name))))
        blue = [p for p in visible(self.tile("ghost_wall"))]
        red = [p for p in visible(self.tile("ghost_refused"))]
        self.assertTrue(all(p[2] > p[0] for p in blue))
        self.assertTrue(all(p[0] > p[2] for p in red))

    def test_json_keys_sorted(self):
        self.assertEqual(list(self.rects), sorted(self.rects))
        self.assertEqual(list(self.meta), sorted(self.meta))


if __name__ == "__main__":
    unittest.main()
