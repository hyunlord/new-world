"""Pure classification boundary tests; synthetic outcomes are not native acceptance."""
import copy
import tempfile
import unittest
from pathlib import Path

import generic_process
from generic_gate import GateContractError
from generic_whole_gate_workspace_classification import (
    CONTRACTS,
    PINS,
    classified_target,
    classify_routes,
)
from mux2_gate_inputs import EvidenceError, Node

ROOT = Path(generic_process.__file__).resolve().parents[2]


def sections():
    return [{'package': 'sim-test', 'target': name, 'kind': kind, 'source': source,
             'nested_passed': 0,
             'cases': [{'name': case, 'status': 'ok', 'mode': None} for case in names]}
            for name, kind, source, _, names, _ in CONTRACTS]


class ClassificationTests(unittest.TestCase):
    def test_reviewed_source_and_exact_original_outcomes(self):
        result = classify_routes(ROOT, dict(PINS), sections())
        self.assertEqual(len(result), 5)
        self.assertEqual(sum(len(row['cases']) for row in result), 21)
        self.assertTrue(all(row['publication_authority'] is False for row in result))

    def test_missing_original_target(self):
        with self.assertRaisesRegex(GateContractError, 'missing or duplicate'):
            classify_routes(ROOT, dict(PINS), sections()[1:])

    def test_duplicate_original_target(self):
        rows = sections()
        with self.assertRaisesRegex(GateContractError, 'missing or duplicate'):
            classify_routes(ROOT, dict(PINS), rows + [rows[0]])

    def test_missing_duplicate_and_unreviewed_cases(self):
        for cases in [[], [{'name': 'unreviewed', 'status': 'ok', 'mode': None}],
                      sections()[0]['cases'] * 2]:
            with self.subTest(cases=cases):
                rows = sections()
                rows[0]['cases'] = cases
                with self.assertRaisesRegex(GateContractError, 'case set differs'):
                    classify_routes(ROOT, dict(PINS), rows)

    def test_ignored_failed_and_mode_changed_rejected(self):
        for key, value in [('status', 'ignored'), ('status', 'FAILED'), ('mode', 'bench')]:
            rows = copy.deepcopy(sections())
            rows[0]['cases'][0][key] = value
            with self.subTest(key=key, value=value), self.assertRaises((GateContractError, EvidenceError)):
                classify_routes(ROOT, dict(PINS), rows)

    def test_target_and_dependency_source_pins_required(self):
        for path in [CONTRACTS[0][2], 'rust/crates/sim-engine/src/lib.rs',
                     'rust/crates/sim-test/src/probe/args.rs']:
            inputs = dict(PINS)
            inputs[path] = '0' * 64
            with self.subTest(path=path), self.assertRaisesRegex(GateContractError, 'source changed or unbound'):
                classify_routes(ROOT, inputs, sections())

    def test_direct_factory_and_ffi_dependencies_are_route_local(self):
        dependencies = {row[0]: row[-1] for row in CONTRACTS}
        needed = {
            'tree_snapshot': ('rust/crates/sim-systems/src/worldgen/mod.rs',
                              'rust/crates/sim-core/src/rules/colony.rs',
                              'rust/crates/sim-engine/src/lib.rs',
                              'rust/crates/sim-systems/src/lib.rs'),
            'engine_bridge_postverify': ('rust/crates/sim-bridge/src/ffi/node.rs',
                                        'rust/crates/sim-bridge/src/ffi/mod.rs'),
        }
        for target, paths in needed.items():
            for path in paths:
                with self.subTest(target=target, path=path):
                    self.assertIn(path, dependencies[target])
                    inputs = dict(PINS)
                    del inputs[path]
                    with self.assertRaisesRegex(GateContractError, 'source changed or unbound'):
                        classify_routes(ROOT, inputs, sections())

    def test_physical_source_tamper_rejected_despite_declared_hash(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            path = CONTRACTS[0][2]
            target = root / path
            target.parent.mkdir(parents=True)
            target.write_bytes((ROOT / path).read_bytes() + b"\n")
            with self.assertRaisesRegex(GateContractError, 'source changed or unbound'):
                classify_routes(root, dict(PINS), sections())

    def test_wrong_source_and_nested_outcome_rejected(self):
        for key, value in [('source', 'wrong.rs'), ('nested_passed', 1)]:
            rows = sections()
            rows[0][key] = value
            with self.subTest(key=key), self.assertRaises((GateContractError, EvidenceError)):
                classify_routes(ROOT, dict(PINS), rows)

    def test_ordinary_and_other_package_targets_remain_unclassified(self):
        result = classify_routes(ROOT, dict(PINS), sections())
        for target in ['worldsim_systems', 'engine_knowledge_ffi', 'colony_jobs', 'sim_test']:
            self.assertFalse(classified_target(Node({'package': 'sim-test', 'target': target, 'kind': 'test'}), result))
        self.assertFalse(classified_target(Node({'package': 'sim-core', 'target': 'tree_snapshot', 'kind': 'test'}), result))
        self.assertTrue(classified_target(Node({'package': 'sim-test', 'target': 'tree_snapshot', 'kind': 'test'}), result))


if __name__ == '__main__':
    unittest.main()
