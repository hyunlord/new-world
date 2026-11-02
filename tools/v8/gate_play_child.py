"""Opt-in launcher child receipts, preserving raw outcomes and merged streams.

A caller-hashed plan admits exact command vectors, not arbitrary shell snippets.
This producer is not build attestation or historical observer qualification.
"""
import hashlib
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, replace
from pathlib import Path

import gate_play_protocol as protocol
from generic_gate import GateContractError, boundary, document, require, text
from generic_observations import keys
from generic_process import atomic_json, file_hash, real_path
from mux2_gate_inputs import Node

# Exact labels owned by the original full test_play.sh. Negative cases have no
# child except the unknown-version Godot version probe.
CASES = ('unknown version', 'GODOT_BIN=/nonexistent', 'bad --speed',
         '--headless-check main', '--headless-check m1', '--headless-check v7')


@dataclass(frozen=True, slots=True)
class Selection:
    suite: Path
    suite_sha256: str
    case: str
    plan: Path
    plan_sha256: str


@boundary
def select_case(path: Path, digest: str, case: str, role: str) -> Selection:
    """Select one immutable child plan; this grants no source/build authority."""
    real_path(path, directory=False)
    raw = path.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == digest, '/suite', 'suite hash differs')
    node = document(raw.decode('utf-8')); keys(node, {'schema_version', 'cases'})
    node.field('schema_version').equals(1)
    cases = node.field('cases'); keys(cases, set(CASES))
    require(case in CASES, '/case', 'unknown original case')
    plans: dict[str, tuple[Path, str]] = {}
    for label in CASES:
        row = cases.field(label)
        if label in CASES[1:3]:
            row.equals(None)
            continue
        keys(row, {'path', 'sha256'})
        plan = Path(text(row.field('path'))); checksum = text(row.field('sha256'))
        real_path(plan, directory=False)
        require(plan != path and file_hash(plan) == checksum, '/plan', 'case plan changed')
        plans[label] = (plan, checksum)
    require(len({plan for plan, _ in plans.values()}) == 4, '/plans', 'case plans must be distinct')
    require(case in plans, '/case', 'no child permitted for negative control')
    require(role == 'version' if case == CASES[0] else role in {'version', 'build', 'import-1', 'import-2', 'headless'},
            '/role', 'unexpected child for original case')
    plan, checksum = plans[case]
    return Selection(path, digest, case, plan, checksum)


@dataclass(frozen=True, slots=True)
class Child:
    argv: tuple[str,...]
    cwd: Path
    executable: Path
    executable_sha256: str
    inputs: tuple[tuple[Path,str],...]
    artifacts: tuple[Path,...]
    output: Path
    timeout: float | None
    merged: bool


@boundary
def admit(path: Path, digest: str, role: str, argv: tuple[str,...]) -> Child:
    raw=path.read_bytes()
    require(hashlib.sha256(raw).hexdigest()==digest,'/plan','plan hash differs from caller anchor')
    node=document(raw.decode('utf-8'));keys(node,{'schema_version','roles'})
    node.field('schema_version').equals(1)
    require(role in {'version','build','import-1','import-2','headless','windowed'},'/role','unknown child role')
    item=node.field('roles').field(role)
    keys(item,{'argv','cwd','executable','executable_sha256','inputs','artifacts','output','timeout_seconds','merged'})
    args=tuple(text(value) for value in item.field('argv').array())
    require(args==argv and bool(argv) and all('\0' not in value for value in argv),'/argv','unplanned command vector')
    cwd=Path(text(item.field('cwd')));executable=Path(text(item.field('executable')))
    real_path(cwd,directory=True);real_path(executable,directory=False)
    require(cwd==Path.cwd().resolve(),'/cwd','actual launcher child cwd differs')
    resolved=shutil.which(argv[0]);require(resolved is not None and Path(resolved).resolve()==executable,'/executable','command resolution differs')
    expected=text(item.field('executable_sha256'));require(file_hash(executable)==expected,'/executable','binary changed')
    inputs=tuple((Path(name),text(Node(value,'/inputs/'+name))) for name,value in item.field('inputs').mapping().items())
    for source,checksum in inputs:require(file_hash(source)==checksum,str(source),'input changed before launch')
    artifacts=tuple(Path(text(value)) for value in item.field('artifacts').array())
    require(len(set(artifacts))==len(artifacts) and all(p.is_absolute() and p==p.resolve() for p in artifacts),'/artifacts','noncanonical or duplicate artifacts')
    output=Path(text(item.field('output')))
    require(output.is_absolute() and output==output.resolve() and not output.exists(),'/output','fresh canonical output required')
    require(not output.is_relative_to(cwd) and not cwd.is_relative_to(output),'/output','output overlaps working tree')
    for source in (path.resolve(),executable,*(p for p,_ in inputs),*artifacts):
        require(not source.is_relative_to(output) and not output.is_relative_to(source),'/output','output overlaps bound input/artifact')
    timeout=item.field('timeout_seconds').value
    require(timeout is None or (isinstance(timeout,(float,int)) and not isinstance(timeout,bool) and 0<timeout<=86400),'/timeout','invalid timeout')
    assert timeout is None or isinstance(timeout,(float,int))
    merged=item.field('merged').value;require(type(merged) is bool,'/merged','boolean required')
    assert isinstance(merged,bool)
    require(timeout==(600 if role in {'import-1','import-2','headless'} else None),'/timeout','original launcher timeout differs')
    require(merged==(role!='version'),'/merged','original launcher stream routing differs')
    return Child(args,cwd,executable,expected,inputs,artifacts,output,timeout,merged)


