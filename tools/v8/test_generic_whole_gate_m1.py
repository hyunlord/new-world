"""Synthetic native case streams and real harmless mirror-producing child fixture."""
import unittest

from gate_test_summary import SummaryError
from generic_gate import GateContractError
from generic_whole_gate_m1 import FFI, PROBE, m1_cases
from generic_whole_gate_native_probes import libtest_scope

PROBE_CHILD = r'''
import json,os,pathlib
root=pathlib.Path(os.environ['WORLDSIM_GATE_TEST_OUT']);pid=os.getpid();sequence=0
for family,seed,ticks in (('m1_colony_replay',5,2880),('m2_colony_replay',2,4320)):
 for branch in ('first','replay'):
  output=pathlib.Path('/tmp')/(family+'_'+str(pid))/branch
  observation=dict(schema_version=1,scene='colony',seed=seed,occurrence='primary',requested_ticks=ticks,requested_terminal_tick=ticks,initial_tick=0,initial_population=2,terminal_tick=ticks,terminal_alive=2,terminal_complete=True,history_complete=True,history_source='fresh_world_append_only_chronicle',knowledge_capability=dict(enabled=True,source='resources.rules.colony.knowledge'),knowledge_lost=[],deaths={name:0 for name in ('starvation','dehydration','exposure','combat','old_age')},checkpoints=[])
  envelope=dict(schema='worldsim_gate_test_observation_v1',process_id=pid,world_sequence=sequence,thread_name=None,local_output=str(output),local_sidecar=str(output/'gate_observations'/('colony_s'+str(seed)+'_primary.json')),observation=observation)
  (root/('probe_pid'+str(pid)+'_world'+str(sequence)+'.json')).write_text(json.dumps(envelope));sequence+=1
'''+ '\nfor name in '+repr(PROBE)+": print('test '+name+' ... ok')\nprint('test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s')\n"


def m1_raw() -> str:
    output=''
    for target,names in (('colony_commands',('synthetic_commands',)),('colony_jobs',('synthetic_jobs',)),('colony_world',('synthetic_world',)),('engine_colony_ffi',FFI),('sim_test',PROBE),('sim_systems',('runtime::jobs::synthetic_case',)),('sim_core',('colony::synthetic_case',))):
        source='unittests src/lib.rs' if target.startswith('sim_') else 'tests/'+target+'.rs'
        output+=f'Running {source} (target/release/deps/{target}-1234)\nrunning {len(names)} '+('test' if len(names)==1 else 'tests')+'\n'
        output+=''.join('test '+name+' ... ok\n' for name in names)
        output+=f'test result: ok. {len(names)} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'
    return output+'TRF_TEST_PASS synthetic\n'


class M1CaseTests(unittest.TestCase):
    def test_actual_case_inventory_keeps_seven_original_sections(self) -> None:
        rows=m1_cases(m1_raw());self.assertEqual(len(rows),7)
        self.assertEqual(rows[3].target,'engine_colony_ffi')
        self.assertEqual(len(rows[4].summary.outcomes),len(PROBE))
        libtest_scope(m1_raw(),'release')

    def test_failed_zero_duplicate_or_wrong_profile_rejected(self) -> None:
        for raw in (m1_raw().replace('/release/','/debug/'),m1_raw()+m1_raw(),m1_raw().replace(PROBE[0],'foreign_case'),m1_raw().replace('runtime::jobs::synthetic_case','runtime::mind::wrong')):
            with self.assertRaises((GateContractError,SummaryError)):_ = m1_cases(raw)
        with self.assertRaises(GateContractError):libtest_scope(m1_raw(),'debug')

if __name__=='__main__':_ = unittest.main()
