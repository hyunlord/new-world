"""Closed structural/suite child ownership; no source, timing or publication authority."""
import json
import os
import select
import shutil
import stat
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal, final

from gate_stall_child import (
    projected_environment,
    range_hash,
    stream_position,
    supervise,
)
from generic_gate import boundary, document, integer, require, text
from generic_process import atomic_json, check_digest, file_hash, real_path
from mux2_gate_inputs import EvidenceError, Json, Node
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


SUITE_PLAN: Final = 'WORLDSIM_ORACLE_SUITE_PLAN'
SUITE_HASH: Final = 'WORLDSIM_ORACLE_SUITE_PLAN_SHA256'
SUITE_REQUEST: Final = 'WORLDSIM_ORACLE_SUITE_REQUEST_FD'
SUITE_CONTROL: Final = 'WORLDSIM_ORACLE_SUITE_CONTROL_FD'
SUITE_SCHEMA: Final = 'oracle_suite_child_v1'
SUITE_NAMES: Final = ('mux2_camp_framing_test','mux2_visibility_validator_test',
    'mux2_marker_layout_test','mux2_coincident_test','mux2_badge_index_test',
    'mux2_badge_packing_test','mux2_overlap_test','mux2_badge_grid_test',
    'mux2_badge_aggregate_test','mux2_bake_receipt_test','mux2_badge_extent_test',
    'mux2_wall_cache_test','mux2_badge_local_test','mux2_person_cache_test',
    'mux2_extent_owner_test','mux2_stall_test')
SUITE_KEYS: Final = (SUITE_PLAN,SUITE_HASH,SUITE_REQUEST,SUITE_CONTROL)


@dataclass(frozen=True, slots=True)
class SuiteRow:
    ordinal: int
    identity: str
    argv: tuple[str,...]
    limit: int
    log: Path
    evidence: Path


@dataclass(frozen=True, slots=True)
class SuitePlan:
    path: Path
    digest: str
    root: Path
    evidence: Path
    logs: Path
    rows: tuple[SuiteRow,...]
    inputs: tuple[tuple[Path,str],...]
    environment: tuple[tuple[str,str],...]


@dataclass(frozen=True, slots=True)
class SuiteResult:
    settlement: Literal['SETTLED','UNKNOWN']
    shell_pid: int
    shell_exit: int | None
    completed: tuple[str,...]
    error: str


def suite_rows(root: Path, evidence: Path, logs: Path, godot: Path) -> tuple[SuiteRow,...]:
    del root  # cwd is bound by SuitePlan, never substituted into the literal argv.
    rows: list[SuiteRow]=[]
    for index,name in enumerate(SUITE_NAMES):
        tail=('--','--expect-lazy') if index==12 else ('--','--expect-cache') if index in (13,14) else ('--','--self-test') if index==15 else ()
        rows.append(SuiteRow(index+1,name,(str(godot),'--headless','--path','.',
            '--script','res://tools/v8/'+name+'.gd',*tail),120 if index==15 else 300,
            logs/('stall-self-test.log' if index==15 else name+'.log'),evidence/f'{index+1:02d}-{name}'))
    return tuple(rows)


def suite_current(plan: SuitePlan) -> None:
    require(file_hash(plan.path)==plan.digest,'/suite','plan changed')
    for path,digest in plan.inputs:
        require(file_hash(path)==digest,str(path),'suite input changed')


