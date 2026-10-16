"""Pass-five approval replaces a stale same-pixel pass-four mask only by layer."""
import hashlib
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from PIL import Image
from hair_registry_p4 import HairFrame, HairRegistry, ResolvedHair


class RegistryPrecedenceTests(unittest.TestCase):
    def test_p5_same_key_overrides_p4(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            directory = root / 'tools/art_pipeline'
            directory.mkdir(parents=True)
            sprite = Image.new('RGBA', (3, 5), (80, 40, 20, 255))
            for version in (4, 5):
                mask = Image.new('RGBA', sprite.size, (0, 0, 0, 255))
                mask.putpixel((version-4, 0), (3, 1, 0, 255))
                path = root / f'mask{version}.png'
                mask.save(path)
                row = {'body': 'adult_b', 'activity': 'idle', 'rgba_sha256': hashlib.sha256(sprite.tobytes()).hexdigest(),
                       'mask_path': path.name, 'mask_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                       'size': list(sprite.size), 'source': 'template_v5', 'verdict': 'ok'}
                (directory / f'reviewed_hair_maps_p{version}.json').write_text(json.dumps({'frames': {'a': row}}))
            resolved = HairRegistry.load(root).resolve(HairFrame('adult_b', 'idle', sprite))
            self.assertEqual(resolved.image.getpixel((1, 0))[0], 3)
            self.assertEqual(resolved.image.getpixel((0, 0))[0], 0)
            before = HairRegistry.load(root, directory / 'reviewed_hair_maps_p4.json')
            self.assertEqual(before.resolve(HairFrame('adult_b', 'idle', sprite)).image.getpixel((0, 0))[0], 3)

    def test_bend_frame_zero_is_width_exempt(self):
        from hair_registry_p4 import ResolvedHair
        sprite = Image.new('RGBA', (8, 5), (80, 40, 20, 255))
        mask = Image.new('RGBA', sprite.size, (3, 1, 0, 255))
        key = ('adult_b', 'gather_wood', hashlib.sha256(sprite.tobytes()).hexdigest())
        registry = HairRegistry({key: ResolvedHair(mask, 'p4_stored', verdict='ok')}, {})
        self.assertEqual(registry.resolve(HairFrame('adult_b', 'gather_wood', sprite, 0)).image.getpixel((0, 0))[0], 3)
        self.assertEqual(registry.resolve(HairFrame('adult_b', 'gather_wood', sprite, 1)).image.getpixel((0, 0))[0], 0)

    def test_key_mask_is_width_exempt_but_sleep_is_not(self):
        sprite = Image.new('RGBA', (8, 5), (80, 40, 20, 255))
        mask = Image.new('RGBA', sprite.size, (3, 1, 0, 255))
        sha = hashlib.sha256(sprite.tobytes()).hexdigest()
        frozen = ResolvedHair(mask, 'hair_key', verdict='ok')
        registry = HairRegistry({('adult_b', 'idle', sha): frozen,
                                 ('adult_b', 'asleep', sha): frozen}, {})
        self.assertEqual(registry.resolve(HairFrame('adult_b', 'idle', sprite)).image.getpixel((0, 0))[0], 3)
        self.assertEqual(registry.resolve(HairFrame('adult_b', 'asleep', sprite)).image.getpixel((0, 0))[0], 0)


if __name__ == '__main__':
    unittest.main()
