"""Frozen anatomical tables survive packing without UV or alpha drift."""
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from PIL import Image

import regions
from regions_v2 import build_table, table_to_data


class RegionV2AtlasTests(unittest.TestCase):
    def test_frozen_table_classifies_packed_pose_without_height_bands(self) -> None:
        # Given a reviewed hair colour moved to a lower row in a packed pose.
        color = (100, 80, 60, 255)
        reference = Image.new('RGBA', (1, 1), color)
        approved = Image.new('RGBA', (1, 1), (3, 0, 0, 255))
        data = {'tables': {'adult_a': table_to_data(build_table(reference, approved, 'adult_a'))}}
        with TemporaryDirectory() as directory:
            root = Path(directory)
            page = Image.new('RGBA', (16, 16))
            page.putpixel((4, 8), color)
            page.save(root / 'world.png')
            manifest = root / 'world_atlas.json'
            manifest.write_text(json.dumps({'pages': ['res://world.png'], 'sprites': {
                'V_a2_01': {'page': 0, 'rect': [3, 4, 3, 6], 'body': 'adult_a', 'view': 'S', 'activity': 'sit'}}}))
            table_file = root / 'tables.json'
            table_file.write_text(json.dumps(data))
            # When encoding with explicitly frozen production tables.
            with patch.object(regions, 'ROOT', root):
                output = regions.build_atlases(manifest, tables_path=table_file)
            # Then UVs and alpha remain exact while colour carries anatomical meaning.
            self.assertEqual(output['sprites']['V_a2_01']['rect'], [3, 4, 3, 6])
            with Image.open(root / 'world_regions.png') as encoded:
                self.assertEqual(encoded.getpixel((4, 8)), (3, 2, 0, 255))
                self.assertEqual(encoded.getpixel((3, 4)), (0, 0, 0, 0))


class ReviewedMapTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.sprite = Image.new('RGBA', (2, 2), (100, 80, 60, 255))
        approved = Image.new('RGBA', (2, 2), (3, 2, 0, 255))
        self.reviewed = Image.new('RGBA', (2, 2), (2, 1, 0, 255))
        self.reviewed.save(self.root / 'reviewed.png')
        self.table = self.root / 'tables.json'
        self.table.write_text(json.dumps({'tables': {'adult_a': table_to_data(
            build_table(self.sprite, approved, 'adult_a'))}}))
        self.record = {'body': 'adult_a', 'rgba_sha256': hashlib.sha256(self.sprite.tobytes()).hexdigest(),
            'size': [2, 2], 'regions_path': 'reviewed.png',
            'regions_sha256': hashlib.sha256((self.root / 'reviewed.png').read_bytes()).hexdigest(),
            'source': 'approved-reference'}
        self.registry = self.root / 'maps.json'
        self.manifest = self.root / 'world_atlas.json'
        self.manifest.write_text(json.dumps({'pages': ['res://world.png'], 'sprites': {
            'V_a2_01': {'page': 0, 'rect': [3, 4, 2, 2], 'body': 'adult_a', 'view': 'N', 'activity': 'carry'}}}))

    def run_atlas(self):
        page = Image.new('RGBA', (16, 16))
        page.paste(self.sprite, (3, 4))
        page.save(self.root / 'world.png')
        self.registry.write_text(json.dumps({'maps': [self.record]}))
        with patch.object(regions, 'ROOT', self.root):
            return regions.build_atlases(self.manifest, tables_path=self.table,
                                        reviewed_maps_path=self.registry)

    def test_exact_reviewed_pixels_override_v2_preserving_uv_and_evidence(self) -> None:
        result = self.run_atlas()
        row = result['sprites']['V_a2_01']
        self.assertEqual(row['rect'], [3, 4, 2, 2])
        self.assertEqual(row['classification_source'], 'reviewed_map')
        self.assertIn('v2_rgba_sha256', row)
        with Image.open(self.root / 'world_regions.png') as image:
            self.assertEqual(image.crop((3, 4, 5, 6)).tobytes(), self.reviewed.tobytes())

    def test_changed_pixel_uses_v2(self) -> None:
        self.sprite.putpixel((0, 0), (101, 80, 60, 255))
        result = self.run_atlas()
        self.assertEqual(result['sprites']['V_a2_01']['classification_source'], 'regions_v2')

    def test_corrupt_frozen_file_hash_is_rejected(self) -> None:
        self.record['regions_sha256'] = '0' * 64
        with self.assertRaises(regions.ReviewedMapError):
            self.run_atlas()

    def test_conflicting_duplicate_maps_are_rejected(self) -> None:
        self.run_atlas()
        alternative = Image.new('RGBA', (2, 2), (4, 1, 0, 255))
        alternative.save(self.root / 'alternative.png')
        duplicate = dict(self.record, regions_path='alternative.png',
            regions_sha256=hashlib.sha256((self.root / 'alternative.png').read_bytes()).hexdigest())
        self.registry.write_text(json.dumps({'maps': [self.record, duplicate]}))
        with patch.object(regions, 'ROOT', self.root), self.assertRaises(regions.ReviewedMapError):
            regions.build_atlases(self.manifest, tables_path=self.table,
                                 reviewed_maps_path=self.registry)

    def test_reviewed_alpha_mismatch_is_rejected(self) -> None:
        self.reviewed.putpixel((0, 0), (0, 0, 0, 0))
        self.reviewed.save(self.root / 'reviewed.png')
        self.record['regions_sha256'] = hashlib.sha256((self.root / 'reviewed.png').read_bytes()).hexdigest()
        with self.assertRaises(regions.ReviewedMapError):
            self.run_atlas()


if __name__ == '__main__':
    unittest.main()
