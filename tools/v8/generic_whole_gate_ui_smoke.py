# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""Original step9 UI worlds; unequal horizons require actual supplemental runs."""
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import boundary, document, integer, require, text
from generic_observations import ordinary_pair
from generic_ui_companion import native_commands
from generic_ui_smoke_run import HELPER, PINS, SCRIPTS
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

PRODUCERS: Final = ('smoke.boot','smoke.labels')


@dataclass(frozen=True, slots=True)
class Smoke:
    ui: WholeUIEvidence
    original_sha256: str


def collect_smoke(session: Node, stage: Node) -> tuple[Smoke, ...]:
    """Select exactly original900-frame boot and labels without replaying either."""
    stage.field('step').equals(9);stage.field('name').equals('smoke')
    argv=tuple(text(v) for v in stage.field('argv').array())
    require(len(argv)==3 and argv[:2]==('bash','tools/v8/smoke.sh'),'/argv','foreign original smoke command')
    _,_=bind_outer(session,stage)
    env=stage.field('environment')
    env.field('WORLDSIM_GATE_UI_BOOT').equals('1')
    directory=Path(text(env.field('WORLDSIM_GATE_UI_OUT')))
    check_namespace(stage,directory)
    paths=tuple(directory.iterdir())
    require(len(paths)==2 and all(p.is_file() for p in paths),'/ui','missing/extra original smoke sidecars')
    index: dict[str,Path]={}
    for path in paths:
        producer=text(document(path.read_text()).field('producer'))
        require(producer in PRODUCERS and producer not in index,'/ui','foreign/duplicate original smoke')
        index[producer]=path
    godot=text(env.field('GODOT_BIN'))
    require(godot in session.field('external_inputs').mapping(),'/godot','Godot lacks source-independent hash')
    result: list[Smoke]=[]
    for producer in PRODUCERS:
        path=index[producer];raw=document(path.read_text());rows=raw.field('checkpoints').array()
        labels=('bootstrap','native_tree_exit') if producer=='smoke.boot' else ('bootstrap','initial_frames_complete','terminal')
        require(len(rows)==len(labels),'/checkpoints','incomplete original smoke lifecycle')
        ticks=tuple(integer(row.field('clock').field('tick')) for row in rows)
        require(ticks[0]==0 and tuple(sorted(ticks))==ticks and ticks[-1]<=200000,'/ticks','invalid actual smoke horizon')
        checkpoints=tuple(CheckpointPlan(label,0,1,ticks[n],'{"requested_frames":900}' if label=='initial_frames_complete' else '{}',n==len(labels)-1)
                          for n,label in enumerate(labels))
        native_argv=(godot,'--headless','--path','.')+ (('--quit-after','900') if producer=='smoke.boot' else
                    ('--script',SCRIPTS[producer],'--','--frames','900'))
        plan=WholeUIPlan(path,producer,native_argv,SCRIPTS[producer],HELPER,tuple(sorted((*PINS.items(),WRAPPER))),
                         checkpoints,((InstancePlan('bootstrap',20261010,0,ticks[-1]),),),
                         completion='native_tree_exit' if producer=='smoke.boot' else 'producer_finish')
        observed=collect_ui(session,stage,plan)
        require(not observed.validated.loaded_unqualified,'/history','unexpected smoke load')
        previous: tuple[tuple[int,str,int], ...]=()
        for n,cp in enumerate(rows):
            current=native_commands(cp.field('commands'),ticks[n],n==0)
            require(current[:len(previous)]==previous,'/commands','native command prefix changed')
            previous=current
        require(not observed.validated.instances[0][0].closed,'/history','unexpected closed smoke instance')
        state=observed.validated.fresh_observations()[0]
        require(state.founders==10,'/census','unexpected smoke founder count')
        result.append(Smoke(observed,observed.sidecar_sha256))
    return tuple(result)


@boundary
def evaluate_equal_horizons(session: Node, base: Node, candidate: Node) -> tuple[Json, ...]:
    """Only equal actual horizons; this API never substitutes for missing companions."""
    base.field('arm').equals('base');candidate.field('arm').equals('candidate')
    rows: list[Json]=[]
    for a,b in zip(collect_smoke(session,base),collect_smoke(session,candidate),strict=True):
        require(a.ui.plan.producer==b.ui.plan.producer,'/pair','different smoke occurrence')
        left,right=a.ui.validated.fresh_observations()[0],b.ui.validated.fresh_observations()[0]
        require(left.terminal_tick==right.terminal_tick,'/pair','UNRESOLVED: actual matched-horizon smoke companions required')
        rows.append(ordinary_pair(left,right))
    return tuple(rows)
