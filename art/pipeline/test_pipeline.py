"""Run: python3 -m unittest art.pipeline.test_pipeline -v. Fixtures never ship."""
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory

import numpy as np
from PIL import Image, ImageDraw

from .check import CheckRecord, check_asset, reference_distance, write_checks
from .ingest import ingest
from .layer_extract import extract_layer
from .models import RGB, Asset, AssetKind, PipelineError, Size
from .outline import OUTLINE, enforce_outline
from .palette import map_palette
from .pixelize import pixelize
from .sheet import pack_sheet
from .snap import snap

COLORS = (OUTLINE, (154, 154, 154), (192, 192, 192), (111, 176, 232))

def rgba_pixel(image: Image.Image, point: Size) -> tuple[int, ...]:
    value = image.getpixel(point)
    assert isinstance(value, tuple)
    return value

def fixtures() -> tuple[Image.Image, Image.Image, Image.Image, Image.Image]:
    circle = Image.new('RGBA', (32, 32), (255, 0, 255, 255))
    ImageDraw.Draw(circle).ellipse((4, 4, 27, 27), fill=(154, 154, 154, 255))
    gradient = Image.new('RGBA', (32, 32), (255, 0, 255, 255))
    draw = ImageDraw.Draw(gradient)
    for x in range(4, 28):
        draw.line((x, 4, x, 27), fill=(x * 8, x * 8, x * 8, 255))
    body = Image.new('RGBA', (32, 32), (255, 0, 255, 255))
    draw = ImageDraw.Draw(body)
    draw.rectangle((12, 10, 19, 27), fill=(154, 154, 154, 255))
    draw.rectangle((11, 6, 20, 12), fill=(154, 154, 154, 255))
    hat = body.copy()
    ImageDraw.Draw(hat).rectangle((9, 4, 22, 8), fill=(192, 192, 192, 255))
    return circle, gradient, body, hat

