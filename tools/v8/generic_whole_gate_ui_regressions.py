# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""New-H step46 native sidecars with independent locale capture namespaces."""
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import boundary, document, require, text
from generic_observations import ordinary_pair
from generic_ui_observations import (
    UIInstance,
    parse_native,
    require_fresh_history,
    unsigned_string,
)
from mux2_gate_inputs import Json, Node

import generic_ui_fixed_run as fixed
import generic_ui_why_run as why
from generic_whole_gate_ui import (
    WholeUIEvidence,
    WholeUIPlan,
    bind_outer,
    check_namespace,
    collect_ui,
)
from generic_whole_gate_ui_fixed import WRAPPER

COMMAND: Final = 'ab99605c49b3b4958e31a6475da7b7d2dd1e1e331b3cd1773a9f4d4fd487c601'
EXPECTED: Final = (('toast',''),('marker.layout',''),('why','en'),('why','ko'))
DIRECTORIES: Final = {('toast',''):'toast', ('marker.layout',''):'layout', ('why','en'):'why-en', ('why','ko'):'why-ko'}


@dataclass(frozen=True, slots=True)
class Regression:
    key: tuple[str,str]
    ui: WholeUIEvidence
    prefix: UIInstance | None
    args: tuple[int,int,int] | None
    commands: tuple[fixed.CommandRows, ...] | None


def _kind(raw: Node) -> tuple[str,str]:
    producer=text(raw.field('producer'))
    if producer=='why':
        args=tuple(text(v) for v in raw.field('user_args').array())
        require(args in (('--locale=en',),('--locale=ko',)), '/locale', 'foreign locale invocation')
        return producer,args[0].removeprefix('--locale=')
    return producer,''


def collect_regressions(session: Node, stage: Node) -> tuple[Regression, ...]:
    """Retain all four actual emitted sidecars with exact source-owned commands."""
    stage.field('step').equals(46)
    stage.field('name').equals('mux21_regressions')
    argv=tuple(text(v) for v in stage.field('argv').array())
    require(len(argv)==4 and argv[:2]==('bash','-c') and hashlib.sha256(argv[2].encode()).hexdigest()==COMMAND,
            '/argv','unreviewed regression source command')
    _,_=bind_outer(session,stage)
    env=stage.field('environment')
    require(not any(k.startswith(('WORLDSIM_ORACLE_','WORLDSIM_STALL_')) for k in env.mapping()),
            '/environment','alternate child protocol requires dedicated journal')
    directory=Path(text(env.field('WORLDSIM_GATE_UI_OUT')))
    check_namespace(stage,directory)
    require({p.name for p in directory.iterdir()}==set(DIRECTORIES.values()),
            '/ui','missing/foreign source-owned child output directory')
    indexed: dict[tuple[str,str],Path]={}
    for expected,child_name in DIRECTORIES.items():
        paths=tuple((directory/child_name).glob('*.json'))
        require(len(paths)==1,'/ui','missing/extra native invocation sidecar')
        path=paths[0]
        require(_kind(document(path.read_text()))==expected,'/ui','foreign locale or producer subtree')
        indexed[expected]=path
    godot=text(env.field('GODOT_BIN'))
    require(godot in session.field('external_inputs').mapping(),'/godot','Godot lacks independent session hash')
    result: list[Regression]=[]
    for producer,locale in EXPECTED:
        path=indexed[(producer,locale)];raw=document(path.read_text())
        if producer=='why':
            horizon,args=why.mechanism(raw)
            checkpoints,instances=why.lifecycle_plan(horizon,args)
            native_argv=(godot,'--path','.', '--resolution','1920x1080','--script','res://'+why.LOCALE,'--','--locale='+locale)
            plan=WholeUIPlan(path,producer,native_argv,'res://'+why.PARENT,why.HELPER,
                             tuple(sorted((*why.PINS.items(),WRAPPER))),checkpoints,instances)
            observed=collect_ui(session,stage,plan)
            why.commands(raw)
            for folder, names in (('why',('why-1.0.png','why-1.25.png','why-1.5.png')),
                                  ('placement',('why-placement-1.0.png','why-placement-1.25.png'))):
                images=path.parent/folder
                require({p.name for p in images.iterdir()}==set(names),'/captures','incomplete/foreign locale capture set')
                for name in names:
                    require((images/name).read_bytes().startswith(b'\x89PNG\r\n\x1a\n'),'/captures','invalid original PNG')
            cp=raw.field('checkpoints').array()[1]
            prefix=parse_native(json.dumps(cp.field('observation').value),unsigned_string(cp.field('node_instance_id')))[0]
            _=require_fresh_history(prefix)
            require(prefix.state.founders==10,'/census','unexpected Why prefix population')
            result.append(Regression((producer,locale),observed,prefix,args,None))
        else:
            lifecycle=fixed.lifecycle_plan(producer,0)
            script='res://tools/v8/'+fixed.definition(producer).script
            native_argv=(godot,'--path','.', '--rendering-method','gl_compatibility','--script',script,'--','--cache') if producer=='marker.layout' else (godot,'--headless','--path','.', '--script',script)
            plan=WholeUIPlan(path,producer,native_argv,script,fixed.HELPER,tuple(sorted((*fixed.PINS.items(),WRAPPER))),
                             lifecycle.checkpoints,lifecycle.instances,lifecycle.classification,no_native_reason=lifecycle.no_native_reason)
            observed=collect_ui(session,stage,plan)
            commands=fixed.command_policy(plan,tuple(cp.field('commands') for cp in raw.field('checkpoints').array()))
            result.append(Regression((producer,locale),observed,None,None,commands))
        require(not observed.validated.loaded_unqualified,'/history','unexpected regression load')
        for node in observed.validated.instances:
            for instance in node:
                _=require_fresh_history(instance)
                require(instance.state.founders==10,'/census','unexpected regression founder population')
    check_namespace(stage,directory)
    return tuple(result)


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> tuple[Json, ...]:
    """Pair exact native worlds and independently retained locale capture sets."""
    base.field('arm').equals('base');candidate.field('arm').equals('candidate')
    rows: list[Json]=[]
    for a,b in zip(collect_regressions(session,base),collect_regressions(session,candidate),strict=True):
        require(a.key==b.key and a.args==b.args and a.commands==b.commands,'/pair','regression inputs differ')
        if a.prefix is not None and b.prefix is not None:
            require(a.prefix.state.terminal_tick==b.prefix.state.terminal_tick,'/pair','UNRESOLVED: Why prefix horizons differ')
            rows.append(ordinary_pair(a.prefix.state,b.prefix.state))
        for an,bn in zip(a.ui.validated.instances,b.ui.validated.instances,strict=True):
            for x,y in zip(an,bn,strict=True):
                require(x.state.terminal_tick==y.state.terminal_tick,'/pair','UNRESOLVED: actual regression horizons differ')
                rows.append(ordinary_pair(x.state,y.state))
    return tuple(rows)
