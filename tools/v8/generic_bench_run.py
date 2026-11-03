"""Execute interleaved benchmark aggregates and require numerical plus world safety."""
import hashlib
import time
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Literal

from generic_bench import (
    AuditError,
    Product,
    git_bytes,
    justified_ratio,
    loads,
    number,
    process_inputs,
    require,
    sha256,
    validate_products,
    write_new,
)
from generic_bench_safety import Aggregate, audit_bench_safety
from generic_gate import document, integer
from generic_process import ProcessResult, ProcessSpec, execute, file_hash, real_path
from generic_sources import inventory
from mux2_gate_inputs import Json, Node
from read_gate_checks import wait_for_load


def run_bench(base: Product, candidate: Product, purpose: str, script: Path,
              script_sha256: str, output: Path,
              justifications: tuple[dict[str, str], ...] = ()) -> dict[str, Json]:
    """Legacy execution API; caller owns source/build admission and measurement lane."""
    return _run_bench(base,candidate,purpose,script,script_sha256,output,justifications,None)


@dataclass(slots=True)
class _BenchExecution:
    output_created: bool = False
    in_flight: bool = False
    invocations: list[Json] = field(default_factory=list)


class BenchSettledFailure(RuntimeError):
    """Source-owned no-in-flight failure; not numeric or publication acceptance."""
    def __init__(self, original: BaseException, diagnostic: BaseException | None,
                 failure_receipt: Path | None) -> None:
        super().__init__('benchmark failed with owned aggregate execution settled')
        self.original: BaseException = original
        self.diagnostic: BaseException | None = diagnostic
        self.failure_receipt: Path | None = failure_receipt


class BenchOwnershipUnknown(RuntimeError):
    """Retain the actual caller lane; no descendant quiescence is established."""
    def __init__(self, original: BaseException, diagnostic: BaseException | None,
                 ownership_receipt: Path | None) -> None:
        super().__init__('benchmark descendant ownership unknown; retain lane')
        self.original: BaseException = original
        self.diagnostic: BaseException | None = diagnostic
        self.ownership_receipt: Path | None = ownership_receipt


def run_bench_owned(base: Product, candidate: Product, purpose: str, script: Path,
                    script_sha256: str, output: Path,
                    justifications: tuple[dict[str,str], ...] = ()) -> dict[str,Json]:
    """Same workload, explicit source-owned settlement boundary for a live lane owner.

    PASS/0 from actual execute settles the bound original aggregate only because
    its source-admitted successful native path waits all children. Every failed or
    unreturned execution is conservatively UNKNOWN. No receipt or caller boolean
    can grant this state. This function does not acquire/release the caller lane.
    """
    state=_BenchExecution()
    try:
        return _run_bench(base,candidate,purpose,script,script_sha256,output,justifications,state)
    except BaseException as original:
        receipt: Path | None = None
        # Never format the original exception before preserving safe diagnostics.
        # Diagnostic failures/interruption cannot replace the primary or settlement.
        if state.output_created:
            try:
                name='ownership.json' if state.in_flight else 'failure.json'
                path=output/name
                if state.in_flight:
                    value: dict[str,Json] = {'schema':'bench_owned_unknown_v1',
                        'ownership':'INCOMPLETE_DESCENDANTS_UNKNOWN','final_inventory':False,
                        'publication_authorized':False,'exception_type':type(original).__name__}
                else:
                    value={'status':'FAIL_OR_INCOMPLETE','reason':'owned benchmark execution failed after settlement',
                           'invocations':list(state.invocations)}
                write_new(path,value)
                receipt=path
            except BaseException as error:
                if state.in_flight:
                    raise BenchOwnershipUnknown(original,error,None) from error
                raise BenchSettledFailure(original,error,None) from error
        if state.in_flight:
            raise BenchOwnershipUnknown(original,None,receipt) from original
        raise BenchSettledFailure(original,None,receipt) from original


