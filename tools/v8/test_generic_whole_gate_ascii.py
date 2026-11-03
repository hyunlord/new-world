"""Synthetic endpoint/source parser tests; aggregate authentication is mocked explicitly."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from gate_test_summary import SummaryError
from generic_gate import GateContractError
from generic_process import file_hash
from generic_whole_gate_ascii import PINS, AsciiFamily, collect_ascii, evaluate_ascii
from generic_whole_gate_ascii_plan import (
    CARRIER,
    CROP,
    GRID,
    HEARTH,
    PEOPLE,
    TESTS,
    TREES,
)
from mux2_gate_inputs import Node
from test_generic_test_history import graph

ROWS=[{'test': test,'seed': 100,'branch': branch,'start': 0,'end': end} for test,branch,end in
      ((GRID,'primary',0),(PEOPLE,'primary',900),(HEARTH,'primary',900),(TREES,'before_commands',360),
       (TREES,'felled',900),(TREES,'expired',2340),(CARRIER,'primary',1000),(CROP,'primary',900))]
COMMANDS=[{'tick': 360,'command': {'cmd': name,'tick': 360},'accepted': True,'count': 1,'result_debug': 'synthetic'}
          for name in ('designate_chop_tiles','add_stockpile','place_blueprint')]


def native_output(profile='release'):
    return f'Running tests/engine_ascii.rs (target/{profile}/deps/engine_ascii-synthetic)\nrunning 6 tests\n'+''.join('test '+name+' ... ok\n' for name in TESTS)+'test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'


class AsciiTests(unittest.TestCase):
    def fixture(self, data):
        root=data.spec.input_root
        original=Path('/Users/rexxa/work/new-world-m6b-dev')
        producer=Path('/Users/rexxa/work/m6b-data/stage0/ordered-dispatch/whole-gate-engine-ascii-draft')
        inputs=dict(data.spec.inputs)
        for name,digest in PINS.items():
            path=root/name;path.parent.mkdir(parents=True,exist_ok=True)
            source=producer/name if (producer/name).exists() else original/name
            path.write_bytes(source.read_bytes());self.assertEqual(file_hash(path),digest);inputs[name]=digest
        for row in ROWS:
            folder=data.folder(f"{row['test']}_s100_{row['branch']}")
            path=folder/'terminal.json';node=json.loads(path.read_text())
            node['stop_reason']='first_awake_carrier' if row['test']==CARRIER else 'original_endpoint'
            if row['end']>360:node['terminal']['command_log']=COMMANDS
            path.write_text(json.dumps(node))
            if row['test']==CARRIER:
                start=dict(node['terminal']);start['tick']=600
                (folder/'checkpoint_search_start.json').write_text(json.dumps({'state': start,'detail': {'start_tick': 600}}))
                (folder/'checkpoint_event_stop.json').write_text(json.dumps({'state': node['terminal'],'detail': {'predicate': 'awake_carrier_free_right_tile','checked_carriers': 1,'start_tick': 600,'maximum_additional_ticks': 1440,'actual_tick': 1000,'attempts_before_success': 400}}))
        session=Node({'arms':{'base':{'root':str(root),'source_inputs':inputs}}})
        stage={'step':5,'arm':'base','process_receipt':str(data.spec.output_root/'process.json'),
               'environment':{'WORLDSIM_GATE_OBSERVER':'1','WORLDSIM_GATE_TEST_OUT':str(data.spec.output_root)}}
        return session,stage

    def anchored(self,data,stage):
        result=dict(stage)
        result['observer_artifacts']={str(p):file_hash(p) for p in data.spec.output_root.rglob('*') if p.is_file() and 'save_load_pid' in str(p)}
        result['observer_directories']={str(p):p.stat().st_mode for p in (data.spec.output_root,*data.spec.output_root.rglob('*')) if p.is_dir()}
        return Node(result)

    def test_eight_original_worlds_and_zero360_command_boundary(self):
        with graph(ROWS) as data:
            session,stage=self.fixture(data)
            with patch('generic_whole_gate_ascii.aggregate_output',return_value=(native_output(),data.pid)):
                family=collect_ascii(session,self.anchored(data,stage))
                self.assertEqual(len(family.observations),8)
                self.assertEqual([r.terminal_tick for r in family.observations],[0,900,900,360,900,2340,1000,900])
                path=data.folder(TREES+'_s100_before_commands')/'terminal.json'
                node=json.loads(path.read_text());node['terminal']['command_log']=COMMANDS;path.write_text(json.dumps(node))
                with self.assertRaises((GateContractError,SummaryError)):collect_ascii(session,self.anchored(data,stage))

    def test_original_outcome_profile_and_carrier_predicate_are_required(self):
        with graph(ROWS) as data:
            session,stage=self.fixture(data)
            for output in (native_output('debug'),native_output().replace('test '+GRID+' ... ok\n','')):
                with patch('generic_whole_gate_ascii.aggregate_output',return_value=(output,data.pid)), self.assertRaises((GateContractError,SummaryError)):
                    collect_ascii(session,self.anchored(data,stage))
            path=data.folder(CARRIER+'_s100_primary')/'checkpoint_event_stop.json'
            node=json.loads(path.read_text());node['detail']['checked_carriers']=0;path.write_text(json.dumps(node))
            with patch('generic_whole_gate_ascii.aggregate_output',return_value=(native_output(),data.pid)), self.assertRaises((GateContractError,SummaryError)):
                collect_ascii(session,self.anchored(data,stage))

    def test_missing_tree_wrong_seed_and_incomplete_history_rejected(self):
        for mutation in ('tree_branch','seed','history'):
            with self.subTest(mutation=mutation), graph(ROWS) as data:
                session,stage=self.fixture(data)
                folder=data.folder(TREES+'_s100_felled')
                if mutation=='tree_branch':
                    folder.rename(folder.with_name(TREES+'_s100_foreign'))
                else:
                    path=folder/('initial.json' if mutation=='seed' else 'terminal.json')
                    node=json.loads(path.read_text())
                    if mutation=='seed':node['seed']=101
                    else:node['history_complete']=False
                    path.write_text(json.dumps(node))
                with patch('generic_whole_gate_ascii.aggregate_output',return_value=(native_output(),data.pid)), self.assertRaises(GateContractError):
                    collect_ascii(session,self.anchored(data,stage))

    def test_tick600_requires_actual_success_not_search_start_alone(self):
        with graph(ROWS) as data:
            session,stage=self.fixture(data)
            folder=data.folder(CARRIER+'_s100_primary')
            path=folder/'terminal.json';node=json.loads(path.read_text())
            node['terminal']['tick']=600;node['completed_ticks']=600;path.write_text(json.dumps(node))
            with patch('generic_whole_gate_ascii.aggregate_output',return_value=(native_output(),data.pid)), self.assertRaises(GateContractError):
                collect_ascii(session,self.anchored(data,stage))
                path=folder/'checkpoint_event_stop.json';event=json.loads(path.read_text())
                event['state']['tick']=600;event['detail']['actual_tick']=600;event['detail']['attempts_before_success']=0
                path.write_text(json.dumps(event))
                self.assertEqual(collect_ascii(session,self.anchored(data,stage)).observations[6].terminal_tick,600)

    def test_event_mismatch_stays_incomplete_but_never_hides_candidate_loss(self):
        with graph(ROWS) as data:
            session,stage=self.fixture(data)
            with patch('generic_whole_gate_ascii.aggregate_output',return_value=(native_output(),data.pid)):
                left=collect_ascii(session,self.anchored(data,stage))
        changed=list(left.observations);changed[6]=replace(changed[6],terminal_tick=1001)
        right=AsciiFamily(left.plans,tuple(changed),left.emitted_pid+1)
        base=Node({'step':5,'arm':'base'});candidate=Node({'step':5,'arm':'candidate'})
        with patch('generic_whole_gate_ascii.collect_ascii',side_effect=[left,right]):
            result=evaluate_ascii(Node({}),base,candidate)
            self.assertEqual(result['status'],'INCOMPLETE');self.assertEqual(len(result['safety']),7)
            self.assertEqual(len(result['coverage']),8);self.assertTrue(result['unsupported'])
        changed[6]=replace(changed[6],losses=((900,'Fire',None),))
        with patch('generic_whole_gate_ascii.collect_ascii',side_effect=[left,replace(right,observations=tuple(changed))]), self.assertRaises(GateContractError):
            evaluate_ascii(Node({}),base,candidate)
        with patch('generic_whole_gate_ascii.collect_ascii',side_effect=[left,left]):
            self.assertEqual(evaluate_ascii(Node({}),base,candidate)['status'],'PASS')

if __name__=='__main__':unittest.main()
