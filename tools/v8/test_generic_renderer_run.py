"""Synthetic boundary tests only: these are not native renderer qualification."""
import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import generic_renderer_run as runner
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document, typed_json
from generic_process import (
    ProcessResult,
    ProcessSpec,
    environment_for,
    execute,
    file_hash,
)
from generic_ui_receipts import godot_visible_arguments
from mux2_gate_inputs import Json
from test_generic_observations import receipt as probe_receipt
from test_generic_ui_receipts import native, obj

ROOT = Path('/Users/rexxa/work/new-world-m6b-dev')


def write(path: Path, value: Json) -> None:
    _ = path.write_text(json.dumps(value))


def process_receipt(spec: ProcessSpec, pid: int = 42) -> ProcessResult:
    external: dict[str,Json] = {str(p):h for p,h in spec.external_inputs}
    receipt: dict[str, Json] = {'schema_version':1,'status':'PASS','native_exit':0,'pid':pid,
        'argv':list(spec.argv),'cwd':str(spec.cwd),'executable':str(spec.executable),
        'expected_executable_sha256':spec.executable_sha256,'input_root':str(spec.input_root),
        'fresh_directories':list(spec.fresh_directories),'output_root':str(spec.output_root),
        'environment':typed_json(environment_for(spec)),'expected_inputs':dict(spec.inputs),'before_inputs':dict(spec.inputs),
        'after_inputs':dict(spec.inputs),'expected_external_inputs':external,'before_external_inputs':external,
        'after_external_inputs':external,'before_executable_sha256':spec.executable_sha256,
        'after_executable_sha256':spec.executable_sha256,'errors':[],
        'environment_policy':'fixed_defaults_plus_explicit_only_no_inheritance','started_at':'synthetic',
        'ended_at':'synthetic','duration_ns':1,'timeout_seconds':600,'timed_out':False,
        'stdout_sha256':file_hash(spec.output_root/'stdout.log'),'stderr_sha256':file_hash(spec.output_root/'stderr.log')}
    write(spec.output_root/'process.json',receipt)
    return ProcessResult(spec.output_root/'process.json','PASS',0,pid)


def commands() -> list[Json]:
    return [{'tick':360,'cmd':name,'result':{'accepted':True}} for name in
            ('designate_chop_tiles','add_stockpile','place_blueprint')]


def ui_data(spec: ProcessSpec, command_path: Path, boot: int = 0) -> dict[str, Json]:
    expected = runner.plan(spec,boot,command_path)
    checkpoints: list[Json] = []
    for index, cp in enumerate(expected.checkpoints):
        observed = native(cp.tick if index < 2 else boot)
        initial = obj(observed,'instances',0)
        initial.update(seed='20261010',closed=index>=2,initial_population=10,terminal_alive=10,
                       initial_agent_ids=[str(i) for i in range(1,11)],terminal_agent_ids=[str(i) for i in range(1,11)])
        if index >= 2:
            replay = obj(native(cp.tick),'instances',0)
            replay.update(instance_ordinal=1,reason='reset',seed='100',initial_population=10,terminal_alive=10,
                          initial_agent_ids=[str(i) for i in range(1,11)],terminal_agent_ids=[str(i) for i in range(1,11)])
            observed['instances'] = [initial,replay]
        log: dict[str,Json] = {'total':1,'ticks':[0],'kinds':['SetSteward'],'accepted':[1]}
        if index >= 3:
            log.update(total=5,ticks=[0,0,360,360,360],kinds=['SetSteward','SetSteward','DesignateChopTiles','AddStockpile','PlaceBlueprint'],accepted=[1,1,1,1,1])
        checkpoints.append({'index':index,'label':cp.label,'node_instance_id':'42','observation':observed,
                            'tallies':{},'commands':log,'clock':{'tick':cp.tick},
                            'command_log_capability':'ticks_kinds_accepted_only','context':json.loads(cp.context_json)})
    engine,user = godot_visible_arguments(spec.argv)
    return {'schema_version':1,'producer':runner.PRODUCER,'invocation_id':expected.invocation_id,'pid':42,
            'argv':engine,'user_args':user,'project':str(spec.cwd),'script':runner.SCRIPT,
            'script_sha256':dict(spec.inputs)[runner.SCRIPT[6:]],'helper_sha256':dict(spec.inputs)[runner.HELPER[6:]],
            'complete':True,'completion':'producer_finish','native_exit_code':0,'classification':'native_world_producer',
            'no_native_reason':'','observed_native_nodes':1,'checkpoints':checkpoints,'errors':[]}