@boundary
def load_suite(path: Path, digest: str) -> SuitePlan:
    check_digest(digest,'/suite');real_path(path,directory=False)
    require(file_hash(path)==digest,'/suite','plan changed')
    node=document(path.read_text())
    require(set(node.mapping())=={'schema','root','evidence','logs','godot','inputs','environment'},'/suite','fields differ')
    node.field('schema').equals(SUITE_SCHEMA)
    root=Path(text(node.field('root')));evidence=Path(text(node.field('evidence')));logs=Path(text(node.field('logs')))
    for directory in (root,evidence,logs):
        real_path(directory,directory=True)
        require(not path.is_relative_to(directory),'/plan','plan must be external to roots')
    for left,right in ((root,evidence),(root,logs),(evidence,logs)):
        require(not left.is_relative_to(right) and not right.is_relative_to(left),'/suite','overlapping roots')
    godot=Path(text(node.field('godot')));real_path(godot,directory=False)
    inputs=tuple((Path(name),text(Node(value))) for name,value in node.field('inputs').mapping().items())
    required={root/'tools/v8'/name for name in ('run_bounded.sh','gate_oracle_child.py','gate_stall_child.py',
        'generic_gate.py','generic_process.py','generic_sources.py','mux2_gate_inputs.py','mux2_gate_lock.py',
        'mux2_gate_checks.sh','godot_bin.sh','mux2_gate_oracle_log.py',*(name+'.gd' for name in SUITE_NAMES))}
    require(required|{godot,Path(sys.executable).resolve(),Path('/bin/bash').resolve()} <= {p for p,_ in inputs},'/inputs','missing suite helper/tool closure')
    for source,sha in inputs:
        real_path(source,directory=False);check_digest(sha,str(source))
        require(not source.is_relative_to(evidence) and not source.is_relative_to(logs) and source!=path,'/inputs','output/plan overlap')
    environment=tuple((key,text(Node(value))) for key,value in node.field('environment').mapping().items())
    allowed={'PATH','HOME','LANG','LC_ALL','PYTHONNOUSERSITE','PYTHONDONTWRITEBYTECODE','PYTHONCOERCECLOCALE','__CF_USER_TEXT_ENCODING','GODOT_BIN'}
    require(set(dict(environment))<=allowed and dict(environment).get('GODOT_BIN')==str(godot),'/environment','closed suite environment required')
    for name,expected in (('bash',Path('/bin/bash').resolve()),('python3',Path(sys.executable).resolve())):
        found=shutil.which(name,path=dict(environment).get('PATH',''))
        require(found is not None and Path(found).resolve()==expected,'/tools','unbound interpreter')
    result=SuitePlan(path,digest,root,evidence,logs,suite_rows(root,evidence,logs,godot),inputs,environment)
    suite_current(result)
    return result


def suite_lane(descriptor: int) -> None:
    held=os.fstat(descriptor);expected=LOCK.stat()
    require((held.st_dev,held.st_ino)==(expected.st_dev,expected.st_ino),'/lane','foreign suite lane')


def suite_stream(log: Path) -> tuple[int,int]:
    expected=log.stat()
    require(stat.S_ISREG(expected.st_mode),'/log','regular merged log required')
    for descriptor in (1,2):
        actual=os.fstat(descriptor)
        require((actual.st_dev,actual.st_ino)==(expected.st_dev,expected.st_ino),'/log','foreign redirected stream')
    return expected.st_dev,expected.st_ino


def suite_environment(plan: SuitePlan) -> dict[str,str]:
    """Observed original suite Bash plus execing wrapper projection; structural stays separate."""
    result=projected_environment(plan.root,dict(plan.environment))
    result['SHLVL']='1'
    return result


def suite_send(descriptor: int, value: dict[str, Json]) -> None:
    raw=(json.dumps(value,separators=(',',':'))+'\n').encode()
    require(len(raw)<=min(4096,os.fpathconf(descriptor,'PC_PIPE_BUF')),'/request','oversized frame')
    require(os.write(descriptor,raw)==len(raw),'/request','partial frame write')


