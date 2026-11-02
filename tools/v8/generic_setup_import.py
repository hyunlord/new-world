"""Original import pair with live ownership; downstream build authority stays pending."""
import os
import select
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import BinaryIO, Literal, final

from gate_oracle_child import (
    IMPORT_COMMAND,
    IMPORT_SCHEMA,
    SUITE_CONTROL,
    SUITE_HASH,
    SUITE_PLAN,
    SUITE_REQUEST,
    SuiteController,
    import_state,
    load_suite,
    prepare_suite,
)
from generic_gate import document, require, text
from generic_process import (
    ProcessResult,
    ProcessSpec,
    atomic_json,
    environment_for,
    file_hash,
)
from generic_setup_bridge import (
    BridgeResult,
    CargoWaitEvidence,
    held_lane,
    immutable_inputs,
    qualify_wait,
)
from generic_setup_prepare import BRIDGE, COMMANDS, PreparedSetup, absent
from generic_sources import inventory
from generic_workspace_cases import ListingProcess, process_output
from mux2_gate_lock import FD_ENV


@dataclass(frozen=True, slots=True)
class PriorBridge:
    result: BridgeResult
    receipt_sha256: str
    wait: CargoWaitEvidence


def authenticate_bridge(prepared: PreparedSetup, prior: PriorBridge) -> tuple[tuple[Path, str], ...]:
    """Reauthenticate the actual step2 endpoint, not a caller's PASS boolean."""
    context=prepared.inputs; output=context.output
    require(prior.result.receipt==output/'receipt.json' and prior.result.native_exit==0,
            '/prior','wrong step2 receipt/outcome')
    require(file_hash(prior.result.receipt)==prior.receipt_sha256,'/prior','changed step2 receipt')
    absent(output/'failure.json'); immutable_inputs(prepared)
    row=document(prior.result.receipt.read_text())
    for key,value in {'schema':'setup_bridge_owned.v1','step':2,'status':'PASS_STEP2_ENDPOINT_ONLY',
        'ownership':'NORMAL_CARGO_RETURN','native_exit':0,'process_status':'PASS',
        'source_sha':context.source_sha,'source_digest':prepared.source_digest,
        'pid':prior.result.pid,'bridge_sha256':prior.result.bridge_sha256}.items():row.field(key).equals(value)
    wait_inputs=qualify_wait(prepared,prior.wait)
    row.field('wait_inputs').equals({str(p):d for p,d in wait_inputs})
    external=dict(prepared.external)
    for path,digest in wait_inputs:
        require(path not in external or external[path]==digest,'/prior','conflicting input')
        external[path]=digest
    bash=next(t for t in context.tools if t.name=='bash')
    argv=(str(bash.path),'-c',COMMANDS[0][2])
    spec=ProcessSpec(argv,context.root,bash.path,bash.sha256,context.root,prepared.materialized,
                     output/'process',context.environment,tuple(sorted(external.items())))
    result=ProcessResult(output/'process/process.json','PASS',0,prior.result.pid)
    row.field('process_receipt').equals(str(result.receipt));row.field('original_argv').equals(list(argv))
    row.field('environment').equals({k:v for k,v in environment_for(spec).items()})
    raw=inventory(output,['process']);row.field('process_artifacts').equals({k:v for k,v in raw.items()})
    required={str(context.root/p):d for p,d in prepared.materialized}
    required.update({str(p):d for p,d in external.items()});required[str(bash.path)]=bash.sha256
    _=process_output(ListingProcess(spec,result,text(row.field('process_receipt_sha256'))),argv,
                     context.root,environment_for(spec),required)
    target=inventory(context.root,['rust/target'])
    row.field('target_inventory').equals({k:v for k,v in target.items()})
    require(target.get(BRIDGE)==prior.result.bridge_sha256,'/prior','bridge endpoint differs')
    bound={output/p:d for p,d in raw.items()}
    bound.update(external);bound[prior.result.receipt]=prior.receipt_sha256
    bound.update({context.root/p:d for p,d in target.items()})
    return tuple(sorted(bound.items()))


