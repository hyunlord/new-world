"""Real harmless child processes emit synthetic gate37 graph/outcome fixtures."""
import json
import sys
import tempfile
import unittest
from pathlib import Path

import generic_process
from generic_gate import GateContractError
from generic_process import ProcessSpec, environment_for, execute, file_hash
from generic_test_history_graph import FORMAT_SOURCE
from generic_test_steward_plans import STOCK,CANCEL,REPLAY,PRODUCTION,HOUSING,DEFENCE,IGNORED,SOURCE_PINS,TESTS,EXTRA_SOURCE
from generic_whole_gate_native import evaluate_family
from mux2_gate_inputs import Json,Node
from test_generic_test_history import CHILD


def rows(target:str) -> list[dict[str,Json]]:
    result:list[dict[str,Json]]=[]
    def add(test:str,seed:int,branch:str,start:int,end:int,stop:str,parent:str='',label:str='') -> None:
        row:dict[str,Json]={'test':test,'seed':seed,'branch':branch,'start':start,'end':end,'stop':stop}
        if parent: row.update(parent=parent,label=label)
        result.append(row)
    if target=='steward_defence':
        for branch in ('original','replay'):add(DEFENCE,0,branch,0,721,'initial_tick_plus_721')
        return result
    add(STOCK,0,'original',0,1,'native_assertions_reached')
    add(CANCEL,19,'original',0,721,'native_assertions_reached')
    add(CANCEL,19,'loaded',361,721,'360_ticks_after_load','original','continuation')
    add(REPLAY,29,'original',0,720,'native_assertions_reached')
    add(REPLAY,29,'replay',0,720,'720_ticks')
    add(PRODUCTION,0,'original',0,360,'native_assertions_reached')
    for test in HOUSING:
        add(test,29,'original',0,3440,'first_housed_plus_1440')
        add(test,29,'replay',2000,3440,'original_completion_tick','original','first_housed')
        add(test,29,'food',2000,3440,'scope_or_cooldown_assertions_reached','original','first_housed')
        add(test,29,'cooldown',2000,3800,'scope_or_cooldown_assertions_reached','original','first_housed')
        for n in range(4):
            add(test,29,f'manual_{n}',2000,2000,'load_only_manual_intent','original','first_housed')
            add(test,29,f'restored_{n}',2000,3440,'1440_ticks_after_manual_load',f'manual_{n}','manual')
    return result

EXTRA=r'''
def write(test,seed,branch,label,tick,detail=None):
 d=parent/f'{test}_s{seed}_{branch}'
 data=state(tick) if detail is None else dict(state=state(tick),detail=detail)
 (d/('checkpoint_'+label+'.json')).write_text(json.dumps(data))
if TARGET=='colony_steward':
 write(REPLAY,29,'replay','replay_setup',0,dict(intervention='enable replay and inject original sourced commands'))
 for test in HOUSING:
  write(test,29,'original','before_setup',0,dict(classification='controlled_full_production',terrain=test==HOUSING[0],recipe='all Position to hearth; 25x25 Plain; clear radius8 trees/nodes/material; optional two Hill tiles; rebuild nav'))
  write(test,29,'original','after_setup',0)
  write(test,29,'original','first_housed',2000)
  write(test,29,'replay','replay_setup',2000,dict(intervention='enable replay and inject original commands'))
  write(test,29,'cooldown','before_cooldown',2000,dict(classification='mechanism',rule=3,until=3440))
else:
 for branch in ('original','replay'):
  detail=dict(classification='controlled_full_production',intervention='tick once; arm adults with created spear; spawn four wolves pack42; enable steward') if branch=='original' else dict(intervention='same arm and incoming setup as original; reset steward and clear log; replay exported bytes retained under original occurrence')
  write(DEFENCE,0,branch,'before_setup',0,detail)
  write(DEFENCE,0,branch,'before_arming',1,dict(adult_ids=[1,2]))
 for branch,label in (('original','after_setup'),('replay','before_replay_reset'),('replay','after_replay_reset')):write(DEFENCE,0,branch,label,1)
 path=parent/f'{DEFENCE}_s0_original'/'input_exported_replay.bin'
 path.write_text(json.dumps([dict(phase='after_assignment'),dict(phase='before_flight')]))
'''


