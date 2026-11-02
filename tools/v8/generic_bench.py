# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m unittest discover -s tools/v8 -p test_generic_bench.py
"""Bound explicit product arms to fresh interleaved benchmark evidence.

Raw arithmetic and child-journal checks preserve the independently reviewed
Stage-minus1 audit contract. No function grants whole-gate publication authority.
"""
import hashlib
import json
import math
import os
import re
import subprocess
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from statistics import median
from typing import Final, Literal, TypedDict

from generic_gate import decode_json
from mux2_gate_inputs import Json
from read_gate_checks import check_bench_ratio, validate_bench_report

SCENES: Final = ('n200', 'n1000', 'n2000', 'n200_script', 'n200_side64', 'n1000_wolves')
RSS_SCENES: Final = ('n200', 'n2000', 'n200_side64')
# Frozen producer contract: sim-test/src/bench.rs SCENES and constants.
WORKLOADS: Final = {
    'n200': (200, 192, False, 0),
    'n1000': (1000, 192, False, 0),
    'n2000': (2000, 192, False, 0),
    'n200_script': (200, 192, True, 0),
    'n200_side64': (200, 64, False, 0),
    'n1000_wolves': (1000, 192, False, 4),
}

class AuditError(Exception):
    """A path-qualified evidence rejection, never a whole-gate verdict."""

class Round(TypedDict):
    means: dict[str, float]
    p99s: dict[str, float]
    rss_agent: float
    first: float
    last: float
    child_count: int
    wait_count: int
    maximum_load: float

def require(condition: bool, reason: str) -> None:
    if not condition: raise AuditError(reason)

def mapping(value: Json, label: str) -> Mapping[str, Json]:
    match value:
        case dict() as result: return result
        case _: raise AuditError(f'{label}: expected object')

def sequence(value: Json, label: str) -> list[Json]:
    match value:
        case list() as result: return result
        case _: raise AuditError(f'{label}: expected array')

def number(value: Json, label: str, *, positive: bool = False) -> float:
    require(type(value) in (int, float), f'{label}: expected numeric value, not bool')
    match value:
        case int() | float():
            result = float(value)
        case _: raise AuditError(f'{label}: expected number')
    require(math.isfinite(result), f'{label}: nonfinite')
    if positive: require(result > 0, f'{label}: not positive')
    return result

def loads(value: Json, label: str) -> list[float]:
    match value:
        case str():
            try: result = [float(part) for part in value.split()]
            except ValueError as exc: raise AuditError(f'{label}: bad load string') from exc
        case list(): result = [number(part, label) for part in value]
        case _: raise AuditError(f'{label}: expected load triplet')
    require(len(result) == 3 and all(math.isfinite(x) and x >= 0 for x in result), f'{label}: invalid triplet')
    return result

def unique(pairs: list[tuple[str, Json]]) -> dict[str, Json]:
    result: dict[str, Json] = {}
    for key, value in pairs:
        require(key not in result, f'duplicate JSON key {key}')
        result[key] = value
    return result

def invalid_constant(value: str) -> None:
    raise AuditError(f'nonfinite JSON constant {value}')

def parse(text: str, label: str) -> Mapping[str, Json]:
    try:
        value = decode_json(text, pairs_hook=unique, constant_hook=invalid_constant)
    except json.JSONDecodeError as exc:
        raise AuditError(f'{label}: invalid JSON') from exc
    def finite(item: Json) -> None:
        match item:
            case float(): require(math.isfinite(item), f'{label}: nonfinite JSON value')
            case list():
                for child in item: finite(child)
            case dict():
                for child in item.values(): finite(child)
            case None | bool() | int() | str(): pass
    finite(value)
    return mapping(value, label)

def read(path: Path, hashes: dict[str, str]) -> Mapping[str, Json]:
    require(path.is_file() and not path.is_symlink(), f'{path}: not a regular evidence file')
    try: data = path.read_bytes()
    except OSError as exc: raise AuditError(f'{path}: missing/unreadable evidence') from exc
    hashes[str(path)] = hashlib.sha256(data).hexdigest()
    return parse(data.decode(), str(path))

def equal(actual: Json, expected: Json, label: str) -> None:
    same = actual == expected
    if type(actual) in (int, float) and type(expected) in (int, float):
        a = number(actual, label); b = number(expected, label)
        same = abs(a-b) <= 4 * max(math.ulp(a), math.ulp(b))
    if (type(actual) is bool) != (type(expected) is bool): same = False
    require(same, f'{label}: raw/report mismatch {actual!r} != {expected!r}')

def identity(actual: Json, expected: str | int | bool, label: str) -> None:
    require(type(actual) is type(expected) and actual == expected,
            f'{label}: workload identity mismatch {actual!r} != {expected!r}')