def _run_bench(base: Product, candidate: Product, purpose: str, script: Path,
              script_sha256: str, output: Path,
              justifications: tuple[dict[str, str], ...], state: _BenchExecution | None) -> dict[str, Json]:
    """Execute sequential admitted arms. Caller owns the global measurement lock.

    Build receipts must be generated/authenticated by the generic source runner,
    not accepted from an untrusted CLI. Source cleanliness and effective data
    inventories are checked by that runner before and after this invocation.
    """
    validate_products(base, candidate, purpose)
    require(bool(base.input_files) and bool(candidate.input_files), 'authenticated runtime input inventories required')
    require(script.is_absolute() and script.resolve() == script, 'canonical harness input required')
    require(sha256(script) == script_sha256, 'command input changed')
    require(output.is_absolute() and output.parent.resolve() == output.parent, 'canonical fresh output required')
    require(not any(output.is_relative_to(product.root) for product in (base, candidate)),
            'evidence output must be outside products')
    output.mkdir(exist_ok=False)
    if state is not None:state.output_created=True
    changelog_bytes = git_bytes(candidate.root, 'show', f'{candidate.source_sha}:CHANGELOG.md')
    changelog = changelog_bytes.decode()
    # Validate every supplied explanation even if all observed ratios later pass.
    _ = justified_ratio(1, 1, 'n200', 'mean', justifications, changelog)
    invocations: list[Json] = state.invocations if state is not None else []
    aggregates: list[Aggregate] = []
    try:
        arms: tuple[tuple[Literal['base', 'candidate'], Product], ...] = (('base', base), ('candidate', candidate))
        for repeat in range(1, 4):
            for arm, product in arms:
                validate_products(base, candidate, purpose)
                require(sha256(script) == script_sha256, 'command input drift')
                triplet = wait_for_load()
                run_dir = output / f'{arm}-r{repeat}'
                spec = aggregate_spec(product, arm, repeat, script, script_sha256, output)
                argv = spec.argv
                record: dict[str, Json] = {'arm': arm, 'repeat': repeat, 'argv': [part for part in argv],
                    'cwd': str(product.root), 'source_sha': product.source_sha,
                    'binary_sha256': product.binary_sha256, 'script_sha256': script_sha256,
                    'build_receipt_sha256': product.build_receipt_sha256,
                    'load_1_5_15': list(triplet), 'started_unix': time.time(),
                    'exit_code': None, 'finished_unix': None}
                if product.runtime_inputs: record['runtime_inputs'] = dict(product.runtime_inputs)
                write_new(output / f'{arm}-r{repeat}.started.json', record)
                if state is not None:state.in_flight=True
                result = execute(spec)
                if state is not None:
                    require(result.status == 'PASS' and result.native_exit == 0 and
                            type(result.pid) is int and result.pid > 0 and result.receipt == run_dir/'process.json',
                            'owned aggregate did not return a bound successful completion')
                    state.in_flight=False
                record.update(exit_code=result.native_exit, finished_unix=time.time(),
                              producer_pid=result.pid, process_status=result.status,
                              process_receipt=str(result.receipt), process_receipt_sha256=sha256(result.receipt))
                record['stdout_sha256'] = sha256(run_dir / 'stdout.log')
                record['stderr_sha256'] = sha256(run_dir / 'stderr.log')
                write_new(output / f'{arm}-r{repeat}.finished.json', record)
                invocations.append(record)
                require(result.status == 'PASS' and result.native_exit == 0,
                        f'{arm} round {repeat}: producer status {result.status}, exit {result.native_exit}')
                aggregates.append(Aggregate(arm, repeat, product, spec, result, sha256(result.receipt)))
                validate_products(base, candidate, purpose)
                require(sha256(script) == script_sha256, 'command input drift')
        audit = audit_bench_safety(output, tuple(aggregates), purpose, script, script_sha256, justifications)
        audit['invocations'] = invocations
        audit['purpose'] = purpose
        audit['changelog_sha256'] = hashlib.sha256(changelog_bytes).hexdigest()
        write_new(output / 'result.json', audit)
        for product in (base, candidate):
            if product.runtime_inputs: _ = process_inputs(product)
        return audit
    except (AuditError, ValueError, KeyError, OSError, UnicodeError) as exc:
        if state is not None:raise
        write_new(output / 'failure.json', {'status': 'FAIL_OR_INCOMPLETE', 'reason': str(exc),
                                          'invocations': invocations})
        raise


def aggregate_spec(product: Product, arm: Literal['base','candidate'], repeat: int,
                   script: Path, script_sha256: str, output: Path) -> ProcessSpec:
    """Closed execution/retained-admission plan, not caller-selectable argv."""
    require(arm in ('base','candidate') and type(repeat) is int and repeat in (1,2,3), 'invalid occurrence')
    runtime_inputs = dict(process_inputs(product))
    require(bool(runtime_inputs) and len(dict(product.input_files)) == len(product.input_files), 'missing/duplicate product inputs')
    external_inputs: tuple[tuple[Path,str], ...] = ()
    if script.is_relative_to(product.root):
        require(runtime_inputs.get(script.relative_to(product.root).as_posix()) == script_sha256,
                'script missing from authenticated product inventory')
    else:
        external_inputs = ((script,script_sha256),)
    run_dir = output/f'{arm}-r{repeat}'
    argv = (str(product.binary),'bench','--out',str(run_dir),'--commands',str(script),
            '--repeats','1','--commit',product.source_sha)
    return ProcessSpec(argv,product.root,product.binary,product.binary_sha256,product.root,
                       tuple(sorted(runtime_inputs.items())),run_dir,
                       (('WORLDSIM_GIT_SHA',product.source_sha),('WORLDSIM_GATE_OBSERVER','1')),
                       external_inputs=external_inputs)


