"""Synthetic load graphs and actual harmless aggregate receipts; no native qualification."""
import json
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

import generic_process
from generic_gate import GateContractError
from generic_process import ProcessSpec,execute,environment_for,file_hash
from generic_test_endpoint import Endpoint
from generic_test_endpoint_types import Snapshot,Loss,PendingReason
from generic_test_history_graph import FORMAT_SOURCE
from generic_whole_gate_migrations import V10,SOURCE_PINS,STRICT_READER_PINS,collect_migration,proposed_window
from mux2_gate_inputs import Json,Node
from test_generic_test_history import CHILD


def endpoint(losses:tuple[Loss,...]=()) -> Endpoint:
    start=Snapshot(100,(1,2),(1,2),True,losses,True,len(losses),(),0,tuple((name,0) for name in ('starvation','dehydration','exposure','combat','old_age')))
    return Endpoint(start,replace(start,tick=160),None,False,(PendingReason.EXTERNAL_ORIGIN_UNRESOLVED,),'0'*64,'60_ticks')


class MigrationTests(unittest.TestCase):
    def test_proposal_keeps_inherited_loss_unknown_founders_and_nonadoption(self) -> None:
        before=endpoint((Loss(5,'Fire',None),));after=replace(before,terminal=replace(before.terminal,losses=(*before.terminal.losses,Loss(120,'Fire',1))))
        row=proposed_window(before,before)
        self.assertEqual(row['status'],'PASS');self.assertFalse(row['strict_ordinary_admission'])
        self.assertEqual(row['founding_census'],'UNKNOWN_EXTERNAL_ORIGIN')
        self.assertEqual(row['candidate_inherited_losses'],[[5,'Fire',None]])
        self.assertEqual(proposed_window(before,after)['status'],'FAIL')
        death=replace(before,terminal=replace(before.terminal,alive_ids=(1,)))
        self.assertEqual(proposed_window(before,death)['status'],'FAIL')

    def test_actual_aggregate_retains_all_four_external_files_and_eight_loads(self) -> None:
        original=Path(generic_process.__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();h=root/'h';(h/'tools/v8').mkdir(parents=True)
            python=Path(sys.executable).resolve();bash=Path('/bin/bash');gate=h/'tools/v8/gate.sh';gate.write_text(f'step 37 migration {python} driver.py\n')
            product=root/'product';product.mkdir();inputs:dict[str,str]={}
            for name in (*(SOURCE_PINS | STRICT_READER_PINS),FORMAT_SOURCE):
                path=product/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes((original/name).read_bytes());inputs[name]=file_hash(path)
            fixtures=root/'fixtures';fixtures.mkdir();external:dict[str,str]={};manifest:dict[str,str]={};rows:list[dict[str,Json]]=[]
            for case in range(4):
                path=fixtures/f'{case}.save';path.write_bytes(b'WORLDSIM'+(10).to_bytes(4,'little')+bytes([case]));external[str(path)]=file_hash(path);manifest[path.name]=file_hash(path)
                for branch in ('migrated','reloaded'):
                    row:dict[str,Json]={'test':V10,'seed':case,'branch':f'fixture_{case}_{branch}','start':100+case,'end':160+case,'external_origin':True}
                    if branch=='migrated':row['external']=True
                    else:row.update(parent=f'fixture_{case}_migrated',label='current')
                    rows.append(row)
            manifest_path=root/'manifest.json';manifest_path.write_text(json.dumps(manifest));external[str(manifest_path)]=file_hash(manifest_path)
            (product/'plan.json').write_text(json.dumps(rows))
            child=CHILD[:CHILD.index("for test in sorted({r['test'] for r in rows})")].replace("'synthetic planned boundary'","'60_ticks'")
            child+='\nfor case in range(4):\n d=parent/('+repr(V10)+"+f'_s{case}_fixture_{case}_migrated')\n path=pathlib.Path("+repr(str(fixtures))+"+f'/{case}.save')\n artifact=d/'save_external_input.wsave';raw=path.read_bytes();artifact.write_bytes(raw)\n checkpoint=d/'save_external_input.json';data=json.loads(checkpoint.read_text());data.update(byte_count=len(raw),input_version_le_bytes=list(raw[8:12]));checkpoint.write_text(json.dumps(data))\n (d/'checkpoint_source.json').write_text(json.dumps(dict(state=state(100+case),detail=dict(path=str(path),version=10))))\n"
            child+="print('test "+V10+" ... ok')\nprint('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s')\n"
            (product/'child.py').write_text(child);(product/'driver.py').write_text("import subprocess,sys\nprint('Running tests/steward_defence_migration.rs (synthetic)',flush=True)\nprint('running 1 test',flush=True)\nsubprocess.run([sys.executable,'child.py'],check=True)\n")
            for name in ('plan.json','child.py','driver.py'):inputs[name]=file_hash(product/name)
            output=root/'output';step=output/'base'/'37';observer=step/'native-worlds';observer.mkdir(parents=True)
            argv=(str(python),'driver.py');spec=ProcessSpec((str(bash),'-c','exec "$@" 2>&1','whole-step',*argv),product,bash,file_hash(bash),product,tuple(sorted(inputs.items())),step/'process',(('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GATE_TEST_OUT',str(observer)),('MUX2_V10_FIXTURES',str(fixtures)),('MUX2_V10_MANIFEST',str(manifest_path))))
            actual=execute(spec);self.assertEqual(actual.native_exit,0)
            stage=Node({'step':37,'arm':'base','state':'RETURNED','native_exit':0,'pid':actual.pid,'argv':list(argv),'cwd':str(product),'environment':environment_for(spec),'process_receipt':str(actual.receipt),'process_receipt_sha256':file_hash(actual.receipt),'observer_artifacts':{str(p):file_hash(p) for p in observer.rglob('*') if p.is_file()},'observer_directories':{str(p):p.stat().st_mode for p in (observer,*observer.rglob('*')) if p.is_dir()}})
            session=Node({'output':str(output),'harness':{'root':str(h),'inputs':{'tools/v8/gate.sh':file_hash(gate)}},'bash':{'path':str(bash),'sha256':file_hash(bash)},'arms':{'base':{'root':str(product),'source_inputs':inputs}},'external_inputs':external})
            result=collect_migration(session,stage,'steward_defence_migration')
            self.assertEqual(len(result.endpoints),8);self.assertEqual(len(result.external),5)
            self.assertTrue(all(row.founding is None and not row.history_complete for row in result.histories))
            self.assertNotEqual(result.pid,actual.pid)
            (fixtures/'0.save').write_bytes(b'wrong')
            with self.assertRaises(GateContractError):collect_migration(session,stage,'steward_defence_migration')

if __name__=='__main__':unittest.main()

class LegacyFixtureTests(unittest.TestCase):
    def test_legacy_nine_file_plan_and_b5_mechanism(self) -> None:
        # Replace only external fixture pins in this synthetic test; production pins stay frozen.
        from unittest.mock import patch
        from generic_whole_gate_migrations import LEGACY, B5, LEGACY_ROWS
        original=Path(generic_process.__file__).resolve().parents[2]
        for target,test in (('legacy_migration_current',LEGACY),('b5_flora',B5)):
            with self.subTest(target=target),tempfile.TemporaryDirectory() as temporary:
                root=Path(temporary).resolve();product=root/'product';product.mkdir();inputs:dict[str,str]={}
                for name in (*(SOURCE_PINS | STRICT_READER_PINS),FORMAT_SOURCE):
                    path=product/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes((original/name).read_bytes());inputs[name]=file_hash(path)
                fixtures=root/'fixtures';fixtures.mkdir();external:dict[str,str]={};source_rows:list[tuple[str,str,int,int,int]]=[];source_bytes:dict[str,str]={}
                for index,(relative,_,version,tick,seed) in enumerate(LEGACY_ROWS):
                    path=fixtures/relative;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'WORLDSIM'+version.to_bytes(4,'little')+bytes([index]));external[str(path)]=file_hash(path);source_rows.append((relative,file_hash(path),version,tick,seed))
                    source_bytes[str(index)]=str(path)
                    if version==6 and tick==2000:
                        commands=path.parent/'continuation-2000-3000.json';commands.write_text('[{"tick":2001},{"tick":2002}]');external[str(commands)]=file_hash(commands)
                rows:list[dict[str,Json]]=[]
                if target=='legacy_migration_current':
                    for case,(_,_,version,tick,seed) in enumerate(source_rows):
                        for branch in ('initial','fresh','twin'):
                            duration=0 if branch=='initial' else (24 if version==9 else 1000)
                            row:dict[str,Json]={'test':test,'seed':seed,'branch':f'fixture_{case}_{branch}','start':tick,'end':tick+duration,'external_origin':True,'stop':'load_and_resave_only' if branch=='initial' else 'current_policy_duration_reached'}
                            if branch=='twin':row.update(parent=f'fixture_{case}_initial',label='current')
                            else:row['external']=True
                            rows.append(row)
                else:
                    for branch,start,end,stop in (('migrated',4000,4024,'24_ticks_before_chop_mechanism'),('twin',4000,4024,'24_ticks'),('post_chop_mechanism_load',4050,4050,'post_chop_load_only')):
                        row={'test':test,'seed':0,'branch':branch,'start':start,'end':end,'external_origin':True,'stop':stop}
                        if branch=='twin':row.update(parent='migrated',label='current')
                        else:row['external']=True
                        rows.append(row)
                (product/'plan.json').write_text(json.dumps(rows))
                child=CHILD[:CHILD.index("for test in sorted({r['test'] for r in rows})")].replace("'synthetic planned boundary'","r['stop']")
                child+='\nTARGET='+repr(target)+'\nTEST='+repr(test)+'\nSOURCES='+repr(source_rows)+'\nFIXTURES=pathlib.Path('+repr(str(fixtures))+')\n'
                child+=r'''
def replace_external(folder,path):
 raw=path.read_bytes();(folder/'save_external_input.wsave').write_bytes(raw)
 c=folder/'save_external_input.json';d=json.loads(c.read_text());d.update(byte_count=len(raw),input_version_le_bytes=list(raw[8:12]));c.write_text(json.dumps(d))
if TARGET=='legacy_migration_current':
 for case,(relative,digest,version,tick,seed) in enumerate(SOURCES):
  path=FIXTURES/relative
  for branch in ('initial','fresh'):
   folder=parent/f'{TEST}_s{seed}_fixture_{case}_{branch}';replace_external(folder,path)
   if branch=='initial':(folder/'checkpoint_source.json').write_text(json.dumps(dict(state=state(tick),detail=dict(path=str(path),sha256=digest,version=version))))
   if branch=='fresh' and version==6 and tick==2000:(folder/'input_captured_bills.bin').write_bytes((path.parent/'continuation-2000-3000.json').read_bytes())
else:
 replace_external(parent/f'{TEST}_s0_migrated',FIXTURES/SOURCES[-1][0])
 folder=parent/f'{TEST}_s0_post_chop_mechanism_load'
 (folder/'checkpoint_classification.json').write_text(json.dumps(dict(state=state(4050),detail=dict(classification='mechanism_load_only',source='same test migrated branch after explicit chop setup; not ordinary continuation',interventions='cancel active jobs; M3 rules; move interval 1; positioned workers; stabilized needs; chop command'))))
'''
                child+=f"\nprint('test {test} ... ok')\nprint('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s')\n"
                (product/'child.py').write_text(child);(product/'driver.py').write_text(f"import subprocess,sys\nprint('Running tests/{target}.rs (synthetic)',flush=True)\nprint('running 1 test',flush=True)\nsubprocess.run([sys.executable,'child.py'],check=True)\n")
                for name in ('plan.json','child.py','driver.py'):inputs[name]=file_hash(product/name)
                python=Path(sys.executable).resolve();bash=Path('/bin/bash');h=root/'h';(h/'tools/v8').mkdir(parents=True);gate=h/'tools/v8/gate.sh';gate.write_text(f'step 37 migration {python} driver.py\n')
                output=root/'output';step=output/'base'/'37';observer=step/'native-worlds';observer.mkdir(parents=True);argv=(str(python),'driver.py')
                spec=ProcessSpec((str(bash),'-c','exec "$@" 2>&1','whole-step',*argv),product,bash,file_hash(bash),product,tuple(sorted(inputs.items())),step/'process',(('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GATE_TEST_OUT',str(observer)),('WORLDSIM_MUX_FIXTURES',str(fixtures)),('WORLDSIM_B5_LEGACY9',str(fixtures/source_rows[-1][0]))))
                actual=execute(spec);self.assertEqual(actual.native_exit,0)
                stage=Node({'step':37,'arm':'base','state':'RETURNED','native_exit':0,'pid':actual.pid,'argv':list(argv),'cwd':str(product),'environment':environment_for(spec),'process_receipt':str(actual.receipt),'process_receipt_sha256':file_hash(actual.receipt),'observer_artifacts':{str(p):file_hash(p) for p in observer.rglob('*') if p.is_file()},'observer_directories':{str(p):p.stat().st_mode for p in (observer,*observer.rglob('*')) if p.is_dir()}})
                session=Node({'output':str(output),'harness':{'root':str(h),'inputs':{'tools/v8/gate.sh':file_hash(gate)}},'bash':{'path':str(bash),'sha256':file_hash(bash)},'arms':{'base':{'root':str(product),'source_inputs':inputs}},'external_inputs':external})
                with patch('generic_whole_gate_migrations.LEGACY_ROWS',tuple(source_rows)),patch('generic_whole_gate_migrations.LEGACY_PROVENANCE',{}):
                    result=collect_migration(session,stage,target)
                    self.assertEqual(len(result.endpoints),27 if target=='legacy_migration_current' else 3)
                    self.assertTrue(all(not row.history_complete and row.founding is None for row in result.histories))
