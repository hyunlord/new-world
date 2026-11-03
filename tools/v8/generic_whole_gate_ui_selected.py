# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""Original renderer and selected-pack UI worlds; native siblings remain separate."""
import json
from pathlib import Path

import generic_renderer_run as renderer
import generic_ui_pack_run as pack
from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_ui_observations import parse_native, require_fresh_history, unsigned_string
from mux2_gate_inputs import Json, Node

from generic_ui_receipts import CheckpointPlan, InstancePlan
from generic_whole_gate_ui import (
    WholeUIEvidence,
    WholeUIPlan,
    bind_outer,
    check_namespace,
    collect_ui,
)
from generic_whole_gate_ui_fixed import WRAPPER


def _select(session: Node, stage: Node, step: int, name: str, producer: str) -> tuple[Path,Path,Path,str,Node]:
    stage.field('step').equals(step);stage.field('name').equals(name)
    argv=tuple(text(v) for v in stage.field('argv').array())
    require(len(argv)==3 and argv[:2]==('bash','tools/v8/'+name+'.sh'),'/argv','foreign original UI command')
    root,_=bind_outer(session,stage)
    log=Path(argv[2]);require(log.is_absolute(),'/log','unbound relative producer output')
    env=stage.field('environment');directory=Path(text(env.field('WORLDSIM_GATE_UI_OUT')))
    check_namespace(stage,directory)
    paths=tuple(directory.iterdir())
    require(len(paths)==1 and paths[0].is_file(),'/ui','missing/extra original UI sidecars')
    raw=document(paths[0].read_text());raw.field('producer').equals(producer)
    godot=text(env.field('GODOT_BIN'))
    require(godot in session.field('external_inputs').mapping(),'/godot','Godot lacks independent binding')
    return root,log,paths[0],godot,raw


def _artifact(stage: Node, path: Path) -> None:
    stage.field('observer_artifacts').field(str(path)).equals(file_hash(path))


def collect_renderer(session: Node, stage: Node) -> WholeUIEvidence:
    _,log,path,godot,raw=_select(session,stage,23,'renderer_agreement',renderer.PRODUCER)
    require('RENDERER_AGREEMENT_TICK' not in stage.field('environment').mapping(),'/tick','nondefault renderer horizon')
    commands=log/'commands/colony_s100_commands.json';_artifact(stage,commands)
    cps=raw.field('checkpoints').array();require(len(cps)==5,'/checkpoints','incomplete renderer')
    horizon=integer(cps[1].field('clock').field('tick'))
    context=json.dumps({'commands_path':str(commands),'commands_sha256':file_hash(commands)})
    checkpoints=(CheckpointPlan('bootstrap',0,1,0),CheckpointPlan('before_reset',0,1,horizon,context),
                 CheckpointPlan('after_reset',0,2,0),CheckpointPlan('replay_complete',0,2,900),
                 CheckpointPlan('terminal',0,2,900,releases_node=True))
    argv=(godot,'--headless','--path','.', '--script',renderer.SCRIPT,'--','--out',str(log/'godot_people.json'),
          '--commands',str(commands),'--seed','100','--tick','900')
    # ASCII semantics are authenticated by the separate native sibling collector.
    pins={k:v for k,v in renderer.PINS.items() if k!='rust/crates/sim-test/src/probe/ascii.rs'}
    plan=WholeUIPlan(path,renderer.PRODUCER,argv,renderer.SCRIPT,renderer.HELPER,tuple(sorted((*pins.items(),WRAPPER))),
                     checkpoints,((InstancePlan('bootstrap',20261010,0,horizon),InstancePlan('reset',100,0,900)),))
    result=collect_ui(session,stage,plan)
    renderer.command_policy(tuple(cps),commands)
    for key in ('observation','clock','tallies','commands'):cps[-1].field(key).equals(cps[-2].field(key).value)
    for state in result.validated.fresh_observations():require(state.founders==10,'/census','unexpected renderer founders')
    return result


def collect_pack(session: Node, stage: Node) -> WholeUIEvidence:
    root,log,path,godot,raw=_select(session,stage,36,'data_only_ext','pack.ui')
    cps=raw.field('checkpoints').array();require(len(cps)==5,'/checkpoints','incomplete selected-pack UI')
    horizon=integer(cps[1].field('clock').field('tick'))
    context={'manifest_path':str(root/pack.PACK),'manifest_sha256':pack.PINS[pack.PACK],
             'fixture_classification':'ordinary_selected_pack','synthetic_DTO_checks':'presentation_validation_only'}
    checkpoints=(CheckpointPlan('bootstrap',0,1,0),CheckpointPlan('paused_selected_pack',0,1,horizon,json.dumps(context)),
                 CheckpointPlan('before_explicit_tick',0,1,horizon),CheckpointPlan('after_explicit_tick',0,1,horizon+1),
                 CheckpointPlan('terminal',0,1,horizon+1,releases_node=True))
    argv=(godot,'--headless','--path',str(root),'--script','res://'+pack.SCRIPT,'--',str(log/'ui'))
    pins={**pack.PINS,'tools/v8/data_only_ext.sh':'0f27e5d899e359a1a891682d1afbd3ce2090ebb8251ed65556bf5a59c3e2234e'}
    plan=WholeUIPlan(path,'pack.ui',argv,'res://'+pack.SCRIPT,pack.HELPER,tuple(sorted((*pins.items(),WRAPPER))),
                     checkpoints,((InstancePlan('bootstrap',20261010,0,horizon+1),),))
    result=collect_ui(session,stage,plan);pack.commands(raw)
    report_path=log/'ui/data-only-ext-results.json';_artifact(stage,report_path)
    report=document(report_path.read_text());report.field('failures').equals(0)
    report.field('selected_pack').equals(str(root/pack.PACK));report.field('rendered').equals(False)
    checks=report.field('checks').array();assertions=[r for r in checks if 'passed' in r.mapping()]
    require(len(assertions)>=20,'/oracle','selected-pack assertions missing')
    for assertion in assertions:assertion.field('passed').equals(True)
    require(len([r for r in checks if r.field('check').value=='rendered capture' and
                 r.field('status').value=='NOT_RUN_HEADLESS'])==1,'/oracle','headless status missing')
    for state in result.validated.fresh_observations():require(state.founders==10,'/census','unexpected pack founders')
    return result


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> tuple[Json,...]:
    base.field('arm').equals('base');candidate.field('arm').equals('candidate')
    step=integer(base.field('step'));candidate.field('step').equals(step)
    require(step in (23,36),'/step','unsupported selected UI family')
    collect=collect_renderer if step==23 else collect_pack
    a,b=collect(session,base),collect(session,candidate)
    pairs=list(zip(a.validated.fresh_observations(),b.validated.fresh_observations(),strict=True))
    if step==36:
        prefixes: list[Observation]=[]
        for observed in (a,b):
            cp=document(observed.plan.sidecar.read_text()).field('checkpoints').array()[1]
            prefix=parse_native(json.dumps(cp.field('observation').value),unsigned_string(cp.field('node_instance_id')))[0]
            prefixes.append(require_fresh_history(prefix))
        pairs.append((prefixes[0],prefixes[1]))
    rows: list[Json]=[]
    for left,right in pairs:
        require(left.terminal_tick==right.terminal_tick,'/pair','UNRESOLVED: unequal actual UI horizons')
        rows.append(ordinary_pair(left,right))
    return tuple(rows)
