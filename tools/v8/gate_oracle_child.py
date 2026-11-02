"""Structural-only direct-child ownership; no source, timing or publication authority."""
import os
import select
import shutil
import stat
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from gate_stall_child import (
    projected_environment,
    range_hash,
    stream_position,
    supervise,
)
from generic_gate import boundary, document, integer, require, text
from generic_process import atomic_json, check_digest, file_hash, real_path
from mux2_gate_inputs import EvidenceError, Node
from mux2_gate_lock import FD_ENV, LOCK

PLAN: Final = 'WORLDSIM_ORACLE_CHILD_PLAN'
HASH: Final = 'WORLDSIM_ORACLE_CHILD_PLAN_SHA256'
CONTROL: Final = 'WORLDSIM_ORACLE_CHILD_CONTROL_FD'
SCHEMA: Final = 'oracle_bounded_child_v1'


@dataclass(frozen=True, slots=True)
class Plan:
    path: Path
    digest: str
    root: Path
    evidence: Path
    log: Path
    argv: tuple[str, ...]
    inputs: tuple[tuple[Path, str], ...]
    environment: tuple[tuple[str, str], ...]


@dataclass(frozen=True, slots=True)
class OwnedResult:
    settlement: Literal['SETTLED', 'UNKNOWN']
    wrapper_pid: int | None
    native_pid: int | None
    wrapper_exit: int | None
    native_exit: int | None
    error: str


@boundary
def load(path: Path, digest: str) -> Plan:
    check_digest(digest, '/plan')
    require(file_hash(path)==digest,'/plan','plan changed')
    node=document(path.read_text())
    require(set(node.mapping())=={'schema','mode','root','evidence','log','argv','inputs','environment'},'/plan','fields differ')
    node.field('schema').equals(SCHEMA);node.field('mode').equals('structural')
    root=Path(text(node.field('root')));evidence=Path(text(node.field('evidence')));log=Path(text(node.field('log')))
    for directory in (root,evidence,log.parent):real_path(directory,directory=True)
    require(log.is_absolute() and log==log.resolve() and not log.is_relative_to(root)
            and not evidence.is_relative_to(root) and not root.is_relative_to(evidence)
            and not log.is_relative_to(evidence),'/paths','overlapping/noncanonical paths')
    argv=tuple(text(row) for row in node.field('argv').array())
    require(len(argv)==7 and argv[1:]==('--rendering-driver','opengl3','--path','.', '--script','res://tools/v8/m6a_plant_art_test.gd'),'/argv','not original structural recipe')
    inputs=tuple((Path(name),text(Node(value,'/inputs/'+name))) for name,value in node.field('inputs').mapping().items())
    required={root/'tools/v8'/name for name in ('run_bounded.sh','gate_oracle_child.py','gate_stall_child.py',
        'generic_gate.py','generic_process.py','generic_sources.py','mux2_gate_inputs.py','mux2_gate_lock.py','m6a_plant_art_test.gd')}
    require(required|{Path(argv[0]),Path(sys.executable).resolve(),Path('/bin/bash').resolve()} <= {p for p,_ in inputs},'/inputs','missing helper/tool closure')
    for source,sha in inputs:
        check_digest(sha,str(source));require(file_hash(source)==sha,str(source),'input changed')
        require(not source.is_relative_to(evidence) and source!=log and source!=path,'/inputs','output/plan overlap')
    environment=tuple((key,text(Node(value,'/environment/'+key))) for key,value in node.field('environment').mapping().items())
    allowed={'PATH','HOME','LANG','LC_ALL','PYTHONNOUSERSITE','PYTHONDONTWRITEBYTECODE','PYTHONCOERCECLOCALE','__CF_USER_TEXT_ENCODING'}
    require(set(dict(environment))<=allowed and 'PATH' in dict(environment),'/environment','unapproved environment keys')
    for name,expected in (('bash',Path('/bin/bash').resolve()),('python3',Path(sys.executable).resolve())):
        found=shutil.which(name,path=dict(environment)['PATH'])
        require(found is not None and Path(found).resolve()==expected,'/tools','PATH selects an unbound interpreter')
    return Plan(path,digest,root,evidence,log,argv,inputs,environment)