def workload(child: Mapping[str, Json], scene: str, *, timed: bool) -> None:
    agents, side, script, wolves = WORKLOADS[scene]
    fields = {'label': scene, 'agents': agents, 'side': side}
    if timed:
        fields.update({'script': script, 'wolves': wolves, 'seed': 15100,
                       'warmup_ticks': 1440, 'measured_ticks': 2880,
                       'build_profile': 'release'})
    for key, value in fields.items():
        require(key in child, f'{scene}/{key}: missing workload identity')
        identity(child[key], value, f'{scene}/{key}')

def round_values(root: Path, commit: str, hashes: dict[str, str], *, require_ceilings: bool) -> Round:
    report = read(root / 'bench.json', hashes)
    validate_bench_report(report, require_ceilings=require_ceilings)
    equal(report['commit'], commit, str(root / 'bench.json'))
    identity(report['seed'], 15100, 'seed')
    identity(report['warmup_ticks'], 1440, 'warmup_ticks')
    identity(report['measured_ticks'], 2880, 'measured_ticks')
    rows = [mapping(row, 'scene') for row in sequence(report['scenes'], 'scenes')]
    require(len(rows) == 6 and all(scene in [row['label'] for row in rows] for scene in SCENES), 'scene coverage')
    raw = {scene: read(root / 'runs' / f'{scene}_r1.json', hashes) for scene in SCENES}
    means: dict[str, float] = {}; p99s: dict[str, float] = {}
    for row in rows:
        scene = str(row['label']); child = raw[scene]
        workload(child, scene, timed=True)
        agents, side, script, _ = WORKLOADS[scene]
        for key, value in (('agents', agents), ('side', side), ('script', script)):
            identity(row[key], value, f'{scene}/summary/{key}')
        identity(row['repeats'], 1, 'repeats')
        for field in ('tick_mean_us', 'tick_p99_us', 'tick_max_us'):
            value = number(child[field], scene + field, positive=True)
            equal(row[field], value, scene + field)
        means[scene] = number(child['tick_mean_us'], scene)
        p99s[scene] = number(child['tick_p99_us'], scene)
        require(loads(child['load_before'], scene)[0] < 8, f'{scene}: child start load >=8')
        for kind in ('rss_kib', 'alloc_bytes'):
            fields = ('before_build', 'after_build', 'after_run') if kind == 'rss_kib' else ('after_build',)
            for field in fields:
                value = number(mapping(child[kind], kind)[field], kind, positive=True)
                equal(row[f'{kind}_{field}'], value, scene + kind + field)
        repeats = sequence(row['per_repeat'], 'per_repeat'); require(len(repeats) == 1, 'per_repeat coverage')
        per = mapping(repeats[0], 'per_repeat')
        for report_key, raw_key in (('mean','tick_mean_us'),('p99','tick_p99_us'),('max','tick_max_us'),('load_before','load_before'),('load_after','load_after'),('alive_end','alive_end'),('fingerprint','fingerprint')):
            equal(per[report_key], child[raw_key], scene + report_key)
    def slope(kind: str, big: str, small: str, denominator: int) -> float:
        return (number(mapping(raw[big][kind], kind)['after_build'], kind) - number(mapping(raw[small][kind], kind)['after_build'], kind)) / denominator
    alloc_agent = slope('alloc_bytes', 'n2000', 'n200', 1800)
    alloc_tile = slope('alloc_bytes', 'n200', 'n200_side64', 32768)
    equal(report['alloc_per_agent'], alloc_agent, 'alloc_per_agent'); equal(report['alloc_per_tile'], alloc_tile, 'alloc_per_tile')
    equal(report['bytes_per_agent'], slope('rss_kib','n2000','n200',1800)*1024, 'bytes_per_agent')
    equal(report['bytes_per_tile'], slope('rss_kib','n200','n200_side64',32768)*1024, 'bytes_per_tile')
    equal(report['memory_ceiling_source'], 'alloc', 'memory ceiling source')
    rss: dict[tuple[str, int], float] = {}
    for repeat in range(1,6):
        for scene in RSS_SCENES:
            child = read(root / 'runs' / f'rss_{scene}_r{repeat}.json', hashes)
            workload(child, scene, timed=False)
            identity(child['repeat'], repeat, 'rss repeat'); identity(child['build_only'], True, 'build_only')
            require(loads(child['load_before'], scene)[0] < 8, f'{scene}: RSS child load >=8')
            rss[scene, repeat] = number(mapping(child['rss_kib'], 'rss')['after_build'], 'rss', positive=True)
    agents = [(rss['n2000',r]-rss['n200',r])*1024/1800 for r in range(1,6)]
    tiles = [(rss['n200',r]-rss['n200_side64',r])*1024/32768 for r in range(1,6)]
    outer = mapping(report['rss_outer'], 'rss_outer')
    outer_expected: dict[str, Json] = {'repeats': 5, 'per_agent': list(agents), 'per_tile': list(tiles),
                                      'per_agent_median': median(agents), 'per_tile_median': median(tiles),
                                      'agent_bound': 5120, 'tile_bound': 128}
    for key, expected in outer_expected.items():
        equal(outer[key], expected, 'rss_outer/' + key)
    computed = {'n1000_mean_le_5ms':means['n1000']<=5000,'n2000_mean_le_10ms':means['n2000']<=10000,'bytes_per_agent_le_4096':alloc_agent<=4096,'bytes_per_tile_le_96':alloc_tile<=96,'rss_per_agent_le_5120':median(agents)<=5120,'rss_per_tile_le_128':median(tiles)<=128}
    ceilings = mapping(report['ceilings'], 'ceilings'); require(ceilings.keys() == computed.keys(), 'ceiling coverage')
    for key, passed in computed.items():
        equal(ceilings[key], 'PASS' if passed else 'MISS', key)
        if require_ceilings: require(passed, f'{root.name}: C3 {key} failed')
    first,last,count,waits,maximum = journal(root, hashes)
    return Round(means=means,p99s=p99s,rss_agent=median(agents),first=first,last=last,child_count=count,wait_count=waits,maximum_load=maximum)

