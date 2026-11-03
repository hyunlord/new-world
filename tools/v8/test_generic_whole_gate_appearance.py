"""Synthetic parser/ancestry fixtures; no product runtime qualification."""
import json
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

from generic_gate import GateContractError
from generic_process import file_hash
from generic_test_endpoint import EndpointEvidence, authenticate_endpoint
from generic_test_history_graph import HistoryEvidence, identity, resolve_graph
from generic_whole_gate_appearance_plan import appearance_plan, SAVE_TEST, BASELINE_TEST, KIN_TEST, RENDER_TEST
from generic_whole_gate_appearance import renderer_mirrors
from test_generic_test_history import graph
from mux2_gate_inputs import Node

ROWS = [dict(test=SAVE_TEST,seed=3,branch='primary',start=0,end=3000),
        dict(test=SAVE_TEST,seed=3,branch='twin',start=0,end=2000),
        dict(test=SAVE_TEST,seed=3,branch='loaded',start=2000,end=3000,parent='primary',label='tick2000'),
        dict(test=BASELINE_TEST,seed=0,branch='primary',start=0,end=100),
        dict(test=KIN_TEST,seed=3,branch='primary',start=0,end=1440)]

class AppearanceTests(unittest.TestCase):
    def test_five_source_occurrences_and_saved_origin(self):
        with graph(ROWS) as data:
            for test,seed in ((BASELINE_TEST,0),(KIN_TEST,3)):
                for path in data.folder(f'{test}_s{seed}_primary').glob('*.json'):
                    node=json.loads(path.read_text())
                    for key in ('initial','terminal','founding'):
                        if key in node:
                            node[key]['knowledge_capability']['enabled']=False
                            node[key]['knowledge_lost']=None
                    path.write_text(json.dumps(node))
            folder=data.folder(KIN_TEST+'_s3_primary')
            terminal=json.loads((folder/'terminal.json').read_text())['terminal']
            event=folder/'checkpoint_event_stop.json'
            event.write_text(json.dumps(dict(state=terminal,detail=dict(predicate='mother_and_partner_present',days_advanced=1,ticks_per_day=1440,maximum_days=120,mother_rows=1,partner_rows=2))))
            hashes=tuple((str(p.relative_to(data.spec.output_root)),file_hash(p)) for p in data.spec.output_root.rglob('*') if p.is_file() and 'save_load_pid' in str(p))
            evidence=HistoryEvidence(data.spec.output_root,data.pid,hashes,data.spec.input_root,data.spec.inputs)
            plans=appearance_plan(evidence);self.assertEqual(len(plans),5)
            endpoints={identity(p.occurrence):authenticate_endpoint(EndpointEvidence(evidence.root,evidence.pid,evidence.output_hashes),p.occurrence) for p in plans}
            histories=resolve_graph(evidence,tuple(p.occurrence for p in plans),endpoints)
            self.assertTrue(all(h.history_complete for h in histories))
            self.assertEqual(histories[2].edges[0].tick,2000)
            node=json.loads(event.read_text());node['detail']['mother_rows']=0;event.write_text(json.dumps(node))
            changed=replace(evidence,output_hashes=tuple((name,file_hash(evidence.root/name)) for name,_ in hashes))
            with self.assertRaises(GateContractError):appearance_plan(changed)

    def test_two_renderers_require_original_commands_and_emitter(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary).resolve();anchors={};pid=123
            script=(Node({'tick':360,'cmd':'synthetic_bound_command'}),)
            for seq,branch in enumerate(('render','bridge-replay')):
                output=Path('/tmp/engine_appearance_ffi')
                if branch=='bridge-replay':output/=branch
                observation=dict(schema_version=1,scene='colony',seed=100,occurrence='primary',requested_ticks=900,requested_terminal_tick=900,initial_tick=0,initial_population=2,terminal_tick=900,terminal_alive=2,terminal_complete=True,history_complete=True,history_source='fresh_world_append_only_chronicle',knowledge_capability=dict(enabled=True,source='resources.rules.colony.knowledge'),knowledge_lost=[],deaths={k:0 for k in ('starvation','dehydration','exposure','combat','old_age')},checkpoints=[dict(phase='before_tick_commands',tick=360,alive=2,script_index=0,script_spec=script[0].value,resolved_command_index=None,debug_kill_agent=None,history_complete=True,knowledge_lost=[])])
                path=root/f'probe_pid{pid}_world{seq}.json'
                path.write_text(json.dumps(dict(schema='worldsim_gate_test_observation_v1',process_id=pid,world_sequence=seq,thread_name=RENDER_TEST,local_output=str(output),local_sidecar=str(output/'gate_observations/colony_s100_primary.json'),observation=observation)))
                anchors[str(path)]=file_hash(path)
            stage=Node({'environment':{'WORLDSIM_GATE_TEST_OUT':str(root)},'observer_artifacts':anchors})
            self.assertEqual(len(renderer_mirrors(stage,pid,script)),2)
            with self.assertRaises(GateContractError):renderer_mirrors(stage,pid+1,script)
            with self.assertRaises(GateContractError):renderer_mirrors(stage,pid,())

if __name__=='__main__':unittest.main()
