"""Temporary fixture registries never enter the real asset set."""
import json
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from PIL import Image

from .models import PipelineError, decode_json, json_array, json_object
from .registry import run_registry


class RegistryTests(unittest.TestCase):
    def make_registry(self, directory: Path) -> Path:
        image = directory / 'source.png'
        Image.new('RGBA', (4, 4), (20, 18, 16, 255)).save(image)
        palette = directory / 'palette.json'
        _ = palette.write_text(json.dumps({
            'colors': [{'name': 'outline', 'hex': '#141210', 'source': 'fixture'}],
            'people': {'skins': ['#D8A884'], 'hairs': ['#2A1E16'],
                       'tunics': ['#EFE2C0'], 'elder_hair': '#E8E8E8'},
            'channels': {'skin': '#9A9A9A'},
        }))
        entries = [{'name': name, 'image': 'source.png', 'box': [4, 4],
                    'anchor': [2, 4], 'kind': 'object', 'family': 'fixture',
                    'reference': 'source.png'} for name in ('first', 'second')]
        manifest = directory / 'registry.json'
        _ = manifest.write_text(json.dumps({'palette': 'palette.json', 'assets': entries}))
        return manifest

    def test_success_preserves_metadata_and_final_checks(self) -> None:
        with TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = self.make_registry(root)
            output = root / 'output'
            result = run_registry(manifest, output)
            self.assertTrue(result.passed)
            self.assertEqual(result.asset_count, 2)
            metadata = json_object(decode_json((output / 'sheet.json').read_text()))
            self.assertEqual(json_object(metadata['palettes'])['skin'], 'd8a884')
            self.assertEqual(json_object(metadata['palettes'])['hairs'], ['2a1e16'])
            self.assertEqual(metadata['channels'], {'skin': '9a9a9a'})
            self.assertEqual(metadata['skins'], ['d8a884'])
            self.assertIn('FINAL', (output / 'CHECKS.md').read_text())
            self.assertTrue((output / 'registry_stats.json').exists())
            cli = subprocess.run([sys.executable, '-m', 'art.pipeline.registry',
                                  str(manifest), str(output)], capture_output=True, text=True, check=False)
            self.assertEqual(cli.returncode, 0, cli.stderr)

    def test_failed_check_retains_report_withholds_atlas(self) -> None:
        with TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = self.make_registry(root)
            Image.new('RGBA', (4, 4), (255, 0, 0, 255)).save(root / 'source.png')
            output = root / 'output'
            result = run_registry(manifest, output)
            self.assertFalse(result.passed)
            self.assertIn('FAIL', (output / 'CHECKS.md').read_text())
            self.assertFalse((output / 'sheet.png').exists())
            cli = subprocess.run([sys.executable, '-m', 'art.pipeline.registry',
                                  str(manifest), str(output)], capture_output=True, text=True, check=False)
            self.assertEqual(cli.returncode, 1, cli.stderr)

    def test_missing_reference_duplicate_and_empty_rejected(self) -> None:
        with TemporaryDirectory() as temp:
            root = Path(temp)
            for failure in ('missing', 'duplicate', 'empty'):
                manifest = self.make_registry(root)
                data = json_object(decode_json(manifest.read_text()))
                assets = json_array(data['assets'])
                if failure == 'missing':
                    del json_object(assets[0])['reference']
                elif failure == 'duplicate':
                    json_object(assets[1])['name'] = json_object(assets[0])['name']
                else:
                    data['assets'] = []
                _ = manifest.write_text(json.dumps(data))
                with self.subTest(failure=failure), self.assertRaises(PipelineError):
                    _ = run_registry(manifest, root / 'output')

if __name__ == '__main__':
    _ = unittest.main()