@dataclass(frozen=True, slots=True)
class BenchResult:
    receipt: Path | None
    sha256: str | None
    status: Literal['COMPONENT_PASS','FAIL','PENDING']
    errors: tuple[str, ...] = ()
    files: tuple[tuple[str,str], ...] = ()


def _snapshot(output: Path) -> dict[str,str]:
    real_path(output,directory=True)
    names = sorted(path.name for path in output.iterdir())
    files = inventory(output,names) if names else {}
    # Empty directory additions also change evidence membership.
    for path in output.rglob('*'):
        if path.is_dir():
            real_path(path,directory=True)
            files[path.relative_to(output).as_posix()+'/'] = ''
    return files


def _lifecycle(spec: ProcessSpec, pid: int, started: float, finished: float) -> None:
    row=document((spec.output_root/'process.json').read_text())
    spawn=document((spec.output_root/'process-spawn.json').read_text())
    start=document((spec.output_root/'process-start.json').read_text())
    spawn.field('pid').equals(pid)
    times=tuple(datetime.fromisoformat(value) for value in
                (row.field('started_at').text(),spawn.field('spawned_at').text(),row.field('ended_at').text()))
    require(all(value.tzinfo is not None and value.utcoffset() is not None for value in times), 'naive process timestamp')
    require(started <= times[0].timestamp() <= times[1].timestamp() <= times[2].timestamp() <= finished,
            'aggregate/process chronology differs')
    require(integer(row.field('duration_ns')) > 0, 'nonpositive process duration')
    require('timeout_seconds' not in row.mapping() and 'timed_out' not in row.mapping(), 'unexpected process timeout')
    row.field('build_provenance_verified').equals(False)
    immutable=('schema_version','argv','cwd','executable','expected_executable_sha256','input_root',
               'expected_inputs','output_root','environment','expected_external_inputs','fresh_directories',
               'environment_policy','started_at','build_provenance_verified','before_executable_sha256',
               'before_inputs','before_external_inputs')
    for key in immutable:start.field(key).equals(row.field(key).value)
    initial: dict[str,Json] = {'status':'FAIL','pid':None,'native_exit':None,'errors':[],'after_external_inputs':{}}
    for key,value in initial.items():
        start.field(key).equals(value)
    require(not {'ended_at','duration_ns','timeout_seconds','timed_out'} & start.mapping().keys(), 'invalid start completion fields')


