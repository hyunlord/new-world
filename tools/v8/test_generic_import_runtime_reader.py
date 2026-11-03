"""Public reader with mocked upstream bridge authority; no native processes."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import gate_oracle_child as controller
from generic_gate import document
from generic_import_runtime_reader import read_import_closure
from import_reader_fixture import ImportFixture
from mux2_gate_inputs import Json


class ImportReaderTests(unittest.TestCase):
    def test_original_checkpoint_positive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f = ImportFixture(Path(directory).resolve())
            with patch('generic_import_runtime_reader.authenticate_bridge', return_value=f.bridge_inputs):
                result = read_import_closure(f.prepared, f.prior, f.output/'receipt.json', f.seal())
            result.field('cache').equals(f.receipt.field('cache_terminal').value)

    def test_rejected_original_chain_mutations(self) -> None:
        changes: tuple[tuple[str, str, Json], ...] = (
            ('children/suite-start.json', 'owner_pid', 999),
            ('children/suite-terminal.json', 'completed', ['import.second']),
            ('children/suite-terminal.json', 'wrapper_waits_observed', True),
            ('children/01-import.first/request.json', 'parent_pid', 999),
            ('children/01-import.first/native-terminal.json', 'native_pid', 999),
            ('children/01-import.first/native-terminal.json', 'native_exit', 7),
            ('children/02-import.second/native-terminal.json', 'timed_out', True),
            ('children/02-import.second/native-terminal.json', 'stream_sha256', '0'*64),
            ('children/02-import.second/import-checkpoint.json', 'target', {}),
        )
        for name, key, value in changes:
            with self.subTest(name=name, key=key), tempfile.TemporaryDirectory() as directory:
                f = ImportFixture(Path(directory).resolve())
                path = f.output/name
                row = document(path.read_text())
                row.mapping()[key] = value
                f.write(path, row.value)
                with patch('generic_import_runtime_reader.authenticate_bridge', return_value=f.bridge_inputs), self.assertRaises(ValueError):
                    _ = read_import_closure(f.prepared, f.prior, f.output/'receipt.json', f.seal())

    def test_cache_is_not_readopted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f = ImportFixture(Path(directory).resolve())
            _ = (f.root/'.godot/imported').write_text('changed after original native checkpoint')
            with patch('generic_import_runtime_reader.authenticate_bridge', return_value=f.bridge_inputs), self.assertRaises(ValueError):
                _ = read_import_closure(f.prepared, f.prior, f.output/'receipt.json', f.seal())

    def test_self_asserted_terminal_cache_is_not_authority(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f = ImportFixture(Path(directory).resolve())
            f.receipt.mapping()['cache_terminal'] = {}
            with patch('generic_import_runtime_reader.authenticate_bridge', return_value=f.bridge_inputs), self.assertRaises(ValueError):
                _ = read_import_closure(f.prepared, f.prior, f.output/'receipt.json', f.seal())

    def test_producer_source_and_stale_receipt_rejected(self) -> None:
        for mutation in ('source', 'receipt'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                f = ImportFixture(Path(directory).resolve())
                digest = f.seal()
                path = (f.root/'tools/v8/gate_oracle_child.py' if mutation == 'source'
                        else f.output/'receipt.json')
                _ = path.write_text(path.read_text()+' ')
                with patch('generic_import_runtime_reader.authenticate_bridge', return_value=f.bridge_inputs), self.assertRaises(ValueError):
                    _ = read_import_closure(f.prepared, f.prior, f.output/'receipt.json', digest)

    def test_late_source_and_prior_receipt_drift(self) -> None:
        original_scan = controller.import_state
        for mutation in ('source', 'prior'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                f = ImportFixture(Path(directory).resolve())
                path = (f.root/'tools/v8/run_bounded.sh' if mutation == 'source' else f.prior.result.receipt)

                def mutate_after_scan(plan: controller.SuitePlan, target: Path = path) -> dict[str, Json]:
                    result = original_scan(plan)
                    _ = target.write_text(target.read_text()+' ')
                    return result

                with patch('generic_import_runtime_reader.authenticate_bridge', return_value=f.bridge_inputs), \
                     patch('generic_import_runtime_reader.controller.import_state', side_effect=mutate_after_scan), \
                     self.assertRaises(ValueError):
                    _ = read_import_closure(f.prepared, f.prior, f.output/'receipt.json', f.seal())

    def test_source_and_plan_closure_cannot_be_trimmed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            f = ImportFixture(Path(directory).resolve())
            with patch('generic_import_runtime_reader.authenticate_bridge', return_value=()), self.assertRaises(ValueError):
                _ = read_import_closure(f.prepared, f.prior, f.output/'receipt.json', f.seal())


if __name__ == '__main__':
    _ = unittest.main()
