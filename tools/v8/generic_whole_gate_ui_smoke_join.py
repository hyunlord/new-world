# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: called by whole-gate acceptance with A's original auxiliary anchor.
"""Revalidate actual independently waited companions against original smoke worlds."""
import os
from pathlib import Path

from generic_gate import boundary, document, integer, require
from generic_observations import Observation, ordinary_pair
from generic_process import ProcessResult, file_hash
from generic_whole_gate_journal import recheck
from mux2_gate_inputs import Json, Node

from generic_ui_receipts import validate_ui
from generic_whole_gate_ui_companions import (
    command_history,
    companion_plan,
    companion_spec,
    originals,
)


@boundary
def read_returned_process(receipt: Path, expected_sha256: str) -> ProcessResult:
    """Deserialize original anchored execute evidence without inferring a descendant."""
    require(file_hash(receipt)==expected_sha256,'/process','actual receipt changed')
    process=document(receipt.read_text())
    process.field('status').equals('PASS');process.field('native_exit').equals(0)
    pid=integer(process.field('pid'));require(pid>0,'/pid','missing actual native PID')
    return ProcessResult(receipt,'PASS',0,pid)


@boundary
def evaluate_family(session: Node, auxiliary_anchor: Node, descriptor: int | None = None) -> tuple[Json,...]:
    """Anchor must come from A's actual auxiliary completion journal, never recaptured."""
    output=Path(session.field('output').text())/'auxiliaries'/'smoke'
    recheck(output,auxiliary_anchor)
    terminal=document((output/'companions.json').read_text())
    terminal.field('schema').equals('whole_gate_smoke_companions.v1');terminal.field('status').equals('PASS')
    original=originals(session)
    expected={(arm,arm,producer,'own') for arm in ('base','candidate') for producer in ('smoke.boot','smoke.labels')}
    for producer in ('smoke.boot','smoke.labels'):
        if original['base',producer].ui.plan.instances[0][0].terminal_tick != original['candidate',producer].ui.plan.instances[0][0].terminal_tick:
            expected.update((('candidate','base',producer,'matched'),('base','candidate',producer,'matched')))
    seen: set[tuple[str,str,str,str]]=set()
    states: dict[tuple[str,str,str],Observation]={}
    for row in terminal.field('invocations').array():
        arm,origin,producer,role=(row.field(k).text() for k in ('arm','origin_arm','producer','role'))
        key=(arm,origin,producer,role)
        require(key in expected and key not in seen,'/companion','foreign/duplicate actual invocation');seen.add(key)
        actual=original[origin,producer]
        row.field('original_sha256').equals(actual.original_sha256)
        folder=output/(arm+'-'+producer.removeprefix('smoke.')+'-'+role)
        recheck(folder,row.field('anchor'))
        receipt=folder/'process.json';row.field('process').equals(str(receipt))
        row.field('process_sha256').equals(file_hash(receipt))
        process=document(receipt.read_text());process.field('status').equals('PASS');process.field('native_exit').equals(0)
        result=read_returned_process(receipt,row.field('process_sha256').text())
        pid=result.pid
        row.field('pid').equals(pid);row.field('native_exit').equals(0);row.field('status').equals('PASS')
        # This is deserialization of A-anchored actual execute() evidence, never a child inferred from Bash.
        spec=companion_spec(session,arm,actual,folder,descriptor)
        if spec.pass_fds:
            process.field('pass_fds').equals(list(spec.pass_fds))
            identities: dict[str,Json]={}
            for fd in spec.pass_fds:
                info=os.fstat(fd)
                identities[str(fd)]=[info.st_dev,info.st_ino,info.st_mode]
            process.field('descriptor_identities').equals(identities)
        else:
            require('pass_fds' not in process.mapping() and 'descriptor_identities' not in process.mapping(),
                    '/owner','original retained descriptor was omitted')
        sidecar=folder/f'smoke.companion-{pid}.json'
        validated=validate_ui(sidecar,file_hash(sidecar),companion_plan(spec,actual),result,file_hash(receipt))
        require(not validated.loaded_unqualified and not validated.instances[0][0].closed,'/history','unexpected companion lifecycle')
        state=validated.fresh_observations()[0];require(state.founders==10,'/census','unexpected companion founders')
        if role=='own':
            left,right=actual.ui.validated.instances[0][0],validated.instances[0][0]
            require((left.seed,left.initial_agent_ids,left.terminal_agent_ids,left.state)==
                    (right.seed,right.initial_agent_ids,right.terminal_agent_ids,right.state),'/history','original/own state differs')
            require(command_history(sidecar)==command_history(actual.ui.plan.sidecar),'/commands','original/own command differs')
        states[arm,producer,role]=state
    require(seen==expected,'/companion','missing source-required actual invocation')
    rows: list[Json]=[]
    for producer in ('smoke.boot','smoke.labels'):
        left=original['base',producer].ui.validated.fresh_observations()[0]
        right=original['candidate',producer].ui.validated.fresh_observations()[0]
        if left.terminal_tick==right.terminal_tick:rows.append(ordinary_pair(left,right))
        else:
            rows.append(ordinary_pair(left,states['candidate',producer,'matched']))
            rows.append(ordinary_pair(states['base',producer,'matched'],right))
    terminal.field('comparisons').equals(list(rows))
    recheck(output,auxiliary_anchor)
    return tuple(rows)
