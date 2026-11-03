"""Pure synthetic classification checks, never native acceptance evidence."""
import shutil
import tempfile
import unittest
from pathlib import Path

import generic_process
from generic_gate import GateContractError
from generic_whole_gate_config_pure import CASES, PINS, SOURCE, classify_config_pure
from mux2_gate_inputs import EvidenceError, Json, Node

ROOT = Path(generic_process.__file__).resolve().parents[2]


def section() -> dict[str, Json]:
    return {'package': 'sim-systems', 'target': 'baseline_worldgen_matches_v7_final', 'kind': 'test',
            'source': SOURCE, 'nested_passed': 0,
            'cases': [{'name': name, 'status': 'ok', 'mode': None} for name in CASES]}


class CorePureTests(unittest.TestCase):
    def test_exact_original_case_census_is_classified(self) -> None:
        result = classify_config_pure(ROOT, dict(PINS), [section()])
        self.assertEqual(len(result), 1)
        Node(result[0]).field('classification').equals('PURE_WORLDGEN_CONFIG_NO_WORLD')
        Node(result[0]).field('publication_authority').equals(False)

    def test_missing_duplicate_or_foreign_target_rejected(self) -> None:
        variants: tuple[list[Json], ...] = ([], [section(), section()], [{**section(), 'package': 'sim-core'}])
        for rows in variants:
            with self.subTest(rows=rows), self.assertRaises(GateContractError):
                _ = classify_config_pure(ROOT, dict(PINS), rows)

    def test_changed_full_case_set_rejected(self) -> None:
        for names in ((), CASES[:1], (CASES[0], CASES[0]), (*CASES, 'new_unreviewed_case')):
            row = section()
            row['cases'] = [{'name': name, 'status': 'ok', 'mode': None} for name in names]
            with self.subTest(names=names), self.assertRaises(GateContractError):
                _ = classify_config_pure(ROOT, dict(PINS), [row])

    def test_original_oracle_status_and_mode_required(self) -> None:
        for status, mode in (('ignored', None), ('FAILED', None), ('ok', 'bench')):
            row = section()
            row['cases'] = [{'name': name, 'status': status, 'mode': mode} for name in CASES]
            with self.subTest(status=status, mode=mode), self.assertRaises((GateContractError, EvidenceError)):
                _ = classify_config_pure(ROOT, dict(PINS), [row])

    def test_wrong_source_or_nested_outcome_rejected(self) -> None:
        for key, value in (('source', 'foreign.rs'), ('nested_passed', 1)):
            row = section()
            row[key] = value
            with self.subTest(key=key), self.assertRaises((GateContractError, EvidenceError)):
                _ = classify_config_pure(ROOT, dict(PINS), [row])

    def test_every_source_and_fixture_binding_is_required(self) -> None:
        for name in PINS:
            inputs = dict(PINS)
            del inputs[name]
            with self.subTest(name=name), self.assertRaises(GateContractError):
                _ = classify_config_pure(ROOT, inputs, [section()])

    def test_physical_source_and_fixture_mutation_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in PINS:
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                _ = shutil.copyfile(ROOT / name, target)
            for name in PINS:
                path = root / name
                original = path.read_bytes()
                _ = path.write_bytes(original + b'\n')
                with self.subTest(name=name), self.assertRaises(GateContractError):
                    _ = classify_config_pure(root, dict(PINS), [section()])
                _ = path.write_bytes(original)

    def test_other_targets_never_acquire_classification(self) -> None:
        rows: list[Json] = [section(), {'package': 'sim-systems', 'target': 'sim_core', 'kind': 'lib'}]
        result = classify_config_pure(ROOT, dict(PINS), rows)
        self.assertEqual([Node(row).field('target').text() for row in result], ['baseline_worldgen_matches_v7_final'])


    def test_ignored_targets_and_nested_recipe_remain_unclassified(self) -> None:
        unresolved = ('content_registry_migration', 'legacy_migration_current', 'm6a_hunger_admission',
                      'm6b_steward_save_migration', 'steward_defence_migration', 'recipe_registry', 'm6a_content')
        extras: list[Json] = [
            {'package': 'sim-core' if name in {'recipe_registry', 'm6a_content'} else 'sim-test',
             'target': name, 'kind': 'test', 'cases': [{'name': 'unexecuted', 'status': 'ignored'}]}
            for name in unresolved]
        rows: list[Json] = [section(), *extras]
        result = classify_config_pure(ROOT, dict(PINS), rows)
        self.assertEqual([Node(row).field('target').text() for row in result],
                         ['baseline_worldgen_matches_v7_final'])


if __name__ == '__main__':
    _ = unittest.main()
