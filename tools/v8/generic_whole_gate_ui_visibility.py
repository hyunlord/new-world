# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""Original39 generator/loader with source-qualified aggregate save continuity."""
import json
from pathlib import Path
from typing import Final

from generic_gate import boundary, document, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_ui_observations import require_fresh_history
from generic_whole_gate_contract import current_product_inputs
from mux2_gate_inputs import BRIDGE, Json, Node
from mux2_gate_visual import validate_visibility

import generic_ui_visibility_run as visibility
from generic_ui_history import SaveLink, join_validated_history
from generic_whole_gate_ui import (
    WholeUIEvidence,
    WholeUIPlan,
    bind_outer,
    check_namespace,
    collect_ui,
)
from generic_whole_gate_ui_fixed import WRAPPER

CAPTURE: Final = ('tools/v8/mux2_gate_receipts.py','a3d7c3b910ca51c04006d22ce3d41d3f289a13cfe9d6f614cea0ef24c7e463e4')
VISUAL: Final = ('tools/v8/mux2_gate_visual.py','c7490898aab7d602e4e6b7dbb9a4b269fc819899fb5c86b1d39161c2e542cce3')


def _artifact(stage: Node, path: Path) -> None:
    stage.field('observer_artifacts').field(str(path)).equals(file_hash(path))


def collect_visibility(session: Node, stage: Node) -> tuple[Observation,...]:
    stage.field('step').equals(39);stage.field('name').equals('mux2_visibility')
    argv=tuple(text(v) for v in stage.field('argv').array())
    require(len(argv)==9 and argv[:6]==('python3',CAPTURE[0],'capture','--kind','visibility','--out')
            and argv[7]=='--godot','/argv','foreign original visibility command')
    root,_=bind_outer(session,stage);out=Path(argv[6]);godot=argv[8]
    require(out.is_absolute() and godot in session.field('external_inputs').mapping(),'/capture','unbound capture paths')
    env=stage.field('environment');env.field('GODOT_BIN').equals(godot)
    directory=Path(text(env.field('WORLDSIM_GATE_UI_OUT')));check_namespace(stage,directory)
    paths=tuple(directory.iterdir());require(len(paths)==2 and all(p.is_file() for p in paths),'/ui','missing/extra visibility producer')
    indexed: dict[str,Path]={}
    for path in paths:
        name=text(document(path.read_text()).field('producer'))
        require(name in ('visibility.generator','visibility.loader') and name not in indexed,'/ui','foreign/duplicate visibility producer')
        indexed[name]=path
    capture_path=out/'receipt.json';_artifact(stage,capture_path)
    capture=document(capture_path.read_text())
    for key,value in (('schema',1),('kind','visibility'),('complete',True),('exit_code',0),('root',str(root)),('godot',godot)):
        capture.field(key).equals(value)
    arm=text(stage.field('arm'));capture.field('source_head').equals(session.field('arms').field(arm).field('source_sha').value)
    capture.field('godot_sha256').equals(session.field('external_inputs').field(godot).value)
    bridge=current_product_inputs(session,arm,'bridge').field('executable_sha256').text()
    capture.field('inputs').field(BRIDGE).equals(bridge)
    for name,digest in capture.field('artifacts').mapping().items():
        _artifact(stage,Path(name));require(file_hash(Path(name))==Node(digest).text(),'/capture','capture artifact changed')
    manifest_path=out/'saves/manifest.json';_artifact(stage,manifest_path)
    manifest=document(manifest_path.read_text())
    for key,value in (('schema',1),('seed',0),('steward','on'),('bridge_sha256',bridge),('script_sha256',visibility.PINS[visibility.GENERATOR])):
        manifest.field(key).equals(value)
    scenes=tuple(manifest.field('scenes').array());require(len(scenes)==4,'/manifest','four exact scenes required')
    for row,(name,day,hour,tick) in zip(scenes,visibility.TARGETS,strict=True):
        save=out/'saves'/(name+'.wsave');_artifact(stage,save)
        for key,value in (('id',name),('day',day),('hour',hour),('tick',tick),('save',str(save)),('sha256',file_hash(save))):
            row.field(key).equals(value)
    pins=tuple(sorted((*visibility.PINS.items(),WRAPPER,CAPTURE,VISUAL)))
    collected: list[WholeUIEvidence]=[]
    prefixes: tuple[visibility.Rows,...]=()
    expected_commands: list[list[str]]=[]
    for generator in (True,False):
        producer='visibility.generator' if generator else 'visibility.loader'
        script='res://'+(visibility.GENERATOR if generator else visibility.LOADER)
        native=(godot,'--path',str(root),'--resolution','1920x1080')+(('--headless',) if generator else ())
        native+=('--script',script,'--')
        native+=(str(out/'saves'),) if generator else ('--manifest',str(manifest_path),'--out',str(out/'run'))
        expected_commands.append(['bash',str(root/WRAPPER[0]),'1800' if generator else '1200',*native])
        checkpoints,instances=visibility.lifecycle_plan(out,scenes,generator)
        plan=WholeUIPlan(indexed[producer],producer,native,script,visibility.HELPER,pins,checkpoints,instances)
        evidence=collect_ui(session,stage,plan);collected.append(evidence)
        logs=visibility.commands(document(evidence.plan.sidecar.read_text()),generator,prefixes)
        if generator:prefixes=tuple(logs[2+i*2] for i in range(4))
    command_log=out/'command.log';_artifact(stage,command_log)
    actual_commands=[document(line).value for line in command_log.read_text().splitlines() if line.startswith('["bash",')]
    require(actual_commands==expected_commands,'/capture','original sequential commands differ')
    source,destination=collected
    require(source.emitted_pid!=destination.emitted_pid,'/history','reused native invocation PID')
    raw_source=document(source.plan.sidecar.read_text());raw_destination=document(destination.plan.sidecar.read_text())
    observations: list[Observation]=[]
    for evidence in collected:
        for native_node in evidence.validated.instances:
            for instance in native_node:
                if instance.reason!='load':observations.append(require_fresh_history(instance))
    for i,row in enumerate(scenes):
        link=SaveLink(Path(text(row.field('save'))),text(row.field('sha256')),manifest_path,file_hash(manifest_path),
                      visibility.TARGETS[i][0],2+i*2,3+i*2,1+i*5,BRIDGE)
        joined=join_validated_history(raw_source,raw_destination,destination.validated,i,link,bridge,
                                      source.sidecar_sha256,source.process_sha256,destination.sidecar_sha256,destination.process_sha256)
        # Aggregate provenance is explicit; this never upgrades observer PIDs into independent child waits.
        proof=document(joined.evidence_json).mapping()
        proof['process_qualification']='SOURCE_BOUND_AGGREGATE_ONLY'
        chain=json.dumps(proof,sort_keys=True,separators=(',',':'))
        state=joined.observation
        observations.append(Observation(state.initial_tick,state.terminal_tick,state.founders,state.alive,state.knowledge_enabled,state.losses,chain))
    require(len(observations)==10 and all(state.founders==10 for state in observations),'/coverage','incomplete visibility native coverage')
    validate_visibility(root,out);check_namespace(stage,directory)
    return tuple(observations)


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> tuple[Json,...]:
    base.field('arm').equals('base');candidate.field('arm').equals('candidate')
    return tuple(ordinary_pair(a,b) for a,b in zip(collect_visibility(session,base),collect_visibility(session,candidate),strict=True))
