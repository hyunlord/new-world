#!/usr/bin/env python3
# How to run: python3 -m unittest discover -s tools/v8 -p test_mux2_gate_fixtures.py
"""Original capture-manifest and legacy flat-map coverage for immutable format10 inputs."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from mux2_gate_fixtures import validate_fixtures


class OriginalFixtureManifestTests(unittest.TestCase):
    def _create(self, root: Path) -> tuple[Path, list[dict[str, str]]]:
        scenes: list[dict[str, str]] = []
        for index in range(4):
            path = root / f'scene{index}.wsave'
            raw = b'WORLDSIM' + (10).to_bytes(4, 'little') + bytes([index])
            path.write_bytes(raw)
            scenes.append({'id': f'scene{index}', 'save': str(path),
                           'sha256': hashlib.sha256(raw).hexdigest()})
        return root / 'manifest.json', scenes

    def test_original_scenes_and_flat_map_both_accept_exact_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            manifest, scenes = self._create(root)
            manifest.write_text(json.dumps({'schema': 1, 'scenes': scenes,
                                            'bridge_sha256': 'a' * 64, 'seed': 0}))
            self.assertIsNone(validate_fixtures(root, manifest))
            manifest.write_text(json.dumps({Path(row['save']).name: row['sha256']
                                            for row in scenes}))
            self.assertIsNone(validate_fixtures(root, manifest))

    def test_rejects_corruption_duplicates_missing_extra_and_escape(self) -> None:
        cases = ('wrong_hash', 'duplicate_path', 'duplicate_id', 'missing_scene',
                 'extra_scene', 'missing_file', 'extra_file', 'escape', 'relative',
                 'traversal', 'wrong_format', 'symlink', 'missing_hash', 'bad_schema',
                 'bad_scenes', 'directory_as_save')
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                fixtures = root / 'fixtures'
                fixtures.mkdir()
                manifest, scenes = self._create(fixtures)
                document = {'schema': 1, 'scenes': scenes}
                first = Path(scenes[0]['save'])
                match case:
                    case 'wrong_hash':
                        scenes[0]['sha256'] = '0' * 64
                    case 'duplicate_path':
                        scenes[1]['save'] = scenes[0]['save']
                    case 'duplicate_id':
                        scenes[1]['id'] = scenes[0]['id']
                    case 'missing_scene':
                        scenes.pop()
                    case 'extra_scene':
                        scenes.append(dict(scenes[0]))
                    case 'missing_file':
                        first.unlink()
                    case 'extra_file':
                        (fixtures / 'extra.save').write_bytes(first.read_bytes())
                    case 'escape':
                        outside = root / first.name
                        outside.write_bytes(first.read_bytes())
                        scenes[0]['save'] = str(outside)
                    case 'relative':
                        scenes[0]['save'] = first.name
                    case 'traversal':
                        scenes[0]['save'] = str(fixtures / '..' / 'fixtures' / first.name)
                    case 'wrong_format':
                        first.write_bytes(b'WORLDSIM' + (11).to_bytes(4, 'little'))
                        scenes[0]['sha256'] = hashlib.sha256(first.read_bytes()).hexdigest()
                    case 'symlink':
                        outside = root / first.name
                        first.rename(outside)
                        first.symlink_to(outside)
                    case 'missing_hash':
                        del scenes[0]['sha256']
                    case 'bad_schema':
                        document['schema'] = 2
                    case 'bad_scenes':
                        document['scenes'] = {}
                    case 'directory_as_save':
                        first.unlink()
                        first.mkdir()
                manifest.write_text(json.dumps(document))
                with self.assertRaises(ValueError):
                    validate_fixtures(fixtures, manifest)

    def test_flat_map_still_rejects_wrong_hash_inventory_and_symlink(self) -> None:
        for case in ('wrong_hash', 'missing_key', 'extra_key', 'symlink'):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                manifest, scenes = self._create(root)
                hashes = {Path(row['save']).name: row['sha256'] for row in scenes}
                match case:
                    case 'wrong_hash':
                        hashes['scene0.wsave'] = '0' * 64
                    case 'missing_key':
                        del hashes['scene0.wsave']
                    case 'extra_key':
                        hashes['extra.wsave'] = 'a' * 64
                    case 'symlink':
                        first = root / 'scene0.wsave'
                        moved = root / 'outside.bin'
                        first.rename(moved)
                        first.symlink_to(moved)
                manifest.write_text(json.dumps(hashes))
                with self.assertRaises(ValueError):
                    validate_fixtures(root, manifest)


if __name__ == '__main__':
    unittest.main()
