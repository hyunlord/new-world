"""Bind benchmark child worlds to authenticated aggregate facts and ordinary safety.

The caller owns source/build freeze and provenance of actual ProcessSpec/Result
objects. This adapter rechecks concrete receipts; it grants no publication authority.
"""
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from generic_bench import (
    RSS_SCENES,
    SCENES,
    WORKLOADS,
    Product,
    compare_rounds,
    git_bytes,
    process_inputs,
    validate_products,
)
from generic_gate import boundary, document, integer, require
from generic_observations import keys
from generic_perf_observations import PerfObservation, Plan, paired_safety, parse_native
from generic_process import (
    ProcessResult,
    ProcessSpec,
    environment_for,
    file_hash,
    real_path,
)
from mux2_gate_inputs import Json, Node


@dataclass(frozen=True, slots=True)
class Aggregate:
    arm: Literal['base', 'candidate']
    round_index: int
    product: Product
    process: ProcessSpec
    result: ProcessResult
    receipt_sha256: str


def children() -> tuple[tuple[str, str, int, str], ...]:
    return tuple([('timed', scene, 1, f'{scene}_r1.json') for scene in SCENES] +
                 [('rss', scene, repeat, f'rss_{scene}_r{repeat}.json')
                  for repeat in range(1, 6) for scene in RSS_SCENES])


def _remember(path: Path, digest: str, hashes: dict[str, str]) -> None:
    prior = hashes.get(str(path))
    require(prior is None or prior == digest, str(path), 'previously audited evidence changed')
    hashes[str(path)] = digest


def _read(path: Path, hashes: dict[str, str]) -> str:
    digest = file_hash(path)
    raw = path.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == digest, str(path), 'read bytes differ from audited digest')
    require(file_hash(path) == digest, str(path), 'evidence changed during read')
    _remember(path, digest, hashes)
    return raw.decode()


def _bind_aggregate(item: Aggregate, root: Path, script: Path, script_sha256: str,
                    hashes: dict[str, str]) -> None:
    spec, result, product = item.process, item.result, item.product
    out = root / f'{item.arm}-r{item.round_index}'
    expected_argv = (str(product.binary), 'bench', '--out', str(out), '--commands', str(script),
                     '--repeats', '1', '--commit', product.source_sha)
    require(spec.argv == expected_argv and spec.output_root == out and
            spec.cwd == spec.input_root == product.root and spec.executable == product.binary and
            spec.executable_sha256 == product.binary_sha256, '/aggregate', 'foreign aggregate plan')
    require(bool(product.input_files) and bool(spec.inputs) and len(dict(spec.inputs)) == len(spec.inputs), '/inputs', 'missing or duplicate effective inputs')
    require(all(dict(spec.inputs).get(name) == digest for name, digest in process_inputs(product)),
            '/inputs', 'product input inventory differs')
    bound_script = dict(spec.external_inputs).get(script)
    if script.is_relative_to(spec.input_root):
        bound_script = dict(spec.inputs).get(script.relative_to(spec.input_root).as_posix())
    require(bound_script == script_sha256, '/script', 'actual script is not input-bound')
    env = environment_for(spec)
    require(env.get('WORLDSIM_GATE_OBSERVER') == '1' and env.get('WORLDSIM_GIT_SHA') == product.source_sha,
            '/environment', 'observer or source environment differs')
    require(result.receipt == out / 'process.json' and result.status == 'PASS' and result.native_exit == 0
            and type(result.pid) is int and result.pid > 0, '/process', 'aggregate failed or lacks actual PID')
    receipt = document(_read(result.receipt, hashes))
    require(hashes[str(result.receipt)] == item.receipt_sha256, '/process', 'receipt hash mismatch')
    expected: dict[str, Json] = {
        'schema_version': 1, 'status': 'PASS', 'native_exit': 0, 'pid': result.pid, 'errors': [],
        'argv': list(spec.argv), 'cwd': str(spec.cwd), 'executable': str(spec.executable),
        'input_root': str(spec.input_root), 'output_root': str(out),
        'environment': dict(env), 'environment_policy': 'fixed_defaults_plus_explicit_only_no_inheritance',
        'fresh_directories': list(spec.fresh_directories),
        'expected_executable_sha256': spec.executable_sha256,
        'before_executable_sha256': spec.executable_sha256, 'after_executable_sha256': spec.executable_sha256,
    }
    for prefix in ('expected', 'before', 'after'):
        expected[prefix + '_inputs'] = dict(spec.inputs)
        expected[prefix + '_external_inputs'] = {str(path): digest for path, digest in spec.external_inputs}
    for key, value in expected.items():
        receipt.field(key).equals(value)
    for name in ('stdout', 'stderr'):
        path = out / (name + '.log')
        _remember(path, file_hash(path), hashes)
        receipt.field(name + '_sha256').equals(hashes[str(path)])
    _remember(spec.executable, file_hash(spec.executable), hashes)
    require(hashes[str(spec.executable)] == spec.executable_sha256, '/binary', 'binary changed since run')
    for name, digest in spec.inputs:
        path = Path(name)
        require(not path.is_absolute() and '..' not in path.parts, '/inputs', 'unsafe input path')
        source = spec.input_root / path
        _remember(source, file_hash(source), hashes)
        require(hashes[str(source)] == digest, '/inputs', 'input changed since run')
    require(len(dict(spec.external_inputs)) == len(spec.external_inputs), '/inputs', 'duplicate external input')
    for path, digest in spec.external_inputs:
        _remember(path, file_hash(path), hashes)
        require(hashes[str(path)] == digest, '/inputs', 'external input changed since run')