def run_suite(path: Path, digest: str, argv: tuple[str,...], limit: int) -> int:
    """Original Bash owns this wrapper; only this wrapper owns the native Popen."""
    plan=load_suite(path,digest)
    matches=tuple(row for row in plan.rows if row.argv==argv and row.limit==limit)
    require(len(matches)==1 and Path.cwd()==plan.root,'/recipe','changed suite recipe')
    row=matches[0];suite_lane(int(os.environ[FD_ENV]))
    request=int(os.environ[SUITE_REQUEST]);control=int(os.environ[SUITE_CONTROL])
    for descriptor in (request,control):require(stat.S_ISFIFO(os.fstat(descriptor).st_mode),'/channel','pipe required')
    observed=dict(os.environ)
    for key in (*SUITE_KEYS,FD_ENV):_=observed.pop(key,None)
    require(observed==suite_environment(plan),'/environment','suite projection differs')
    device,inode=suite_stream(row.log)
    message: dict[str,Json]={'schema':SUITE_SCHEMA,'plan_sha256':digest,'ordinal':row.ordinal,
        'identity':row.identity,'wrapper_pid':os.getpid(),'parent_pid':os.getppid(),
        'cwd':str(plan.root),'argv':list(argv),'limit':limit,'log_device':device,'log_inode':inode}
    row.evidence.mkdir()
    with (row.evidence/'request.json').open('x') as stream:
        json.dump(message,stream,allow_nan=False);stream.flush();os.fsync(stream.fileno())
    suite_send(request,{'event':'request','ordinal':row.ordinal,'wrapper_pid':os.getpid(),
        'plan_sha256':digest,'request_sha256':file_hash(row.evidence/'request.json')})
    require(os.read(control,1)==b'G','/control','suite startup not authorized')
    admission=document((row.evidence/'wrapper-start.json').read_text())
    require(set(admission.mapping())=={'schema','request','wrapper_wait_observed'},'/admission','fields differ')
    admission.field('schema').equals(SUITE_SCHEMA);admission.field('wrapper_wait_observed').equals(False)
    admission.field('request').equals(message)
    for key in (*SUITE_KEYS,FD_ENV):_=os.environ.pop(key,None)
    start=stream_position(row.log);begin=time.monotonic_ns();pid: int | None=None
    code: int | None=None;timed_out=False;settled=False;error=''
    def started(value: int) -> None:
        nonlocal pid
        pid=value
        atomic_json(row.evidence/'native-start.json',{'schema':SUITE_SCHEMA,'request':message,
            'native_pid':pid,'environment':{k:v for k,v in observed.items()},'stream_start':start})
    try:
        suite_current(plan);_=suite_stream(row.log);require_connected(control)
        code,timed_out=supervise(argv,limit,started,control);settled=True
    except BaseException as failure:
        error=type(failure).__name__+': '+str(failure)
        raise
    finally:
        end=stream_position(row.log)
        atomic_json(row.evidence/'native-terminal.json',{'schema':SUITE_SCHEMA,'request':message,
            'native_pid':pid,'native_exit':code,'timed_out':timed_out,'settlement':'SETTLED' if settled else 'UNKNOWN',
            'error':error,'stream_start':start,'stream_end':end,'stream_sha256':range_hash(row.log,start,end),
            'started_monotonic_ns':begin,'ended_monotonic_ns':time.monotonic_ns()})
        suite_send(request,{'event':'finished','ordinal':row.ordinal,'wrapper_pid':os.getpid(),'plan_sha256':digest})
        os.close(request)
    require(settled and code is not None,'/suite','native wait unknown')
    assert code is not None
    if timed_out:
        print(f'command exceeded the {limit}-second limit',file=sys.stderr)
        return 124
    return code


@dataclass(frozen=True, slots=True)
class SuiteChannels:
    request_read: int
    request_write: int
    control_read: int
    control_write: int
    plan_sha256: str
    owner_pid: int
    prepared_sha256: str


