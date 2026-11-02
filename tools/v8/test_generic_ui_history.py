"""Synthetic authenticated save chains; no engine process is launched."""
import copy
import json
import unittest
from dataclasses import replace
from pathlib import Path
from typing import override

import test_generic_ui_receipts as fixtures
from generic_gate import GateContractError, document, integer, text
from generic_process import file_hash
from generic_ui_history import (
    JoinedHistory,
    SaveLink,
    UIEvidence,
    join_visibility_history,
)
from generic_ui_receipts import CheckpointPlan, InstancePlan
from mux2_gate_inputs import Json, Node


class HistoryTests(unittest.TestCase):
    generator: fixtures.UIReceiptTests
    loader: fixtures.UIReceiptTests
    link: SaveLink
    loss: dict[str, Json]

    @override
    def setUp(self) -> None:
        self.generator = fixtures.UIReceiptTests()
        self.loader = fixtures.UIReceiptTests()
        for fixture in (self.generator, self.loader):
            fixture.setUp()
            self.addCleanup(fixture.doCleanups)
        g, l = self.generator, self.loader
        save = g.output / 'scene.wsave'
        _ = save.write_bytes(b'synthetic saved bytes; never loaded by an engine')
        manifest = g.output / 'manifest.json'
        row: dict[str, Json] = {'id': 'scene', 'save': str(save), 'sha256': file_hash(save), 'tick': 30}
        _ = manifest.write_text(json.dumps({'scenes': [row]}))
        self.link = SaveLink(save, file_hash(save), manifest, file_hash(manifest), 'scene', 1, 2, 1, 'bridge.dylib')
        self.loss = {'tick': '20', 'node': 'fire', 'node_index': 0, 'holder': str((1 << 64) - 1)}
        prefix: dict[str, Json] = {'save_path': str(save), 'scene_id': 'scene'}
        saved: dict[str, Json] = {**prefix, 'prefix_checkpoint': 1, 'save_sha256': file_hash(save)}
        loaded: dict[str, Json] = {'save_path': str(save), 'save_sha256': file_hash(save), 'manifest_path': str(manifest),
                  'manifest_sha256': file_hash(manifest), 'manifest_row': row}
        g.data['checkpoints'] = [g.checkpoint(0, 'bootstrap', 0), g.checkpoint(1, 'save_prefix', 30),
                                 g.checkpoint(2, 'save_link', 30), g.checkpoint(3, 'terminal', 40)]
        for cp, context in zip(fixtures.checkpoints(g.data)[1:3], (prefix, saved)):
            cp['context'] = context
        for cp in fixtures.checkpoints(g.data):
            native = fixtures.obj(cp, 'observation', 'instances', 0)
            native['initial_population'] = 2
            native['initial_agent_ids'] = ['0', str((1 << 64) - 1)]
            if cp['index']:
                native['knowledge_lost'] = [copy.deepcopy(self.loss)]
                native['chronicle_cursor'] = 1
            else:
                native['terminal_alive'] = 2
                native['terminal_agent_ids'] = fixtures.arr(native, 'initial_agent_ids')[:]
        g.plan = replace(g.plan, checkpoints=tuple(CheckpointPlan(text(Node(cp).field('label')), 0, 1, integer(Node(cp).field('clock').field('tick')),
                              json.dumps(cp['context']), cp['label']=='terminal') for cp in fixtures.checkpoints(g.data)),
                         instances=((InstancePlan('bootstrap', 7, 0, 40),),))
        l.data['checkpoints'] = [l.checkpoint(0, 'bootstrap', 0), l.checkpoint(1, 'loaded_initial', 30),
                                 l.checkpoint(2, 'loaded_terminal', 35)]
        fixtures.obj(l.data, 'checkpoints', 1)['context'] = loaded
        for cp in fixtures.checkpoints(l.data)[1:]:
            boot = fixtures.obj(fixtures.native(0), 'instances', 0)
            boot['closed'] = True
            native = fixtures.obj(cp, 'observation', 'instances', 0)
            native.update(instance_ordinal=1, reason='load', initial_tick='30', history_complete=False,
                          history_gap='initial_history_unauthenticated', chronicle_cursor=1,
                          knowledge_lost=[copy.deepcopy(self.loss)])
            fixtures.obj(cp, 'observation')['instances'] = [boot, native]
        fixtures.arr(l.data, 'checkpoints', 2, 'observation', 'instances', 1, 'knowledge_lost').append(
            {'tick': '35', 'node': 'food', 'node_index': 1, 'holder': None})
        fixtures.obj(l.data, 'checkpoints', 2, 'observation', 'instances', 1)['chronicle_cursor'] = 2
        l.plan = replace(l.plan, checkpoints=tuple(CheckpointPlan(text(Node(cp).field('label')), 0, len(fixtures.arr(cp, 'observation', 'instances')),
                             integer(Node(cp).field('clock').field('tick')), json.dumps(cp['context']), cp['label']=='loaded_terminal')
                             for cp in fixtures.checkpoints(l.data)),
                         instances=((InstancePlan('bootstrap', 7, 0, 0), InstancePlan('load', 7, 30, 35)),))
        self._configure(g, 'visibility.generator', 'mux2_visibility_saves.gd', ())
        self._configure(l, 'visibility.loader', 'mux2_visibility_test.gd', ((save,file_hash(save)), (manifest,file_hash(manifest))))

    def _configure(self, fixture: fixtures.UIReceiptTests, producer: str, script: str, external: tuple[tuple[Path, str], ...]) -> None:
        source = 'tools/v8/' + script
        path = fixture.cwd / source
        path.parent.mkdir(parents=True)
        _ = path.write_text('# synthetic producer')
        _ = (fixture.cwd / 'bridge.dylib').write_bytes(b'synthetic native product')
        inputs = fixture.spec.inputs + ((source,file_hash(path)), ('bridge.dylib',file_hash(fixture.cwd/'bridge.dylib')))
        environment = tuple((k, producer if k=='WORLDSIM_GATE_INVOCATION_ID' else v) for k,v in fixture.spec.environment)
        args = list(fixture.spec.argv)
        args[args.index('--script')+1] = 'res://'+source
        args[-1:] = [str(self.link.save.parent)] if producer=='visibility.generator' else ['--manifest', str(self.link.manifest)]
        fixture.data['argv'] = [v for v in args[4:args.index('--')]]
        fixture.data['user_args'] = [v for v in args[args.index('--')+1:]]
        fixture.process['argv'] = [v for v in args]
        fixture.spec = replace(fixture.spec, argv=tuple(args), inputs=inputs, external_inputs=external, environment=environment)
        fixture.plan = replace(fixture.plan, producer=producer, invocation_id=producer, script='res://'+source, process=fixture.spec)
        fixture.path = fixture.sidecars / (producer+'-123.json')
        fixture.data.update(producer=producer, invocation_id=producer, script='res://'+source, script_sha256=file_hash(path))
        fixture.process.update(environment=dict(fixtures.obj(fixture.process, 'environment'), WORLDSIM_GATE_INVOCATION_ID=producer),
                               expected_inputs=dict(inputs),before_inputs=dict(inputs),after_inputs=dict(inputs))
        for key in ('expected_external_inputs','before_external_inputs','after_external_inputs'):
            fixture.process[key] = {str(p):h for p,h in external}

    def evidence(self, fixture: fixtures.UIReceiptTests) -> UIEvidence:
        _ = fixture.path.write_text(json.dumps(fixture.data))
        _ = fixture.result.receipt.write_text(json.dumps(fixture.process))
        return UIEvidence(fixture.path, file_hash(fixture.path), fixture.plan, fixture.result, file_hash(fixture.result.receipt))

    def join(self) -> JoinedHistory:
        return join_visibility_history(self.evidence(self.generator), self.evidence(self.loader), self.link)

    def test_join_preserves_founders_loss_prefix_and_raw_uncertainty(self) -> None:
        result = self.join()
        self.assertEqual((result.observation.initial_tick,result.observation.terminal_tick,result.observation.founders), (0,35,2))
        self.assertEqual(len(result.observation.losses),2)
        self.assertFalse(fixtures.obj(self.loader.data, 'checkpoints', 1, 'observation', 'instances', 1)['history_complete'])
        self.assertEqual(document(result.evidence_json).field('save_sha256').value, self.link.save_sha256)

    def test_save_bytes_and_external_binding_required(self) -> None:
        _ = self.link.save.write_bytes(b'changed')
        with self.assertRaises(GateContractError): _ = self.join()

    def test_prefix_census_and_history_must_match(self) -> None:
        initial = fixtures.obj(self.loader.data, 'checkpoints', 1, 'observation', 'instances', 1)
        original = copy.deepcopy(initial)
        mutations: tuple[tuple[str, Json], ...] = (('initial_agent_ids',['8']), ('chronicle_cursor',2), ('knowledge_lost',[]),
                            ('history_gap','invalid_chronicle_tick_order'))
        for field, value in mutations:
            initial.clear();initial.update(copy.deepcopy(original));initial[field]=value
            with self.subTest(field=field), self.assertRaises(GateContractError): _ = self.join()

    def test_link_cannot_choose_foreign_checkpoint_or_hash(self) -> None:
        original = self.link
        for change in ({'link_checkpoint':3},{'save_sha256':'0'*64},{'scene_id':'other'},{'bridge_input':'absent'}):
            self.link = replace(original,**change)
            with self.subTest(change=change), self.assertRaises(GateContractError): _ = self.join()

    def test_native_node_index_prefix_must_match(self) -> None:
        fixtures.obj(self.loader.data, 'checkpoints', 1, 'observation', 'instances', 1, 'knowledge_lost', 0)['node_index']=99
        with self.assertRaises(GateContractError): _ = self.join()

    def test_external_save_input_cannot_be_omitted(self) -> None:
        self.loader.spec = replace(self.loader.spec, external_inputs=())
        self.loader.plan = replace(self.loader.plan, process=self.loader.spec)
        for key in ('expected_external_inputs','before_external_inputs','after_external_inputs'):
            self.loader.process[key] = {}
        with self.assertRaisesRegex(GateContractError,'external save/manifest'): _ = self.join()

    def test_crlf_sidecar_hash_binds_exact_bytes(self) -> None:
        generator=self.evidence(self.generator)
        loader=self.evidence(self.loader)
        content=json.dumps(self.generator.data,indent=2).replace('\n','\r\n').encode()
        _=generator.sidecar.write_bytes(content)
        generator=replace(generator,sidecar_sha256=file_hash(generator.sidecar))
        result=join_visibility_history(generator,loader,self.link)
        self.assertEqual(result.observation.founders,2)

    def test_fake_generator_out_flag_rejected(self) -> None:
        args=self.generator.spec.argv
        changed=args[:args.index('--')+1]+('--out',str(self.link.save.parent))
        self.generator.plan=replace(self.generator.plan,process=replace(self.generator.spec,argv=changed))
        with self.assertRaisesRegex(GateContractError,'positional'):
            _=self.join()

    def test_wrong_actual_producer_argument_rejected(self) -> None:
        self.generator.plan = replace(self.generator.plan, process=replace(self.generator.spec, argv=('/fake',)))
        with self.assertRaisesRegex(GateContractError,'process argument'): _ = self.join()

    def test_terminal_corruption_is_not_healed(self) -> None:
        fixtures.obj(self.loader.data, 'checkpoints', 2, 'observation', 'instances', 1)['history_gap']='chronicle_shrank'
        with self.assertRaises(GateContractError): _ = self.join()

    def test_save_link_state_must_equal_prefix(self) -> None:
        fixtures.obj(self.generator.data, 'checkpoints', 2, 'observation', 'instances', 0)['terminal_agent_ids']=['7']
        with self.assertRaisesRegex(GateContractError,'state changed'): _ = self.join()


if __name__ == '__main__': _ = unittest.main()
