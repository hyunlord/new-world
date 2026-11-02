"""Non-TTY native Cargo runner capture; no case, doctest or world admission.

The authenticated outer ProcessSpec supplies the manifest anchor and clean initial
Cargo environment. Cargo's actual child environment/argv/cwd are forwarded intact,
except a declared observer output variable. Stream buffers are bounded to 64 KiB
per stream; blocking downstream writes apply backpressure on independent threads.
"""
import fcntl
import hashlib
import os
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path
from types import FrameType
from typing import BinaryIO

from gate_rustdoc_exec import Transient, admit_transient, postflight
from generic_gate import GateContractError, boundary, document, integer, require, text
from generic_observations import keys
from generic_process import atomic_json, file_hash, real_path
from generic_sources import inventory
from mux2_gate_inputs import Json, Node


def allocate(root: Path) -> tuple[int,Path]:
    real_path(root,directory=True)
    lock=root/'sequence.lock'
    require(not lock.is_symlink(),'/sequence','symlink lock')
    with lock.open('a+') as stream:
        fcntl.flock(stream,fcntl.LOCK_EX)
        _=stream.seek(0);raw=stream.read()
        require(not raw or raw.isdecimal(),'/sequence','invalid counter')
        number=int(raw or '0')+1
        output=root/f'{number:06d}';output.mkdir()
        _=stream.seek(0);_=stream.truncate();_=stream.write(str(number));stream.flush();os.fsync(stream.fileno())
    return number,output


def tee(source: BinaryIO, output: Path, descriptor: int, errors: list[str]) -> None:
    forwarding=True
    try:
        with source,output.open('xb') as log:
            while block:=os.read(source.fileno(),65536):
                _=log.write(block)
                if forwarding:
                    view=memoryview(block)
                    try:
                        while view:view=view[os.write(descriptor,view):]
                    except OSError as error:
                        errors.append('forward: '+str(error));forwarding=False
    except OSError as error:errors.append('capture: '+str(error))


def settle(child: subprocess.Popen[bytes] | None, threads: list[threading.Thread],
           code: int | None, errors: list[str]) -> int | None:
    """No terminal hashes until owned child is reaped and every started tee ends."""
    if child is None:return code
    if code is None:
        if child.returncode is None:
            try:os.kill(child.pid,signal.SIGKILL)
            except ProcessLookupError as error:errors.append('cleanup kill: '+str(error))
        try:code=child.wait()
        except OSError as error:
            errors.append('cleanup wait: '+str(error))
            _,status=os.waitpid(child.pid,0)
            code=os.waitstatus_to_exitcode(status);child.returncode=code
    for index,stream in enumerate((child.stdout,child.stderr)):
        if stream is not None and (index>=len(threads) or threads[index].ident is None):stream.close()
    for thread in threads:
        if thread.ident is not None:thread.join()
    return code


