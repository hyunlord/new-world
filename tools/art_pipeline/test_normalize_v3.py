# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_normalize_v3.py
"""Synthetic lattice and production acceptance regressions."""
import io
import unittest
import numpy as np
from PIL import Image
from pixel_normalize_v3 import recover_cell_v3, evaluate_sprite, quantize_native
from palette_tools import rgb_to_lab
from v3_semantics import semantic_counts, evaluate_animation, evaluate_stages


def enlarged(native: np.ndarray, scale: float, phase: float) -> Image.Image:
    height, width = native.shape[:2]
    ys = np.arange(int(height * scale + phase + 12))
    xs = np.arange(int(width * scale + phase + 12))
    xx = np.floor((xs + .5 - phase - 5) / scale).astype(int)
    yy = np.floor((ys + .5 - phase - 5) / scale).astype(int)
    result = np.full((len(ys), len(xs), 3), [255, 0, 255], dtype=np.uint8)
    valid = (yy[:, None] >= 0) & (yy[:, None] < height) & (xx[None, :] >= 0) & (xx[None, :] < width)
    result[valid] = native[np.clip(yy, 0, height - 1)[:, None], np.clip(xx, 0, width - 1)][valid]
    return Image.fromarray(result)


class NormalizeV3Test(unittest.TestCase):
    def test_all_scale_phase_png_and_jpeg(self) -> None:
        palette = np.array([[72, 64, 44], [95, 84, 60], [131, 110, 80], [174, 150, 111], [212, 182, 142], [210, 25, 30], [30, 130, 70], [40, 100, 200]], dtype=np.uint8)
        native = palette[np.random.default_rng(91).integers(0, len(palette), (28, 20))]
        for scale in (5.3, 5.9, 6.4):
            for phase in (0., 1.7, 3.1):
                for jpeg in (False, True):
                    with self.subTest(scale=scale, phase=phase, jpeg=jpeg):
                        source = enlarged(native, scale, phase)
                        if jpeg:
                            buffer = io.BytesIO()
                            source.save(buffer, format='JPEG', quality=88)
                            source = Image.open(io.BytesIO(buffer.getvalue())).convert('RGB')
                        recovered, metadata = recover_cell_v3(source)
                        self.assertEqual(recovered.size, (20, 28))
                        if jpeg:
                            recovered = quantize_native(recovered, [tuple(map(int, color)) for color in palette])
                        delta = np.linalg.norm(rgb_to_lab(np.asarray(recovered)[..., :3]) - rgb_to_lab(native), axis=2)
                        self.assertGreaterEqual(float((delta < 3).mean()), .90 if jpeg else .98)
                        self.assertLess(abs(metadata['period_xy'][0] - scale), .1)

    def test_harmonic_trap(self) -> None:
        native = np.tile(np.array([40, 45, 200, 205], dtype=np.uint8), 5)[None, :, None]
        native = np.broadcast_to(native, (28, 20, 3)).copy()
        native[1::2] += np.uint8(7)
        recovered, metadata = recover_cell_v3(enlarged(native, 5.9, 1.7))
        self.assertEqual(recovered.size, (20, 28))
        self.assertAlmostEqual(metadata['period_xy'][0], 5.9, delta=.1)

    def test_fidelity_excludes_removed_key(self) -> None:
        sample = Image.new('RGBA', (2, 1), (100, 80, 60, 255))
        sample.putpixel((1, 0), (240, 0, 200, 255))
        final = sample.copy()
        final.putpixel((1, 0), (0, 0, 0, 0))
        metrics = evaluate_sprite(sample, final, [(100, 80, 60)], (2, 1))
        self.assertEqual(metrics['mean_delta_e'], 0)
        self.assertEqual(metrics['keyed_out'], 1)
        self.assertTrue(metrics['passed'])

    def test_semantics_thresholds(self) -> None:
        brown = Image.new('RGBA', (4, 4), (110, 65, 30, 255))
        self.assertEqual(semantic_counts(brown), (0, 0))
        bushes = [brown.copy() for _ in range(4)]
        for image, count in zip(bushes, (6, 2, 1, 0)):
            for x in range(count):
                image.putpixel((x % 4, x // 4), (230, 20, 35, 255))
        metrics = evaluate_stages(bushes, [Image.new('RGBA', (5, 4), (255, 150, 20, 255))],
                                  [Image.new('RGBA', (8, 12))], Image.new('RGBA', (6, 8)), Image.new('RGBA', (2, 3)))
        self.assertTrue(metrics['passed'])
        self.assertFalse(evaluate_sprite(brown, brown, [(110, 65, 30)], (3, 3))['size_pass'])

    def test_animation_rejects_identical_walks(self) -> None:
        idle = Image.new('RGBA', (10, 20), (80, 60, 40, 255))
        result = evaluate_animation(idle, [idle] * 4, [idle] * 4)
        self.assertTrue(result['feet_baseline_pass'])
        self.assertTrue(result['head_top_pass'])
        self.assertFalse(result['walk_pass'])

    def test_format_and_empty_common_are_rejected(self) -> None:
        source = Image.new('RGBA', (3, 3), (80, 60, 40, 255))
        transparent = Image.new('RGBA', (3, 3))
        self.assertFalse(evaluate_sprite(source, transparent, [(80, 60, 40)], (3, 3))['passed'])
        partial = source.copy()
        partial.putpixel((0, 0), (80, 60, 40, 128))
        self.assertFalse(evaluate_sprite(source, partial, [(80, 60, 40)], (3, 3))['binary_alpha'])
        self.assertFalse(evaluate_sprite(source, source, [(81, 60, 40)], (3, 3))['format_pass'])

    def test_animation_accepts_leg_change_rejects_head_and_colors(self) -> None:
        idle = Image.new('RGBA', (10, 20), (80, 60, 40, 255))
        lifted = idle.copy()
        for x in range(8):
            lifted.putpixel((x, 19), (0, 0, 0, 0))
        walk = [lifted, idle, lifted, idle]
        self.assertTrue(evaluate_animation(idle, walk, walk)['passed'])
        high = Image.new('RGBA', (10, 22), (80, 60, 40, 255))
        self.assertFalse(evaluate_animation(idle, [high])['head_top_pass'])
        recolored = Image.new('RGBA', idle.size, (200, 30, 40, 255))
        self.assertFalse(evaluate_animation(idle, [recolored])['color_pass'])
        low = Image.new('RGBA', (10, 21))
        low.paste(idle, (0, 0))
        self.assertFalse(evaluate_animation(idle, [low])['feet_baseline_pass'])

    def test_failed_relative_and_ramp_semantics(self) -> None:
        red = Image.new('RGBA', (4, 4), (230, 20, 35, 255))
        tiny_fire = Image.new('RGBA', (4, 4), (255, 150, 20, 255))
        metric = evaluate_stages([red] * 4, [tiny_fire], [red], red, red)
        self.assertFalse(metric['berry_pass'])
        self.assertFalse(metric['fire_pass'])
        self.assertFalse(metric['child_pass'])
        self.assertFalse(metric['basket_pass'])

if __name__ == '__main__':
    unittest.main()
