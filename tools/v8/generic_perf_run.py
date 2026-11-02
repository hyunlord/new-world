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
from pathlib import Path
from typing import Literal, Protocol, runtime_checkable

import perf_summary
from core_p7_admission import AdmissionError, admit
from generic_bench import AuditError, Product, validate_products, verify_product
from generic_gate import document, require, typed_json
from generic_perf_observations import PerfObservation, Plan, paired_safety, parse_native
from generic_process import ProcessSpec, atomic_json, execute, file_hash
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
                    process = execute(ProcessSpec(run.argv, product.root, product.binary, product.binary_sha256,
                                                  product.root, product.input_files,
                                                  arm_root / 'processes' / run.name,
                                                  (('WORLDSIM_GATE_OBSERVER','1'),
                                                   ('WORLDSIM_GIT_SHA',product.source_sha))))
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
