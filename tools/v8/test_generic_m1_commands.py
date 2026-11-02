"""Original source/command contract checks, not native M1 coverage."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from generic_m1_commands import BODY, COMMANDS, authenticate_source


class M1CommandsTests(unittest.TestCase):
    def test_current_original_chain(self) -> None:
        self.assertEqual(authenticate_source(Path(__file__).with_name('gate.sh')), BODY)
        self.assertEqual(len(COMMANDS), 4)
        self.assertEqual(COMMANDS[0].targets,
                         ('colony_commands', 'colony_jobs', 'colony_world', 'engine_colony_ffi'))
        self.assertEqual(COMMANDS[0].argv[-2:], ('--', '--test-threads=8'))
        for command in COMMANDS[1:]:
            self.assertIsNone(command.threads)
            self.assertNotIn('--', command.argv)
            self.assertNotIn('--no-fail-fast', command.argv)
        self.assertEqual(COMMANDS[2].argv,
                         ('cargo', 'test', '--release', '-p', 'sim-systems', '--lib', 'jobs::'))

    def test_source_drift_cannot_claim_original_chain(self) -> None:
        original = Path(__file__).with_name('gate.sh').read_text()
        mutations = (
            original.replace('--test engine_colony_ffi ', '', 1),
            original.replace('--lib probe::colony', '--lib probe::colony -- --test-threads=8', 1),
            original.replace('&& cargo test --release -p sim-systems', '; cargo test --release -p sim-systems', 1),
            original.replace('grep -q "^TRF_TEST_PASS"', 'grep -q "^TRF_TEST_PASS$"', 1),
            original.replace('summarize_tests m1_tests "$?"', 'summarize_tests m1_tests 0', 1),
            original.replace("' \"$LOG\"\nsummarize_tests m1_tests", "' '$LOG'\nsummarize_tests m1_tests", 1),
            original + '\n' + next(line for line in original.splitlines() if line.startswith('step 17 ')),
        )
        with TemporaryDirectory() as folder:
            path = Path(folder) / 'gate.sh'
            for content in mutations:
                with self.subTest(content=content[-100:]):
                    self.assertNotEqual(content, original)
                    _ = path.write_text(content)
                    with self.assertRaises(ValueError):
                        _ = authenticate_source(path)
