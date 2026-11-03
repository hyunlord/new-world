"""Four source-selected M1/M2 replay probes retained by the native mirror hook."""
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from gate_test_summary import parse_log
from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, ProbeWorld, parse_probe, ordinary_pair
from generic_process import real_path
from generic_test_endpoint_types import bound_bytes, fields
from generic_whole_gate_aggregate import aggregate_output
from generic_whole_gate_m1 import PROBE, check_sources
from mux2_gate_inputs import Json,Node

REPLAYS: Final = {
    'm1_colony_replay':('probe::colony::tests::a_recorded_command_log_replays_to_a_byte_identical_summary',5,2880),
    'm2_colony_replay':('probe::colony::tests::an_m2_command_log_with_bills_and_priorities_replays_identically',2,4320),
}

@dataclass(frozen=True,slots=True)
class Replayed:
    family:str
    branch:str
    seed:int
    pid:int
    sequence:int
    observation:Observation


def libtest_scope(raw:str,profile:str) -> None:
    headers=list(re.finditer(r'^\s*(?:Running (?:unittests )?[^\n]+|Doc-tests [^\n]+)\s*$',raw,re.MULTILINE))
    selected=[(index,header) for index,header in enumerate(headers)
              if re.fullmatch(r'\s*Running unittests src/lib\.rs \([^\n]*/'+profile+r'/deps/sim_test-[^/()]+\)\s*',header.group())]
    require(len(selected)==1,'/probe-lib','missing or duplicate source-selected sim-test library')
    index,header=selected[0];payload=raw[header.end():headers[index+1].start() if index+1<len(headers) else len(raw)]
    summary=parse_log(payload)
    names={row.name:row.status for row in summary.outcomes}
    require(summary.summaries==1 and len(names)==len(summary.outcomes) and summary.failed==0,'/probe-lib','ambiguous or failed native library')
    require(all(names.get(name)=='ok' for name in PROBE),'/probe-lib','selected probe assertions absent or failed')


@boundary
def collect_replays(session:Node,stage:Node) -> tuple[Replayed,...]:
    step=integer(stage.field('step'));require(step in {4,5,17},'/route','unregistered probe library route')
    check_sources(session,stage)
    raw,_=aggregate_output(session,stage);libtest_scope(raw,'debug' if step==4 else 'release')
    environment=stage.field('environment');environment.field('WORLDSIM_GATE_OBSERVER').equals('1')
    root=Path(text(environment.field('WORLDSIM_GATE_TEST_OUT')));real_path(root,directory=True)
    receipt=Path(text(stage.field('process_receipt')))
    require(root.is_relative_to(receipt.parent.parent),'/observer','foreign probe mirror root')
    stage.field('observer_directories').field(str(root)).equals(root.stat().st_mode)
    anchors={name:text(Node(value)) for name,value in stage.field('observer_artifacts').mapping().items()}
    expected={(family,branch) for family in REPLAYS for branch in ('first','replay')}
    selected:dict[tuple[str,str],Replayed]={};identities:set[tuple[int,int]]=set()
    # Other workspace probe families retain their own coverage obligations.
    for path in root.glob('probe_pid*_world*.json'):
        require(str(path) in anchors,str(path),'unbound native mirror')
        node=document(bound_bytes(path,anchors[str(path)]).decode())
        output=Path(text(node.field('local_output')))
        family=next((name for name in REPLAYS if output.parent.name.startswith(name+'_')),None)
        if family is None:continue
        fields(node,'schema process_id world_sequence thread_name local_output local_sidecar observation')
        node.field('schema').equals('worldsim_gate_test_observation_v1')
        pid=integer(node.field('process_id'));sequence=integer(node.field('world_sequence'));require(pid>0,'/pid','positive emitted process required')
        require(path.name==f'probe_pid{pid}_world{sequence}.json' and output.is_absolute()
                and '..' not in output.parts and output.parent.name==f'{family}_{pid}',str(path),'source-specific temporary output or native identity mismatch')
        require((pid,sequence) not in identities,str(path),'duplicated native mirror identity');identities.add((pid,sequence))
        # run() always uses unnamed scoped worker threads, even when workers=1.
        node.field('thread_name').equals(None)
        branch=output.name;key=(family,branch);require(key in expected and key not in selected,str(path),'foreign or duplicated replay branch')
        _,seed,ticks=REPLAYS[family]
        node.field('local_sidecar').equals(str(output/'gate_observations'/f'colony_s{seed}_primary.json'))
        observation=parse_probe(json.dumps(node.field('observation').value),ProbeWorld('colony',seed,'primary',ticks))
        require(observation.checkpoints_json=='[]',str(path),'source replay fixtures have no debug-kill interventions')
        selected[key]=Replayed(family,branch,seed,pid,sequence,observation)
    require(set(selected)==expected,'/probe-replays','missing source-selected ordinary replay occurrence')
    require(len({row.pid for row in selected.values()})==1,'/probe-replays','four library probe worlds must share actual emitter PID')
    for family in REPLAYS:
        require(selected[family,'first'].sequence<selected[family,'replay'].sequence,family,'replay precedes original in actual process')
    return tuple(selected[key] for key in sorted(expected))


@boundary
def evaluate_replays(session:Node,base:Node,candidate:Node) -> dict[str,Json]:
    left,right=collect_replays(session,base),collect_replays(session,candidate)
    safety:list[Json]=[];coverage:list[Json]=[]
    for a,b in zip(left,right,strict=True):
        require((a.family,a.branch,a.seed)==(b.family,b.branch,b.seed),'/pair','replay identity differs')
        safety.append(ordinary_pair(a.observation,b.observation))
        coverage.append({'target':'sim_test','test':REPLAYS[a.family][0],'branch':a.branch,'seed':a.seed,
                         'base_emitted_pid':a.pid,'candidate_emitted_pid':b.pid,
                         'base_world_sequence':a.sequence,'candidate_world_sequence':b.sequence,
                         'input_contract':'source-fixed seed/days/workers1/story-on/neighbours-on/steward-off/no-twins',
                         'oracle':'summary_and_command_log_byte_equality' if a.family=='m1_colony_replay' else 'summary_byte_equality_and_original_priority_quarry_blueprint_three_bill_assertions',
                         'temporary_replay_input':'SOURCE_ASSERTION_ONLY_NOT_RETAINED_NO_INPUT_HASH_CLAIM'})
    return {'status':'PASS','safety':safety,'coverage':coverage,'qualification':'SOURCE_SELECTED_NATIVE_ASSERTIONS_AND_ACTUAL_FOUR_MIRRORS'}
