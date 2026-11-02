"""Synthetic child boundaries; no native simulation or build execution."""
import json
import sys
import tempfile
import unittest
from collections.abc import Callable
from dataclasses import replace
from pathlib import Path

from generic_gate import GateContractError, document
from generic_process import ProcessSpec, execute, file_hash
from generic_sources import inventory
from generic_test_observations import (
    Binding,
    ExpectedOccurrence,
    PendingReason,
    authenticate,
)
from mux2_gate_inputs import Json, Node

CHILD = '''import json, os, pathlib, sys
root = pathlib.Path(os.environ["WORLDSIM_GATE_TEST_OUT"])
d = root / ("save_load_pid" + str(os.getpid())) / "fixture_s7_original"
d.mkdir(parents=True)
def state(t):
 return {"tick":t,"alive":2,"alive_ids":[1,2],"raw_agent_population":2,"raw_agent_ids":[1,2],"census_semantics":"alive excludes BodyHealth::is_dead; raw includes every Agent; wrapper must check ordinary endpoint agreement","knowledge_capability":{"enabled":True,"source":"resources.rules.colony.knowledge"},"knowledge_lost":[],"history_order_valid":True,"chronicle_entries":0,"command_log":[],"births":0,"deaths":{"starvation":0,"dehydration":0,"exposure":0,"combat":0,"old_age":0}}
a=state(0); z=state(5)
initial={"schema_version":1,"test":"fixture","seed":7,"branch":"original","pid":os.getpid(),"initial":a,"history_complete":True,"status":"PENDING_NATIVE_TEST_RESULT"}
terminal={"schema_version":1,"initial":a,"founding":a,"terminal":z,"completed_ticks":5,"stop_reason":"five_ticks","prefix":None,"history_complete":True,"loaded_prefix_requires_external_authentication":False,"status":"ENDPOINT_REACHED_NATIVE_TEST_RESULT_REQUIRED"}
(d/"initial.json").write_text(json.dumps(initial)); (d/"terminal.json").write_text(json.dumps(terminal))
print("test fixture ... ok")
sys.exit(int(sys.argv[1]))
'''


class ObservationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.script = self.root / 'child.py'
        self.script.write_text(CHILD)
        self.output = self.root / 'run'
        executable = Path(sys.executable).resolve()
        self.spec = ProcessSpec((str(executable), str(self.script), '0'), self.root,
                                executable, file_hash(executable), self.root,
                                (('child.py', file_hash(self.script)),), self.output,
                                (('WORLDSIM_GATE_OBSERVER', '1'), ('WORLDSIM_GATE_TEST_OUT', str(self.output))))
        result = execute(self.spec)
        assert result.pid is not None
        self.pid = result.pid
        self.directory = self.output / f'save_load_pid{self.pid}' / 'fixture_s7_original'
        self.expected = ExpectedOccurrence('fixture', 'fixture', 7, 'original', 0, 5, True)
        self.binding = Binding(self.spec, file_hash(result.receipt), self.pid, self.hashes())

    def hashes(self) -> tuple[tuple[str, str], ...]:
        return tuple(inventory(self.output, [f'save_load_pid{self.pid}']).items())

    def change(self, name: str, edit: Callable[[dict[str, Json]], None]) -> None:
        path = self.directory / name
        value = document(path.read_text()).mapping()
        edit(value)
        path.write_text(json.dumps(value))
        self.binding = replace(self.binding, output_hashes=self.hashes())

    def test_real_child_authenticates_exact_endpoint(self) -> None:
        result = authenticate(self.binding, self.expected)
        self.assertEqual(result.pending, ())
        self.assertTrue(result.history_complete)
        self.assertEqual(result.terminal.tick, 5)
        assert result.founding is not None
        self.assertEqual(result.founding.alive_ids, (1, 2))

    def test_stale_output_hash_rejected(self) -> None:
        (self.directory / 'terminal.json').write_text('{}')
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)

    def test_pid_seed_branch_and_full_horizon_are_bound(self) -> None:
        for expected in [replace(self.expected, seed=8), replace(self.expected, branch='other'),
                         replace(self.expected, end_tick=6), replace(self.expected, start_tick=1)]:
            with self.subTest(expected=expected), self.assertRaises((GateContractError, OSError)):
                authenticate(self.binding, expected)
        with self.assertRaises(GateContractError):
            authenticate(replace(self.binding, pid=self.pid + 1), self.expected)

    def test_raw_live_disagreement_rejected_for_ordinary(self) -> None:
        self.change('terminal.json', lambda v: Node(v['terminal']).mapping().update(alive=1, alive_ids=[1]))
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)
        self.assertEqual(authenticate(self.binding, replace(self.expected, ordinary=False)).terminal.alive_ids, (1,))

    def test_duplicate_keys_and_unknown_fields_rejected(self) -> None:
        self.change('terminal.json', lambda v: v.update(unknown=True))
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)
        path = self.directory / 'terminal.json'
        path.write_text('{"schema_version":1,"schema_version":1}')
        self.binding = replace(self.binding, output_hashes=self.hashes())
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)

    def test_bool_is_not_integer(self) -> None:
        self.change('terminal.json', lambda v: v.update(completed_ticks=True))
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)

    def test_symlink_rejected(self) -> None:
        path = self.directory / 'terminal.json'
        saved = self.root / 'saved.json'
        path.rename(saved)
        path.symlink_to(saved)
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)

    def test_native_failure_is_retained(self) -> None:
        output = self.root / 'failed_run'
        spec = replace(self.spec, argv=(*self.spec.argv[:-1], '9'), output_root=output,
                       environment=(('WORLDSIM_GATE_OBSERVER', '1'), ('WORLDSIM_GATE_TEST_OUT', str(output))))
        child = execute(spec)
        assert child.pid is not None
        hashes = tuple(inventory(output, [f'save_load_pid{child.pid}']).items())
        result = authenticate(Binding(spec, file_hash(child.receipt), child.pid, hashes), self.expected)
        self.assertEqual(result.native_exit, 9)
        self.assertIn(PendingReason.NATIVE_PROCESS_FAILED, result.pending)

    def test_missing_native_test_outcome_remains_pending(self) -> None:
        result = authenticate(self.binding, replace(self.expected, native_test='construction::fixture'))
        self.assertIn(PendingReason.NATIVE_TEST_OUTCOME_MISSING, result.pending)

    def loaded(self, external: bool) -> None:
        initial = document((self.directory / 'initial.json').read_text()).field('initial').value
        artifact = self.directory / 'save_prefix.wsave'
        raw = b'WORLDSIM' + (11).to_bytes(4, 'little') + b'synthetic-save-payload'
        artifact.write_bytes(raw)
        checkpoint = self.directory / 'save_prefix.json'
        checkpoint.write_text(json.dumps({'state': initial, 'save_artifact': str(artifact), 'byte_count': len(raw),
                                         'magic': list(raw[:8]), 'input_version_le_bytes': list(raw[8:12]),
                                         'current_format_version': 11, 'external_sha256_required': True}))
        prefix: dict[str, Json] = {'save_artifact': str(artifact), 'checkpoint': str(checkpoint),
                                  'external_hash_authentication_required': True}
        founding = initial
        if external:
            prefix.update(kind='external_legacy_save', founding_census=None)
            founding = None
            (self.directory / 'external-prefix.json').write_text(json.dumps(prefix))
        else:
            prefix['native_prefix_equal'] = True
            (self.directory / 'load-link.json').write_text(json.dumps({'prefix': prefix, 'founding': founding, 'initial': initial}))
        self.change('terminal.json', lambda v: v.update(prefix=prefix, founding=founding,
                    history_complete=False, loaded_prefix_requires_external_authentication=True))

    def test_loaded_equal_bytes_do_not_invent_complete_prefix(self) -> None:
        self.loaded(False)
        result = authenticate(self.binding, self.expected)
        self.assertFalse(result.history_complete)
        self.assertEqual(result.pending, (PendingReason.PREFIX_GRAPH_UNRESOLVED,))

    def test_genuine_external_founders_stay_unknown(self) -> None:
        self.loaded(True)
        result = authenticate(self.binding, self.expected)
        self.assertIsNone(result.founding)
        self.assertFalse(result.history_complete)
        self.assertEqual(result.pending, (PendingReason.EXTERNAL_ORIGIN_UNRESOLVED,))

    def test_loaded_false_completeness_and_invented_external_founders_rejected(self) -> None:
        self.loaded(True)
        self.change('terminal.json', lambda v: v.update(history_complete=True))
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)
        self.change('terminal.json', lambda v: v.update(history_complete=False, founding=v['initial']))
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)

    def test_missing_prefix_graph_is_typed_pending(self) -> None:
        self.loaded(False)
        hashes = tuple((name, digest) for name, digest in self.binding.output_hashes if not name.endswith('.wsave'))
        result = authenticate(replace(self.binding, output_hashes=hashes), self.expected)
        self.assertIn(PendingReason.PREFIX_GRAPH_UNRESOLVED, result.pending)

    def test_prefix_byte_metadata_mismatch_rejected(self) -> None:
        self.loaded(False)
        self.change('save_prefix.json', lambda v: v.update(byte_count=1))
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)

    def test_loss_duplicates_are_preserved_and_future_loss_rejected(self) -> None:
        value = document((self.directory / 'terminal.json').read_text()).mapping()
        state = Node(value['terminal']).mapping()
        duplicate_losses: list[Json] = [{'tick': 2, 'node': 'Fire', 'holder': None}] * 2
        state.update(knowledge_lost=duplicate_losses, chronicle_entries=2)
        self.change('terminal.json', lambda v: v.update(terminal=state))
        result = authenticate(self.binding, self.expected)
        assert result.terminal.losses is not None
        self.assertEqual(len(result.terminal.losses), 2)
        state.update(knowledge_lost=[{'tick': 6, 'node': 'Fire', 'holder': None}])
        self.change('terminal.json', lambda v: v.update(terminal=state))
        with self.assertRaises(GateContractError):
            authenticate(self.binding, self.expected)

    def test_process_receipt_and_executable_identity_are_pinned(self) -> None:
        with self.assertRaises(GateContractError):
            authenticate(replace(self.binding, receipt_sha256='0' * 64), self.expected)
        with self.assertRaises(GateContractError):
            authenticate(replace(self.binding, process=replace(self.spec, executable_sha256='0' * 64)), self.expected)

    def test_output_path_escape_rejected(self) -> None:
        hashes = (*self.binding.output_hashes, ('../child.py', file_hash(self.script)))
        with self.assertRaises(GateContractError):
            authenticate(replace(self.binding, output_hashes=hashes), self.expected)

    def test_fresh_directory_binding_is_mandatory(self) -> None:
        path = self.output / 'process.json'
        receipt = document(path.read_text()).mapping()
        self.assertEqual(receipt['fresh_directories'], [])
        for replacement in ({key: value for key, value in receipt.items() if key != 'fresh_directories'},
                            {**receipt, 'fresh_directories': ['settings']}):
            _ = path.write_text(json.dumps(replacement))
            with self.assertRaises(GateContractError):
                _ = authenticate(replace(self.binding, receipt_sha256=file_hash(path)), self.expected)

    def test_external_input_maps_are_mandatory_even_when_empty(self) -> None:
        path = self.output / 'process.json'
        receipt = document(path.read_text()).mapping()
        for field in ['expected_external_inputs', 'before_external_inputs', 'after_external_inputs']:
            self.assertEqual(receipt[field], {})
            missing = {key: value for key, value in receipt.items() if key != field}
            path.write_text(json.dumps(missing))
            with self.subTest(field=field), self.assertRaises(GateContractError):
                authenticate(replace(self.binding, receipt_sha256=file_hash(path)), self.expected)

    def test_changed_external_input_receipt_rejected(self) -> None:
        external_temp = tempfile.TemporaryDirectory()
        self.addCleanup(external_temp.cleanup)
        external = Path(external_temp.name).resolve() / 'external.dat'
        external.write_bytes(b'fixed external input')
        output = self.root / 'external_run'
        spec = replace(self.spec, output_root=output, external_inputs=((external, file_hash(external)),),
                       environment=(('WORLDSIM_GATE_OBSERVER', '1'), ('WORLDSIM_GATE_TEST_OUT', str(output))))
        child = execute(spec)
        assert child.pid is not None
        hashes = tuple(inventory(output, [f'save_load_pid{child.pid}']).items())
        binding = Binding(spec, file_hash(child.receipt), child.pid, hashes)
        self.assertEqual(authenticate(binding, self.expected).pending, ())
        receipt = document(child.receipt.read_text()).mapping()
        for field in ['expected_external_inputs', 'before_external_inputs', 'after_external_inputs']:
            changed = {**receipt, field: {str(external): '0' * 64}}
            child.receipt.write_text(json.dumps(changed))
            with self.subTest(field=field), self.assertRaises(GateContractError):
                authenticate(replace(binding, receipt_sha256=file_hash(child.receipt)), self.expected)


if __name__ == '__main__':
    unittest.main()