def run(plan_path: Path, digest: str, role: str, argv: tuple[str,...], selection: Selection | None = None) -> int:
    if selection is not None:
        require(select_case(selection.suite, selection.suite_sha256, selection.case, role) == selection
                and (plan_path, digest) == (selection.plan, selection.plan_sha256), '/selection', 'case binding differs')
    child=admit(plan_path,digest,role,argv)
    if selection is not None:
        child=replace(child,inputs=(*child.inputs,(selection.suite,selection.suite_sha256)))
    wrapper_pid=os.getpid();parent_pid=os.getppid()
    if protocol.enabled():
        require(selection is not None, '/owner', 'owned child requires selected original case')
        protocol.child_request(role, argv, plan_path, digest)
    child.output.mkdir(parents=True,exist_ok=False)
    environment=dict(os.environ)
    started=time.time_ns();timed_out=False
    bounded=role in {'import-1','import-2','headless'}
    with (child.output/'stdout.log').open('xb') as stdout,(child.output/'stderr.log').open('xb') as stderr:
        process=subprocess.Popen(child.argv,executable=child.executable,cwd=child.cwd,env=environment,
                                 stdin=subprocess.DEVNULL if bounded else None,
                                 stdout=stdout,stderr=subprocess.STDOUT if child.merged else stderr)
        try:atomic_json(child.output/'started.json',{'schema_version':1,'role':role,'pid':process.pid,'wrapper_pid':wrapper_pid,'parent_pid':parent_pid,
            'argv':[a for a in child.argv],'cwd':str(child.cwd),'environment':{k:v for k,v in environment.items()},
            'started_ns':started,'plan_sha256':digest,'executable_sha256':child.executable_sha256})
        except OSError:
            process.kill();_=process.wait();raise
        try:code=process.wait(timeout=child.timeout)
        except subprocess.TimeoutExpired:
            timed_out=True;process.kill();code=process.wait()
            _=stdout.write(b'command exceeded the 600-second limit\n')
    errors: list[str]=[]
    observed: dict[str,str]={};artifacts: dict[str,str]={}
    try:
        require(file_hash(plan_path)==digest,'/plan','plan changed during child')
        require(file_hash(child.executable)==child.executable_sha256,'/executable','binary changed during child')
        for path,checksum in child.inputs:
            observed[str(path)]=file_hash(path)
            require(observed[str(path)]==checksum,str(path),'input changed during child')
        for path in child.artifacts:
            if path.exists():artifacts[str(path)]=file_hash(path)
    except (GateContractError,OSError) as error:errors.append(str(error))
    native_shell_exit=code % 256 if bounded else (128-code if code<0 else code)
    result=125 if errors else (124 if timed_out else native_shell_exit)
    atomic_json(child.output/'receipt.json',{'schema_version':1,'role':role,'plan_sha256':digest,
        'pid':process.pid,'wrapper_pid':wrapper_pid,'parent_pid':parent_pid,
        'parent_pid_after_wait':os.getppid(),'case':selection.case if selection is not None else None,
        'suite_sha256':selection.suite_sha256 if selection is not None else None,'argv':[a for a in child.argv],'cwd':str(child.cwd),'environment':{k:v for k,v in environment.items()},
        'executable':str(child.executable),'executable_sha256':child.executable_sha256,
        'expected_inputs':{str(p):v for p,v in child.inputs},'after_inputs':{k:v for k,v in observed.items()},
        'declared_artifacts':[str(p) for p in child.artifacts],'artifact_hashes':{k:v for k,v in artifacts.items()},
        'native_exit':code,'launcher_exit':result,'timed_out':timed_out,'timeout_seconds':child.timeout,
        'merged':child.merged,'stdin_policy':'eof' if bounded else 'inherited','started_ns':started,'ended_ns':time.time_ns(),'errors':[e for e in errors],
        'stdout_sha256':file_hash(child.output/'stdout.log'),'stderr_sha256':file_hash(child.output/'stderr.log'),
        'build_provenance_verified':False,'historical_observer_qualified':False})
    if protocol.enabled():
        protocol.child_returned(role, child.output/'receipt.json')
    _=sys.stdout.buffer.write((child.output/'stdout.log').read_bytes());_=sys.stdout.buffer.flush()
    _=sys.stderr.buffer.write((child.output/'stderr.log').read_bytes());_=sys.stderr.buffer.flush()
    if errors:print('play gate child evidence failed: '+'; '.join(errors),file=sys.stderr)
    return result


def main() -> None:
    require(len(sys.argv)>=4 and sys.argv[2]=='--','/argv','expected ROLE -- COMMAND...')
    path=Path(os.environ.get('WORLDSIM_GATE_PLAY_PLAN',''))
    digest=os.environ.get('WORLDSIM_GATE_PLAY_PLAN_SHA256','')
    try:
        selection=None
        suite=os.environ.get('WORLDSIM_GATE_PLAY_SUITE_PLAN')
        if suite is not None:
            require('WORLDSIM_GATE_PLAY_PLAN' not in os.environ
                    and 'WORLDSIM_GATE_PLAY_PLAN_SHA256' not in os.environ,
                    '/plan', 'ambiguous single and suite plan')
            selection=select_case(Path(suite),os.environ.get('WORLDSIM_GATE_PLAY_SUITE_PLAN_SHA256',''),
                                  os.environ.get('WORLDSIM_GATE_PLAY_CASE',''),sys.argv[1])
            path,digest=selection.plan,selection.plan_sha256
        code=run(path,digest,sys.argv[1],tuple(sys.argv[3:]),selection)
    except (GateContractError,OSError,UnicodeError) as error:
        print('play gate child rejected: '+str(error),file=sys.stderr);code=125
    raise SystemExit(code)


if __name__=='__main__':
    main()
