"""Reader bundle tests use synthetic PNG bytes, never visual acceptance evidence."""
import importlib.util
import tempfile
import unittest
from pathlib import Path


class LifeBundleTests(unittest.TestCase):
    def test_reader_bundle_contains_only_anonymous_images_and_instructions(self):
        path = Path(__file__).with_name('run_life_captures_p13.py')
        self.assertTrue(path.exists(), 'P12 life capture runner is not implemented')
        spec = importlib.util.spec_from_file_location('life_capture_runner', path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            source = base / 'private_living_parent_en.png'
            source.write_bytes(b'native-image-fixture')
            target = base / 'bundle'
            mapping = module.prepare_reader_bundle([source], target)
            self.assertEqual((target / 'image_0001.png').read_bytes(), source.read_bytes())
            self.assertEqual(sorted(p.name for p in target.iterdir()), ['image_0001.png', 'instructions.json'])
            self.assertNotIn('parent', (target / 'instructions.json').read_text())
            self.assertEqual(mapping[0]['source'], str(source))
            with self.assertRaises(FileExistsError):
                module.prepare_reader_bundle([source], target)

    def test_quiet_wait_is_inside_window_lock_and_bounded(self):
        import run_life_captures_p13 as runner
        self.assertTrue(hasattr(runner, 'life_launch_command'), 'life quiet-lock launch wrapper missing')
        command = runner.life_launch_command(Path('/repo'), Path('/godot'))
        self.assertEqual(command[:5], ['bash', 'tools/art_pipeline/with_window_lock.sh', '1800', 'bash', '-c'])
        self.assertIn('sleep 30', command[5])
        self.assertIn('exec "$@"', command[5])
        self.assertEqual(command[7], '/Users/rexxa/work/.mac_quiet.lock')
        self.assertEqual(command[8], '/godot')


if __name__ == '__main__':
    unittest.main()
