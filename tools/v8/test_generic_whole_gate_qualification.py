# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_qualification
"""Reader unit fixtures never constitute real product qualification."""
import json
import sys
import tempfile
import unittest
from pathlib import Path
from typing import final

from generic_gate import GateContractError, document
from generic_process import file_hash
from generic_whole_gate_qualification import (
    MODES,
    compare_artifacts,
    identity,
    matrix,
    qualification,
    reference,
)
from generic_whole_gate_qualification_build import authenticate
from mux2_gate_inputs import EvidenceError, Json, Node


@final
class QualificationTests(unittest.TestCase):
    def __init__(self, methodName: str = "runTest") -> None:
        super().__init__(methodName)
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name).resolve()
        self.addCleanup(self.temp.cleanup)

    def ref(self, path: Path) -> Node:
        return Node({'path': str(path), 'sha256': file_hash(path), 'mode': path.lstat().st_mode})

    def test_unrelated_build_command_is_not_product_provenance(self) -> None:
        tool = Path(sys.executable).resolve()
        digest = file_hash(tool)
        selector = self.root/'selector.json'
        tools: dict[str, Json] = {name: {'path': str(tool), 'sha256': digest} for name in ('cargo','rustc')}
        _ = selector.write_text(json.dumps({'selectors': tools}))
        session = Node({'selector_path': str(selector), 'selector_sha256': file_hash(selector),
                        'external_inputs': {str(tool): digest}})
        build = Node({'cargo_sha256': digest, 'rustc_sha256': digest, 'argv': ['not-cargo','--unrelated']})
        with self.assertRaisesRegex(EvidenceError, 'expected'):
            authenticate(session, build, Node({}), 'worldsim_probe', 'derived_off')

    def test_absent(self) -> None:
        self.assertEqual(qualification(Node({})), 'PRODUCT_UNQUALIFIED')

    def test_missing_reference(self) -> None:
        with self.assertRaises((GateContractError, FileNotFoundError)):
            _ = qualification(Node({'observer_qualification': {'path': str(self.root/'missing'), 'sha256': 'a'*64, 'mode': 33188}}))

    def test_reference_mode(self) -> None:
        path = self.root/'evidence.json'
        _ = path.write_text('{}')
        anchor = self.ref(path)
        self.assertEqual(reference(anchor)[1].value, {})
        path.chmod(0o700)
        with self.assertRaises((GateContractError, EvidenceError)):
            _ = reference(anchor)

    def test_reference_bytes(self) -> None:
        path = self.root/'evidence.json'
        _ = path.write_text('{}')
        anchor = self.ref(path)
        _ = path.write_text('{"changed":true}')
        with self.assertRaises((GateContractError, EvidenceError)):
            _ = reference(anchor)

    def fixture(self) -> tuple[Node, tuple[Path, Path, Path], dict[str, Json]]:
        outputs = (self.root/'original', self.root/'off', self.root/'on')
        evidence: dict[str, Json] = {}
        for mode, output in zip(MODES, outputs, strict=True):
            (output/'artifacts').mkdir(parents=True)
            artifact = output/'artifacts/state.json'
            _ = artifact.write_text('{"world":42}')
            evidence[mode] = {'artifacts': {'state.json': self.ref(artifact).value}}
        return Node({'comparator': 'bytes', 'artifacts': ['state.json']}), outputs, evidence

    def test_equal_bytes_only(self) -> None:
        row, outputs, evidence = self.fixture()
        compare_artifacts(row, outputs, Node(evidence))
        self.assertEqual(qualification(Node({})), 'PRODUCT_UNQUALIFIED')

    def test_different_bytes_updated_anchor(self) -> None:
        row, outputs, evidence = self.fixture()
        path = outputs[2]/'artifacts/state.json'
        _ = path.write_text('{"world":43}')
        evidence['derived_on'] = {'artifacts': {'state.json': self.ref(path).value}}
        with self.assertRaises((GateContractError, EvidenceError)):
            compare_artifacts(row, outputs, Node(evidence))

    def test_missing_artifact(self) -> None:
        row, outputs, evidence = self.fixture()
        evidence['derived_on'] = {'artifacts': {}}
        with self.assertRaises((GateContractError, EvidenceError)):
            compare_artifacts(row, outputs, Node(evidence))

    def test_foreign_equal_bytes(self) -> None:
        row, outputs, evidence = self.fixture()
        evidence['derived_on'] = evidence['derived_off']
        with self.assertRaises((GateContractError, EvidenceError)):
            compare_artifacts(row, outputs, Node(evidence))

    def test_empty(self) -> None:
        row, outputs, evidence = self.fixture()
        for mode, output in zip(MODES, outputs, strict=True):
            path = output/'artifacts/state.json'
            _ = path.write_text('')
            evidence[mode] = {'artifacts': {'state.json': self.ref(path).value}}
        with self.assertRaises((GateContractError, EvidenceError)):
            compare_artifacts(row, outputs, Node(evidence))

    def test_unavailable(self) -> None:
        _, outputs, evidence = self.fixture()
        with self.assertRaises((GateContractError, EvidenceError)):
            compare_artifacts(Node({'comparator': 'UNAVAILABLE', 'artifacts': ['state.json']}), outputs, Node(evidence))

    def test_complete_63(self) -> None:
        production = document(Path(__file__).with_name('whole_gate_observer_matrix.json').read_text())
        self.assertEqual(len(production.field('observer_postimages').mapping()), 63)
        self.assertEqual(len(production.field('rows').array()), 53)
        covered = {path.text() for row in production.field('rows').array() for path in row.field('observer_paths').array()}
        self.assertEqual(covered, set(production.field('observer_postimages').mapping()))
        self.assertTrue(any(row.field('comparator').text() == 'UNAVAILABLE' for row in production.field('rows').array()))

    def identities(self) -> tuple[Node, dict[str, Json], Node]:
        roles = self.root/'roles.json'
        _ = roles.write_text('{"files":[]}')
        original: dict[str, Json] = {'observer.rs': ['100644', 'blob', 'digest']}
        effective: dict[str, Json] = {'manifest': {'observer.rs': ['100644', 'digest']}, 'sha256': 'effective'}
        source: dict[str, Json] = {'source_sha': 'product', 'source_before': {'tracked_manifest': original},
            'effective_source': effective, 'layer_manifest_path': str(roles)}
        harness: dict[str, Json] = {'source_sha': 'harness', 'inputs': {'input': 'hash'}}
        session = Node({'harness': harness, 'arms': {'base': source, 'candidate': source}})
        arm: dict[str, Json] = {'source_sha': 'product', 'original_source': original, 'effective_source': effective}
        observers: dict[str, Json] = {'observer.rs': ['100644', 'digest']}
        report: dict[str, Json] = {'schema': 'whole_gate_observer_qualification.v1', 'harness_sha': 'harness',
            'harness_inputs': {'input': 'hash'}, 'observer_postimages': observers,
            'arms': {'base': arm, 'candidate': arm}}
        return session, report, Node({'observer_postimages': observers})

    def test_source_identity_only(self) -> None:
        session, report, required = self.identities()
        identity(session, Node(report), required)
        self.assertEqual(qualification(session), 'PRODUCT_UNQUALIFIED')

    def test_wrong_harness(self) -> None:
        session, report, required = self.identities()
        report['harness_sha'] = 'different'
        with self.assertRaises((GateContractError, EvidenceError)):
            identity(session, Node(report), required)

    def test_wrong_observer_set(self) -> None:
        session, report, required = self.identities()
        report['observer_postimages'] = {}
        with self.assertRaises((GateContractError, EvidenceError)):
            identity(session, Node(report), required)

    def test_missing_product(self) -> None:
        session, report, required = self.identities()
        report['arms'] = {'base': Node(report).field('arms').field('base').value}
        with self.assertRaises((GateContractError, EvidenceError)):
            identity(session, Node(report), required)

    def test_foreign_product(self) -> None:
        session, report, required = self.identities()
        arms = Node(report).field('arms').mapping()
        arm = Node(arms['candidate']).mapping()
        arm['source_sha'] = 'foreign'
        with self.assertRaises((GateContractError, EvidenceError)):
            identity(session, Node(report), required)

    def test_four_added_targets_remain_unqualified(self) -> None:
        production = document(Path(__file__).with_name('whole_gate_observer_matrix.json').read_text())
        rows = {row.field('id').text():row for row in production.field('rows').array()}
        for target in ('engine_ascii','engine_knowledge_ffi','m6b_presentation_defaults','engine_colony_ffi'):
            name = 'rust/crates/sim-test/tests/'+target+'.rs'
            self.assertIn(name, production.field('observer_postimages').mapping())
            row = rows['native_'+target]
            self.assertEqual(row.field('observer_paths').value,[name])
            self.assertEqual(row.field('comparator').text(),'UNAVAILABLE')
            self.assertEqual(row.field('command').text(),'cargo test --release -p sim-test --test '+target+' -- --test-threads=1')
        self.assertEqual(sum(row.field('comparator').text()=='UNAVAILABLE' for row in rows.values()),32)
        self.assertEqual(rows['probe_core'].field('comparator').text(), 'UNAVAILABLE')
        self.assertEqual(rows['probe_core'].field('evidence').text(), 'NOT_RUN')
        self.assertEqual(production.field('origin_source_state').text(),'WORKING_TREE_SNAPSHOT_NOT_COMMITTED')

    def test_unbound_matrix(self) -> None:
        path = self.root/'tools/v8/whole_gate_observer_matrix.json'
        path.parent.mkdir(parents=True)
        _ = path.write_text('{}')
        with self.assertRaises((GateContractError, EvidenceError)):
            _ = matrix(Node({'harness': {'root': str(self.root), 'inputs': {'tools/v8/whole_gate_observer_matrix.json': 'a'*64}}}))

if __name__ == '__main__':
    _ = unittest.main()