def journal(root: Path, hashes: dict[str, str]) -> tuple[float, float, int, int, float]:
    expected = [('timed', s, 1, f'{s}_r1.json') for s in SCENES]
    expected += [('rss', s, r, f'rss_{s}_r{r}.json') for r in range(1,6) for s in RSS_SCENES]
    require({p.name for p in (root/'runs').glob('*.json')} == {row[3] for row in expected}, f'{root}: raw child coverage')
    path = root / 'admission.jsonl'
    try: data = path.read_bytes()
    except OSError as exc: raise AuditError(f'{path}: missing journal') from exc
    hashes[str(path)] = hashlib.sha256(data).hexdigest()
    cursor = 0; waits = 0; first = 0.0; previous = 0.0; maximum = 0.0
    for line in data.decode().splitlines():
        row = parse(line, str(path))
        require(cursor < len(expected), 'extra/duplicate admission')
        kind,scene,repeat,filename = expected[cursor]
        identity(row['schema_version'], 1, 'schema_version')
        for key, value in (('kind',kind),('scene',scene),('repeat',repeat),('output',str(root/'runs'/filename))): identity(row[key], value, 'admission/'+key)
        triplet = loads(row['load_1_5_15'], 'journal loads')
        require(triplet == loads(row['load_raw'], 'load_raw'), 'journal raw/triplet mismatch')
        timestamp = number(row['unix_seconds'], 'unix_seconds', positive=True)
        require(timestamp >= previous, 'journal timestamp reversal')
        if not first: first = timestamp
        previous = timestamp
        require(number(row['waited_seconds'], 'waited_seconds') >= 0, 'negative wait')
        match row['decision']:
            case 'WAIT':
                require(triplet[0] >= 8, 'WAIT with low load'); waits += 1
            case 'ADMIT':
                require(triplet[0] < 8, 'ADMIT at load >=8')
                maximum = max(maximum, triplet[0]); cursor += 1
            case _: raise AuditError('invalid admission decision')
    require(cursor == len(expected), 'missing admission coverage')
    return first, previous, cursor, waits, maximum


def justified_ratio(base: float, candidate: float, scene: str, metric: str,
                    entries: tuple[dict[str, str], ...], changelog: str) -> float:
    """A larger limit needs an exact, committed, scene/metric-specific explanation."""
    require(scene in SCENES and metric in {'mean', 'p99'}, 'unknown ratio metric')
    try:
        passed, ratio = check_bench_ratio(base, candidate)
    except ValueError as exc:
        raise AuditError(str(exc)) from exc
    seen: set[tuple[str, str]] = set()
    selected = False
    for entry in entries:
        require(set(entry) == {'scene', 'metric', 'quote'}, 'bad justification schema')
        key = (entry['scene'], entry['metric'])
        require(key[0] in SCENES and key[1] in {'mean', 'p99'} and key not in seen,
                'unknown or duplicate justification')
        seen.add(key)
        quote = entry['quote']
        require(bool(quote.strip()) and quote in changelog and all(part in quote for part in key),
                'justification quote absent from candidate CHANGELOG or lacks scene/metric')
        selected |= key == (scene, metric)
    require(passed or (selected and ratio <= 1.15), f'{scene}/{metric}: median ratio {ratio} exceeds admitted limit')
    return ratio


@dataclass(frozen=True, slots=True)
class Product:
    """Runner-authenticated product; this adapter rechecks concrete artifacts."""
    root: Path
    binary: Path
    source_sha: str
    binary_sha256: str
    build_receipt: Path
    build_receipt_sha256: str
    input_files: tuple[tuple[str, str], ...] = ()