class PipelineTests(unittest.TestCase):
    def test_each_fixture_full_image_path(self) -> None:
        for index, fixture in enumerate(fixtures()):
            with self.subTest(index=index):
                keyed = ingest(fixture, (0, 0, 32, 32))
                reduced = pixelize(keyed.image, (16, 16))
                mapped = map_palette(reduced, COLORS)
                outlined = enforce_outline(mapped.image)
                padded = snap(outlined, (32, 32))
                asset = Asset(str(index), padded.image, (32, 32), (16, 28))
                report = check_asset(asset, COLORS, padded.image)
                self.assertTrue(report.passed, report.errors)
                self.assertEqual(report.reference_delta_e, 0)
                sheet = pack_sheet([asset], width=32)
                self.assertEqual(sheet.metadata['tiles'][str(index)], [0, 0, 32, 32])
                self.assertEqual(sheet.image.tobytes(), padded.image.tobytes())
                variant = padded.image.copy()
                ImageDraw.Draw(variant).rectangle((14, 14, 17, 17), fill=(*COLORS[2], 255))
                variant = enforce_outline(variant)
                extracted = extract_layer(padded.image, variant)
                layer_asset = Asset(f'layer_{index}', extracted.image, (32, 32),
                                    (16, 28), 'layer', outline_context=padded.image)
                self.assertTrue(check_asset(layer_asset, COLORS).passed)
                self.assertEqual(pack_sheet([layer_asset], width=32).image.tobytes(),
                                 extracted.image.tobytes())

    def test_ingest_near_magenta_crop_and_shared_transform(self) -> None:
        circle, _, body, hat = fixtures()
        circle.putpixel((0, 0), (240, 20, 240, 255))
        self.assertEqual(ingest(circle).bbox, (4, 4, 28, 28))
        crop = ingest(hat).bbox
        shared_body = ingest(body, crop).image
        shared_hat = ingest(hat, crop).image
        self.assertEqual(shared_body.size, shared_hat.size)
        padded_body, padded_hat = snap(shared_body, (32, 32)), snap(shared_hat, (32, 32))
        self.assertEqual(padded_body.offset, padded_hat.offset)
        shared_x, shared_y = 14 - crop[0], 20 - crop[1]
        destination = (shared_x + padded_body.offset[0], shared_y + padded_body.offset[1])
        self.assertEqual(padded_body.image.getpixel(destination), padded_hat.image.getpixel(destination))
        with self.assertRaises(PipelineError):
            _ = ingest(hat, ingest(body).bbox)
        with self.assertRaises(PipelineError):
            _ = ingest(Image.new('RGBA', (4, 4), (255, 0, 255, 255)))

    def test_majority_exact_tie_and_remainder(self) -> None:
        image = Image.new('RGBA', (3, 1))
        image.putdata([(9, 8, 7, 255), (1, 2, 3, 255), (9, 8, 7, 255)])
        self.assertEqual(pixelize(image, (1, 1)).getpixel((0, 0)), (9, 8, 7, 255))
        self.assertEqual(pixelize(image.crop((0, 0, 2, 1)), (1, 1)).getpixel((0, 0)), (1, 2, 3, 255))
        self.assertEqual(pixelize(image, (2, 1)).getpixel((1, 0)), (1, 2, 3, 255))
        with self.assertRaises(PipelineError):
            _ = pixelize(image, (4, 1))

    def test_palette_partial_alpha_and_large_distance(self) -> None:
        image = Image.new('RGBA', (2, 2), (255, 0, 0, 80))
        mapped = map_palette(image, COLORS)
        self.assertEqual(mapped.moved_over_8, 4)
        self.assertGreater(mapped.max_delta_e, 8)
        self.assertEqual(rgba_pixel(mapped.image, (0, 0))[3], 80)
        self.assertIn(rgba_pixel(mapped.image, (0, 0))[:3], COLORS)

    def test_outline_idempotent_without_growth(self) -> None:
        for fixture in fixtures():
            image = ingest(fixture).image
            result = enforce_outline(image)
            self.assertEqual(result.getchannel('A').tobytes(), image.getchannel('A').tobytes())
            self.assertEqual(result.tobytes(), enforce_outline(result).tobytes())
        square = enforce_outline(Image.new('RGBA', (5, 5), (154, 154, 154, 255)))
        self.assertEqual(rgba_pixel(square, (1, 1))[:3], (154, 154, 154))

    def test_snap_rejects_oversize(self) -> None:
        with self.assertRaises(PipelineError):
            _ = snap(Image.new('RGBA', (33, 32)), (32, 32))
        self.assertEqual(snap(Image.new('RGBA', (5, 7)), (32, 32)).offset, (13, 12))

    def test_layer_close_uses_real_variant_colors(self) -> None:
        _, _, body, hat = fixtures()
        base = ingest(body, (0, 0, 32, 32)).image
        variant = ingest(hat, (0, 0, 32, 32)).image
        variant.putpixel((14, 7), rgba_pixel(base, (14, 7)))
        layer = extract_layer(base, variant)
        self.assertGreater(layer.closed_pixels, 0)
        self.assertEqual(layer.image.getpixel((14, 7)), variant.getpixel((14, 7)))
        mask = np.array(layer.image)[:, :, 3] > 0
        np.testing.assert_array_equal(np.array(layer.image)[mask], np.array(variant)[mask])
        with self.assertRaises(PipelineError):
            _ = extract_layer(base, variant.crop((0, 0, 31, 32)))
        variant.putpixel((14, 20), (0, 0, 0, 0))
        with self.assertRaises(PipelineError):
            _ = extract_layer(base, variant)
        self.assertEqual(extract_layer(base, base).difference_pixels, 0)
        outlined_base = enforce_outline(base)
        outlined_hat = enforce_outline(ingest(hat, (0, 0, 32, 32)).image)
        extracted_hat = extract_layer(outlined_base, outlined_hat)
        hat_asset = Asset('hat', extracted_hat.image, (32, 32), (16, 28), 'layer',
                          outline_context=outlined_base)
        self.assertTrue(check_asset(hat_asset, COLORS).passed)
        composite = Image.alpha_composite(outlined_base, extracted_hat.image)
        self.assertEqual(composite.tobytes(), outlined_hat.tobytes())

    def test_checks_noise_detached_parts_and_alpha(self) -> None:
        image = Image.new('RGBA', (16, 16))
        draw = ImageDraw.Draw(image)
        draw.rectangle((2, 2, 5, 5), fill=(*OUTLINE, 255))
        draw.rectangle((10, 10, 11, 13), fill=(*OUTLINE, 255))
        asset = Asset('layer', image, (16, 16), (8, 14), 'layer')
        self.assertTrue(check_asset(asset, COLORS).passed)
        image.putpixel((15, 0), (*OUTLINE, 255))
        self.assertFalse(check_asset(asset, COLORS).passed)
        image.putpixel((15, 0), (*OUTLINE, 20))
        self.assertTrue(any('alpha' in e for e in check_asset(asset, COLORS).errors))

    def test_layer_outline_uses_body_air_boundary(self) -> None:
        base = Image.new('RGBA', (8, 8), (*COLORS[1], 255))
        layer = Image.new('RGBA', (8, 8))
        ImageDraw.Draw(layer).rectangle((2, 2, 5, 5), fill=(*COLORS[2], 255))
        asset = Asset('top', layer, (8, 8), (4, 7), 'layer', outline_context=base)
        self.assertTrue(check_asset(asset, COLORS).passed)
        self.assertFalse(check_asset(Asset('top', layer, (8, 8), (4, 7), 'layer'), COLORS).passed)

    def test_ground_ghost_shadow_checks(self) -> None:
        cases: tuple[tuple[AssetKind, int, RGB], ...] = (('ground', 255, COLORS[1]), ('ghost', 80, COLORS[3]), ('shadow', 90, COLORS[0]))
        for kind, alpha, color in cases:
            image = Image.new('RGBA', (4, 4), (*color, alpha))
            asset = Asset(kind, image, (4, 4), (2, 2), kind)
            self.assertTrue(check_asset(asset, COLORS).passed)
            image.putpixel((0, 0), (0, 255, 0, alpha))
            self.assertFalse(check_asset(asset, COLORS).passed)

    def test_atlas_duplicate_and_box_failures(self) -> None:
        asset = Asset('one', Image.new('RGBA', (4, 4)), (4, 4), (2, 4))
        with self.assertRaises(PipelineError):
            _ = pack_sheet([asset, asset])
        with self.assertRaises(PipelineError):
            _ = pack_sheet([asset], width=3)
        packed = pack_sheet([asset, Asset('two', asset.image, asset.box, asset.anchor)], width=4)
        self.assertEqual(packed.metadata['tiles']['two'], [0, 4, 4, 4])
        self.assertEqual(packed.metadata['anchors']['one'], [2, 4])

    def test_check_writer_scope_and_missing_references(self) -> None:
        image = Image.new('RGBA', (4, 4), (*OUTLINE, 255))
        report = check_asset(Asset('sample', image, (4, 4), (2, 4)), COLORS)
        with TemporaryDirectory() as directory:
            path = Path(directory) / 'CHECKS.md'
            with self.assertRaises(PipelineError):
                write_checks([], path)
            write_checks([CheckRecord(report, 'test', 'synthetic fixture only')], path)
            text = path.read_text()
            self.assertIn('1 submitted registry assets only', text)
            self.assertIn('NOT_MEASURED', text)
            self.assertIn('Dashed geometry requires visual verification', text)
            measured = [CheckRecord(replace(report, name='low', reference_delta_e=1.0), 'test', 'fixture'),
                        CheckRecord(replace(report, name='high', reference_delta_e=20.0), 'test', 'fixture')]
            write_checks(measured, path)
            flagged = next(line for line in path.read_text().splitlines() if '| high |' in line)
            self.assertIn('ABOVE_P90', flagged)
            self.assertIn('PASS', flagged)

    def test_sheet_preserves_legacy_palette_variants(self) -> None:
        palettes = {'skin': 'd8a884', 'elder_hair': 'e8e8e8',
                    'hairs': ['2a1e16', '5a3a22'], 'tunics': ['efe2c0', '6fb0e8']}
        asset = Asset('one', Image.new('RGBA', (4, 4)), (4, 4), (2, 4))
        packed = pack_sheet([asset], palettes=palettes)
        self.assertEqual(packed.metadata['palettes'], palettes)

    def test_palette_canonicalizes_fully_transparent_rgb(self) -> None:
        mapped = map_palette(Image.new('RGBA', (2, 2), (99, 88, 77, 0)), COLORS)
        self.assertEqual(mapped.image.tobytes(), bytes(16))
        self.assertEqual(mapped.moved_over_8, 0)

    def test_final_checks_require_measured_reference(self) -> None:
        image = Image.new('RGBA', (4, 4), (*OUTLINE, 255))
        asset = Asset('sample', image, (4, 4), (2, 4))
        missing = CheckRecord(check_asset(asset, COLORS), 'test', 'fixture')
        measured = CheckRecord(check_asset(asset, COLORS, image), 'test', 'fixture')
        with TemporaryDirectory() as directory:
            path = Path(directory) / 'CHECKS.md'
            with self.assertRaises(PipelineError):
                write_checks([missing], path, final=True)
            self.assertFalse(path.exists())
            write_checks([missing], path)
            self.assertIn('PROVISIONAL', path.read_text())
            write_checks([measured], path, final=True)
            self.assertIn('FINAL', path.read_text())
            self.assertNotIn('NOT_MEASURED', path.read_text())

    def test_reference_is_distribution_not_shape(self) -> None:
        first = Image.new('RGBA', (4, 4), (*COLORS[1], 255))
        second = Image.new('RGBA', (8, 2), (*COLORS[1], 255))
        self.assertEqual(reference_distance(first, second), 0)
        self.assertIsNone(reference_distance(first, Image.new('RGBA', (2, 2))))

if __name__ == '__main__':
    _ = unittest.main()