class RendererTests(unittest.TestCase):
    def exercise(self, defect: str = '') -> tuple[str, Json]:
        with TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            values: list[ProbeProduct] = []
            for arm in ('base','candidate'):
                directory = root/arm
                directory.mkdir()
                for name in runner.PINS:
                    path = directory/name
                    path.parent.mkdir(parents=True,exist_ok=True)
                    _ = path.write_bytes((ROOT/name).read_bytes())
                binary = directory/'synthetic-binary'
                _ = binary.write_text('synthetic only')
                values.append(ProbeProduct('a'*40,binary,file_hash(binary),directory,
                                          tuple((n,file_hash(directory/n)) for n in runner.PINS)))
            products = runner.RendererProducts(Products(*values),Products(*values))
            output = root/'output'
            calls: list[str] = []
            def child(spec: ProcessSpec) -> ProcessResult:
                spec.output_root.mkdir()
                kind = spec.output_root.name
                calls.append(kind)
                _ = (spec.output_root/'stdout.log').write_text('RENDERER_AGREEMENT_GODOT_DONE tick=900 people=1 settled=true\n' if kind == 'godot' else '')
                _ = (spec.output_root/'stderr.log').write_text('')
                if kind in {'generator','ascii'}:
                    duration = 1440 if kind == 'generator' else 900
                    data = probe_receipt()
                    data.update(seed=100,requested_ticks=duration,requested_terminal_tick=duration,terminal_tick=duration)
                    if defect == 'loss' and kind == 'ascii':
                        data['knowledge_lost'] = [{'tick':400,'node':'fire','holder':1}]
                    if defect == 'incomplete' and kind == 'ascii':
                        data['terminal_complete'] = False
                    (spec.output_root/'gate_observations').mkdir()
                    write(spec.output_root/'gate_observations/colony_s100_primary.json',data)
                    if kind == 'generator':
                        (spec.output_root/'commands').mkdir()
                        write(spec.output_root/'commands/colony_s100_commands.json',commands())
                    else:
                        write(spec.output_root/'text_people.json',{'tick':900,'people':[[1,2]],'appearance':[{'agent_id':1,'look':2,'variant':3}]})
                elif kind == 'godot':
                    command_path = spec.external_inputs[0][0]
                    data = ui_data(spec,command_path,1 if defect == 'bootstrap' and spec.output_root.parent.name == 'candidate' else 0)
                    if defect == 'missing_bootstrap':
                        obj(data,'checkpoints',0)['label'] = 'not_bootstrap'
                    if defect == 'incomplete_ui':
                        data['complete'] = False
                    if defect == 'wrong_pid':
                        data['pid'] = 43
                    if defect == 'settle_drift':
                        obj(data,'checkpoints',4,'clock')['tick'] = 901
                    write(spec.output_root/f'{runner.PRODUCER}-42.json',data)
                    write(spec.output_root/'godot_people.json',{'tick':900,'settled':True,'people':[[1,2]],'detail':[{'agent_id':1,'look':2,'variant':3}]})
                result = process_receipt(spec)
                if (defect == 'fake_receipt' and kind == 'ascii') or (defect == 'fake_oracle' and kind == 'oracle'):
                    data = document(result.receipt.read_text()).mapping()
                    data['after_inputs'] = {}
                    write(result.receipt,data)
                if defect == 'mutation' and spec.output_root.parent.name == 'candidate':
                    _ = (output/'base/generator/stdout.log').write_bytes(b'changed\r\n')
                if defect == 'raw_rewrite' and spec.output_root.parent.name == 'candidate':
                    path = output/'base/generator/gate_observations/colony_s100_primary.json'
                    _ = path.write_bytes(path.read_bytes()+b' ')
                return result
            with patch('generic_renderer_run.prepare'), patch('generic_renderer_run.execute',side_effect=child):
                result = runner.run_pair(products,output)
            receipt = document(result.receipt.read_text()).value
            if not defect:
                self.assertEqual(calls,['generator','ascii','godot','oracle']*2)
            return result.status,receipt

    def test_full_four_occurrences_each_arm(self) -> None:
        status,receipt = self.exercise()
        self.assertEqual(status,'PASS',receipt)
        self.assertEqual(len(document(json.dumps(receipt)).field('safety').array()),4)

    def test_incomplete_fake_missing_and_mutated_evidence_fail(self) -> None:
        for defect in ('incomplete','fake_receipt','missing_bootstrap','settle_drift','mutation','incomplete_ui','wrong_pid','fake_oracle','raw_rewrite','loss'):
            with self.subTest(defect=defect):
                status,receipt = self.exercise(defect)
                self.assertEqual(status,'FAIL',receipt)

    def test_unmatched_bootstrap_is_pending(self) -> None:
        status,receipt = self.exercise('bootstrap')
        self.assertEqual(status,'PENDING',receipt)

    def test_exact_workloads_and_native_observer_flags(self) -> None:
        product = ProbeProduct('a'*40,Path('/probe'),'b'*64,Path('/root'),())
        generator = runner.spec_for(product,'generator',Path('/out/generator'),Path('/commands'))
        ascii_spec = runner.spec_for(product,'ascii',Path('/out/ascii'),Path('/commands'))
        self.assertIn('--gate-observations',generator.argv)
        self.assertIn('--gate-observations',ascii_spec.argv)
        self.assertEqual(generator.argv[generator.argv.index('--days')+1],'1')
        self.assertEqual(ascii_spec.argv[ascii_spec.argv.index('--tick')+1],'900')

    def test_domain_oracle_is_exact_original_body(self) -> None:
        product = ProbeProduct('a'*40,Path('/probe'),'b'*64,ROOT,())
        source = runner.oracle_source(product)
        original = (ROOT/'tools/v8/renderer_agreement.sh').read_text()
        self.assertEqual(source,original.split("<<'PY'\n",1)[1][:-3])
        self.assertIn('appearance_bad',source)
        self.assertIn('collections.Counter',source)

    def test_original_python_oracle_runs_and_rejects_domain_failures(self) -> None:
        product = ProbeProduct('a'*40,Path('/probe'),'b'*64,ROOT,())
        source = runner.oracle_source(product)
        python = Path(sys.executable).resolve(strict=True)
        with TemporaryDirectory() as temporary:
            directory = Path(temporary).resolve()
            left = directory/'text.json'
            right = directory/'godot.json'
            write(left,{'tick':900,'people':[[1,2]],'appearance':[{'agent_id':1,'look':2,'variant':3}]})
            for index,defect in enumerate(('', 'tiles','appearance','settling','empty')):
                data: dict[str,Json] = {'tick':900,'settled':defect!='settling','people':[[1,2]],
                                       'detail':[{'agent_id':1,'look':2,'variant':3}]}
                if defect == 'tiles':
                    data['people'] = [[9,9]]
                if defect == 'appearance':
                    data['detail'] = []
                if defect == 'empty':
                    write(left,{'tick':900,'people':[],'appearance':[]})
                    data['people'] = []
                write(right,data)
                spec = ProcessSpec((str(python),'-c',source,str(left),str(right),'900'),directory,
                                   python,file_hash(python),directory,
                                   (('text.json',file_hash(left)),('godot.json',file_hash(right))),directory/str(index),timeout_seconds=600)
                result = execute(spec)
                self.assertEqual(result.status,'FAIL' if defect else 'PASS')
                self.assertIn('RENDERER_AGREEMENT', (result.receipt.parent/'stdout.log').read_text())

    def test_prepare_rejects_foreign_inputs_before_execution(self) -> None:
        product = ProbeProduct('a'*40,Path('/probe'),'b'*64,ROOT,())
        with self.assertRaises(GateContractError):
            runner.prepare(runner.RendererProducts(Products(product,product),Products(product,product)),Path('/fresh-output'))


if __name__ == '__main__':
    _ = unittest.main()