def sha256(path: Path) -> str:
    require(path.is_file() and not path.is_symlink(), f'{path}: expected regular artifact')
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def git_bytes(root: Path, *args: str) -> bytes:
    env = {key: value for key, value in os.environ.items() if not key.startswith('GIT_')}
    env.update(GIT_OPTIONAL_LOCKS='0', GIT_NO_REPLACE_OBJECTS='1')
    result = subprocess.run(['git', '-C', str(root), *args], env=env, capture_output=True, check=False)
    require(result.returncode == 0, f'git {args}: {result.stderr.decode(errors="replace")}')
    return result.stdout


def verify_product(product: Product) -> None:
    require(re.fullmatch('[0-9a-f]{40}', product.source_sha) is not None, 'full product SHA required')
    for value in (product.binary_sha256, product.build_receipt_sha256):
        require(re.fullmatch('[0-9a-f]{64}', value) is not None, 'full artifact SHA256 required')
    require(product.root.is_absolute() and product.root.resolve() == product.root, 'canonical product root required')
    require(product.binary.is_absolute() and product.binary.resolve() == product.binary, 'canonical binary required')
    require(product.binary.is_relative_to(product.root), 'binary outside product build root')
    require(git_bytes(product.root, 'rev-parse', 'HEAD').decode().strip() == product.source_sha, 'product HEAD drift')
    require(sha256(product.binary) == product.binary_sha256, 'binary changed')
    require(sha256(product.build_receipt) == product.build_receipt_sha256, 'build receipt changed')
    receipt = parse(product.build_receipt.read_text(), str(product.build_receipt))
    identity(receipt['source_sha'], product.source_sha, 'build source')
    identity(receipt['binary_sha256'], product.binary_sha256, 'build binary')
    identity(receipt['profile'], 'release', 'build profile')
    identity(receipt['exit_code'], 0, 'build exit')


def validate_products(base: Product, candidate: Product, purpose: str) -> None:
    require(purpose in {'publication', 'self_proof', 'defect_proof'}, 'unknown proof purpose')
    for product in (base, candidate):
        verify_product(product)
    require(not base.root.is_relative_to(candidate.root) and not candidate.root.is_relative_to(base.root),
            'product roots must be independent')
    require(not os.path.samefile(base.binary, candidate.binary), 'products share a binary inode')
    if purpose == 'self_proof':
        require(base.source_sha == candidate.source_sha, 'self proof must compare BASE with BASE')
    elif purpose == 'defect_proof':
        require(base.source_sha != candidate.source_sha, 'defect proof requires distinct source')


def write_new(path: Path, value: Json) -> None:
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        _ = stream.write('\n')


def compare_rounds(root: Path, base_sha: str, candidate_sha: str,
                   justifications: tuple[dict[str, str], ...] = (), changelog: str = '') -> dict[str, Json]:
    """Audit exact six rounds; does not replace source/build or world-safety checks."""
    hashes: dict[str, str] = {}
    names = [f'{arm}-r{repeat}' for repeat in range(1, 4) for arm in ('base', 'candidate')]
    require({path.name for path in root.iterdir() if path.is_dir()} == set(names), 'round directory coverage')
    rounds: dict[str, Round] = {}
    previous = 0.0
    for name in names:
        candidate = name.startswith('candidate')
        run = round_values(root / name, candidate_sha if candidate else base_sha, hashes,
                           require_ceilings=candidate)
        require(run['first'] > previous, 'interleaved round journals overlap or reverse')
        previous = run['last']
        rounds[name] = run
    comparisons: list[Json] = []
    for scene in SCENES:
        row: dict[str, Json] = {'scene': scene}
        metrics: tuple[tuple[str, Literal['means', 'p99s']], ...] = (('mean', 'means'), ('p99', 'p99s'))
        for metric, field in metrics:
            baseline = median(rounds[f'base-r{r}'][field][scene] for r in range(1, 4))
            new = median(rounds[f'candidate-r{r}'][field][scene] for r in range(1, 4))
            row[metric + '_ratio'] = justified_ratio(baseline, new, scene, metric, justifications, changelog)
        comparisons.append(row)
    return {'status': 'NUMERICAL_PASS', 'raw_children': sum(row['child_count'] for row in rounds.values()),
            'timed_children': 36, 'rss_children': 90,
            'waits': sum(row['wait_count'] for row in rounds.values()),
            'maximum_admit_load1': max(row['maximum_load'] for row in rounds.values()),
            'comparisons': comparisons, 'hashes': {key: value for key, value in hashes.items()},
            'limits': [('Not publication authorization; source/build authentication, ordinary world safety, '
                       'measurement exclusivity and remaining gate steps are required.')]}