def authenticate_result(base: Product, candidate: Product, purpose: str, script: Path,
                        script_sha256: str, output: Path, *, receipt_sha256: str | None,
                        justifications: tuple[dict[str,str], ...] = ()) -> BenchResult:
    """Retained numeric/safety admission only; source/build authority stays caller-owned.

    Never executes children, waits for load, rewrites receipts or appends a ledger.
    FAIL/PENDING do not establish descendant quiescence or release a caller's lane.
    """
    errors: list[str] = []; before: dict[str,str] | None = None
    terminal: Path | None = None; terminal_hash: str | None = None
    status: Literal['COMPONENT_PASS','FAIL','PENDING'] = 'FAIL'
    try:
        before=_snapshot(output)
        result_path=output/'result.json';failure=output/'failure.json'
        require(not (result_path.exists() and failure.exists()), 'both result and failure retained')
        terminal=result_path if result_path.exists() else failure if failure.exists() else None
        if terminal is None:
            require(receipt_sha256 is None, 'expected terminal receipt is missing')
            status='PENDING'
        else:
            terminal_hash=file_hash(terminal)
            require(terminal_hash == receipt_sha256, 'terminal digest differs')
            node=document(terminal.read_text())
            if terminal == failure:
                require(set(node.mapping()) == {'status','reason','invocations'}, 'unknown failure schema')
                node.field('status').equals('FAIL_OR_INCOMPLETE')
                _=node.field('reason').text();_=node.field('invocations').array()
                status='FAIL'
            else:
                validate_products(base,candidate,purpose)
                real_path(script,directory=False)
                require(file_hash(script) == script_sha256, 'script changed')
                require(not any(output.is_relative_to(product.root) for product in (base,candidate)), 'output overlaps product')
                aggregates: list[Aggregate] = []; invocations: list[Json] = []
                expected_files={'result.json'}; previous=0.0
                arms: tuple[tuple[Literal['base','candidate'],Product], ...] = (('base',base),('candidate',candidate))
                for repeat in (1,2,3):
                    for arm,product in arms:
                        spec=aggregate_spec(product,arm,repeat,script,script_sha256,output)
                        label=f'{arm}-r{repeat}'
                        start=document((output/(label+'.started.json')).read_text())
                        end=document((output/(label+'.finished.json')).read_text())
                        fixed: dict[str,Json]={'arm':arm,'repeat':repeat,'argv':list(spec.argv),'cwd':str(product.root),
                            'source_sha':product.source_sha,'binary_sha256':product.binary_sha256,
                            'script_sha256':script_sha256,'build_receipt_sha256':product.build_receipt_sha256}
                        if product.runtime_inputs: fixed['runtime_inputs'] = dict(product.runtime_inputs)
                        for key,value in fixed.items():start.field(key).equals(value);end.field(key).equals(value)
                        require(set(start.mapping()) == set(fixed)|{'load_1_5_15','started_unix','exit_code','finished_unix'}, 'started record schema differs')
                        added={'producer_pid','process_status','process_receipt','process_receipt_sha256','stdout_sha256','stderr_sha256'}
                        require(set(end.mapping()) == set(start.mapping())|added, 'finished record schema differs')
                        start.field('exit_code').equals(None);start.field('finished_unix').equals(None)
                        for key in ('load_1_5_15','started_unix'):end.field(key).equals(start.field(key).value)
                        require(loads(start.field('load_1_5_15').value,'aggregate load')[0] < 8, 'aggregate admission load >=8')
                        begin=number(start.field('started_unix').value,'start',positive=True)
                        finish=number(end.field('finished_unix').value,'finish',positive=True)
                        require(previous < begin <= finish, 'interleaved aggregate overlap/reversal')
                        previous=finish
                        pid=integer(end.field('producer_pid'));require(pid > 0, 'invalid aggregate PID')
                        end.field('exit_code').equals(0);end.field('process_status').equals('PASS')
                        process=spec.output_root/'process.json'
                        end.field('process_receipt').equals(str(process));end.field('process_receipt_sha256').equals(file_hash(process))
                        for stream in ('stdout','stderr'):end.field(stream+'_sha256').equals(file_hash(spec.output_root/(stream+'.log')))
                        _lifecycle(spec,pid,begin,finish)
                        aggregates.append(Aggregate(arm,repeat,product,spec,ProcessResult(process,'PASS',0,pid),file_hash(process)))
                        invocations.append(end.mapping())
                        expected_files.update({label+'.started.json',label+'.finished.json',label+'/process-start.json',label+'/process-spawn.json'})
                changelog=git_bytes(candidate.root,'show',f'{candidate.source_sha}:CHANGELOG.md')
                audit=audit_bench_safety(output,tuple(aggregates),purpose,script,script_sha256,justifications)
                audit.update(invocations=invocations,purpose=purpose,changelog_sha256=hashlib.sha256(changelog).hexdigest())
                node.equals(audit)
                for name in Node(audit['hashes']).mapping():
                    path=Path(name)
                    if path.is_relative_to(output):expected_files.add(path.relative_to(output).as_posix())
                # Aggregate receipt/logs are mandatory even for test doubles of the safety oracle.
                for item in aggregates:
                    expected_files.update(f'{item.arm}-r{item.round_index}/{name}' for name in ('process.json','stdout.log','stderr.log'))
                actual_files={name for name in before if not name.endswith('/')}
                require(actual_files == expected_files, 'unexpected/missing benchmark evidence files')
                expected_dirs={parent.as_posix()+'/' for name in expected_files for parent in Path(name).parents if parent != Path('.')}
                require({name for name in before if name.endswith('/')} == expected_dirs, 'unexpected/missing benchmark evidence directories')
                status='COMPONENT_PASS'
    except (AuditError, OSError, ValueError, KeyError, UnicodeError) as error:
        errors.append(str(error))
    finally:
        for product in (base, candidate):
            if product.runtime_inputs:
                try: _ = process_inputs(product)
                except (AuditError, OSError, ValueError) as error: errors.append('runtime: '+str(error))
        if before is not None:
            try:require(_snapshot(output) == before, 'retained evidence changed during authentication')
            except (AuditError,OSError,ValueError) as error:errors.append('final: '+str(error))
    return BenchResult(terminal,terminal_hash,'FAIL' if errors else status,tuple(errors),
                       tuple(sorted(before.items())) if before is not None else ())
