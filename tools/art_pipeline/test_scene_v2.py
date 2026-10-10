"""Scale fidelity and full-render readability regression controls."""
from __future__ import annotations

import unittest
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from palette_tools import load_palette
from scene_ground import ground_roles, tile_variant
from scene_layout import Placement, Sprite, render
from scene_v2 import CROP, MAP_OFFSET, frame, readability, attribution_frame

ROOT = Path(__file__).resolve().parents[2]


class SceneScaleTests(unittest.TestCase):
    def test_zoom_four_is_exact_nearest_replication(self) -> None:
        yy, xx = np.indices((1024, 1024))
        pixels = np.stack((xx % 256, yy % 256, (xx ^ yy) % 256), axis=-1).astype(np.uint8)
        world = Image.fromarray(pixels).convert("RGBA")
        actual = np.asarray(frame(world, 4, Image.new("RGBA", (1920, 1080))))
        native = np.asarray(world.crop(CROP))
        expected = np.repeat(np.repeat(native, 4, axis=0), 4, axis=1)
        np.testing.assert_array_equal(actual, expected)

    def test_overview_preserves_every_native_texel(self) -> None:
        world = Image.new("RGBA", (1024, 1024), (67, 83, 101, 255))
        result = frame(world, 1, Image.new("RGBA", (1920, 1080)))
        x, y = MAP_OFFSET
        self.assertEqual(result.size, (1920, 1080))
        np.testing.assert_array_equal(np.asarray(result)[y:y+1024,x:x+1024], np.asarray(world))

    def test_visible_and_identical_controls_at_both_scales(self) -> None:
        ground = Image.new("RGBA", (1024, 1024), (20, 20, 20, 255))
        sprite = Sprite("09_villager_gatherer", Image.new("RGBA", (12, 24), "white"), (6, 24))
        people = [Placement(sprite, 512, 512, 1)]
        hud = Image.new("RGBA", (1920, 1080))
        for zoom in (1, 4):
            full = frame(render(ground, people), zoom, hud)
            empty = frame(render(ground, people, False), zoom, hud)
            self.assertEqual(readability(full, empty, people, zoom)["visible_fraction"], 1)
            control = readability(empty, empty, people, zoom)
            self.assertEqual(control["visible_fraction"], 0)
            self.assertEqual(control["mean_delta_e76"], 0)

    def test_direction_ground_ramps_have_distinct_source_colors(self) -> None:
        for direction in ("D1", "D2", "D3"):
            palette = load_palette(ROOT / "docs/art" / direction / "palette.hex")
            roles = ground_roles(palette)
            for ramp in (roles.grass, roles.earth, roles.water):
                self.assertGreaterEqual(len(set(ramp)), 2, direction)
                self.assertTrue(set(ramp).issubset(palette))
                self.assertNotIn(ramp[0], ramp[1:])
            self.assertIn(roles.shore, palette)

    def test_attribution_excludes_overlap_objects_and_hud(self) -> None:
        sprite = Sprite("adult", Image.new("RGBA", (10, 10), "white"), (0, 0))
        blocker = Sprite("rock", Image.new("RGBA", (2, 10), "gray"), (0, 0))
        people = [Placement(sprite,512,512,1), Placement(sprite,517,512,2), Placement(blocker,512,512)]
        hud = Image.new("RGBA", (1920,1080))
        x,y = MAP_OFFSET
        ImageDraw.Draw(hud).rectangle((x+519,y+512,x+519,y+521),fill="black")
        ids = np.asarray(attribution_frame(people,1,hud))
        self.assertEqual(int(np.sum(ids==1)),30)
        self.assertEqual(int(np.sum(ids==2)),90)
        ground = Image.new("RGBA",(1024,1024),"black")
        world = ground.copy()
        underlay = ground.copy()
        for placement in people:
            world.alpha_composite(placement.sprite.image,placement.box[:2])
            if placement.villager_id is None:
                underlay.alpha_composite(placement.sprite.image,placement.box[:2])
        full = frame(world,1,hud)
        empty = frame(underlay,1,hud)
        rows = readability(full,empty,people,1,hud)["villagers"]
        self.assertEqual([row["attributed_pixel_count"] for row in rows],[30,90])
        self.assertAlmostEqual(rows[0]["mean_luma_difference"],1)
        self.assertAlmostEqual(rows[1]["mean_luma_difference"],1)

    def test_transparent_box_does_not_dilute_attributed_mean(self) -> None:
        sprite_image = Image.new("RGBA",(20,10))
        ImageDraw.Draw(sprite_image).rectangle((0,0,4,9),fill="white")
        people = [Placement(Sprite("adult",sprite_image,(0,0)),512,512,1)]
        ground = Image.new("RGBA",(1024,1024),"black")
        hud = Image.new("RGBA",(1920,1080))
        full,empty = frame(render(ground,people),1,hud),frame(ground,1,hud)
        row = readability(full,empty,people,1,hud)["villagers"][0]
        self.assertAlmostEqual(row["mean_luma_difference"],1)
        self.assertAlmostEqual(row["box_mean_luma_difference"],.25)
        self.assertAlmostEqual(row["changed_fraction"],.25)

    def test_tile_hash_matches_required_formula(self) -> None:
        for x in range(64):
            for y in range(64):
                self.assertEqual(tile_variant(x,y), ((x*73856093)^(y*19349663))%4)


if __name__ == "__main__":
    unittest.main()
