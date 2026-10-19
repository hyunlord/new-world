"""Installation checks hair masks before writing any production asset."""
from dataclasses import replace
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from PIL import Image

import install_v4
from test_install_v4 import reviewed_fixture


class HairInstallTests(unittest.TestCase):
    def test_adult_missing_mask_stops_before_writes(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            plan = install_v4.build_plan(reviewed_fixture(root))
            image = Image.new('RGBA', (2, 5), (100, 80, 60, 255))
            actor = replace(plan.sprites[0], body='adult_a', activity='idle', sprite=image)
            plan = replace(plan, sprites=(actor,))
            with self.assertRaisesRegex(install_v4.InstallError, 'hair mask'):
                install_v4.apply_plan(plan, plan.selection_sha256)
            self.assertFalse((root / 'assets/art/production').exists())

    def test_new_adult_mask_can_be_computed_from_frozen_training(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            plan = install_v4.build_plan(reviewed_fixture(root))
            image = Image.new('RGBA', (2, 5), (100, 80, 60, 255))
            actor = replace(plan.sprites[0], body='adult_a', activity='idle', sprite=image)
            plan = replace(plan, sprites=(actor,))
            palette = root / 'tools/art_pipeline/hair_palette_p4.json'
            palette.parent.mkdir(parents=True)
            palette.write_text(json.dumps({'bodies': {'adult_a': {
                'seen_rgb': [[100, 80, 60]], 'hair_rgb': [[100, 80, 60]]}}}))
            install_v4.apply_plan(plan, plan.selection_sha256)
            self.assertTrue((root / 'assets/art/production/N3/01.png').exists())

    def test_child_and_asleep_require_no_mask(self) -> None:
        for body, activity in (('child', 'idle'), ('adult_a', 'asleep')):
            with self.subTest(body=body, activity=activity), TemporaryDirectory() as directory:
                root = Path(directory)
                plan = install_v4.build_plan(reviewed_fixture(root))
                actor = replace(plan.sprites[0], body=body, activity=activity)
                plan = replace(plan, sprites=(actor,))
                install_v4.apply_plan(plan, plan.selection_sha256)
                self.assertTrue((root / 'assets/art/production/N3/01.png').exists())


if __name__ == '__main__':
    unittest.main()
