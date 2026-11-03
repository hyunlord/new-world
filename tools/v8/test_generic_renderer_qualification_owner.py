"""Pure source-boundary negatives; mocked source recheck is not live build evidence."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_renderer_qualification_owner as owner
from generic_build import WorkspaceSourceProof
from generic_gate import GateContractError
from generic_process import file_hash
from generic_renderer_run import PINS
from generic_setup_prepare import PreparedSetup, SetupInputs, Tool
from mux2_gate_inputs import Json, Node
from test_generic_setup_bridge import unused_wait


class SourceTest(unittest.TestCase):
    def fixture(self) -> tuple[Node, PreparedSetup, Node]:
        directory = Path(owner.__file__).parent
        names = ('generic_renderer_qualification_owner.py', 'generic_renderer_qualification_specs.py',
                 'generic_setup_bridge.py', 'generic_setup_import.py', 'generic_import_runtime_reader.py',
                 'generic_import_checkpoint_reader.py', 'gate_oracle_child.py', 'generic_renderer_run.py',
                 'generic_whole_gate_renderer_compare.py', 'generic_original_harness_projection.py')
        source: dict[str, Json] = {'o.rs': ['100644', 'a'*40, 'b'*64]}
        effective: dict[str, Json] = {'manifest': {'o.rs': ['100644', 'c'*64]}}
        proof = WorkspaceSourceProof(json.dumps({'tracked_manifest': source}), json.dumps(effective), (), 'DERIVED_OBSERVER')
        context = SetupInputs(Path('/synthetic-root'), 'd'*40, Path('/cargo'), (Tool('unused', Path('/unused'), '0'*64),), (), (), (),
                              Path('/lock'), 'e'*64, Path('/settings'), Path('/bridge'))
        materialized = (('o.rs', 'c'*64), *( (name, PINS[name]) for name in
                       ('tools/v8/renderer_agreement.sh', 'tools/v8/u_script.json')))
        prepared = PreparedSetup(context, 'f'*64, materialized, (), (), derived_source=proof)
        arm: dict[str, Json] = {'source_sha': 'd'*40, 'source_before': {'tracked_manifest': source},
                                'effective_source': effective, 'source_inputs': {'o.rs': 'c'*64}}
        session = Node({'arms': {'base': arm}, 'harness': {'inputs': {
            'tools/v8/'+name: file_hash(directory/name) for name in names}}})
        matrix = Node({'observer_postimages': {'o.rs': ['100644', 'c'*64]}})
        return session, prepared, matrix

    def test_derived_modes_bind_exact_source(self) -> None:
        session, prepared, matrix = self.fixture()
        with patch.object(owner, 'matrix', return_value=matrix), patch.object(owner, 'immutable_inputs') as recheck:
            for mode in ('derived_off', 'derived_on'):
                owner.mode_source(session, 'base', mode, prepared)
            self.assertEqual(recheck.call_count, 2)

    def test_original_projection_is_explicitly_unavailable(self) -> None:
        session, prepared, _ = self.fixture()
        with self.assertRaisesRegex(GateContractError, 'H-only projection'):
            owner.mode_source(session, 'base', 'original_off', prepared)

    def test_source_and_observer_drift_reject(self) -> None:
        session, prepared, matrix = self.fixture()
        assert prepared.derived_source is not None
        changes = (
            replace(prepared, inputs=replace(prepared.inputs, source_sha='0'*40)),
            replace(prepared, derived_source=None),
            replace(prepared, derived_source=replace(prepared.derived_source, baseline_json='{"tracked_manifest":{}}')),
            replace(prepared, derived_source=replace(prepared.derived_source, effective_json='{"manifest":{}}')),
            replace(prepared, materialized=(('o.rs', '0'*64),)),
        )
        with patch.object(owner, 'matrix', return_value=matrix), patch.object(owner, 'immutable_inputs'):
            for changed in changes:
                with self.subTest(changed=changed), self.assertRaises(ValueError):
                    owner.mode_source(session, 'base', 'derived_on', changed)
        wrong = Node({'observer_postimages': {'o.rs': ['100644', '0'*64]}})
        with patch.object(owner, 'matrix', return_value=wrong), self.assertRaises(ValueError):
            owner.mode_source(session, 'base', 'derived_on', prepared)

    def test_three_mode_preflight_rejects_before_native_work(self) -> None:
        session, prepared, _ = self.fixture()
        session.mapping()['materialization'] = {}
        with TemporaryDirectory() as directory:
            base = Path(directory).resolve()
            requests: list[owner.RendererRequest] = []
            for mode in ('original_off', 'derived_off', 'derived_on'):
                root = base/mode
                root.mkdir()
                selected = replace(prepared, inputs=replace(prepared.inputs, root=root))
                requests.append(owner.RendererRequest(base/'session', 'a'*64, 'base', mode,
                                                      selected, unused_wait(selected), base/(mode+'-output')))
            with patch.object(owner, 'load_session', return_value=session), patch.object(owner, 'run_bridge_owned') as bridge:
                with self.assertRaisesRegex(GateContractError, 'H-only projection'):
                    _ = owner.prepare_modes(tuple(requests), -1)
                bridge.assert_not_called()
            with self.assertRaisesRegex(GateContractError, 'three-mode'):
                _ = owner.prepare_modes(tuple(requests[1:]), -1)

    def test_harness_and_selection_drift_reject(self) -> None:
        session, prepared, matrix = self.fixture()
        for arm, mode in (('foreign', 'derived_on'), ('base', 'foreign')):
            with self.assertRaises(GateContractError):
                owner.mode_source(session, arm, mode, prepared)
        harness = session.field('harness').field('inputs').mapping()
        harness['tools/v8/generic_renderer_qualification_owner.py'] = '0'*64
        with patch.object(owner, 'matrix', return_value=matrix), self.assertRaises(GateContractError):
            owner.mode_source(session, 'base', 'derived_on', prepared)


if __name__ == '__main__':
    _ = unittest.main()