@dataclass(frozen=True, slots=True)
class ImportResult:
    receipt: Path
    first_exit: int
    second_exit: int
    shell_exit: int
    status: Literal['PASS_STEP3_ONLY','FAIL_STEP3']
    build_product_authority: Literal[False] = False


@final
class ImportFailure(RuntimeError):
    def __init__(self, primary: BaseException, owner: 'ImportOwner | None') -> None:
        super().__init__('import pair failed; live owner and original exception retained')
        self.primary=primary
        self.owner=owner
        self.secondary: BaseException | None=None


@final
class ImportOwner:
    """Retained live shell/controller on every failure; never closes the caller lane."""
    def __init__(self, prepared: PreparedSetup, prior: PriorBridge, output: Path,
                 controller: SuiteController, stream: BinaryIO) -> None:
        self.prepared=prepared;self.prior=prior;self.output=output
        self.controller=controller;self.stream=stream

    def finish(self) -> ImportResult:
        control=self.controller
        try:
            while control.shell.poll() is None:
                control.service_ready()
                _=select.select((control.request_read,),(),(),0.05)
            # Drain all durable notifications after the actual outer wait, including coalesced frames.
            while select.select((control.request_read,),(),(),0)[0]:
                before=(len(control.completed),control.buffer)
                control.service_ready()
                if before==(len(control.completed),control.buffer):break
            result=control.snapshot()
            require(result.settlement=='SETTLED','/ownership','import ownership UNKNOWN; retain caller lane')
            require(result.shell_exit is not None,'/shell','missing actual wait')
            assert result.shell_exit is not None
            native=[document((r.evidence/'native-terminal.json').read_text()) for r in control.plan.rows]
            codes: list[int]=[]
            for node in native:
                code=node.field('native_exit').value
                require(type(code) is int and -255<=code<=255,'/native','invalid raw exit')
                assert type(code) is int
                codes.append(code)
            expected=124 if native[1].field('timed_out').value is True else codes[1]%256
            require(result.shell_exit==expected,'/shell','original semicolon aggregate differs')
            _=authenticate_bridge(self.prepared,self.prior)
            # Join the terminal state to the last native checkpoint, never establish a new baseline.
            checkpoint=document((control.plan.rows[-1].evidence/'import-checkpoint.json').read_text()).mapping()
            require(control.import_checkpoint==checkpoint and import_state(control.plan)==checkpoint,
                    '/imports','terminal target/cache/settings differ from last native checkpoint')
            status='PASS_STEP3_ONLY' if result.shell_exit==0 else 'FAIL_STEP3'
            _=control.close_settled();self.stream.close()
            held_lane(control.lane_fd)
            artifacts=inventory(self.output,['children','logs','plan.json'])
            receipt=self.output/'receipt.json'
            atomic_json(receipt,{'schema':'setup_import_pair.v1','status':status,'step':3,
                'first_native_exit':codes[0],'second_native_exit':codes[1],
                'timed_out':[n.field('timed_out').value for n in native],
                'shell_exit':result.shell_exit,'shell_pid':result.shell_pid,
                'ownership':'SETTLED_ORIGINAL_IMMEDIATE_CHILDREN','prior_bridge_sha256':self.prior.receipt_sha256,
                'plan_sha256':control.plan.digest,'cache_terminal':checkpoint['cache'],
                'settings_terminal':checkpoint['settings'],
                'source_sha':self.prepared.inputs.source_sha,'bridge_sha256':self.prior.result.bridge_sha256,
                'runtime_closure':'CALLER_OWNED_UNQUALIFIED','build_product_authority':False,
                'artifacts':{k:v for k,v in artifacts.items()},'steps4_5_6':'NOT_RUN','full_gate_complete':False})
            require(inventory(self.output,['children','logs','plan.json'])==artifacts,'/artifacts','terminal output changed')
            require(import_state(control.plan)==checkpoint,'/imports','target/cache/settings changed during terminal publication')
            return ImportResult(receipt,codes[0],codes[1],result.shell_exit,status)
        except BaseException as failure:
            # Cancellation precedes any exception formatting. Caller retains actual owner recovery.
            error=ImportFailure(failure,self)
            try:
                control.cancel('import pair operation failed')
                atomic_json(self.output/'failure.json',{'schema':'setup_import_pair.v1','status':'FAIL',
                    'ownership':'UNKNOWN_KEEP_CALLER_LANE','shell_pid':control.shell.pid,
                    'shell_exit':control.shell.returncode,'completed':list(control.completed),
                    'build_product_authority':False,'full_gate_complete':False})
            except BaseException as secondary:
                error.secondary=secondary
                raise error from secondary
            raise error from failure


