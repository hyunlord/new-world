"""Synthetic original sections and real source files; never native evidence."""
import shutil
import tempfile
import unittest
from pathlib import Path

import generic_process
from generic_gate import GateContractError
from generic_whole_gate_core_material import classify_core_material
from generic_whole_gate_core_material_pins import CONTRACTS, PINS
from mux2_gate_inputs import EvidenceError, Json, Node

ROOT = Path(generic_process.__file__).resolve().parents[2]


def sections() -> list[Json]:
    return [{'package': 'sim-core', 'target': name, 'kind': 'test', 'source': source,
             'nested_passed': 0,
             'cases': [{'name': case, 'status': 'ok', 'mode': None} for case in cases]}
            for name, source, cases in CONTRACTS]


def copied(root: Path) -> None:
    for name in PINS:
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        _ = shutil.copyfile(ROOT / name, path)


class MaterialClassificationTests(unittest.TestCase):
    def test_complete_original_targets_admitted_without_safety_claim(self) -> None:
        result = classify_core_material(ROOT, dict(PINS), sections())
        self.assertEqual(len(result), 2)
        self.assertEqual(sum(len(Node(row).field('cases').array()) for row in result), 30)
        for row in result:
            Node(row).field('publication_authority').equals(False)
            Node(row).field('ordinary_safety').equals('NOT_APPLICABLE_NO_WORLD')

    def test_missing_duplicate_or_foreign_target_rejected(self) -> None:
        for index in (0, 1):
            original = sections()
            foreign = {**Node(original[index]).mapping(), 'package': 'sim-test'}
            variants = (original[:index] + original[index+1:], original + [original[index]],
                        original[:index] + [foreign] + original[index+1:])
            for rows in variants:
                with self.subTest(index=index, rows=rows), self.assertRaises(GateContractError):
                    _ = classify_core_material(ROOT, dict(PINS), rows)

    def test_full_case_census_required_for_each_target(self) -> None:
        for index, (_, _, names) in enumerate(CONTRACTS):
            for changed in (names[:-1], (*names, names[0]), (*names, 'unreviewed')):
                rows = sections()
                row = dict(Node(rows[index]).mapping())
                row['cases'] = [{'name': name, 'status': 'ok', 'mode': None} for name in changed]
                rows[index] = row
                with self.subTest(index=index, changed=changed), self.assertRaises(GateContractError):
                    _ = classify_core_material(ROOT, dict(PINS), rows)

    def test_oracles_source_and_nested_records_remain_mandatory(self) -> None:
        changes: tuple[tuple[str, Json], ...] = (('source', 'wrong'), ('nested_passed', 1),
            ('cases', [{'name': 'failed', 'status': 'FAILED', 'mode': None}]))
        for index in (0, 1):
            for key, value in changes:
                rows = sections()
                rows[index] = {**Node(rows[index]).mapping(), key: value}
                with self.subTest(index=index, key=key), self.assertRaises((GateContractError, EvidenceError)):
                    _ = classify_core_material(ROOT, dict(PINS), rows)
            for status, mode in (('FAILED', None), ('ignored', None), ('ok', 'bench')):
                rows = sections()
                row = dict(Node(rows[index]).mapping())
                row['cases'] = [{'name': name, 'status': status, 'mode': mode} for name in CONTRACTS[index][2]]
                rows[index] = row
                with self.subTest(index=index, status=status, mode=mode), self.assertRaises((GateContractError, EvidenceError)):
                    _ = classify_core_material(ROOT, dict(PINS), rows)

    def test_every_source_and_data_hash_is_required(self) -> None:
        for path in PINS:
            inputs = dict(PINS)
            del inputs[path]
            with self.subTest(path=path), self.assertRaises(GateContractError):
                _ = classify_core_material(ROOT, inputs, sections())

    def test_every_physical_source_and_data_mutation_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            copied(root)
            for name in PINS:
                path = root / name
                original = path.read_bytes()
                _ = path.write_bytes(original + b'\n')
                with self.subTest(name=name), self.assertRaises(GateContractError):
                    _ = classify_core_material(root, dict(PINS), sections())
                _ = path.write_bytes(original)

    def test_added_ron_namespace_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            copied(root)
            extra = root / 'rust/crates/sim-core/data/material/stone/unreviewed.ron'
            _ = extra.write_text('unreviewed')
            with self.assertRaisesRegex(GateContractError, 'material directory census'):
                _ = classify_core_material(root, dict(PINS), sections())

    def test_missing_and_symlink_data_rejected(self) -> None:
        for symlink in (False, True):
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                copied(root)
                name = 'rust/crates/sim-core/data/material/stone/granite.ron'
                path = root / name
                path.unlink()
                if symlink:
                    path.symlink_to(ROOT / name)
                with self.subTest(symlink=symlink), self.assertRaises(GateContractError):
                    _ = classify_core_material(root, dict(PINS), sections())

    def test_unresolved_library_and_nested_target_remain_unclassified(self) -> None:
        rows = sections() + [{'package': 'sim-core', 'target': name, 'kind': kind}
                             for name, kind in (('sim_core', 'lib'), ('recipe_registry', 'test'), ('m6a_content', 'test'))]
        result = classify_core_material(ROOT, dict(PINS), rows)
        self.assertEqual([Node(row).field('target').text() for row in result],
                         ['material_harness', 'material_loader_integration'])


if __name__ == '__main__':
    _ = unittest.main()