def _round(item: Aggregate, script: Path, hashes: dict[str, str]) -> tuple[PerfObservation, ...]:
    out = item.process.output_root
    expected = children()
    sidecars = out / 'gate_observations'
    real_path(sidecars, directory=True)
    require({path.name for path in sidecars.iterdir()} == {row[3] + '.safety.json' for row in expected},
            str(sidecars), 'missing or extra child sidecars')
    journal = _read(out / 'child-processes.jsonl', hashes).splitlines()
    require(len(journal) == len(expected) * 3, '/children', 'missing or extra child phases')
    observations: list[PerfObservation] = []
    fields = {'schema', 'sequence', 'child_index', 'phase', 'parent_pid', 'pid', 'argv', 'cwd',
              'output', 'kind', 'scene', 'repeat', 'native_exit', 'signal', 'success', 'error'}
    for index, (kind, scene, repeat, filename) in enumerate(expected):
        output = out / 'runs' / filename
        agents, side, scripted, wolves = WORKLOADS[scene]
        argv = [str(item.product.binary), 'bench-one']
        if kind == 'rss':
            argv.extend(['--build-only', str(repeat)])
        argv.extend(['--scene-label', scene, '--out', str(output)])
        if kind == 'timed' and scripted:
            argv.extend(['--commands', str(script)])
        child_pid: int | None = None
        for offset, phase in enumerate(('launch', 'spawned', 'completed')):
            row = document(journal[index * 3 + offset])
            keys(row, fields)
            identity: dict[str, Json] = {
                'schema': 'bench_child_process_v1', 'sequence': index * 3 + offset,
                'child_index': index, 'phase': phase, 'parent_pid': item.result.pid,
                'argv': list(argv), 'cwd': str(item.process.cwd), 'output': str(output),
                'kind': kind, 'scene': scene, 'repeat': repeat, 'signal': None, 'error': None,
                'native_exit': 0 if phase == 'completed' else None,
                'success': True if phase == 'completed' else None,
            }
            for key, value in identity.items():
                row.field(key).equals(value)
            if phase == 'spawned':
                child_pid = integer(row.field('pid'))
                require(child_pid > 0 and child_pid != item.result.pid, '/child/pid', 'invalid child PID')
            row.field('pid').equals(child_pid)
        require(child_pid is not None, '/child', 'missing completed child')
        assert child_pid is not None
        workload: dict[str, Json] = {'label': scene, 'side': side}
        workload.update({'script': scripted, 'wolves': wolves} if kind == 'timed' else {'repeat': repeat})
        plan = Plan('bench' if kind == 'timed' else 'build_only', 4320 if kind == 'timed' else 0,
                    15100, agents, str(output), child_pid, json.dumps(workload))
        raw = _read(output, hashes)
        observation = _read(sidecars / (filename + '.safety.json'), hashes)
        observations.append(parse_native(observation, raw, plan))
    return tuple(observations)


@boundary
def audit_bench_safety(root: Path, aggregates: tuple[Aggregate, ...], purpose: str,
                       script: Path, script_sha256: str,
                       justifications: tuple[dict[str, str], ...] = ()) -> dict[str, Json]:
    """Require actual runner facts; matching synthetic receipts alone are not authority."""
    order = tuple((arm, repeat) for repeat in range(1, 4) for arm in ('base', 'candidate'))
    require(tuple((item.arm, item.round_index) for item in aggregates) == order,
            '/aggregates', 'exact six ordered aggregate occurrences required')
    base, candidate = aggregates[0].product, aggregates[1].product
    validate_products(base, candidate, purpose)
    require(all(item.product == (base if item.arm == 'base' else candidate) for item in aggregates),
            '/products', 'product changed across rounds')
    real_path(root, directory=True)
    require(file_hash(script) == script_sha256, '/script', 'script hash mismatch')
    changelog = git_bytes(candidate.root, 'show', f'{candidate.source_sha}:CHANGELOG.md').decode()
    numeric = compare_rounds(root, base.source_sha, candidate.source_sha, justifications, changelog)
    hashes = {name: Node(value).text() for name, value in Node(numeric['hashes']).mapping().items()}
    observations: dict[tuple[str, int], tuple[PerfObservation, ...]] = {}
    for item in aggregates:
        _bind_aggregate(item, root, script, script_sha256, hashes)
        observations[item.arm, item.round_index] = _round(item, script, hashes)
    safety: list[Json] = []
    for repeat in range(1, 4):
        for index, (left, right) in enumerate(zip(observations['base', repeat], observations['candidate', repeat], strict=True)):
            safety.append({'round': repeat, 'child_index': index, 'safety': paired_safety(left, right)})
    validate_products(base, candidate, purpose)
    require(file_hash(script) == script_sha256, '/script', 'script changed during audit')
    for name, expected in hashes.items():
        require(file_hash(Path(name)) == expected, name, 'evidence changed during audit')
    return {'status': 'BENCH_NUMERICAL_AND_SAFETY_PASS', 'worlds': 126, 'pairs': 63,
            'numerical': numeric, 'safety': safety, 'hashes': dict(hashes), 'publication_authority': False,
            'authority_dependency': 'runner-authenticated products/source freeze and actual aggregate ProcessSpec/Result'}
