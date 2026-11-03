# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""Source-owned original step34 three-producer lifecycle and safety join."""
import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from generic_gate import boundary, document, integer, require, text
from generic_observations import ordinary_pair
from generic_ui_observations import require_fresh_history
from mux2_gate_inputs import Json, Node

from generic_ui_fixed_run import (
    HELPER,
    PINS,
    CommandRows,
    command_policy,
    definition,
    lifecycle_plan,
)
from generic_whole_gate_ui import (
    WholeUIEvidence,
    WholeUIPlan,
    bind_outer,
    check_namespace,
    collect_ui,
)

RENDER_COMMAND: Final = '904cf9c187415cdd517a0b8aa09b6495ac2ca6b4f2b4c2f096cd68902d3d352d'
WRAPPER: Final = ('tools/v8/run_bounded.sh', '64b0d037c1a854add96cccf25ae7a0bcfc40be78ab4406f092514e4891009bf7')
PRODUCERS: Final = ('render.ground','render.people','render.trees')


@dataclass(frozen=True, slots=True)
class FixedEvidence:
    ui: WholeUIEvidence
    commands: tuple[CommandRows, ...]


def collect_fixed(session: Node, stage: Node) -> tuple[FixedEvidence, ...]:
    """Select exactly the original source-bound step34 sequence, not caller plans."""
    stage.field('step').equals(34)
    stage.field('name').equals('mart2_render')
    argv = tuple(text(v) for v in stage.field('argv').array())
    require(len(argv)==4 and argv[:2]==('bash','-c') and
            hashlib.sha256(argv[2].encode()).hexdigest()==RENDER_COMMAND,
            '/argv', 'unreviewed render aggregate source command')
    root,_ = bind_outer(session,stage)
    environment = stage.field('environment')
    require(not any(name.startswith(('WORLDSIM_ORACLE_', 'WORLDSIM_STALL_')) for name in environment.mapping()),
            '/environment', 'dedicated child protocol requires its own journal join')
    directory = Path(text(environment.field('WORLDSIM_GATE_UI_OUT')))
    check_namespace(stage,directory)
    paths = sorted(directory.iterdir())
    require(len(paths)==len(PRODUCERS) and all(p.is_file() for p in paths), '/ui', 'missing/extra render producer')
    indexed: dict[str, Path] = {}
    for path in paths:
        producer = text(document(path.read_text()).field('producer'))
        require(producer in PRODUCERS and producer not in indexed, '/ui', 'foreign/duplicate render occurrence')
        indexed[producer] = path
    result: list[FixedEvidence] = []
    godot = text(environment.field('GODOT_BIN'))
    external = session.field('external_inputs').mapping()
    require(godot in external, '/godot', 'Godot executable lacks independent session binding')
    for producer in PRODUCERS:
        path = indexed[producer]
        raw = document(path.read_text())
        checkpoints = raw.field('checkpoints').array()
        require(len(checkpoints)>1, '/ui', 'missing pre-reset bootstrap checkpoint')
        bootstrap_tick = integer(checkpoints[1].field('clock').field('tick'))
        lifecycle = lifecycle_plan(producer,bootstrap_tick)
        script = 'res://tools/v8/'+definition(producer).script
        plan = WholeUIPlan(path,producer,(godot,'--headless','--path','.', '--script',script),
                           script,HELPER,tuple(sorted((*PINS.items(), WRAPPER))),lifecycle.checkpoints,
                           lifecycle.instances,lifecycle.classification,no_native_reason=lifecycle.no_native_reason)
        observed = collect_ui(session,stage,plan)
        require(not observed.validated.loaded_unqualified, '/history', 'unexpected render load')
        commands = command_policy(plan,tuple(cp.field('commands') for cp in checkpoints))
        for node in observed.validated.instances:
            for instance in node:
                _ = require_fresh_history(instance)
                require(instance.state.founders==(1000 if instance.reason=='stress_reset' else 10), '/census', 'unexpected render founders')
        result.append(FixedEvidence(observed,commands))
    require(root==Path(text(stage.field('cwd'))), '/cwd', 'aggregate project mismatch')
    check_namespace(stage,directory)
    return tuple(result)


@boundary
def evaluate_family(session: Node, base: Node, candidate: Node) -> tuple[Json, ...]:
    """Pair complete step34; differing bootstrap horizons are still unresolved."""
    base.field('arm').equals('base')
    candidate.field('arm').equals('candidate')
    left,right = collect_fixed(session,base),collect_fixed(session,candidate)
    rows: list[Json] = []
    for a,b in zip(left,right,strict=True):
        require(a.ui.plan.producer==b.ui.plan.producer and a.commands==b.commands, '/pair', 'render occurrence/commands differ')
        for an,bn in zip(a.ui.validated.instances,b.ui.validated.instances,strict=True):
            require(len(an)==len(bn), '/pair', 'native retained instance coverage differs')
            for x,y in zip(an,bn,strict=True):
                require(x.state.terminal_tick==y.state.terminal_tick, '/pair', 'UNRESOLVED: unequal actual bootstrap horizons')
                require((x.reason,x.seed,x.initial_agent_ids,x.terminal_agent_ids,x.state)==
                        (y.reason,y.seed,y.initial_agent_ids,y.terminal_agent_ids,y.state), '/pair', 'render native census/history differs')
                if y.reason=='stress_reset':
                    require(not y.state.knowledge_enabled and y.state.alive==1000, '/stress', 'stress population/capability differs')
                else:
                    rows.append(ordinary_pair(x.state,y.state))
    return tuple(rows)