class StewardTests(unittest.TestCase):
    def setUp(self) -> None:
        temp=tempfile.TemporaryDirectory();self.addCleanup(temp.cleanup);root=Path(temp.name).resolve()
        original=Path(generic_process.__file__).resolve().parents[2]
        harness=root/'h';(harness/'tools/v8').mkdir(parents=True);gate=harness/'tools/v8/gate.sh'
        python=Path(sys.executable).resolve();bash=Path('/bin/bash');gate.write_text(f'step 37 defence {python} driver.py\n')
        output=root/'output';output.mkdir();arms:dict[str,Json]={};stages:list[Node]=[]
        for arm in ('base','candidate'):
            product=root/arm;product.mkdir();inputs:dict[str,str]={}
            for name in (*('rust/crates/sim-test/tests/'+target+'.rs' for target in SOURCE_PINS),EXTRA_SOURCE,FORMAT_SOURCE):
                path=product/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes((original/name).read_bytes());inputs[name]=file_hash(path)
            driver='import subprocess,sys\n'
            for target,names in TESTS.items():
                driver+=f"print('Running tests/{target}.rs (synthetic-fixture)',flush=True)\n"
                if target in ('colony_steward','steward_defence'):
                    child=CHILD.replace("'plan.json'",repr(target+'.json')).replace("'synthetic planned boundary'","r['stop']")
                    child=child[:child.index("for test in sorted({r['test'] for r in rows})")]
                    child+='\nTARGET='+repr(target)+'\nREPLAY='+repr(REPLAY)+'\nHOUSING='+repr(HOUSING)+'\nDEFENCE='+repr(DEFENCE)+'\n'+EXTRA
                    (product/(target+'.json')).write_text(json.dumps(rows(target)))
                    (product/(target+'.py')).write_text(child)
                    for file in (target+'.json',target+'.py'):inputs[file]=file_hash(product/file)
                    driver+=f'subprocess.run([sys.executable,{target+".py"!r}],check=True)\n'
                driver+=f'print("running {len(names)} tests")\n'
                for name in names:driver+=f'print("test {name} ... '+('ignored' if name==IGNORED else 'ok')+'")\n'
                ignored=int(IGNORED in names)
                driver+=f'print("test result: ok. {len(names)-ignored} passed; 0 failed; {ignored} ignored; 0 measured; 0 filtered out; finished in 0.01s")\n'
            (product/'driver.py').write_text(driver);inputs['driver.py']=file_hash(product/'driver.py');arms[arm]={'root':str(product),'source_inputs':inputs}
            step=output/arm/'37';observer=step/'native-worlds';observer.mkdir(parents=True)
            argv=(str(python),'driver.py');spec=ProcessSpec((str(bash),'-c','exec "$@" 2>&1','whole-step',*argv),product,bash,file_hash(bash),product,tuple(sorted(inputs.items())),step/'process',(('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GATE_TEST_OUT',str(observer))))
            actual=execute(spec);self.assertEqual(actual.native_exit,0)
            stages.append(Node({'step':37,'arm':arm,'state':'RETURNED','native_exit':0,'pid':actual.pid,'argv':list(argv),'cwd':str(product),'environment':environment_for(spec),'process_receipt':str(actual.receipt),'process_receipt_sha256':file_hash(actual.receipt),'observer_artifacts':{str(p):file_hash(p) for p in observer.rglob('*') if p.is_file()},'observer_directories':{str(p):p.stat().st_mode for p in (observer,*observer.rglob('*')) if p.is_dir()}}))
        self.session=Node({'output':str(output),'harness':{'root':str(harness),'inputs':{'tools/v8/gate.sh':file_hash(gate)}},'bash':{'path':str(bash),'sha256':file_hash(bash)},'arms':arms});self.base,self.candidate=stages

    def test_full_production_and_loaded_housing_with_precise_mechanisms(self) -> None:
        result=evaluate_family(self.session,self.base,self.candidate)
        self.assertEqual(result['status'],'INCOMPLETE')
        self.assertEqual(len(Node(result['coverage']).array()),32)
        self.assertEqual(len(Node(result['safety']).array()),30)
        self.assertEqual(len(Node(result['mechanisms']).array()),2)
        self.assertIn('migration external environment missing',str(result['unsupported']))

    def test_reject_changed_declared_setup_even_when_reanchored(self) -> None:
        row=dict(self.candidate.mapping());files=dict(self.candidate.field('observer_artifacts').mapping())
        file=next(Path(name) for name in files if HOUSING[0] in name and name.endswith('checkpoint_before_setup.json'))
        data=json.loads(file.read_text());data['detail']['terrain']=False;file.write_text(json.dumps(data));files[str(file)]=file_hash(file);row['observer_artifacts']=files
        with self.assertRaises(GateContractError):evaluate_family(self.session,self.base,Node(row))

if __name__=='__main__':unittest.main()
