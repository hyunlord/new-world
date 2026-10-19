"""Regression coverage for frozen hair provenance."""
import hashlib
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from PIL import Image
import hair_inventory_p4 as inventory


class InventoryTests(unittest.TestCase):
    def test_extraction_preserves_shade_and_source_alpha(self):
        sprite = Image.new('RGBA', (2, 3), (10, 20, 30, 255))
        regions = Image.new('RGBA', sprite.size, (2, 1, 0, 255))
        regions.putpixel((0, 0), (3, 2, 0, 255))
        result = inventory.installed_mask(sprite, regions, 'adult_a', 'idle')
        self.assertEqual(result.getpixel((0, 0)), (3, 2, 0, 255))
        self.assertEqual(result.getpixel((1, 0)), (0, 0, 0, 255))

    def test_children_and_lying_are_empty(self):
        for body, activity, size in [('child', 'idle', (2, 3)), ('adult_a', 'asleep', (2, 3)), ('adult_a', 'idle', (3, 2))]:
            sprite = Image.new('RGBA', size, (1, 2, 3, 255))
            result = inventory.installed_mask(sprite, Image.new('RGBA', size, (3, 1, 0, 255)), body, activity)
            self.assertEqual(result.getchannel('R').getextrema(), (0, 0))

    def test_partner_rejects_changed_pixels(self):
        sprite = Image.new('RGBA', (2, 3), (10, 20, 30, 255))
        regions = Image.new('RGBA', sprite.size, (3, 1, 0, 255))
        with self.assertRaises(inventory.InventoryError):
            inventory.validate_partner(sprite, regions, 'changed', [2, 3])
        inventory.validate_partner(sprite, regions, hashlib.sha256(sprite.tobytes()).hexdigest(), [2, 3])

    def test_partner_rejects_alpha_mismatch(self):
        sprite = Image.new('RGBA', (2, 3), (10, 20, 30, 255))
        regions = Image.new('RGBA', sprite.size, (3, 1, 0, 0))
        with self.assertRaises(inventory.InventoryError):
            inventory.validate_partner(sprite, regions, hashlib.sha256(sprite.tobytes()).hexdigest(), [2, 3])

    def test_frozen_registry_detects_changed_png_before_reuse(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'mask.png'
            Image.new('RGBA', (2, 3)).save(path)
            registry = {'version': 1, 'source_atlas_sha256': 'unused', 'frames': {'a': {
                'mask_path': 'mask.png', 'mask_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                'size': [2, 3]}}}
            Image.new('RGBA', (2, 3), (3, 1, 0, 255)).save(path)
            with self.assertRaises(inventory.InventoryError):
                inventory.validate_frozen(root, registry)


if __name__ == '__main__':
    unittest.main()
