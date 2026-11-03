"""Tiny actual Python/Bash/Git fixtures only; original Godot/Cargo qualification NOT_RUN."""
import json
import os
import subprocess
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from typing import cast
from unittest.mock import patch

import gate_oracle_child
from generic_gate import GateContractError
from generic_play_main_projection import prepare_main_projection
from generic_play_sources import HistoricalBindings, HistoricalSource, SourceGrants
from generic_process import atomic_json, file_hash
from generic_sources import freeze_source
from generic_whole_gate_play import run
from generic_whole_gate_play_environment import validate_environment
from generic_whole_gate_play_join import collect_play
from generic_whole_gate_play_plan import build_plan
from generic_whole_gate_play_sources import next_admin, object_manifest
from mux2_gate_inputs import Node
from mux2_gate_lock import FD_ENV
from test_generic_play_owner import OwnerTests


class WholePlayTests(unittest.TestCase):
    def test_git_objects_match_actual_checkout_and_admin_collision(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();root=base/'repo';root.mkdir()
            def git(*args: str) -> str:
                return subprocess.run(['git','-C',str(root),*args],check=True,capture_output=True,text=True).stdout.strip()
            _=git('init');_=git('config','user.name','Fixture');_=git('config','user.email','fixture@invalid')
            _=(root/'x').write_text('actual original blob\n');_=git('add','x');_=git('commit','-m','fixture')
            commit=git('rev-parse','HEAD')
            manifest,_=object_manifest(root,commit)
            self.assertEqual(manifest,freeze_source(root,commit)['tracked_manifest'])
            common=root/'.git'
            for prefix in ('first','second'):
                wanted=next_admin(common,'main');target=base/prefix/'main'
                _=git('worktree','add','--detach',str(target),commit)
                actual=subprocess.run(['git','-C',str(target),'rev-parse','--absolute-git-dir'],check=True,capture_output=True,text=True).stdout.strip()
                self.assertEqual(wanted,Path(actual))

    def test_actual_bash_bookkeeping_and_semantic_override_rejection(self) -> None:
        with TemporaryDirectory() as folder:
            root=Path(folder).resolve();python=str(Path(sys.executable).resolve())
            env={'PATH':str(Path(python).parent)+':/usr/bin:/bin','HOME':str(root),'LANG':'C','LC_ALL':'C',
                 'WORLDSIM_GATE_PLAY_REQUEST_FD':'3','WORLDSIM_GATE_PLAY_CONTROL_FD':'4',
                 'WORLDSIM_GATE_PLAY_OWNER_OUT':str(root/'owner'),FD_ENV:'5'}
            if '__CF_USER_TEXT_ENCODING' in os.environ:env['__CF_USER_TEXT_ENCODING']=os.environ['__CF_USER_TEXT_ENCODING']
            child=subprocess.run(['/bin/bash','-c',"python3 -c 'import os,json;print(json.dumps(dict(os.environ)))'"],
                cwd=root,env=env,check=True,capture_output=True,text=True)
            actual=Node(json.loads(child.stdout));expected={key:value for key,value in env.items() if not key.startswith('WORLDSIM_')}
            validate_environment(actual,expected,root,root,root/'play')
            foreign=dict(actual.mapping());foreign['CARGO_HOME']='/foreign'
            with self.assertRaises(GateContractError):validate_environment(Node(foreign),expected,root,root,root/'play')
            foreign=dict(actual.mapping());foreign['PWD']='/foreign'
            with self.assertRaises(GateContractError):validate_environment(Node(foreign),expected,root,root,root/'play')

    def test_source_owned_plan_has_exact_cases_children_and_fresh_roots(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();root=base/'root';(root/'tools/v8').mkdir(parents=True)
            observer=root/'tools/v8/gate_legacy_observer.gd';_=observer.write_text('fixture observer')
            # Synthetic H files exercise the real launcher check; session admission stays mocked.
            harness=base/'harness';(harness/'tools').mkdir(parents=True)
            launcher=root/'tools/play.sh';_=launcher.write_text('#!/bin/bash\nexit 0\n');launcher.chmod(0o755)
            selected=harness/'tools/play.sh';_=selected.write_bytes(launcher.read_bytes());selected.chmod(0o755)
            python=Path(sys.executable).resolve();tool=(python,file_hash(python));session_file=base/'session.json';_=session_file.write_text('{}')
            session=Node({'output':str(base/'out'),'environment':{'HOME':str(base)},'arms':{'candidate':{'root':str(root),
                'source_inputs':{'tools/v8/gate_legacy_observer.gd':file_hash(observer),'tools/play.sh':file_hash(launcher)}}},
                'harness':{'root':str(harness),'inputs':{'tools/play.sh':file_hash(selected)}},
                'external_inputs':{str(python):tool[1]},'python':{'path':str(python)}})
            env={'PATH':str(python.parent)+':/bin:/usr/bin','HOME':str(base),'GODOT_BIN':str(python),'CARGO_TARGET_DIR':str(root/'rust/target')}
            sources=tuple(HistoricalSource(v,'a'*40,'b'*64,'c'*64) for v in ('main','m1','v7'))
            history=HistoricalBindings(sources,tool,tool,tool,())
            target=base/'out/steps/candidate/21-play_check/play-owned'
            with patch('generic_whole_gate_play_plan.load_session',return_value=session),patch('generic_whole_gate_play_plan.process_spec',return_value=SimpleNamespace(environment=tuple(env.items()))),patch('generic_whole_gate_play_plan.bindings',return_value=(history,{v:('cargo','build','-p','sim-bridge') for v in ('main','m1','v7')})):
                plan=build_plan(session_file,file_hash(session_file),'candidate',target)
            suite=json.loads(plan.suite.read_text())['cases']
            self.assertEqual(len(suite),6);self.assertEqual(sum(value is None for value in suite.values()),2)
            roles=[role for value in suite.values() if value is not None for role in json.loads(Path(value['path']).read_text())['roles']]
            self.assertEqual(len(roles),16);self.assertEqual(roles.count('version'),4)
            self.assertEqual(dict(plan.environment)['CARGO_TARGET_DIR'],'target')
            for _,role,values in plan.child_environments:
                if role=='build':self.assertEqual(dict(values)['CARGO_TARGET_DIR'],'target')
            for value in suite.values():
                if value is None:continue
                child_roles=json.loads(Path(value['path']).read_text())['roles']
                if 'build' not in child_roles:continue
                build=child_roles['build'];resolved=Path(build['cwd'])/dict(plan.environment)['CARGO_TARGET_DIR']
                self.assertFalse(resolved.is_relative_to(root))
                self.assertEqual(Path(build['artifacts'][0]).parent.parent,resolved)
            self.assertEqual(dict(plan.environment)['PLAY_ROOT'],str(base/'out/play-workspaces/candidate/play-roots'))
            self.assertFalse((base/'out/play-workspaces/candidate/play-roots').exists())
            self.assertIn((target/'plan.json',file_hash(target/'plan.json')),plan.inputs)

    def test_main_export_preserves_original_and_distinguishes_actual_observer_bridge(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();original=base/'play/main';original.mkdir(parents=True)
            root=base/'base-o';(root/'tools/v8').mkdir(parents=True)
            helper=root/'tools/v8/gate_ui_observer.gd';_=helper.write_text('observer fixture')
            project=root/'project.godot';_=project.write_text('fixture project')
            original_binary=original/'bridge';_=original_binary.write_bytes(b'original bridge')
            observed=base/'observer-bridge';_=observed.write_bytes(b'observer bridge')
            receipt=base/'actual-build-receipt';_=receipt.write_text('actual receipt stand-in')
            joined=base/'join.json';atomic_json(joined,{'build_receipt':str(receipt),'build_receipt_sha256':file_hash(receipt),'build_pid':123})
            source=HistoricalSource('main','a'*40,'b'*64,'c'*64)
            snapshot={'source':'original fixture'}
            grants=cast(SourceGrants,SimpleNamespace(source=lambda _:source,check=lambda:None,check_built=lambda _:None,
                built={'main':SimpleNamespace(path=original_binary,sha256=file_hash(original_binary),receipt=receipt,receipt_sha256=file_hash(receipt),pid=123)},
                play_root=base/'play',started={'main':snapshot}))
            session=Node({'arms':{'base':{'source_sha':source.commit,'root':str(root),'effective_source':{'manifest':{}},
                'source_inputs':{'project.godot':file_hash(project),'tools/v8/gate_ui_observer.gd':file_hash(helper)}}}})
            product=Node({'executable':str(observed),'executable_sha256':file_hash(observed),'build_receipt':str(receipt),'build_receipt_sha256':file_hash(receipt)})
            with patch('generic_play_main_projection.load_session',return_value=session),patch('generic_play_main_projection.freeze_source',return_value=snapshot),patch('generic_play_main_projection.current_product_inputs',return_value=product),patch('generic_play_main_projection.validate_play_build',return_value=file_hash(original_binary)):
                projection=prepare_main_projection(grants,joined,file_hash(joined),base/'session','fixture',base/'export')
            provenance=json.loads((base/'export/gate_observer/main-provenance.json').read_text())
            self.assertNotEqual(provenance['original_bridge_sha256'],provenance['observer_bridge_sha256'])
            self.assertEqual(original_binary.read_bytes(),b'original bridge');projection.recheck()
            _=(base/'export/project.godot').write_text('drift')
            with self.assertRaises(GateContractError):projection.recheck()

    def test_collector_rejects_original_unowned_launcher_as_new_owner(self) -> None:
        session=Node({'output':'/owned/session'})
        stage=Node({'step':21,'name':'play_check','arm':'candidate','argv':['bash','tools/v8/test_play.sh']})
        with patch('generic_whole_gate_play_join.bind_outer',return_value=(Path('/source'),Node({}))), self.assertRaises(GateContractError):
            collect_play(session,stage)

    def test_wrapper_retains_actual_six_case_waits_and_pending_qualification(self) -> None:
        with TemporaryDirectory() as folder:
            base=Path(folder).resolve();plan=OwnerTests().fixture(base)
            atomic_json(base/'plan.json',{'fixture':'source plan stand-in; no native qualification'})
            plan=replace(plan,inputs=(*plan.inputs,(base/'plan.json',file_hash(base/'plan.json'))))
            with (base/'lane').open('w+b') as lane, patch.object(gate_oracle_child,'LOCK',base/'lane'), patch('generic_whole_gate_play.build_plan',return_value=plan),patch('generic_whole_gate_play.load_session',return_value=Node({})):
                code=run(base/'session','fixture-session','candidate',base,lane.fileno())
            result=json.loads((base/'play-result.json').read_text())
            self.assertEqual(code,0);self.assertEqual(result['original_case_exits'],[3,2,4,0,0,0])
            self.assertEqual(result['qualification'],'INCOMPLETE')
            self.assertEqual(len(list(base.glob('native-*/receipt.json'))),16)
            self.assertEqual(file_hash(plan.output/'terminal.json'),result['owner_terminal_sha256'])


if __name__=='__main__':unittest.main()
