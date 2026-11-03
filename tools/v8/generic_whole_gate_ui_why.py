# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""Source-owned original40 Why capture; independent of locale regressions46."""
import json
from pathlib import Path

from generic_gate import boundary, document, require, text
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_ui_observations import parse_native, require_fresh_history, unsigned_string
from mux2_gate_inputs import Json, Node
from mux2_gate_visual import validate_why

import generic_ui_why_run as why
from generic_whole_gate_ui import WholeUIPlan, bind_outer, check_namespace, collect_ui
from generic_whole_gate_ui_fixed import WRAPPER
from generic_whole_gate_ui_visibility import CAPTURE, VISUAL


def collect_why(session: Node, stage: Node) -> tuple[tuple[int,int,int],tuple[Observation,...]]:
    stage.field('step').equals(40);stage.field('name').equals('mux2_why')
    argv=tuple(text(v) for v in stage.field('argv').array())
    require(len(argv)==9 and argv[:6]==('python3',CAPTURE[0],'capture','--kind','why','--out') and argv[7]=='--godot',
            '/argv','foreign original Why capture command')
    root,_=bind_outer(session,stage);out=Path(argv[6]);godot=argv[8]
    require(out.is_absolute() and godot in session.field('external_inputs').mapping(),'/capture','unbound Why paths')
    env=stage.field('environment');env.field('GODOT_BIN').equals(godot)
    directory=Path(text(env.field('WORLDSIM_GATE_UI_OUT')));check_namespace(stage,directory)
    paths=tuple(directory.glob('*.json'));require(len(paths)==1,'/ui','missing/extra Why sidecar')
    raw=document(paths[0].read_text());horizon,args=why.mechanism(raw)
    checkpoints,instances=why.lifecycle_plan(horizon,args)
    native=(godot,'--path',str(root),'--resolution','1920x1080','--script','res://'+why.PARENT)
    plan=WholeUIPlan(paths[0],'why',native,'res://'+why.PARENT,why.HELPER,
                     tuple(sorted((*why.PINS.items(),WRAPPER,CAPTURE,VISUAL))),checkpoints,instances)
    evidence=collect_ui(session,stage,plan);why.commands(raw)
    receipt=out/'receipt.json';stage.field('observer_artifacts').field(str(receipt)).equals(file_hash(receipt))
    capture=document(receipt.read_text())
    for key,value in (('schema',1),('kind','why'),('complete',True),('exit_code',0),('root',str(root)),('godot',godot)):
        capture.field(key).equals(value)
    capture.field('godot_sha256').equals(session.field('external_inputs').field(godot).value)
    capture.field('source_head').equals(session.field('arms').field(text(stage.field('arm'))).field('source_sha').value)
    for name,digest in capture.field('artifacts').mapping().items():
        stage.field('observer_artifacts').field(name).equals(Node(digest).value)
        require(file_hash(Path(name))==Node(digest).text(),'/capture','Why capture artifact changed')
    log=out/'command.log';stage.field('observer_artifacts').field(str(log)).equals(file_hash(log))
    commands=[document(line).value for line in log.read_text().splitlines() if line.startswith('["bash",')]
    require(commands==[['bash',str(root/WRAPPER[0]),'180',*native]],'/capture','original Why argv differs')
    for image in validate_why(log):
        stage.field('observer_artifacts').field(str(image)).equals(file_hash(image))
    cp=raw.field('checkpoints').array()[1]
    prefix=parse_native(json.dumps(cp.field('observation').value),unsigned_string(cp.field('node_instance_id')))[0]
    states=(require_fresh_history(prefix),*evidence.validated.fresh_observations())
    require(len(states)==2 and all(state.founders==10 for state in states),'/coverage','incomplete Why native coverage')
    check_namespace(stage,directory)
    return args,states


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> tuple[Json,...]:
    base.field('arm').equals('base');candidate.field('arm').equals('candidate')
    left,a=collect_why(session,base);right,b=collect_why(session,candidate)
    require(left==right,'/intervention','different original Why native intervention')
    rows: list[Json]=[]
    for x,y in zip(a,b,strict=True):
        require(x.terminal_tick==y.terminal_tick,'/horizon','UNRESOLVED: original Why horizons differ')
        rows.append(ordinary_pair(x,y))
    return tuple(rows)