@boundary
def execute(manifest: Path, digest: str, argv: tuple[str,...]) -> int:
    raw=manifest.read_bytes()
    require(hashlib.sha256(raw).hexdigest()==digest,'/manifest','caller anchor mismatch')
    plan=document(raw.decode('utf-8'))
    keys(plan,({'doctest_policy'} if 'doctest_policy' in plan.mapping() else set[str]()) | {'schema_version','invocation_id','step','arm','profile','output_root','launcher_sha256','python_sha256','inputs','artifacts','observer_variable'})
    plan.field('schema_version').equals(1)
    root=Path(text(plan.field('output_root')))
    sequence,output=allocate(root)
    parent=os.getppid();launcher=os.getpid();cwd=Path.cwd().resolve();started=time.time_ns()
    original_env=dict(os.environ);environment=dict(original_env)
    additions: dict[str,str]={};errors: list[str]=[];received: list[int]=[];queued: list[int]=[]
    threads: list[threading.Thread]=[];cleanup_active=False
    child: subprocess.Popen[bytes] | None=None
    code: int | None=None;artifact_id: str | None=None;before_binary: str | None=None;after_binary: str | None=None
    binary: Path | None=None;transient: Transient | None=None;transient_after: dict[str,Json] | None=None
    inputs: dict[str,str]={};after_inputs: dict[str,str]={}
    identity: dict[str,Json]={'schema_version':1,'component':'NATIVE_CAPTURE_COMPONENT',
        'invocation_id':text(plan.field('invocation_id')),'step':integer(plan.field('step')),
        'arm':text(plan.field('arm')),'profile':text(plan.field('profile')),'sequence':sequence,
        'parent_pid':parent,'launcher_pid':launcher,'argv':[arg for arg in argv],'cwd':str(cwd),
        'manifest_sha256':digest,'started_ns':started}
    atomic_json(output/'pending.json',identity)
    old_handlers={sig:signal.getsignal(sig) for sig in (signal.SIGINT,signal.SIGTERM,signal.SIGHUP)}
    def forward(signum: int, _frame: FrameType | None) -> None:
        received.append(signum)
        if child is None or cleanup_active:queued.append(signum)
        else:child.send_signal(signum)
    try:
        require(not any(os.isatty(fd) for fd in (0,1,2)),'/tty','only non-TTY outer ProcessSpec is supported')
        require(file_hash(Path(__file__).resolve())==text(plan.field('launcher_sha256')) and
                file_hash(Path(sys.executable).resolve())==text(plan.field('python_sha256')),'/launcher','launcher/interpreter changed')
        require(bool(argv) and all('\0' not in arg for arg in argv),'/argv','missing binary or NUL')
        binary=Path(argv[0]).absolute()
        require(not root.is_relative_to(cwd) and not cwd.is_relative_to(root),'/output','receipts overlap child cwd')
        artifacts=plan.field('artifacts').array()
        matched=[item for item in artifacts if text(item.field('path'))==str(binary)]
        if not matched and 'doctest_policy' in plan.mapping():
            transient=admit_transient(plan,digest,argv,cwd,output)
            binary=transient.lexical;before_binary=file_hash(transient.canonical)
        else:
            real_path(binary,directory=False)
            require(len(matched)==1,'/artifact','binary must match exactly one authenticated artifact')
            item=matched[0];keys(item,{'id','path','sha256'});artifact_id=text(item.field('id'))
            before_binary=file_hash(binary)
            require(before_binary==text(item.field('sha256')),'/artifact','binary differs from build admission')
        inputs={name:text(Node(value,'/inputs/'+name)) for name,value in plan.field('inputs').mapping().items()}
        for name,checksum in inputs.items():
            path=Path(name)
            require(not path.is_relative_to(root) and path!=binary,'/inputs','generated outputs are not immutable source')
            require(file_hash(path)==checksum,name,'immutable input changed before child')
        claim=root/('artifact-'+hashlib.sha256(str(binary).encode()).hexdigest()+'.claim')
        with claim.open('x') as stream:_=stream.write(str(sequence))
        variable=plan.field('observer_variable').value
        require(variable is None or variable=='WORLDSIM_GATE_TEST_OUT','/observer','unsupported observer delta')
        if variable is not None:
            assert isinstance(variable,str)
            require(variable not in environment,'/observer','existing observer scope cannot be overwritten')
            observer=output/'observer';observer.mkdir();additions[variable]=str(observer);environment.update(additions)
        for sig in old_handlers:_=signal.signal(sig,forward)
        child=subprocess.Popen(argv,executable=binary,cwd=cwd,env=environment,stdin=None,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        atomic_json(output/'started.json',{**identity,'child_pid':child.pid,'artifact_id':artifact_id,
            'before_binary_sha256':before_binary,'environment':{k:v for k,v in environment.items()},
            'observer_additions':{k:v for k,v in additions.items()}})
        while queued:child.send_signal(queued.pop(0))
        assert child.stdout is not None and child.stderr is not None
        threads=[threading.Thread(target=tee,args=(stream,output/name,fd,errors)) for stream,name,fd in
                 ((child.stdout,'stdout.log',1),(child.stderr,'stderr.log',2))]
        for thread in threads:
            try:thread.start()
            except RuntimeError as error:raise GateContractError('THREAD_START','/tee',str(error)) from error
        code=child.wait()
    except (GateContractError,OSError,UnicodeError) as error:
        errors.append(str(error))
    finally:
        cleanup_active=True
        try:code=settle(child,threads,code,errors)
        finally:
            for sig,handler in old_handlers.items():_=signal.signal(sig,handler)
    if child is not None:
        assert binary is not None
        try:
            if transient is not None:transient_after=postflight(transient)
            after_binary=file_hash(binary if transient is None else transient.canonical)
            require(after_binary==before_binary,'/artifact','binary mutated during child')
            require(file_hash(manifest)==digest,'/manifest','manifest changed during child')
            require(file_hash(Path(__file__).resolve())==text(plan.field('launcher_sha256')) and
                    file_hash(Path(sys.executable).resolve())==text(plan.field('python_sha256')),'/launcher','launcher/interpreter changed during child')
            for name,checksum in inputs.items():
                after_inputs[name]=file_hash(Path(name));require(after_inputs[name]==checksum,name,'immutable input changed during child')
        except (GateContractError,OSError) as error:errors.append(str(error))
    logs: dict[str,str]={}
    for name in ('stdout.log','stderr.log'):
        path=output/name
        if path.exists():logs[name]=file_hash(path)
    output_hashes=inventory(output,[p.name for p in output.iterdir()])
    extra: dict[str,Json]={} if transient is None else {'transient':{**transient.evidence,'identity_after':transient_after}}
    atomic_json(output/'receipt.json',{**identity,**extra,'child_pid':None if child is None else child.pid,
        'artifact_id':artifact_id,'native_exit':code,'native_signal':-code if code is not None and code<0 else None,
        'received_signals':[s for s in received],'environment':{k:v for k,v in environment.items()},
        'original_environment':{k:v for k,v in original_env.items()},'observer_additions':{k:v for k,v in additions.items()},
        'before_binary_sha256':before_binary,'after_binary_sha256':after_binary,
        'expected_inputs':{k:v for k,v in inputs.items()},'after_inputs':{k:v for k,v in after_inputs.items()},
        'stdout_sha256':logs.get('stdout.log'),'stderr_sha256':logs.get('stderr.log'),'ended_ns':time.time_ns(),
        'output_inventory':{k:v for k,v in output_hashes.items()},'errors':[e for e in errors],'capture_status':'FAIL' if errors or code is None else 'COMPLETE',
        'workspace_case_accounting':'INCOMPLETE','doctests':'INCOMPLETE','world_coverage':'INCOMPLETE'})
    return 125 if errors or code is None else code


def main() -> None:
    try:
        code=execute(Path(os.environ.get('CARGO_CAPTURE_MANIFEST','')),os.environ.get('CARGO_CAPTURE_MANIFEST_SHA256',''),tuple(sys.argv[1:]))
    except (GateContractError,OSError,UnicodeError) as error:
        print('Cargo capture rejected: '+str(error),file=sys.stderr);code=125
    if code<0:
        if -code not in (signal.SIGKILL,signal.SIGSTOP):_=signal.signal(-code,signal.SIG_DFL)
        os.kill(os.getpid(),-code)
    raise SystemExit(code)


if __name__=='__main__':
    main()
