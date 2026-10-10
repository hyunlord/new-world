"""Hair-only frozen atlas maps preserve reviewed pixels, provenance and UVs."""
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

import numpy as np
from PIL import Image

import regions


class ReviewedMapTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.sprite = Image.new('RGBA', (2, 4), (100, 80, 60, 255))
        self.reviewed = Image.new('RGBA', (2, 4), (0, 0, 0, 255))
        self.reviewed.putpixel((0, 0), (3, 2, 0, 255))
        self.reviewed.save(self.root / 'reviewed.png')
        self.record = {'body': 'adult_a', 'rgba_sha256': hashlib.sha256(self.sprite.tobytes()).hexdigest(),
            'size': [2, 4], 'mask_path': 'reviewed.png',
            'mask_sha256': hashlib.sha256((self.root / 'reviewed.png').read_bytes()).hexdigest(),
            'source': 'installed_map', 'activity': 'carry', 'verdict': 'pending'}
        self.registry = self.root / 'maps.json'
        palette = self.root / 'tools/art_pipeline/hair_palette_p4.json'
        palette.parent.mkdir(parents=True)
        palette.write_text(json.dumps({'bodies': {'adult_a': {
            'seen_rgb': [[100, 80, 60]], 'hair_rgb': [[100, 80, 60]]}}}))
        self.manifest = self.root / 'world_atlas.json'
        self.manifest.write_text(json.dumps({'pages': ['res://world.png'], 'sprites': {
            'V_a2_01': {'page': 0, 'rect': [3, 4, 2, 4], 'body': 'adult_a', 'view': 'N', 'activity': 'carry'}}}))

    def run_atlas(self):
        page = Image.new('RGBA', (16, 16))
        page.paste(self.sprite, (3, 4))
        page.save(self.root / 'world.png')
        self.registry.write_text(json.dumps({'version': 1, 'frames': {'V_a2_01': self.record}}))
        with patch.object(regions, 'ROOT', self.root):
            return regions.build_atlases(self.manifest, reviewed_maps_path=self.registry)

    def test_exact_reviewed_pixels_preserve_uv_and_evidence(self) -> None:
        result = self.run_atlas()
        row = result['sprites']['V_a2_01']
        self.assertEqual(row['rect'], [3, 4, 2, 4])
        self.assertEqual(row['classification_source'], 'installed_map')
        self.assertEqual(row['reviewed_mask_sha256'], self.record['mask_sha256'])
        with Image.open(self.root / 'world_regions.png') as image:
            self.assertEqual(image.crop((3, 4, 5, 8)).tobytes(), self.reviewed.tobytes())
            self.assertEqual(image.getpixel((2, 4)), (0, 0, 0, 0))

    def test_same_id_changed_pixels_use_algorithm(self) -> None:
        self.sprite.putpixel((0, 0), (101, 80, 60, 255))
        result = self.run_atlas()
        self.assertEqual(result['sprites']['V_a2_01']['classification_source'], 'hair_maps')
        with Image.open(self.root / 'world_regions.png') as image:
            self.assertEqual(set(np.unique(np.asarray(image)[..., 0])), {0, 3})

    def test_rejected_mask_cannot_be_reused(self) -> None:
        self.record['verdict'] = 'reject'
        with self.assertRaisesRegex(regions.ReviewedMapError, 'rejected'):
            self.run_atlas()

    def test_corrupt_frozen_file_hash_is_rejected(self) -> None:
        self.record['mask_sha256'] = '0' * 64
        with self.assertRaises(regions.ReviewedMapError):
            self.run_atlas()

    def test_conflicting_duplicate_maps_are_rejected(self) -> None:
        self.run_atlas()
        alternative = self.reviewed.copy()
        alternative.putpixel((0, 0), (3, 1, 0, 255))
        alternative.save(self.root / 'alternative.png')
        duplicate = dict(self.record, mask_path='alternative.png',
            mask_sha256=hashlib.sha256((self.root / 'alternative.png').read_bytes()).hexdigest())
        self.registry.write_text(json.dumps({'version': 1, 'frames': {'V_a2_01': self.record, 'other': duplicate}}))
        with patch.object(regions, 'ROOT', self.root), self.assertRaises(regions.ReviewedMapError):
            regions.build_atlases(self.manifest, reviewed_maps_path=self.registry)

    def test_reviewed_alpha_mismatch_is_rejected(self) -> None:
        self.reviewed.putpixel((0, 0), (0, 0, 0, 0))
        self.save_reviewed()
        with self.assertRaises(regions.ReviewedMapError):
            self.run_atlas()

    def test_full_body_region_is_rejected(self) -> None:
        self.reviewed.putpixel((1, 2), (4, 1, 0, 255))
        self.save_reviewed()
        with self.assertRaises(regions.ReviewedMapError):
            self.run_atlas()

    def test_frozen_size_mismatch_is_rejected(self) -> None:
        self.record['size'] = [2, 5]
        with self.assertRaises(regions.ReviewedMapError):
            self.run_atlas()

    def test_adult_without_hair_is_rejected(self) -> None:
        self.reviewed.putpixel((0, 0), (0, 0, 0, 255))
        self.save_reviewed()
        with self.assertRaises(regions.ReviewedMapError):
            self.run_atlas()

    def save_reviewed(self) -> None:
        self.reviewed.save(self.root / 'reviewed.png')
        self.record['mask_sha256'] = hashlib.sha256((self.root / 'reviewed.png').read_bytes()).hexdigest()


if __name__ == '__main__':
    unittest.main()