def start_import_pair(prepared: PreparedSetup, prior: PriorBridge, output: Path,
                      descriptor: int) -> ImportOwner:
    """Launch exact original two-command shell after actual step2 reauthentication."""
    owner: ImportOwner | None=None
    stream: BinaryIO | None=None
    try:
        held_lane(descriptor)
        bound=dict(authenticate_bridge(prepared,prior));context=prepared.inputs
        require(output.is_absolute() and output==output.resolve() and not output.is_relative_to(context.root)
                and not context.root.is_relative_to(output) and not output.is_relative_to(context.output)
                and not context.output.is_relative_to(output)
                and not output.is_relative_to(context.settings) and not context.settings.is_relative_to(output),'/output','overlapping/noncanonical output')
        absent(output);absent(context.root/'.godot');absent(context.settings)
        require(IMPORT_COMMAND+'; '+IMPORT_COMMAND==COMMANDS[1][2],'/recipe','import command drift')
        bound.update({context.root/p:d for p,d in prepared.materialized})
        for tool in context.tools:bound[tool.path]=tool.sha256
        output.mkdir();evidence=output/'children';logs=output/'logs';evidence.mkdir();logs.mkdir()
        godot=next(t.path for t in context.tools if t.name=='godot')
        path=output/'plan.json'
        atomic_json(path,{'schema':IMPORT_SCHEMA,'root':str(context.root),'evidence':str(evidence),
            'logs':str(logs),'godot':str(godot),'inputs':{str(p):d for p,d in sorted(bound.items())},
            'environment':{k:v for k,v in context.environment}})
        plan=load_suite(path,file_hash(path));channels=prepare_suite(plan,descriptor)
        environment=dict(context.environment)
        environment.update({SUITE_PLAN:str(path),SUITE_HASH:plan.digest,SUITE_REQUEST:str(channels.request_write),
                            SUITE_CONTROL:str(channels.control_read),FD_ENV:str(descriptor)})
        stream=(logs/'imports.log').open('xb')
        try:
            shell=subprocess.Popen(('bash','-c',COMMANDS[1][2]),cwd=context.root,env=environment,
                stdin=subprocess.DEVNULL,stdout=stream,stderr=stream,
                pass_fds=(descriptor,channels.request_write,channels.control_read))
        except BaseException:
            for fd in (channels.request_read,channels.request_write,channels.control_read,channels.control_write):os.close(fd)
            raise
        controller=SuiteController(plan,shell,descriptor,channels)
        owner=ImportOwner(prepared,prior,output,controller,stream)
        os.close(channels.request_write);os.close(channels.control_read)
        controller.start()
        return owner
    except BaseException as failure:
        error=ImportFailure(failure,owner)
        try:
            if owner is not None:owner.controller.cancel('import startup failed')
            elif stream is not None:stream.close()
        except BaseException as secondary:
            error.secondary=secondary
            raise error from secondary
        raise error from failure
