# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: invoked by generic_whole_gate.py after both original step9 returns.
"""Actual direct Godot companions, separate from immutable original smoke stages."""
import json
from dataclasses import dataclass, replace
from pathlib import Path

from generic_gate import document, require
from generic_observations import Observation, ordinary_pair
from generic_process import ProcessSpec, atomic_json, execute, file_hash
from generic_ui_companion import native_commands
from generic_ui_smoke_run import HELPER, SCRIPTS
from generic_whole_gate_contract import current_product_inputs, load_session
from generic_whole_gate_environment import process_spec
from generic_whole_gate_journal import anchored, entries, recheck
from mux2_gate_inputs import Json, Node

from generic_ui_receipts import CheckpointPlan, InstancePlan, UIPlan, validate_ui
from generic_whole_gate_ui_smoke import Smoke, collect_smoke


@dataclass(frozen=True, slots=True)
class CompanionResult:
    code: int
    output: Path


def originals(session: Node) -> dict[tuple[str,str], Smoke]:
    result: dict[tuple[str,str], Smoke]={}
    for arm in ('base','candidate'):
        stages=[row for row in entries(session,arm) if row.field('step').value==9]
        require(len(stages)==1,'/smoke','exactly one original stage9 required')
        for original in collect_smoke(session,stages[0]):
            result[arm,original.ui.plan.producer]=original
    return result


def companion_spec(session: Node, arm: str, original: Smoke, output: Path,
                   descriptor: int | None) -> ProcessSpec:
    """Derive a real direct invocation, preserving A's bound source/env/lane FD."""
    bridge=current_product_inputs(session,arm,'bridge')
    godot=Path(session.field('environment').field('GODOT_BIN').text())
    digest=session.field('external_inputs').field(str(godot)).text()
    tick=original.ui.validated.fresh_observations()[0].terminal_tick
    argv=(str(godot),'--headless','--path','.', '--script',SCRIPTS['smoke.companion'],'--',
          '--ticks',str(tick),'--original-producer',original.ui.plan.producer,
          '--original-sidecar-sha256',original.original_sha256)
    additions={'WORLDSIM_GATE_OBSERVER':'1','WORLDSIM_GATE_UI_OUT':str(output),
               'WORLDSIM_GATE_INVOCATION_ID':'smoke.companion:'+str(output)}
    wrapped=process_spec(session,arm,argv,output,additions,descriptor)
    environment=dict(wrapped.environment)
    environment['WORLDSIM_TEST_SETTINGS_DIR']=str(output/'settings')
    inputs=dict(wrapped.inputs)
    for name,value in bridge.field('runtime_inputs').mapping().items():
        relative=Path(name).relative_to(wrapped.input_root).as_posix()
        expected=Node(value).text()
        require(relative not in inputs or inputs[relative]==expected,'/bridge','conflicting actual build endpoint')
        inputs[relative]=expected
    return replace(wrapped,argv=argv,executable=godot,executable_sha256=digest,
                   inputs=tuple(sorted(inputs.items())),environment=tuple(sorted(environment.items())),
                   fresh_directories=('settings',),timeout_seconds=600)


def companion_plan(spec: ProcessSpec, original: Smoke) -> UIPlan:
    tick=original.ui.validated.fresh_observations()[0].terminal_tick
    context=json.dumps({'requested_tick':tick,'original_producer':original.ui.plan.producer,
                        'original_sidecar_sha256':original.original_sha256})
    return UIPlan('smoke.companion','smoke.companion:'+str(spec.output_root),spec,
                  SCRIPTS['smoke.companion'],HELPER,
                  (CheckpointPlan('bootstrap',0,1,0),CheckpointPlan('companion_terminal',0,1,tick,context,True)),
                  ((InstancePlan('bootstrap',20261010,0,tick),),))


