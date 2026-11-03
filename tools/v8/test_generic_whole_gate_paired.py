"""Offline receipt closure tests; no product qualification."""
import json
import tempfile
import unittest
from pathlib import Path

from generic_fixture_registry import Invocation
from generic_gate import GateContractError, document
from generic_process import file_hash
from mux2_gate_inputs import Json

import generic_whole_gate_paired as subject


class ClosureTests(unittest.TestCase):
    def test_original_namespace_and_modes_survive_reading(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            component = root / 'component'
            component.mkdir()
            evidence = component / 'receipt.json'
            _ = evidence.write_text('{}')
            acceptance = root / 'acceptance.json'
            _ = acceptance.write_text('{}')
            raw = {'original_artifacts': {str(evidence): file_hash(evidence)},
                   'original_modes': {str(p): p.lstat().st_mode for p in (root, component, evidence)}}
            closed = subject.closure(document(json.dumps(raw)), acceptance)
            subject.recheck(closed)
            _ = (component / 'unbound.json').write_text('{}')
            with self.assertRaisesRegex(GateContractError, 'namespace'):
                subject.recheck(closed)

    def test_file_mode_and_byte_mutations_are_not_recaptured(self) -> None:
        for mode in (False, True):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp).resolve()
                evidence = root / 'receipt.json'
                _ = evidence.write_text('{}')
                raw = {'original_artifacts': {str(evidence): file_hash(evidence)},
                       'original_modes': {str(p): p.lstat().st_mode for p in (root, evidence)}}
                closed = subject.closure(document(json.dumps(raw)), root / 'acceptance.json')
                if mode:
                    evidence.chmod(0o600)
                else:
                    _ = evidence.write_text('{"foreign":true}')
                with self.assertRaises(GateContractError):
                    subject.recheck(closed)


class PairedTests(unittest.TestCase):
    def test_real_embedded_arm_paths_need_no_fabricated_base_stage(self) -> None:
        from generic_direct_run import partition, step_fixtures
        from generic_fixture_registry import ExpansionContext, expand, load_registry
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            registry = load_registry(Path(__file__).parent / 'gate_fixtures.json')
            acceptance = root / 'steps/candidate/44-core_shadow/acceptance/acceptance.json'
            folder = acceptance.parent / 'component'
            folder.mkdir(parents=True)
            arms: dict[str, dict[str, str]] = {}
            products: dict[str, dict[str, str]] = {}
            plans: list[Invocation] = []
            for arm in ('base', 'candidate'):
                source = root / arm
                source.mkdir()
                executable = source / 'probe'
                _ = executable.write_text('TEST_FIXTURE_ONLY')
                arms[arm] = {'root':str(source), 'source_sha':'a'*40 if arm=='base' else 'b'*40}
                products[arm] = {'root':str(source),'executable':str(executable),'executable_sha256':file_hash(executable)}
                plans.extend(p for p in expand(registry, ExpansionContext(source,source,folder/'runs',executable,executable))
                             if p.arm==arm and p.fixture_id in step_fixtures(44))
            components: list[Json] = []
            for name, ids in partition(registry).items():
                selected = [p for p in plans if p.fixture_id in ids]
                if not selected:
                    continue
                invocations: list[Json] = []
                for p in selected:
                    p.out.mkdir(parents=True)
                    for stream in ('stdout','stderr'):
                        _ = (p.out/(stream+'.log')).write_text('')
                    product = products[p.arm]
                    native = {'status':'PASS','native_exit':0,'errors':[],'pid':123,
                        'cwd':str(p.cwd),'input_root':str(p.cwd),'output_root':str(p.out),
                        'argv':list(p.argv or ()), 'executable':product['executable'],
                        'environment':{'WORLDSIM_GATE_OBSERVER':'1','WORLDSIM_GIT_SHA':arms[p.arm]['source_sha']},
                        **{k:product['executable_sha256'] for k in ('expected_executable_sha256','before_executable_sha256','after_executable_sha256')},
                        **{stream+'_sha256':file_hash(p.out/(stream+'.log')) for stream in ('stdout','stderr')}}
                    _ = (p.out/'process.json').write_text(json.dumps(native))
                    invocations.append({'id':p.id,'fixture':p.fixture_id,'arm':p.arm,'repeat':p.repeat,
                        'seeds':list(p.seeds),'roles':list(p.roles),'cwd':str(p.cwd),'out':str(p.out),'argv':list(p.argv or ())})
                path = folder/(name+'.json')
                _ = path.write_text(json.dumps({'status':'PASS','errors':[]}))
                components.append({'component':name,'status':'PASS','receipt':str(path),'receipt_sha256':file_hash(path),'invocations':invocations})
            receipt = folder/'receipt.json'
            _ = receipt.write_text(json.dumps({'schema':'direct_step_components_v1','step':44,'status':'PASS','errors':[],
                'registry_sha256':registry.sha256,'fixtures':list(step_fixtures(44)),
                'base':arms['base']['source_sha'],'candidate':arms['candidate']['source_sha'],'products':products,'components':components}))
            files = {str(p):file_hash(p) for p in acceptance.parent.rglob('*') if p.is_file()}
            modes = {str(p):p.lstat().st_mode for p in (acceptance.parent,*acceptance.parent.rglob('*'))}
            _ = acceptance.write_text(json.dumps({'schema':'whole_gate_acceptance.v1','step':44,'status':'COMPONENT_PASS','component_status':'PASS',
                'component_receipt':str(receipt),'component_receipt_sha256':file_hash(receipt),
                'base':arms['base']['source_sha'],'candidate':arms['candidate']['source_sha'],
                'original_artifacts':files,'original_modes':modes}))
            session = document(json.dumps({'arms':arms}))
            stage = document(json.dumps({'step':44,'arm':'candidate','state':'RETURNED','native_exit':0,
                'process_receipt':str(acceptance.parent.parent/'process/process.json'),
                'observer_artifacts':{str(acceptance):file_hash(acceptance)}}))
            result = subject.authenticate(session,stage,registry)
            self.assertEqual({p.arm for p in result.plans},{'base','candidate'})
            self.assertEqual(len(result.plans),len(plans))
            _ = (plans[0].out/'process.json').write_text('{}')
            with self.assertRaisesRegex(GateContractError,'bytes changed'):
                _ = subject.authenticate(session,stage,registry)