def prepare_suite(plan: SuitePlan, lane_fd: int) -> SuiteChannels:
    """Before Popen: reject stale output and create only this suite's channels."""
    suite_current(plan);suite_lane(lane_fd)
    require(not list(plan.evidence.iterdir()) and not list(plan.logs.iterdir()),'/suite','stale evidence/logs')
    request_read,request_write=os.pipe()
    try:
        control_read,control_write=os.pipe()
    except BaseException:
        os.close(request_read);os.close(request_write)
        raise
    try:
        path=plan.evidence/'suite-prepared.json'
        atomic_json(path,{'schema':SUITE_SCHEMA,'plan_sha256':plan.digest,'owner_pid':os.getpid(),
            'nonce':os.urandom(32).hex(),'request_inode':os.fstat(request_read).st_ino,
            'control_inode':os.fstat(control_write).st_ino})
        return SuiteChannels(request_read,request_write,control_read,control_write,plan.digest,os.getpid(),file_hash(path))
    except BaseException:
        for descriptor in (request_read,request_write,control_read,control_write):os.close(descriptor)
        raise


@final
class SuiteController:
    """Mutable live suite owner; never closes the caller's lane or invents wrapper waits."""
    def __init__(self, plan: SuitePlan, shell: subprocess.Popen[bytes], lane_fd: int,
                 channels: SuiteChannels) -> None:
        self.channels=channels
        self.plan=plan;self.shell=shell;self.lane_fd=lane_fd
        self.request_read=channels.request_read;self.control_write=channels.control_write
        self.buffer=b'';self.active: tuple[SuiteRow,dict[str,Json]] | None=None
        self.completed: list[str]=[];self.anchors: dict[Path,str]={}
        self.ranges: list[tuple[Path,int,int,str,int,int]]=[]
        self.error='';self.halted=False;self.closed=False;self.started=False

    def start(self) -> None:
        """Fallible attachment occurs after the caller retains this live owner."""
        plan=self.plan;shell=self.shell
        try:
            require(not self.started and not self.halted,'/suite','controller already started/cancelled')
            suite_lane(self.lane_fd);suite_current(plan)
            require(self.channels.owner_pid==os.getpid() and self.channels.plan_sha256==plan.digest,'/suite','foreign preparation')
            prepared=plan.evidence/'suite-prepared.json'
            require(file_hash(prepared)==self.channels.prepared_sha256,'/suite','prelaunch preparation changed')
            self.anchors[prepared]=self.channels.prepared_sha256
            keys={p.name for p in plan.evidence.iterdir()}
            require(keys in ({'suite-prepared.json'},{'suite-prepared.json',plan.rows[0].evidence.name}),'/suite','unexpected startup evidence')
            command=('bash','tools/v8/mux2_gate_checks.sh','oracles',str(plan.logs))
            require(shell.args==command or shell.args==list(command),'/suite','not original shell argv')
            for descriptor in (self.request_read,self.control_write):require(stat.S_ISFIFO(os.fstat(descriptor).st_mode),'/channel','pipe required')
            preparation=document(prepared.read_text())
            preparation.field('request_inode').equals(os.fstat(self.request_read).st_ino)
            preparation.field('control_inode').equals(os.fstat(self.control_write).st_ino)
            atomic_json(plan.evidence/'suite-start.json',{'schema':SUITE_SCHEMA,'plan_sha256':plan.digest,
                'shell_pid':shell.pid,'owner_pid':os.getpid(),'argv':list(command),'cwd':str(plan.root)})
            self.anchors[plan.evidence/'suite-start.json']=file_hash(plan.evidence/'suite-start.json')
            self.started=True
        except BaseException as failure:
            try:self.cancel('controller operation failed')
            except BaseException as cleanup:raise failure from cleanup
            raise

    def cancel(self, reason: str) -> None:
        self.error=self.error or reason or 'cancelled';self.halted=True
        if self.control_write>=0:
            descriptor=self.control_write;self.control_write=-1;os.close(descriptor)

    def current(self) -> None:
        suite_current(self.plan);suite_lane(self.lane_fd)
        for path,digest in self.anchors.items():require(file_hash(path)==digest,str(path),'suite evidence changed')
        for path,first,last,digest,device,inode in self.ranges:
            current=path.stat()
            require((current.st_dev,current.st_ino)==(device,inode) and range_hash(path,first,last)==digest,str(path),'producer log changed')

    def service_ready(self) -> None:
        """Nonblocking one-read progress; exceptions retain this owner for recovery."""
        try:
            require(self.started,'/suite','startup not published')
            self.current()
            ready,_,_=select.select((self.request_read,),(),(),0)
            if not ready:return
            chunk=os.read(self.request_read,4096)
            if not chunk:
                require(not self.buffer,'/request','truncated frame')
                return
            self.buffer+=chunk
            while b'\n' in self.buffer:
                raw,self.buffer=self.buffer.split(b'\n',1)
                require(len(raw)<4096,'/request','oversized frame')
                self._message(document(raw.decode()))
            require(len(self.buffer)<4096,'/request','oversized partial frame')
        except BaseException as failure:
            try:self.cancel('controller operation failed')
            except BaseException as cleanup:raise failure from cleanup
            raise

    def _message(self, node: Node) -> None:
        event=node.field('event').text()
        if event=='finished':
            require(set(node.mapping())=={'event','ordinal','wrapper_pid','plan_sha256'},'/request','completion fields differ')
            require(self.active is not None,'/request','no active row')
            assert self.active is not None
            row,request=self.active
            node.field('ordinal').equals(row.ordinal);node.field('wrapper_pid').equals(request['wrapper_pid'])
            node.field('plan_sha256').equals(self.plan.digest)
            self._close_row(row,request)
            return
        require(event=='request' and not self.halted and self.active is None,'/request','unexpected/concurrent request')
        require(len(self.completed)<16 and self.shell.poll() is None,'/request','suite is not live')
        row=self.plan.rows[len(self.completed)]
        require(set(node.mapping())=={'event','ordinal','wrapper_pid','plan_sha256','request_sha256'},'/request','notification fields differ')
        node.field('ordinal').equals(row.ordinal);node.field('plan_sha256').equals(self.plan.digest)
        real_path(row.evidence,directory=True);real_path(row.log,directory=False)
        require({p.name for p in row.evidence.iterdir()}=={'request.json'},'/request','stale row evidence')
        digest=text(node.field('request_sha256'));check_digest(digest,'/request')
        require(file_hash(row.evidence/'request.json')==digest,'/request','request changed')
        request_node=document((row.evidence/'request.json').read_text())
        request_node.field('wrapper_pid').equals(node.field('wrapper_pid').value)
        request=request_node.mapping()
        node=request_node
        require(set(request)=={'schema','plan_sha256','ordinal','identity','wrapper_pid','parent_pid','cwd','argv','limit','log_device','log_inode'},'/request','fields differ')
        expected: dict[str,Json]={'schema':SUITE_SCHEMA,'plan_sha256':self.plan.digest,'ordinal':row.ordinal,'identity':row.identity,
            'parent_pid':self.shell.pid,'cwd':str(self.plan.root),'argv':list(row.argv),'limit':row.limit}
        for key,value in expected.items():node.field(key).equals(value)
        require(integer(node.field('wrapper_pid'))>0 and integer(node.field('wrapper_pid'))!=self.shell.pid,'/pid','invalid wrapper')
        log=row.log.stat();require(stat.S_ISREG(log.st_mode),'/log','regular log required')
        node.field('log_device').equals(log.st_dev);node.field('log_inode').equals(log.st_ino)
        self.current()
        require(file_hash(row.evidence/'request.json')==digest,'/request','request changed during admission')
        self.anchors[row.evidence/'request.json']=digest
        self.active=(row,request)
        atomic_json(row.evidence/'wrapper-start.json',{'schema':SUITE_SCHEMA,'request':request,'wrapper_wait_observed':False})
        self.anchors[row.evidence/'wrapper-start.json']=file_hash(row.evidence/'wrapper-start.json')
        self.current()
        require(os.write(self.control_write,b'G')==1,'/grant','short grant')

    def _close_row(self, row: SuiteRow, request: dict[str,Json]) -> None:
        self.current()
        start=document((row.evidence/'native-start.json').read_text())
        end=document((row.evidence/'native-terminal.json').read_text())
        require(set(start.mapping())=={'schema','request','native_pid','environment','stream_start'},'/native','start fields differ')
        require(set(end.mapping())=={'schema','request','native_pid','native_exit','timed_out','settlement','error','stream_start','stream_end','stream_sha256','started_monotonic_ns','ended_monotonic_ns'},'/native','terminal fields differ')
        for node in (start,end):node.field('schema').equals(SUITE_SCHEMA);node.field('request').equals(request)
        end.field('settlement').equals('SETTLED');end.field('error').equals('')
        pid=integer(start.field('native_pid'));require(pid>0 and pid!=request['wrapper_pid'],'/pid','invalid native')
        end.field('native_pid').equals(pid);start.field('environment').equals({k:v for k,v in suite_environment(self.plan).items()})
        code=end.field('native_exit').value
        require(type(code) is int,'/exit','raw integer exit required')
        assert type(code) is int
        require(-255<=code<=255,'/exit','invalid exit')
        require(type(end.field('timed_out').value) is bool,'/timeout','boolean required')
        first=integer(start.field('stream_start'));last=integer(end.field('stream_end'))
        end.field('stream_start').equals(first);end.field('stream_sha256').equals(range_hash(row.log,first,last))
        require(integer(end.field('ended_monotonic_ns'))>integer(end.field('started_monotonic_ns')),'/time','invalid interval')
        for name in ('native-start.json','native-terminal.json'):self.anchors[row.evidence/name]=file_hash(row.evidence/name)
        self.ranges.append((row.log,first,last,text(end.field('stream_sha256')),integer(Node(request['log_device'])),integer(Node(request['log_inode']))))
        self.current()
        self.completed.append(row.identity);self.active=None
        if code!=0 or end.field('timed_out').value is True:self.halted=True

    def snapshot(self) -> SuiteResult:
        """Only an actual shell wait plus all admitted native waits can settle this owner."""
        try:
            self.current();code=self.shell.poll()
            if code is not None and code<0:
                self.cancel('outer shell signaled; descendant settlement unknown')
            settled=code is not None and self.active is None and not self.buffer and not self.error
            if code==0:require(len(self.completed)==16,'/suite','successful shell lacks all rows')
            return SuiteResult('SETTLED' if settled else 'UNKNOWN',self.shell.pid,code,tuple(self.completed),self.error)
        except BaseException as failure:
            try:self.cancel('controller operation failed')
            except BaseException as cleanup:raise failure from cleanup
            raise

    def close_settled(self) -> SuiteResult:
        result=self.snapshot()
        require(result.settlement=='SETTLED','/suite','live owner must retain UNKNOWN recovery state')
        if not self.closed:
            try:
                logs: dict[str,Json]={str(self.plan.rows[index].log):file_hash(self.plan.rows[index].log)
                    for index in range(len(self.completed))}
                atomic_json(self.plan.evidence/'suite-terminal.json',{'schema':SUITE_SCHEMA,
                    'plan_sha256':self.plan.digest,'shell_pid':result.shell_pid,'shell_exit':result.shell_exit,
                    'settlement':result.settlement,'completed':list(result.completed),'closed_log_sha256':logs,
                    'wrapper_waits_observed':False,'oracle_admission':'CALLER_REQUIRED','error':result.error})
                self.current()
                require(logs=={str(self.plan.rows[index].log):file_hash(self.plan.rows[index].log)
                    for index in range(len(self.completed))},'/logs','closed logs changed')
                if self.control_write>=0:os.close(self.control_write);self.control_write=-1
                os.close(self.request_read);self.closed=True
            except BaseException as failure:
                try:self.cancel('controller operation failed')
                except BaseException as cleanup:raise failure from cleanup
                raise
        return result
