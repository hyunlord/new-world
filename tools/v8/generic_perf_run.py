# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Bounded perf/nav runner. Products are source-owner authenticated release builds.

Timing stays report-only. Ordinary safety is mandatory, including the added
candidate stress companion. This component cannot authorize publication.
"""
import hashlib
import json
import math
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Literal, Protocol, runtime_checkable

import perf_summary
from core_p7_admission import AdmissionError, admit
from generic_bench import (
    AuditError,
    Product,
    process_inputs,
    validate_products,
    verify_product,
)
from generic_gate import boundary, document, integer, require, typed_json
from generic_observations import keys
from generic_perf_observations import PerfObservation, Plan, paired_safety, parse_native
from generic_process import (
    ProcessSpec,
    atomic_json,
    check_digest,
    environment_for,
    execute,
    file_hash,
    real_path,
)
from generic_sources import inventory
from mux2_gate_inputs import Json
from mux2_gate_lock import LOCK, lane


@runtime_checkable
class Reducer(Protocol):
    def summarize(self, out: Path) -> object: ...
    def compare(self, base: object, cand: object) -> str: ...


def reducer(module: object = perf_summary) -> Reducer:
    if not isinstance(module, Reducer):
        raise TypeError('standing reducer lacks summarize/compare')
    return module


@dataclass(frozen=True, slots=True)
class Run:
    name: str
    argv: tuple[str, ...]
    raw: Path
    ticks: int
    seed: int
    agents: int | None
    workload: str
    kind: Literal['perf', 'nav']
    companion: bool = False


def plans(fixture: str, product: Product, output: Path, arm: str) -> tuple[Run, ...]:
    require(arm in {'base', 'candidate'}, fixture, 'unknown arm')
    binary = str(product.binary)
    if fixture == 'nav.soak':
        raw = output / 'raw/nav_bench.json'
        return (Run('nav', (binary, 'nav-bench', '--out', str(raw), '--commands',
                           str(product.root / 'tools/v8/soak_script.json')),
                    raw.parent, 4320, 15100, 1000, json.dumps({'script':'soak_script.json'}), 'nav'),)
    require(fixture == 'perf.natural_quick', fixture, 'unsupported producer')
    result: list[Run] = []
    for repeat in range(1, 4):
        raw = output / f'raw/natural_r{repeat}'
        for seed in (19600, 19603, 19605, 19606):
            result.append(Run(f'natural-r{repeat}-s{seed}',
                              (binary,'natural','--scene','game_v26','--seeds',f'{seed}..{seed+1}',
                               '--days','120','--out',str(raw)), raw, 172800, seed, None,
                              json.dumps({'mode':'natural','scene':'game_v26','side':64,'feed':'none'}), 'perf'))
    for mode, directory in (('quick','quick'), ('stress','stress_v26')):
        raw = output / 'raw' / directory
        extra = () if mode == 'quick' else ('--seeds','15100..15101','--agents','500','--side','64',
                                            '--days','1','--feed','fed','--rules','game_v26')
        result.append(Run(mode, (binary, mode, *extra, '--out', str(raw)), raw, 1440, 15100, 500,
                          json.dumps({'mode':mode,'scene':'stress_baseline','side':64,'feed':'fed'}),
                          'perf', mode == 'stress' and arm == 'candidate'))
    return tuple(result)


def freeze(root: Path) -> dict[str, str]:
    """Include every file, and reject links/special nodes rather than skipping them."""
    result: dict[str, str] = {}
    for path in sorted(root.rglob('*')):
        require(not path.is_symlink(), str(path), 'symlink output')
        if path.is_dir():
            continue
        result[path.relative_to(root).as_posix()] = file_hash(path)
    return result


def final_inventory(root: Path) -> tuple[dict[str, str], list[str]]:
    """Retain readable artifacts without pretending a partial inventory is complete."""
    hashes: dict[str, str] = {}
    errors: list[str] = []
    try:
        for path in sorted(root.rglob('*')):
            try:
                require(not path.is_symlink(), str(path), 'symlink output')
                if not path.is_dir():
                    hashes[path.relative_to(root).as_posix()] = file_hash(path)
            except (ValueError, OSError) as error:
                errors.append(str(error))
    except OSError as error:
        errors.append(str(error))
    return hashes, errors


def raw_coverage(output: Path, retained: dict[Path, str], indexes: dict[Path, list[Json]]) -> None:
    expected = {path for path in retained if path.is_relative_to(output) and 'raw' in path.relative_to(output).parts}
    expected |= indexes.keys()
    actual = set(output.glob('*/raw/**/*')) | set(output.glob('*/raw'))
    directories: set[Path] = set()
    for path in expected:
        while path.parent != output and path.parent.name != 'base' and path.parent.name != 'candidate':
            path = path.parent
            directories.add(path)
    require(actual == expected | directories, str(output), 'unregistered raw artifact/directory or missing report')


def append_index(path: Path, raw: Path, indexes: dict[Path, list[Json]]) -> str:
    data = path.read_bytes()
    rows = [node.value for node in document(data.decode('utf-8')).field('runs').array()]
    prior = indexes.get(path, [])
    require(len(rows) == len(prior) + 1 and rows[:-1] == prior,
            str(path), 'index changed earlier entries or did not append exactly one world')
    document(json.dumps(rows[-1])).field('file').equals(raw.name)
    indexes[path] = rows
    return hashlib.sha256(data).hexdigest()


def audit(run: Run, product: Product, pid: int, raw: Path) -> tuple[PerfObservation, dict[Path, str]]:
    raw_bytes = raw.read_bytes()
    raw_text = raw_bytes.decode('utf-8')
    report = document(raw_text)
    report.field('build_profile').equals('release')
    if run.kind == 'perf':
        report.field('git_sha').equals(product.source_sha)
        report.field('contended').equals(False)
        report.field('warmup').field('ticks').equals(60)
        report.field('tick_summary').field('ticks').equals(run.ticks - 60)
        for name in ('mean_ms','p99_ms','p999_ms','max_ms'):
            require(report.field('tick_summary').field(name).number() >= 0, str(raw), 'negative timing')
        require(raw.with_suffix('.tick_us.bin').stat().st_size == run.ticks * 4,
                str(raw), 'missing/truncated complete tick timing samples')
        entries = document((run.raw / 'runs.json').read_text()).field('runs').array()
        require(len(entries) == len(list(run.raw.glob('*.tick_us.bin'))), str(raw), 'index coverage')
        matching = [entry for entry in entries if entry.field('file').text() == raw.name]
        require(len(matching) == 1, str(raw), 'missing/duplicate index occurrence')
        for entry in entries:
            entry.field('status').equals('ok')
        matching[0].field('seed').equals(run.seed)
    else:
        report.field('warmup_ticks').equals(1440)
        report.field('measured_ticks').equals(2880)
        movement = report.field('movement_us').number()
        budget = report.field('budget_us').number()
        require(movement >= 0 and budget > 0, str(raw), 'invalid navigation timing')
        report.field('within_budget').equals(movement <= budget)
    sidecar = run.raw / 'gate_observations' / (raw.name + '.safety.json')
    plan = Plan(run.kind, run.ticks, run.seed, run.agents, str(raw), pid, run.workload)
    sidecar_bytes = sidecar.read_bytes()
    observation = parse_native(sidecar_bytes.decode('utf-8'), raw_text, plan)
    require(raw.read_bytes() == raw_bytes and sidecar.read_bytes() == sidecar_bytes,
            str(raw), 'artifact changed during audit')
    return observation, {raw: hashlib.sha256(raw_bytes).hexdigest(),
                         sidecar: hashlib.sha256(sidecar_bytes).hexdigest()}


def prepare(fixture: str, base: Product, candidate: Product, output: Path,
            purpose: str = 'publication') -> dict[str, tuple[Run, ...]]:
    validate_products(base, candidate, purpose)
    require(output.is_absolute() and output == output.resolve() and not output.exists(),
            str(output), 'fresh canonical output required')
    for product in (base, candidate):
        require(not output.is_relative_to(product.root) and not product.root.is_relative_to(output),
                str(output), 'outputs overlap product')
        require(bool(product.input_files) and len(dict(product.input_files)) == len(product.input_files),
                str(product.root), 'missing/duplicate effective input inventory')
        require(inventory(product.root, [name for name, _ in product.input_files]) == dict(product.input_files),
                str(product.root), 'effective input inventory drift')
        _ = process_inputs(product)
        if fixture == 'nav.soak':
            require('tools/v8/soak_script.json' in dict(product.input_files), fixture, 'script not source-bound')
    if fixture == 'nav.soak':
        require(dict(base.input_files)['tools/v8/soak_script.json'] ==
                dict(candidate.input_files)['tools/v8/soak_script.json'], fixture, 'paired script changed')
    return {arm: plans(fixture, product, output / arm, arm)
            for arm, product in (('base',base), ('candidate',candidate))}


def run_pair(fixture: str, base: Product, candidate: Product, output: Path, *,
             deadline: float, purpose: str = 'publication') -> Path:
    require(type(deadline) in (int, float) and math.isfinite(deadline), fixture, 'finite timing admission deadline required')
    planned = prepare(fixture, base, candidate, output, purpose)
    output.mkdir(parents=True)
    worlds: list[Json] = []
    safety: list[Json] = []
    report: dict[str, Json] = {'status':'PENDING', 'fixture':fixture, 'publication_authorized':False,
                              'timing_policy':'standing report-only; safety mandatory',
                              'measurement_lock':str(LOCK), 'worlds':worlds, 'safety':safety, 'errors':[]}
    if base.runtime_inputs or candidate.runtime_inputs:
        report['runtime_inputs'] = {arm: dict(product.runtime_inputs) for arm, product in (('base', base), ('candidate', candidate))}
    observations: dict[tuple[str,str], PerfObservation] = {}
    retained: dict[Path, str] = {}
    indexes: dict[Path, list[Json]] = {}
    process_directories: dict[Path, set[str]] = {}
    admission = output / 'admission.jsonl'
    admission_bytes = b''
    admission_label = ''
    try:
        with lane():
            for arm, product in (('base',base), ('candidate',candidate)):
                arm_root = output / arm
                (arm_root / 'processes').mkdir(parents=True)
                for run in planned[arm]:
                    require(all(file_hash(path) == digest for path, digest in retained.items()),
                            run.name, 'previous completed evidence changed')
                    if admission.exists():
                        require(admission.read_bytes() == admission_bytes, str(admission), 'admission journal changed')
                    admission_label = f'{arm}/{run.name}'
                    _ = admit(admission, admission_label, deadline=deadline)
                    updated = admission.read_bytes()
                    require(updated.startswith(admission_bytes) and len(updated) > len(admission_bytes),
                            str(admission), 'admission must append without rewriting')
                    admission_bytes = updated
                    retained[admission] = hashlib.sha256(updated).hexdigest()
                    run.raw.mkdir(parents=True, exist_ok=True)
                    before = set(run.raw.glob('*.json'))
                    process = execute(_process_spec(run, product, arm_root / 'processes' / run.name))
                    process_files = freeze(process.receipt.parent)
                    process_directories[process.receipt.parent] = set(process_files)
                    retained.update({process.receipt.parent/name: digest for name,digest in process_files.items()})
                    require(process.status == 'PASS' and process.native_exit == 0 and process.pid is not None,
                            run.name, 'native process did not complete')
                    raw_files = set(run.raw.glob('*.json')) - before - {run.raw / 'runs.json'}
                    require(len(raw_files) == 1, run.name, 'expected exactly one actual new world report')
                    raw = raw_files.pop()
                    if run.kind == 'nav':
                        require(raw == Path(run.argv[3]), run.name, 'wrong native nav output')
                        require(any(line.startswith('nav_bench:') for line in
                                    (process.receipt.parent/'stdout.log').read_text().splitlines()),
                                run.name, 'standing nav stdout oracle missing')
                    if process.pid is None:
                        raise AssertionError('validated native PID')
                    observed, bound_bytes = audit(run, product, process.pid, raw)
                    observations[arm, run.name] = observed
                    retained.update(bound_bytes)
                    if run.kind == 'perf':
                        index = run.raw/'runs.json'
                        retained[index] = append_index(index, raw, indexes)
                        samples = raw.with_suffix('.tick_us.bin')
                        retained[samples] = file_hash(samples)
                    worlds.append({'arm':arm, 'occurrence':run.name, 'companion':run.companion,
                                   'source_sha':product.source_sha, 'process':str(process.receipt),
                                   'raw':str(raw), 'pid':process.pid})
            raw_coverage(output, retained, indexes)
            for directory, names in process_directories.items():
                require(set(freeze(directory)) == names, str(directory), 'completed process artifact set changed')
            require(set(output.glob('*/raw/**/gate_observations/*')) ==
                    {path for path in retained if path.name.endswith('.safety.json')},
                    str(output), 'extra/missing native world observations')
            safety_errors: list[str] = []
            for run in planned['base']:
                try:
                    safety.append({'occurrence':run.name, **paired_safety(observations['base',run.name],
                                                                         observations['candidate',run.name])})
                except ValueError as error:
                    safety.append({'occurrence':run.name, 'status':'FAIL', 'error':str(error)})
                    safety_errors.append(str(error))
            if fixture == 'perf.natural_quick':
                summaries = {arm: reducer().summarize(output/arm/'raw') for arm in ('base','candidate')}
                report['timing_summaries'] = typed_json(summaries)
                report['standing_timing_comparison'] = reducer().compare(summaries['base'], summaries['candidate'])
                report['coverage_expansion'] = 'original BASE14/candidate13 plus candidate unchanged stress companion: 14/14'
            else:
                report['standing_oracle'] = 'native exit and report-only within_budget; no new timing threshold'
            for product in (base, candidate):
                verify_product(product)
                _ = process_inputs(product)
                require(inventory(product.root, [n for n,_ in product.input_files]) == dict(product.input_files),
                        str(product.root), 'end input drift')
            require(all(file_hash(path) == digest for path,digest in retained.items()),
                    str(output), 'raw or sidecar changed after audit')
            require(not safety_errors, fixture, '; '.join(safety_errors))
            report['status'] = 'COMPONENT_PASS'
    except AdmissionError as error:
        report['errors'] = [str(error)]
        try:
            updated = admission.read_bytes() if admission.exists() else b''
            require(updated.startswith(admission_bytes), str(admission), 'failed admission rewrote journal prefix')
            appended = updated[len(admission_bytes):]
            if appended:
                for row in appended.decode('utf-8').splitlines():
                    entry = document(row)
                    entry.field('status').equals('NOT_RUN')
                    entry.field('label').equals(admission_label)
                retained[admission] = hashlib.sha256(updated).hexdigest()
        except (ValueError, OSError) as journal_error:
            report['status'] = 'FAIL'
            report['errors'] = [str(error), str(journal_error)]
    except (AuditError, ValueError, OSError, KeyError) as error:
        report['status'] = 'FAIL'
        report['errors'] = [str(error)]
    frozen, freeze_errors = final_inventory(output)
    report['output_inventory_complete'] = not freeze_errors
    if freeze_errors:
        report['status'] = 'FAIL'
        previous = report['errors']
        report['errors'] = [*(previous if isinstance(previous, list) else []), *freeze_errors]
    if any(frozen.get(path.relative_to(output).as_posix()) != digest for path, digest in retained.items()):
        report['status'] = 'FAIL'
        previous = report['errors']
        report['errors'] = [*(previous if isinstance(previous, list) else []),
                            'audited artifact changed before final output freeze']
    report['output_sha256'] = typed_json(frozen)
    target = output / 'result.json'
    atomic_json(target, report)
    return target

@dataclass(frozen=True, slots=True)
class PerfResult:
    receipt: Path
    sha256: str
    status: Literal['COMPONENT_PASS','FAIL','PENDING']


def _process_spec(run: Run, product: Product, output: Path) -> ProcessSpec:
    return ProcessSpec(run.argv,product.root,product.binary,product.binary_sha256,product.root,
                       process_inputs(product),output,(('WORLDSIM_GATE_OBSERVER','1'),('WORLDSIM_GIT_SHA',product.source_sha)))


def _completed(spec: ProcessSpec, pid: int) -> datetime:
    folder=spec.output_root
    require(set(freeze(folder))=={'process.json','process-start.json','process-spawn.json','stdout.log','stderr.log'},str(folder),'process file coverage')
    terminal=document((folder/'process.json').read_text())
    start=document((folder/'process-start.json').read_text())
    spawn=document((folder/'process-spawn.json').read_text())
    immutable: dict[str,Json]={'schema_version':1,'argv':list(spec.argv),'cwd':str(spec.cwd),'executable':str(spec.executable),
        'expected_executable_sha256':spec.executable_sha256,'input_root':str(spec.input_root),'expected_inputs':dict(spec.inputs),
        'output_root':str(folder),'environment':{k:v for k,v in environment_for(spec).items()},'expected_external_inputs':{},
        'fresh_directories':[],'environment_policy':'fixed_defaults_plus_explicit_only_no_inheritance','build_provenance_verified':False,
        'before_executable_sha256':spec.executable_sha256,'before_inputs':dict(spec.inputs),'before_external_inputs':{},'after_external_inputs':{}}
    keys(start,set(immutable)|{'started_at','pid','native_exit','status','errors'})
    keys(terminal,set(start.mapping())|{'ended_at','duration_ns','after_executable_sha256','after_inputs','stdout_sha256','stderr_sha256','limit'})
    keys(spawn,{'pid','spawned_at'})
    for key,value in immutable.items():
        start.field(key).equals(value);terminal.field(key).equals(value)
    initial: tuple[tuple[str,Json],...]=(('pid',None),('native_exit',None),('status','FAIL'),('errors',[]))
    for key,value in initial:
        start.field(key).equals(value)
    require(type(pid) is int and pid>0,str(folder),'positive actual PID required')
    terminal.field('pid').equals(pid);spawn.field('pid').equals(pid)
    terminal.field('status').equals('PASS');terminal.field('native_exit').equals(0);terminal.field('errors').equals([])
    terminal.field('after_executable_sha256').equals(spec.executable_sha256);terminal.field('after_inputs').equals(dict(spec.inputs))
    terminal.field('started_at').equals(start.field('started_at').value)
    terminal.field('limit').equals('Endpoint executable/input binding only; no build, descendants, output schema, or whole-gate authority.')
    require(integer(terminal.field('duration_ns'))>0,str(folder),'positive duration required')
    for stream in ('stdout','stderr'):terminal.field(stream+'_sha256').equals(file_hash(folder/(stream+'.log')))
    times=[datetime.fromisoformat(node.text()) for node in (start.field('started_at'),spawn.field('spawned_at'),terminal.field('ended_at'))]
    require(all(value.tzinfo is not None for value in times) and times==sorted(times),str(folder),'invalid process timestamp chain')
    return times[0]


def _admissions(path: Path, labels: tuple[str,...], starts: tuple[datetime,...], deadline: float, pending: bool, next_label: str | None=None) -> None:
    if not path.exists():
        require(pending and not labels,str(path),'missing admission journal')
        return
    rows=[document(line) for line in path.read_text().splitlines()]
    position=0
    for index,label in enumerate(labels):
        admitted=False
        for window in range(9):
            require(position<len(rows),str(path),'missing admitted occurrence')
            row=rows[position];position+=1
            keys(row,{'label','utc','window','loads_1_5_15','status'})
            row.field('label').equals(label);row.field('window').equals(window)
            now=row.field('utc').number();loads=[value.number() for value in row.field('loads_1_5_15').array()]
            require(math.isfinite(now) and len(loads)==3 and all(math.isfinite(v) and v>=0 for v in loads),str(path),'invalid admission sample')
            allowed=now<deadline and loads[0]<8
            row.field('status').equals('ADMITTED' if allowed else 'NOT_RUN')
            if allowed:
                require(now<=starts[index].timestamp()+0.000001,str(path),'admission after native start')
                admitted=True
                break
            require(now<deadline and window<8 and now+900<deadline,str(path),'admission continued after hard stop')
        require(admitted,str(path),'no admitted terminal')
    if pending and position<len(rows):
        require(next_label is not None,str(path),'admission after all planned worlds')
        for window,row in enumerate(rows[position:]):
            keys(row,{'label','utc','window','loads_1_5_15','status'})
            row.field('label').equals(next_label);row.field('window').equals(window);row.field('status').equals('NOT_RUN')
            now=row.field('utc').number();loads=[value.number() for value in row.field('loads_1_5_15').array()]
            require(window<=8 and math.isfinite(now) and len(loads)==3 and all(math.isfinite(v) and v>=0 for v in loads),str(path),'invalid pending sample')
            require(now>=deadline or loads[0]>=8,str(path),'false NOT_RUN admission')
            if position+window+1<len(rows):
                require(now<deadline and window<8 and now+900<deadline,str(path),'pending admission continued after hard stop')
        position=len(rows)
    require(position==len(rows),str(path),'extra admission groups')


@boundary
def authenticate_result(fixture: str, base: Product, candidate: Product, output: Path, *,
                        receipt_sha256: str, deadline: float, purpose: str='publication') -> PerfResult:
    """Rejoin retained output bytes; source-owner build/compile-root admission is external."""
    require(fixture in {'perf.natural_quick','nav.soak'},'/fixture','unsupported producer')
    require(type(deadline) in (int,float) and math.isfinite(deadline),'/deadline','finite deadline required')
    validate_products(base,candidate,purpose)
    real_path(output,directory=True)
    target=output/'result.json';check_digest(receipt_sha256,'/receipt')
    require(file_hash(target)==receipt_sha256,str(target),'terminal hash differs')
    node=document(target.read_text())
    common={'status','fixture','publication_authorized','timing_policy','measurement_lock','worlds','safety','errors','output_inventory_complete','output_sha256'}
    if base.runtime_inputs or candidate.runtime_inputs:
        common.add('runtime_inputs')
        node.field('runtime_inputs').equals({arm: dict(product.runtime_inputs) for arm, product in (('base', base), ('candidate', candidate))})
    extras={'timing_summaries','standing_timing_comparison','coverage_expansion'} if fixture=='perf.natural_quick' else {'standing_oracle'}
    require(common<=set(node.mapping())<=common|extras,str(target),'terminal fields differ')
    state=node.field('status').text()
    require(state in {'COMPONENT_PASS','FAIL','PENDING'},'/status','unknown perf status')
    assert state in ('COMPONENT_PASS','FAIL','PENDING')
    node.field('fixture').equals(fixture);node.field('publication_authorized').equals(False)
    node.field('timing_policy').equals('standing report-only; safety mandatory');node.field('measurement_lock').equals(str(LOCK))
    errors=tuple(row.text() for row in node.field('errors').array());safety=node.field('safety').array()
    for row in safety: _=row.mapping()
    if 'timing_summaries' in node.mapping():_=node.field('timing_summaries').mapping()
    for key in extras-{'timing_summaries'}:
        if key in node.mapping():_=node.field(key).text()
    require(type(node.field('output_inventory_complete').value) is bool,'/inventory','boolean required')
    node.field('output_inventory_complete').equals(True)
    hashes=freeze(output);_=hashes.pop('result.json')
    node.field('output_sha256').equals({k:v for k,v in hashes.items()})
    def products_current() -> None:
        validate_products(base,candidate,purpose)
        for product in (base,candidate):
            _ = process_inputs(product)
            require(not output.is_relative_to(product.root) and not product.root.is_relative_to(output),'/output','product overlap')
            require(bool(product.input_files) and len(dict(product.input_files))==len(product.input_files),'/inputs','missing/duplicate inventory')
            require(inventory(product.root,[name for name,_ in product.input_files])==dict(product.input_files),'/inputs','input drift')
        if fixture=='nav.soak':
            require('tools/v8/soak_script.json' in dict(base.input_files) and 'tools/v8/soak_script.json' in dict(candidate.input_files),'/script','missing script binding')
            require(dict(base.input_files)['tools/v8/soak_script.json']==dict(candidate.input_files)['tools/v8/soak_script.json'],'/script','paired script differs')
    products_current()
    planned=tuple((arm,product,run) for arm,product in (('base',base),('candidate',candidate)) for run in plans(fixture,product,output/arm,arm))
    worlds=node.field('worlds').array()
    require(len(worlds)<=len(planned),'/worlds','extra occurrences')
    observations: dict[tuple[str,str],PerfObservation]={};retained: dict[Path,str]={};indexes: dict[Path,list[Json]]={}
    starts: list[datetime]=[];labels: list[str]=[];raws: set[Path]=set()
    for row,(arm,product,run) in zip(worlds,planned):
        keys(row,{'arm','occurrence','companion','source_sha','process','raw','pid'})
        for key,value in (('arm',arm),('occurrence',run.name),('source_sha',product.source_sha),('companion',run.companion)):
            row.field(key).equals(value)
        folder=output/arm/'processes'/run.name;row.field('process').equals(str(folder/'process.json'))
        raw=Path(row.field('raw').text());real_path(raw,directory=False)
        require(raw.parent==run.raw and raw.suffix=='.json' and raw.name!='runs.json' and raw not in raws,'/raw','wrong or reused native report')
        raws.add(raw);pid=integer(row.field('pid'));require(pid>0,'/pid','positive PID required')
        if run.kind=='nav':require(raw==Path(run.argv[3]),'/raw','wrong nav filename')
        starts.append(_completed(_process_spec(run,product,folder),pid));labels.append(arm+'/'+run.name)
        if run.kind=='nav':
            require(any(line.startswith('nav_bench:') for line in (folder/'stdout.log').read_text().splitlines()),'/stdout','nav oracle missing')
        observed,bound=audit(run,product,pid,raw);observations[arm,run.name]=observed;retained.update(bound)
        if run.kind=='perf':
            index=run.raw/'runs.json';indexes.setdefault(index,[]).append({'file':raw.name,'seed':run.seed,'status':'ok'})
            retained[raw.with_suffix('.tick_us.bin')]=file_hash(raw.with_suffix('.tick_us.bin'))
    if state=='COMPONENT_PASS':
        keys(node,common|extras);require(not errors and len(worlds)==len(planned),'/worlds','incomplete success')
        for index,expected in indexes.items():
            rows=document(index.read_text()).field('runs').array()
            require(len(rows)==len(expected),str(index),'wrong index cardinality')
            for row,wanted in zip(rows,expected):
                for key,value in document(json.dumps(wanted)).mapping().items():row.field(key).equals(value)
            retained[index]=file_hash(index)
        raw_coverage(output,retained,indexes)
        expected_safety=[{'occurrence':run.name,**paired_safety(observations['base',run.name],observations['candidate',run.name])}
                         for run in plans(fixture,base,output/'base','base')]
        node.field('safety').equals(typed_json(expected_safety))
        if fixture=='perf.natural_quick':
            summaries={arm:reducer().summarize(output/arm/'raw') for arm in ('base','candidate')}
            node.field('timing_summaries').equals(typed_json(summaries))
            node.field('standing_timing_comparison').equals(reducer().compare(summaries['base'],summaries['candidate']))
            node.field('coverage_expansion').equals('original BASE14/candidate13 plus candidate unchanged stress companion: 14/14')
        else:node.field('standing_oracle').equals('native exit and report-only within_budget; no new timing threshold')
        _admissions(output/'admission.jsonl',tuple(labels),tuple(starts),deadline,False)
        require({p.name for p in output.iterdir()}=={'result.json','admission.jsonl','base','candidate'},'/outputs','extra top-level outputs')
        for arm in ('base','candidate'):
            require({p.name for p in (output/arm).iterdir()}=={'raw','processes'},'/outputs','extra arm outputs')
            require({p.name for p in (output/arm/'processes').iterdir()}=={run.name for a,_,run in planned if a==arm},'/processes','extra process occurrence')
    else:
        require(bool(errors),'/errors','noncompletion lacks diagnostic')
        if state=='PENDING':
            require(len(worlds)<len(planned) and not safety,'/pending','admission pending cannot finish coverage or safety')
            next_label=planned[len(worlds)][0]+'/'+planned[len(worlds)][2].name if len(worlds)<len(planned) else None
            _admissions(output/'admission.jsonl',tuple(labels),tuple(starts),deadline,True,next_label)
            # Admission runs before raw/process creation, but after the next arm's
            # empty processes directory. Retained children cannot disappear from worlds.
            next_arm=planned[len(worlds)][0]
            active={arm for arm,_,_ in planned[:len(worlds)]}|{next_arm}
            expected_top={'result.json',*active}
            if (output/'admission.jsonl').exists():expected_top.add('admission.jsonl')
            require({p.name for p in output.iterdir()}==expected_top,'/pending','unregistered pending outputs')
            for arm in active:
                completed={run.name for a,_,run in planned[:len(worlds)] if a==arm}
                require({p.name for p in (output/arm).iterdir()}==({'processes','raw'} if completed else {'processes'}),
                        '/pending','unregistered pre-admission arm artifacts')
                require({p.name for p in (output/arm/'processes').iterdir()}==completed,
                        '/pending','omitted or unregistered completed children')
            raw_coverage(output,retained,indexes)
    products_current()
    require(freeze(output)=={**hashes,'result.json':receipt_sha256},'/outputs','artifacts changed during authentication')
    return PerfResult(target,receipt_sha256,state)