def require_connected(descriptor: int) -> None:
    """Reject currently observable EOF; not an atomic disconnect/spawn guarantee."""
    require(stat.S_ISFIFO(os.fstat(descriptor).st_mode),'/control','pipe required')
    ready,_,_=select.select((descriptor,),(),(),0)
    if ready:
        _=os.read(descriptor,1)
        require(False,'/control','disconnected or unexpected control byte')


def run(path: Path, digest: str, argv: tuple[str,...], limit: int) -> int:
    plan=load(path,digest)
    require(argv==plan.argv and limit==180 and Path.cwd()==plan.root,'/recipe','changed structural recipe')
    lane=os.fstat(int(os.environ[FD_ENV]));canonical=LOCK.stat()
    require((lane.st_dev,lane.st_ino)==(canonical.st_dev,canonical.st_ino),'/lane','foreign lane')
    control=int(os.environ[CONTROL]);require(stat.S_ISFIFO(os.fstat(control).st_mode),'/control','pipe required')
    parent=document((plan.evidence/'wrapper-start.json').read_text())
    parent.field('wrapper_pid').equals(os.getpid());parent.field('parent_pid').equals(os.getppid())
    parent.field('plan_sha256').equals(digest)
    require(os.read(control,1)==b'G','/control','startup not authorized')
    observed=dict(os.environ)
    for key in (PLAN,HASH,CONTROL,FD_ENV):_=observed.pop(key,None)
    require(observed==projected_environment(plan.root,dict(plan.environment)),'/environment','shell projection differs')
    for key in (PLAN,HASH,CONTROL,FD_ENV):_=os.environ.pop(key,None)
    start=stream_position(plan.log);begin=time.monotonic_ns();pid: int | None=None
    code: int | None=None;timed_out=False;settled=False;error=''
    def started(value: int) -> None:
        nonlocal pid
        pid=value
        atomic_json(plan.evidence/'native-start.json',{'schema':SCHEMA,'plan_sha256':digest,'wrapper_pid':os.getpid(),
            'parent_pid':os.getppid(),'native_pid':pid,'argv':list(argv),'environment':dict(observed),'stream_start':start})
    try:
        require_connected(control)
        # A disconnect after this check may race with Popen; supervise owns cleanup.
        code,timed_out=supervise(argv,limit,started,control)
        settled=True
    except BaseException as failure:
        error=type(failure).__name__+': '+str(failure)
        raise
    finally:
        end=stream_position(plan.log)
        atomic_json(plan.evidence/'native-terminal.json',{'schema':SCHEMA,'plan_sha256':digest,'wrapper_pid':os.getpid(),
            'parent_pid':os.getppid(),'native_pid':pid,'native_exit':code,'timed_out':timed_out,'timeout_seconds':limit,
            'settlement':'SETTLED' if settled else 'UNKNOWN','error':error,'stream_start':start,'stream_end':end,
            'stream_sha256':range_hash(plan.log,start,end),'started_monotonic_ns':begin,'ended_monotonic_ns':time.monotonic_ns()})
    require(settled and code is not None,'/wait','missing owned wait')
    assert code is not None
    if timed_out:
        print('command exceeded the 180-second limit',file=sys.stderr)
        return 124
    return code


