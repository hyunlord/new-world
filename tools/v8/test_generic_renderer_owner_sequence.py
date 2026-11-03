"""Real run() control-flow tests with synthetic native/admission boundaries; no runtime proof."""
import fcntl
import os
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from generic_gate import document
from renderer_owner_test_fixture import OwnerFixture


class OwnerSequenceTests(unittest.TestCase):
    def retained_lane(self, fixture: OwnerFixture, descriptor: int) -> None:
        self.assertEqual(os.fstat(descriptor).st_ino, fixture.lock.stat().st_ino)
        with fixture.lock.open('rb') as competing, self.assertRaises(BlockingIOError):
            fcntl.flock(competing.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_bridge_failure_stops_before_import(self) -> None:
        with TemporaryDirectory() as directory:
            f = OwnerFixture(Path(directory).resolve())
            f.failure = 'bridge'
            with f.lock.open('rb') as lane, f.owner(lane.fileno()) as runner:
                with self.assertRaisesRegex(RuntimeError, 'bridge failure'):
                    _ = runner.run()
                self.assertEqual(f.events, ['bridge'])
                self.assertIsNone(runner.bridge)
                self.assertIsNone(runner.imports)
                self.assertFalse((f.output/'renderer.json').exists())
                self.retained_lane(f, lane.fileno())

    def test_import_failure_keeps_actual_returned_owner(self) -> None:
        for failure, events in (('start', ['bridge', 'import-start']),
                                ('finish', ['bridge', 'import-start', 'import-finish'])):
            with self.subTest(failure=failure), TemporaryDirectory() as directory:
                f = OwnerFixture(Path(directory).resolve())
                f.failure = failure
                with f.lock.open('rb') as lane, f.owner(lane.fileno()) as runner:
                    with self.assertRaises(RuntimeError):
                        _ = runner.run()
                    self.assertEqual(f.events, events)
                    self.assertIs(runner.bridge, f.bridge)
                    self.assertIs(runner.imports, f.import_owner)
                    self.assertIsNone(runner.imported)
                    self.assertEqual(runner.processes, ())
                    self.assertFalse((f.output/'renderer.json').exists())
                    self.retained_lane(f, lane.fileno())

    def test_later_producer_failure_retains_all_returned_processes(self) -> None:
        for producer, count in (('ascii', 2), ('godot', 3)):
            with self.subTest(producer=producer), TemporaryDirectory() as directory:
                f = OwnerFixture(Path(directory).resolve())
                f.failure = producer
                with f.lock.open('rb') as lane, f.owner(lane.fileno()) as runner:
                    with self.assertRaises(ValueError):
                        _ = runner.run()
                    self.assertIs(runner.imports, f.import_owner)
                    self.assertIs(runner.imported, f.import_result)
                    self.assertEqual(len(runner.processes), count)
                    self.assertEqual(len(runner.process_hashes), count)
                    self.assertEqual(runner.processes[0][1].status, 'PASS')
                    self.assertEqual(runner.processes[-1][1].native_exit, 7)
                    self.assertFalse((f.output/'renderer.json').exists())
                    self.retained_lane(f, lane.fileno())

    def test_final_checkpoint_failure_prevents_terminal(self) -> None:
        with TemporaryDirectory() as directory:
            f = OwnerFixture(Path(directory).resolve())
            f.failure = 'final-checkpoint'
            with f.lock.open('rb') as lane, f.owner(lane.fileno()) as runner:
                with self.assertRaisesRegex(RuntimeError, 'final checkpoint'):
                    _ = runner.run()
                self.assertEqual(len(runner.processes), 3)
                self.assertFalse((f.output/'renderer.json').exists())
                self.retained_lane(f, lane.fileno())

    def test_final_history_receipt_session_and_artifact_checks_gate_terminal(self) -> None:
        for failure in ('history', 'receipt', 'session', 'artifact'):
            with self.subTest(failure=failure), TemporaryDirectory() as directory:
                f = OwnerFixture(Path(directory).resolve())
                f.failure = failure
                with f.lock.open('rb') as lane, f.owner(lane.fileno()) as runner:
                    with self.assertRaises((ValueError, RuntimeError)):
                        _ = runner.run()
                    self.assertEqual(len(runner.processes), 3)
                    self.assertFalse((f.output/'renderer.json').exists())
                    self.retained_lane(f, lane.fileno())

    def test_all_three_producers_return_before_ordinary_terminal(self) -> None:
        with TemporaryDirectory() as directory:
            f = OwnerFixture(Path(directory).resolve())
            with f.lock.open('rb') as lane, f.owner(lane.fileno()) as runner:
                result = runner.run()
                self.assertEqual(result.commands, (f.artifact_root/'colony_s100_commands.json').read_bytes())
                self.assertEqual(f.events, ['bridge', 'import-start', 'import-finish', 'checkpoint',
                                           'checkpoint', 'generator', 'checkpoint', 'ascii',
                                           'checkpoint', 'godot', 'checkpoint'])
                terminal = document((f.output/'renderer.json').read_text())
                terminal.field('status').equals('ORDINARY_RENDERER_RETURNED')
                terminal.field('qualification').equals('NOT_RUN')
                terminal.field('matrix_admission').equals('UNAVAILABLE')
                self.assertEqual(len(terminal.field('processes').array()), 3)
                self.retained_lane(f, lane.fileno())


if __name__ == '__main__':
    _ = unittest.main()
