"""Synthetic source-bound endpoint tests; no native qualification is inferred."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from gate_test_summary import SummaryError
from generic_gate import GateContractError
from generic_process import file_hash
from generic_whole_gate_knowledge_defaults import (
    PINS,
    ROWS,
    KnowledgeDefaults,
    collect_knowledge_defaults,
    evaluate_knowledge_defaults,
)
from mux2_gate_inputs import Node
from test_generic_test_history import graph


def plan(target):
    return [{'test': test,'seed': seed,'branch': 'primary','start': 0,'end': end} for test,seed,end,_ in ROWS[target]]


def output(target,profile='release'):
    names=[row[0] for row in ROWS[target]]
    return f'Running tests/{target}.rs (target/{profile}/deps/{target}-synthetic)\nrunning {len(names)} tests\n'+''.join('test '+name+' ... ok\n' for name in names)+f'test result: ok. {len(names)} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'


class KnowledgeDefaultsTests(unittest.TestCase):
    def fixture(self,data,target):
        root=data.spec.input_root;inputs=dict(data.spec.inputs)
        producer=Path('/Users/rexxa/work/m6b-data/stage0/ordered-dispatch/whole-gate-knowledge-defaults-draft')
        original=Path('/Users/rexxa/work/new-world-m6b-dev')
        for name,digest in PINS.items():
            source=producer/name if (producer/name).exists() else original/name
            path=root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(source.read_bytes())
            self.assertEqual(file_hash(path),digest);inputs[name]=digest
        for test,seed,_,capability in ROWS[target]:
            for path in data.folder(f'{test}_s{seed}_primary').glob('*.json'):
                node=json.loads(path.read_text())
                for key in ('initial','terminal','founding'):
                    if key in node:
                        node[key]['knowledge_capability']['enabled']=capability
                        node[key]['knowledge_lost']=[] if capability else None
                if path.name=='terminal.json':node['stop_reason']='original_endpoint'
                path.write_text(json.dumps(node))
        session=Node({'arms':{'base':{'root':str(root),'source_inputs':inputs}}})
        stage={'step':5,'arm':'base','process_receipt':str(data.spec.output_root/'process.json'),
               'environment':{'WORLDSIM_GATE_OBSERVER':'1','WORLDSIM_GATE_TEST_OUT':str(data.spec.output_root)}}
        return session,stage

    def anchored(self,data,stage):
        result=dict(stage)
        result['observer_artifacts']={str(p):file_hash(p) for p in data.spec.output_root.rglob('*') if p.is_file() and 'save_load_pid' in str(p)}
        result['observer_directories']={str(p):p.stat().st_mode for p in (data.spec.output_root,*data.spec.output_root.rglob('*')) if p.is_dir()}
        return Node(result)

    def test_six_source_worlds_keep_exact_capability_and_horizons(self):
        count=0
        for target in ROWS:
            with graph(plan(target)) as data:
                session,stage=self.fixture(data,target)
                with patch('generic_whole_gate_knowledge_defaults.aggregate_output',return_value=(output(target),data.pid)):
                    family=collect_knowledge_defaults(session,self.anchored(data,stage),target)
                    self.assertEqual([r.terminal_tick for r in family.observations],[r[2] for r in ROWS[target]])
                    self.assertEqual([r.knowledge_enabled for r in family.observations],[r[3] for r in ROWS[target]])
                    count+=len(family.observations)
        self.assertEqual(count,6)

    def test_wrong_original_profile_or_missing_case_rejected(self):
        target='engine_knowledge_ffi'
        with graph(plan(target)) as data:
            session,stage=self.fixture(data,target)
            for raw in (output(target,'debug'),output(target).replace('test '+ROWS[target][0][0]+' ... ok\n','')):
                with patch('generic_whole_gate_knowledge_defaults.aggregate_output',return_value=(raw,data.pid)), self.assertRaises((GateContractError,SummaryError)):
                    collect_knowledge_defaults(session,self.anchored(data,stage),target)

    def test_wrong_horizon_and_baseline_capability_rejected(self):
        target='engine_knowledge_ffi'
        for wrong in ('horizon','capability'):
            with graph(plan(target)) as data:
                session,stage=self.fixture(data,target)
                test,seed,_,_=ROWS[target][0];path=data.folder(f'{test}_s{seed}_primary')/'terminal.json'
                node=json.loads(path.read_text())
                if wrong=='horizon':node['terminal']['tick']=2
                else:node['terminal']['knowledge_capability']['enabled']=True;node['terminal']['knowledge_lost']=[]
                path.write_text(json.dumps(node))
                with patch('generic_whole_gate_knowledge_defaults.aggregate_output',return_value=(output(target),data.pid)), self.assertRaises(GateContractError):
                    collect_knowledge_defaults(session,self.anchored(data,stage),target)

    def test_strict_candidate_loss_does_not_follow_six_green_outcomes(self):
        families=[]
        for target in ROWS:
            with graph(plan(target)) as data:
                session,stage=self.fixture(data,target)
                with patch('generic_whole_gate_knowledge_defaults.aggregate_output',return_value=(output(target),data.pid)):
                    families.append(collect_knowledge_defaults(session,self.anchored(data,stage),target))
        base=Node({'step':5,'arm':'base'});candidate=Node({'step':5,'arm':'candidate'})
        with patch('generic_whole_gate_knowledge_defaults.collect_knowledge_defaults',side_effect=[families[0],families[0],families[1],families[1]]):
            result=evaluate_knowledge_defaults(Node({}),base,candidate)
            self.assertEqual(len(result['coverage']),6);self.assertEqual(len(result['safety']),6)
        changed=list(families[0].observations);changed[1]=replace(changed[1],losses=((200,'Fire',None),))
        candidate_family=KnowledgeDefaults(tuple(changed),families[0].emitted_pid+1)
        with patch('generic_whole_gate_knowledge_defaults.collect_knowledge_defaults',side_effect=[families[0],candidate_family]), self.assertRaises(GateContractError):
            evaluate_knowledge_defaults(Node({}),base,candidate)

if __name__=='__main__':unittest.main()