def command_history(path: Path) -> tuple[tuple[int,str,int], ...]:
    previous: tuple[tuple[int,str,int], ...]=()
    for n,cp in enumerate(document(path.read_text()).field('checkpoints').array()):
        current=native_commands(cp.field('commands'),int(cp.field('clock').field('tick').number()),n==0)
        require(current[:len(previous)]==previous,'/commands','native prefix changed')
        previous=current
    return previous


def run_companions(path: Path, digest: str, descriptor: int | None = None) -> CompanionResult:
    """Never rerun originals; preserve partial/unknown evidence and real native waits."""
    session=load_session(path,digest)
    original=originals(session)
    output=Path(session.field('output').text())/'auxiliaries'/'smoke'
    output.mkdir(parents=True,exist_ok=False)
    requests: list[tuple[str,str,str,str]]=[]
    for arm in ('base','candidate'):
        for producer in ('smoke.boot','smoke.labels'):
            requests.append((arm,arm,producer,'own'))
    for producer in ('smoke.boot','smoke.labels'):
        left=original['base',producer].ui.validated.fresh_observations()[0]
        right=original['candidate',producer].ui.validated.fresh_observations()[0]
        if left.terminal_tick != right.terminal_tick:
            requests.extend((('candidate','base',producer,'matched'),('base','candidate',producer,'matched')))
    rows: list[Json]=[]
    states: dict[tuple[str,str,str],Observation]={}
    code=0
    for arm,origin,producer,role in requests:
        actual=original[origin,producer]
        folder=output/(arm+'-'+producer.removeprefix('smoke.')+'-'+role)
        spec=companion_spec(session,arm,actual,folder,descriptor)
        result=execute(spec)
        row: dict[str,Json]={'arm':arm,'origin_arm':origin,'producer':producer,'role':role,
                            'original_sha256':actual.original_sha256,'process':str(result.receipt),
                            'process_sha256':file_hash(result.receipt),'status':result.status,
                            'native_exit':result.native_exit,'pid':result.pid,'anchor':anchored(folder)}
        rows.append(row)
        if result.native_exit is None or result.native_exit < 0:
            code=2
            break
        if result.status!='PASS' or result.native_exit!=0:
            code=1
            break
        sidecar=folder/f'smoke.companion-{result.pid}.json'
        validated=validate_ui(sidecar,file_hash(sidecar),companion_plan(spec,actual),result,file_hash(result.receipt))
        require(not validated.loaded_unqualified and not validated.instances[0][0].closed,
                '/companion','unexpected native history')
        state=validated.fresh_observations()[0]
        require(state.founders==10,'/companion','unexpected population')
        if role=='own':
            left,right=actual.ui.validated.instances[0][0],validated.instances[0][0]
            require((left.seed,left.initial_agent_ids,left.terminal_agent_ids,left.state)==
                    (right.seed,right.initial_agent_ids,right.terminal_agent_ids,right.state),
                    '/companion','original native history differs')
            require(command_history(sidecar)==command_history(actual.ui.plan.sidecar),'/companion','original native commands differ')
        states[arm,producer,role]=state
        _=load_session(path,digest)
        _=originals(session)
    comparisons: list[Json]=[]
    if code==0:
        for producer in ('smoke.boot','smoke.labels'):
            left=original['base',producer].ui.validated.fresh_observations()[0]
            right=original['candidate',producer].ui.validated.fresh_observations()[0]
            if left.terminal_tick==right.terminal_tick:
                comparisons.append(ordinary_pair(left,right))
            else:
                comparisons.append(ordinary_pair(left,states['candidate',producer,'matched']))
                comparisons.append(ordinary_pair(states['base',producer,'matched'],right))
    for raw in rows:
        retained=Node(raw)
        recheck(Path(retained.field('process').text()).parent,retained.field('anchor'))
    terminal=output/'companions.json'
    atomic_json(terminal,{'schema':'whole_gate_smoke_companions.v1','status':'PASS' if code==0 else 'UNKNOWN' if code==2 else 'FAIL',
                          'invocations':rows,'comparisons':comparisons,'full_gate_claim':False})
    return CompanionResult(code,output)