def inspect(path: Path, digest: str, wrapper_pid: int | None, wrapper_exit: int | None) -> OwnedResult:
    """Join local owned receipts; independent source admission remains a caller duty."""
    try:
        plan=load(path,digest)
        require(wrapper_pid is not None and wrapper_exit is not None,'/wrapper','wait unknown')
        parent=document((plan.evidence/'wrapper-start.json').read_text())
        finish=document((plan.evidence/'wrapper-terminal.json').read_text())
        native=document((plan.evidence/'native-terminal.json').read_text())
        start=document((plan.evidence/'native-start.json').read_text())
        for row in (parent,finish,native,start):
            row.field('schema').equals(SCHEMA);row.field('plan_sha256').equals(digest);row.field('wrapper_pid').equals(wrapper_pid)
            row.field('parent_pid').equals(parent.field('parent_pid').value)
        finish.field('wrapper_exit').equals(wrapper_exit)
        native.field('settlement').equals('SETTLED');native.field('error').equals('')
        require(type(native.field('timed_out').value) is bool,'/timeout','boolean required')
        pid=integer(native.field('native_pid'));start.field('native_pid').equals(pid)
        require(pid>0 and pid!=wrapper_pid,'/pid','invalid native pid')
        code=native.field('native_exit').value
        require(type(code) is int,'/exit','raw integer exit required')
        assert type(code) is int
        require(-255<=code<=255,'/exit','raw exit out of range')
        native.field('timeout_seconds').equals(180)
        expected=124 if native.field('timed_out').value is True else code%256
        require(wrapper_exit==expected,'/exit','wrapper/native mismatch')
        start.field('argv').equals(list(plan.argv));start.field('environment').equals({key:value for key,value in projected_environment(plan.root,dict(plan.environment)).items()})
        first=integer(native.field('stream_start'));last=integer(native.field('stream_end'))
        start.field('stream_start').equals(first);native.field('stream_sha256').equals(range_hash(plan.log,first,last))
        require(integer(native.field('ended_monotonic_ns'))>integer(native.field('started_monotonic_ns')),'/time','invalid interval')
        closed=document((plan.evidence/'log-closed.json').read_text())
        closed.field('sha256').equals(file_hash(plan.log));closed.field('plan_sha256').equals(digest)
        return OwnedResult('SETTLED',wrapper_pid,pid,wrapper_exit,code,'')
    except (OSError,ValueError,EvidenceError) as failure:
        return OwnedResult('UNKNOWN',wrapper_pid,None,wrapper_exit,None,str(failure))


def capture(path: Path, digest: str, lane_fd: int) -> OwnedResult:
    """Own only this wrapper; caller retains lane/recovery ownership on UNKNOWN."""
    plan=load(path,digest)
    require(not list(plan.evidence.iterdir()) and not plan.log.exists(),'/output','fresh evidence/log required')
    held=os.fstat(lane_fd);canonical=LOCK.stat()
    require((held.st_dev,held.st_ino)==(canonical.st_dev,canonical.st_ino),'/lane','foreign lane')
    read,write=os.pipe();pid: int | None=None;code: int | None=None;error=''
    environment=dict(plan.environment,**{PLAN:str(path),HASH:digest,CONTROL:str(read),FD_ENV:str(lane_fd)})
    command=('bash','tools/v8/run_bounded.sh','180',*plan.argv)
    try:
        with plan.log.open('xb',buffering=0) as stream, subprocess.Popen(
            command,cwd=plan.root,env=environment,stdout=stream,stderr=subprocess.STDOUT,
            pass_fds=(lane_fd,read)) as process:
            pid=process.pid;os.close(read);read=-1
            try:
                atomic_json(plan.evidence/'wrapper-start.json',{'schema':SCHEMA,'plan_sha256':digest,
                    'parent_pid':os.getpid(),'wrapper_pid':pid,'argv':list(command),'environment':dict(environment)})
                _=os.write(write,b'G')
                code=process.wait()
            except BaseException:
                os.close(write);write=-1
                code=process.wait()
                raise
    except (OSError,ValueError,KeyboardInterrupt,SystemExit) as failure:
        error=type(failure).__name__+': '+str(failure)
    finally:
        if read>=0:os.close(read)
        if write>=0:os.close(write)
    atomic_json(plan.evidence/'wrapper-terminal.json',{'schema':SCHEMA,'plan_sha256':digest,'parent_pid':os.getpid(),
        'wrapper_pid':pid,'wrapper_exit':code,'error':error})
    if plan.log.exists():
        atomic_json(plan.evidence/'log-closed.json',{'schema':SCHEMA,'plan_sha256':digest,'sha256':file_hash(plan.log)})
    return inspect(path,digest,pid,code) if not error else OwnedResult('UNKNOWN',pid,None,code,None,error)
