#!/usr/bin/env python3
"""M-CORE paired evidence controller; no builds and no historical-ledger ratios.

freeze BASE_BIN_DIR NEW_EXE ACCEPTED_AUDIT OUT_CONFIG
run CONFIG NEW_SESSION_DIR
check SESSION_DIR [--output FRESH_EXTERNAL_RECEIPT]

Freeze after building NEW_EXE from the current checkout. This binds bytes and
source, but is not a claim that this tool observed the compiler invocation.
BASE provenance uses its preserved build-head, empty source diff, lock and
SHA256SUMS, and the immutable Git Rust tree; no old checkout is required.
A session cannot be resumed or splice another session's members. Invalid raw
attempts remain in attempts/, and only whole valid pairs enter accepted/.
"""
from __future__ import annotations

import hashlib
import json
import math
import os
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Mapping, Sequence, TypeAlias

Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
ROOT: Final = Path(__file__).resolve().parents[2]
SCENES: Final = ('n200', 'n1000', 'n2000', 'n200_script', 'n200_side64', 'n1000_wolves')
RSS_SCENES: Final = ('n200', 'n2000', 'n200_side64')
AUDIT_SHA: Final = '928d1fde99b32158ab8c667d444a9d7855e20ed402c662a06f1c03d835af0392'
BASE_MANIFEST_SHA: Final = '90983feb84c8c7d1ee97c9860a0ceafba79909fd0629b1cea71e47276430ef92'
EXPECTED_BASE: Final = '9855bd082ad88a4c49296410f206f148d98c4df9'
SHARE: Final = 0.6990029820337981
TICK_BAR: Final = 0.700498508983101
CEILINGS: Final = ('n1000_mean_le_5ms', 'n2000_mean_le_10ms', 'bytes_per_agent_le_4096',
                   'bytes_per_tile_le_96', 'rss_per_agent_le_5120', 'rss_per_tile_le_128')


@dataclass(frozen=True, slots=True)
class EvidenceError(Exception):
    """A concrete missing or inconsistent evidence obligation."""
    detail: str

    def __str__(self) -> str:
        return self.detail


@dataclass(frozen=True, slots=True)
class Document:
    """Checked JSON object boundary; nested values retain their explicit variants."""
    values: Mapping[str, Json]

    @classmethod
    def parse(cls, value: Json) -> Document:
        match value:
            case dict() as fields:
                return cls(fields)
            case _:
                raise EvidenceError('expected JSON object')

    @classmethod
    def read(cls, path: Path) -> Document:
        return cls.parse(json.loads(path.read_text()))

    def text(self, key: str) -> str:
        value = self.values.get(key)
        match value:
            case str() as text:
                return text
            case _:
                raise EvidenceError(f'expected string: {key}')

    def number(self, key: str) -> float:
        value = self.values.get(key)
        if type(value) not in (int, float) or not math.isfinite(value):
            raise EvidenceError(f'expected finite number: {key}')
        return float(value)

    def child(self, key: str) -> Document:
        return Document.parse(self.values.get(key))

    def rows(self, key: str) -> list[Document]:
        value = self.values.get(key)
        match value:
            case list() as rows:
                return [Document.parse(row) for row in rows]
            case _:
                raise EvidenceError(f'expected object array: {key}')


def require(condition: bool, detail: str) -> None:
    """Fail closed without converting missing evidence into a pass."""
    if not condition:
        raise EvidenceError(detail)


def sha(path: Path) -> str:
    """Stream immutable input bytes into SHA256."""
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write(path: Path, value: Json, *, exclusive: bool = False) -> None:
    """Write a deterministic receipt; callers never overwrite raw measurements."""
    with path.open('x' if exclusive else 'w') as stream:
        stream.write(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n')


def git(*args: str) -> str:
    """Read the current repository; never mutate its checkout."""
    return subprocess.check_output(['git', '-C', str(ROOT), *args], text=True).strip()


def source_hash() -> str:
    """Bind tracked and nonignored untracked Rust inputs, excluding build products."""
    names = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '-z', '--cached',
                                     '--others', '--exclude-standard', '--', 'rust']).split(b'\0')
    digest = hashlib.sha256()
    for raw in sorted(set(name for name in names if name)):
        path = ROOT / os.fsdecode(raw)
        require(path.is_file(), f'missing source input: {path}')
        digest.update(len(raw).to_bytes(8, 'little'))
        digest.update(raw)
        digest.update(bytes.fromhex(sha(path)))
    return digest.hexdigest()


def loads(value: str) -> tuple[float, float, float]:
    """Parse all three macOS/Linux uptime load samples, never only the first."""
    parts = value.replace(',', ' ').replace('{', ' ').replace('}', ' ').split()
    require(len(parts) == 3, f'invalid 1/5/15 load sample: {value}')
    sample = tuple(float(part) for part in parts)
    require(all(math.isfinite(n) and n >= 0 for n in sample), 'invalid load number')
    return sample[0], sample[1], sample[2]


def frozen_file(path: Path) -> Json:
    return {'path': str(path.resolve()), 'sha256': sha(path)}


def verify_file(record: Document) -> Path:
    path = Path(record.text('path'))
    require(sha(path) == record.text('sha256'), f'changed frozen file: {path}')
    return path


def audit_check(path: Path) -> None:
    """Use the accepted design share only as a bar; recheck its nine raw hashes."""
    require(sha(path) == AUDIT_SHA, 'unaccepted design-share audit hash')
    audit = Document.read(path)
    require(audit.values.get('complete') is True, 'incomplete design-share audit')
    rows = audit.rows('rows')
    require(len(rows) == 9, 'design share requires nine rows')
    for row in rows:
        require(row.text('status') == 'VALID', 'invalid design-share row')
        require(sha(path.parent / row.text('file')) == row.text('sha256'), 'changed design-share raw')
    n1000 = [r for r in audit.rows('scenes') if r.text('scene') == 'n1000']
    require(len(n1000) == 1 and abs(n1000[0].number('share_s') - SHARE) < 1e-15, 'wrong accepted share')


def validate_base(head: str, declared_base: str, rust_tree: str) -> None:
    """The frozen build may add reports but must represent the nominated BASE Rust tree."""
    require(declared_base == EXPECTED_BASE, 'wrong nominated BASE commit')
    expected_tree = git('rev-parse', EXPECTED_BASE + ':rust')
    require(rust_tree == expected_tree == git('rev-parse', head + ':rust'), 'frozen BASE differs from nominated Rust tree')
    require(subprocess.run(['git', '-C', str(ROOT), 'merge-base', '--is-ancestor', EXPECTED_BASE, head], check=False).returncode == 0,
            'BASE build HEAD is not a descendant of nominated BASE')


def validate_candidate(measured_head: str, current_head: str, expected_source: str, current_source: str) -> None:
    """Reports may follow measurement; unrelated history or changed Rust may not."""
    require(subprocess.run(['git', '-C', str(ROOT), 'merge-base', '--is-ancestor', measured_head, current_head], check=False).returncode == 0,
            'measured HEAD is not an ancestor of publication HEAD')
    require(expected_source == current_source, 'candidate source changed')


def validate_frozen_lock(lock: Path, manifest: Path, expected_manifest_sha: str) -> None:
    """Bind the untracked BASE lock to its preserved build manifest, not a git blob."""
    require(lock.is_file() and manifest.is_file(), 'missing frozen BASE lock proof')
    require(sha(manifest) == expected_manifest_sha, 'changed frozen BASE build manifest')
    entries = [line.split(maxsplit=1) for line in manifest.read_text().splitlines()]
    matches = [entry[0] for entry in entries
               if len(entry) == 2 and Path(entry[1].strip()).is_absolute()
               and Path(entry[1].strip()).resolve() == lock.resolve()]
    require(len(matches) == 1 and matches[0] == sha(lock), 'BASE lock/manifest mismatch')


def freeze(base: Path, candidate: Path, audit: Path, output: Path) -> None:
    """Bind the existing BASE build proof and current candidate without rebuilding."""
    require(not output.exists(), 'freeze output already exists')
    audit_check(audit)
    head = (base / 'build-head.txt').read_text().strip()
    validate_base(head, EXPECTED_BASE, git('rev-parse', head + ':rust'))
    require(not (base / 'source-diff.txt').read_bytes(), 'BASE has unexplained source diff')
    manifest = (base / 'SHA256SUMS').read_text().splitlines()
    listed = {line.split(maxsplit=1)[1].strip(): line.split()[0] for line in manifest}
    for name in ('worldsim_perf', 'Cargo.lock'):
        path = (base / name).resolve()
        require(listed.get(str(path)) == sha(path), f'BASE manifest mismatch: {name}')
    validate_frozen_lock(base / 'Cargo.lock', base / 'SHA256SUMS', BASE_MANIFEST_SHA)
    config: dict[str, Json] = {
        'schema': 1, 'repo': str(ROOT), 'candidate_head': git('rev-parse', 'HEAD'),
        'candidate_source_sha256': source_hash(), 'base_head': head, 'expected_base': EXPECTED_BASE,
        'base_rust_tree': git('rev-parse', f'{head}:rust'),
        'base': frozen_file(base / 'worldsim_perf'), 'new': frozen_file(candidate),
        'base_proof': [frozen_file(base / name) for name in ('build-head.txt', 'source-diff.txt', 'Cargo.lock', 'SHA256SUMS')],
        'audit': frozen_file(audit), 'commands': frozen_file(ROOT / 'tools/v8/bench_script.json'),
        'controller': frozen_file(Path(__file__)), 'launcher': frozen_file(ROOT / 'tools/v8/core_perf.sh'),
        'share_s': SHARE, 'tick_ratio_bar': TICK_BAR,
        'new_post_timing_saturation': True, 'load_limit_exclusive': 8,
        'build_binding_limit': 'Hashes bind supplied executables and sources; compiler invocation is external build evidence.',
    }
    write(output, config)


def verify_config(config: Document) -> None:
    """Gate reuse allows report descendants with identical measured inputs and executables."""
    require(config.text('repo') == str(ROOT), 'wrong repository')
    validate_candidate(config.text('candidate_head'), git('rev-parse', 'HEAD'), config.text('candidate_source_sha256'), source_hash())
    validate_base(config.text('base_head'), config.text('expected_base'), config.text('base_rust_tree'))
    require(config.text('base_rust_tree') == git('rev-parse', config.text('base_head') + ':rust'), 'BASE tree mismatch')
    for key in ('base', 'new', 'commands', 'controller', 'launcher', 'audit'):
        verify_file(config.child(key))
    for proof in config.rows('base_proof'):
        verify_file(proof)
    base_dir = verify_file(config.child('base')).parent
    validate_frozen_lock(base_dir / 'Cargo.lock', base_dir / 'SHA256SUMS', BASE_MANIFEST_SHA)
    audit_check(verify_file(config.child('audit')))


def admit(log: Path, label: str) -> tuple[float, float, float] | None:
    """Wait for <8; three high windows >=900 seconds apart defer timing."""
    high_windows = 0
    next_window = 0.0
    while True:
        sample = os.getloadavg()
        now = time.monotonic()
        high_window = sample[0] >= 8 and now >= next_window
        if high_window:
            high_windows += 1
            next_window = now + 900
        with log.open('a') as stream:
            stream.write(json.dumps({'label': label, 'utc': time.time(), 'monotonic': now,
                                     'loads_1_5_15': sample, 'high_window': high_windows if high_window else None,
                                     'status': 'ADMITTED' if sample[0] < 8 else 'NOT_RUN'}) + '\n')
        if sample[0] < 8:
            return sample
        if high_windows >= 3:
            return None
        time.sleep(20)


def sequence(build_only: bool) -> list[tuple[int, str]]:
    """The standing timed and RSS matrix, in repeat-major order."""
    return [(repeat, scene) for repeat in range(1, (5 if build_only else 3) + 1)
            for scene in (RSS_SCENES if build_only else SCENES)]


def validate_schedule(records: Sequence[Document]) -> None:
    """Consume adjacent pairs in exact matrix and retry order, not grouped order."""
    offset = 0
    previous_end = 0.0
    for build_only in (False, True):
        for repeat, scene in sequence(build_only):
            accepted = False
            for attempt in range(1, 4):
                require(offset + 2 <= len(records), 'incomplete chronological pair')
                pair = records[offset:offset + 2]
                roles = ('base', 'new') if repeat % 2 else ('new', 'base')
                accepted_flags = []
                for row, role in zip(pair, roles, strict=True):
                    require(row.values.get('build_only') is build_only and row.text('scene') == scene
                            and row.number('repeat') == repeat and row.number('attempt') == attempt
                            and row.text('role') == role, 'chronological pair/matrix/retry order mismatch')
                    require(row.number('started') >= previous_end and row.number('ended') >= row.number('started'), 'overlapping or reordered invocations')
                    previous_end = row.number('ended')
                    require(row.text('status') in ('VALID', 'INVALID'), 'unknown attempt validity status')
                    flag = row.values.get('pair_accepted')
                    require(type(flag) is bool, 'pair acceptance must be Boolean')
                    accepted_flags.append(flag)
                require(accepted_flags[0] == accepted_flags[1], 'accepted half-pair')
                valid = all(row.text('status') == 'VALID' for row in pair)
                require(accepted_flags[0] == valid, 'valid pair discarded or invalid pair accepted')
                offset += 2
                accepted = valid
                if accepted:
                    break
            require(accepted, 'no valid pair after three attempts')
    require(offset == len(records), 'extra records or retry after accepted pair')


def validate_dimensions(raw: Document, scene: str, build_only: bool, repeat: int) -> None:
    """Labels cannot disguise a smaller or differently configured measurement."""
    require(scene in SCENES, 'unknown scene')
    agents = 2000 if scene == 'n2000' else (1000 if scene in ('n1000', 'n1000_wolves') else 200)
    side = 64 if scene == 'n200_side64' else 192
    require(raw.text('label') == scene and raw.number('agents') == agents and raw.number('side') == side,
            'raw scene population/map mismatch')
    actual_kind = raw.values.get('build_only', False)
    require(type(actual_kind) is bool and actual_kind is build_only, 'raw measurement kind mismatch')
    if build_only:
        require(scene in RSS_SCENES and raw.number('repeat') == repeat, 'RSS scene/repeat mismatch')
    else:
        require(raw.values.get('script') is (scene == 'n200_script') and raw.number('wolves') == (4 if scene == 'n1000_wolves' else 0),
                'raw script/wolf configuration mismatch')
        require(raw.number('seed') == 15100 and raw.number('warmup_ticks') == 1440
                and raw.number('measured_ticks') == 2880 and raw.text('build_profile') == 'release', 'wrong timed protocol')


def run(config_path: Path, out: Path) -> int:
    """Execute only frozen binaries sequentially; never accept half of a pair."""
    config = Document.read(config_path)
    verify_config(config)
    out.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(config_path, out / 'config.json')
    for role in ('base', 'new'):
        (out / 'accepted' / role).mkdir(parents=True)
    (out / 'attempts').mkdir()
    records: list[Json] = []
    for build_only in (False, True):
        for repeat, scene in sequence(build_only):
            accepted = False
            for attempt in range(1, 4):
                pair: list[Json] = []
                roles = ('base', 'new') if repeat % 2 else ('new', 'base')
                for role in roles:
                    label = f'{"rss" if build_only else "timed"}-{scene}-r{repeat}-a{attempt}-{role}'
                    external = admit(out / 'admission.jsonl', label)
                    if external is None:
                        write(out / 'attempts.json', records + pair)
                        write(out / 'receipt.json', {'status': 'PENDING', 'reason': 'three high-load windows; timing deferred'})
                        return 75
                    verify_config(config)
                    raw = out / 'attempts' / (label + '.json')
                    argv = [str(verify_file(config.child(role))), 'bench-one', '--scene-label', scene,
                            '--out', str(raw), '--commands', str(verify_file(config.child('commands')))]
                    if build_only:
                        argv += ['--build-only', str(repeat)]
                    environment = os.environ.copy()
                    environment.pop('WORLDSIM_MIND_HISTORY_SATURATE', None)
                    if role == 'new' and not build_only:
                        environment['WORLDSIM_MIND_HISTORY_SATURATE'] = '1'
                    started = time.time()
                    with (out / 'attempts' / (label + '.log')).open('w') as stream:
                        result = subprocess.run(argv, cwd=ROOT, env=environment, stdout=stream, stderr=subprocess.STDOUT, check=False)
                    ended = time.time()
                    verify_config(config)
                    internal = None
                    parse_error = None
                    if result.returncode == 0 and raw.exists():
                        try:
                            internal = loads(Document.read(raw).text('load_before'))
                        except (EvidenceError, ValueError) as error:
                            parse_error = str(error)
                    valid = internal is not None and internal[0] < 8
                    row: dict[str, Json] = {'role': role, 'scene': scene, 'repeat': repeat, 'attempt': attempt,
                        'build_only': build_only, 'started': started, 'ended': ended, 'external_load': list(external),
                        'internal_load': list(internal) if internal else None, 'exit_code': result.returncode,
                        'raw': str(raw.resolve()), 'sha256': sha(raw) if raw.exists() else None,
                        'status': 'VALID' if valid else 'INVALID', 'pair_accepted': False, 'parse_error': parse_error}
                    pair.append(row)
                accepted = all(Document.parse(row).text('status') == 'VALID' for row in pair)
                for value in pair:
                    row = dict(Document.parse(value).values)
                    row['pair_accepted'] = accepted
                    records.append(row)
                    if accepted:
                        role = Document.parse(row).text('role')
                        shutil.copyfile(Document.parse(row).text('raw'), out / 'accepted' / role / f'{"rss" if build_only else "timed"}-{scene}-r{repeat}.json')
                write(out / 'attempts.json', records)
                if accepted:
                    break
            if not accepted:
                write(out / 'receipt.json', {'status': 'PENDING', 'reason': 'three invalid pair attempts retained'})
                return 75
    for role in ('base', 'new'):
        subprocess.run([str(verify_file(config.child(role))), 'bench-summarise', '--runs', str(out / 'accepted' / role),
                        '--out', str(out / f'{role}-summary.json'), '--commit', config.text('base_head' if role == 'base' else 'candidate_head')], cwd=ROOT, check=True)
    return check(out)


def read_records(path: Path) -> list[Document]:
    return Document.parse({'rows': json.loads(path.read_text())}).rows('rows')


def system_us(raw: Document, name: str) -> float:
    """Use arithmetic total system cost divided by measured ticks, not median shares."""
    rows = [row for row in raw.rows('systems') if row.text('name') == name]
    require(len(rows) == 1, f'missing or duplicated measured system: {name}')
    ticks = raw.number('measured_ticks')
    require(ticks == 2880, 'wrong measured tick count')
    cost = rows[0].number('total_ms') * 1000 / ticks
    require(cost > 0, f'nonpositive system cost: {name}')
    return cost


def mean_ratio(old: Sequence[float], new: Sequence[float]) -> float:
    """Ratio of matched arithmetic means; never mean of per-pair ratios."""
    require(len(old) == len(new) == 3 and min(old) > 0, 'three positive matched base repeats required')
    return statistics.mean(new) / statistics.mean(old)


def saturation(raws: Mapping[str, list[Document]]) -> Json:
    """Companion evidence, explicitly separate from standing after-build C3."""
    values: dict[str, list[float]] = {}
    methods: set[str] = set()
    missing: list[str] = []
    for scene, runs in raws.items():
        values[scene] = []
        for index, raw in enumerate(runs, 1):
            if 'mind_history_memory' not in raw.values:
                missing.append(f'{scene}/r{index}')
                continue
            evidence = raw.child('mind_history_memory')
            after = evidence.child('after')
            people = evidence.number('people')
            require(people > 0 and evidence.number('saturated_people') == people, 'unsaturated people in companion')
            require(evidence.number('capacity') == 16 and evidence.values.get('measured_after_saturation') is True,
                    'companion does not prove capacity16 measured afterward')
            require(after.number('min_records') == after.number('max_records') == 16
                    and after.number('records') == people * 16, 'companion ring occupancy mismatch')
            allocation = after.number('allocator_live_bytes')
            require(allocation > 0 and evidence.number('allocator_live_bytes') == allocation, 'invalid companion allocator checkpoint')
            require(abs(allocation - evidence.child('before').number('allocator_live_bytes')
                        - evidence.number('allocator_delta_bytes')) < 0.5, 'companion allocator delta mismatch')
            methods.add(evidence.text('method'))
            values[scene].append(allocation)
    if missing:
        return {'status': 'PENDING', 'missing': missing, 'standing_c3_instrument_unchanged': True}
    require(len(methods) == 1, 'mixed saturation methods')
    medians = {scene: statistics.median(rows) for scene, rows in values.items()}
    agent = (medians['n2000'] - medians['n200']) / 1800
    tile = (medians['n200'] - medians['n200_side64']) / (192**2 - 64**2)
    return {'status': 'MEASURED', 'method': next(iter(methods)), 'allocator_medians': medians,
            'bytes_per_agent': agent, 'bytes_per_tile': tile,
            'comparison_to_4096_bytes_per_agent': 'PASS' if agent <= 4096 else 'MISS',
            'comparison_to_96_bytes_per_tile': 'PASS' if tile <= 96 else 'MISS',
            'standing_c3_instrument_unchanged': True,
            'limitation': 'Synthetic full-ring capacity companion, not natural occupancy or a replacement for after-build standing C3.'}


def check(out: Path, output: Path | None = None) -> int:
    """Recheck evidence; an external output leaves the frozen session untouched."""
    if output is not None:
        output = output.resolve()
        require(not output.is_relative_to(out.resolve()), 'fresh receipt must be outside the frozen session')
        require(not output.exists(), 'fresh receipt output already exists')
    config = Document.read(out / 'config.json')
    verify_config(config)
    prior_receipt = out / 'receipt.json'
    if prior_receipt.exists():
        prior = Document.read(prior_receipt)
        if 'files' in prior.values:
            require(prior.text('config_sha256') == sha(out / 'config.json'), 'changed session configuration')
            for record in prior.rows('files'):
                path = verify_file(record)
                require(path.is_relative_to(out), 'receipt file escaped session')
    records = read_records(out / 'attempts.json')
    validate_schedule(records)
    admissions = [Document.parse(json.loads(line)) for line in (out / 'admission.jsonl').read_text().splitlines()]
    admitted_rows = [row for row in admissions if row.text('status') == 'ADMITTED']
    admitted = {row.text('label'): row for row in admitted_rows}
    require(len(admitted) == len(admitted_rows), 'duplicate external admission label')
    selected: dict[tuple[bool, str, int, str], Document] = {}
    previous_end = 0.0
    for row in records:
        raw_path = Path(row.text('raw'))
        require(raw_path.parent == (out / 'attempts').resolve(), 'raw is outside this session')
        require(row.number('started') >= previous_end, 'overlapping or reordered timed invocations')
        previous_end = row.number('ended')
        require(previous_end >= row.number('started'), 'negative invocation duration')
        if raw_path.exists():
            require(sha(raw_path) == row.text('sha256'), 'changed raw attempt')
        build_only = row.values.get('build_only')
        require(type(build_only) is bool, 'missing build-only discriminator')
        scene, repeat, role = row.text('scene'), int(row.number('repeat')), row.text('role')
        require(role in ('base', 'new'), 'unknown binary role')
        if row.values.get('pair_accepted') is not True:
            continue
        require(row.text('status') == 'VALID' and row.number('exit_code') == 0, 'invalid accepted member')
        raw = Document.read(raw_path)
        validate_dimensions(raw, scene, build_only, repeat)
        external = row.values.get('external_load')
        require(type(external) is list and len(external) == 3, 'missing external 1/5/15 loads')
        sample = loads(' '.join(str(value) for value in external))
        internal = loads(raw.text('load_before'))
        require(sample[0] < 8 and internal[0] < 8, 'contended accepted member')
        require(list(internal) == row.values.get('internal_load'), 'internal admission differs from raw')
        admission = admitted.get(raw_path.stem)
        require(admission is not None and admission.values.get('loads_1_5_15') == external, 'missing matching external admission')
        require(0 <= row.number('started') - admission.number('utc') <= 30, 'admission is not immediately before execution')
        if build_only:
            require(raw.number('repeat') == repeat, 'RSS repeat mismatch')
        else:
            require(raw.number('seed') == 15100 and raw.number('warmup_ticks') == 1440
                    and raw.number('measured_ticks') == 2880 and raw.text('build_profile') == 'release', 'wrong timed protocol')
        slot = (build_only, scene, repeat, role)
        require(slot not in selected, 'duplicate accepted repeat')
        selected[slot] = raw
        accepted_path = out / 'accepted' / role / f'{"rss" if build_only else "timed"}-{scene}-r{repeat}.json'
        require(sha(accepted_path) == sha(raw_path), 'summary input differs from raw pair')
    expected = {(build, scene, repeat, role) for build in (False, True)
                for repeat, scene in sequence(build) for role in ('base', 'new')}
    require(set(selected) == expected, 'missing or extra standing timed/RSS pairs')
    for role in ('base', 'new'):
        require(len(list((out / 'accepted' / role).glob('*.json'))) == 33, 'unexpected summary input file')
    summaries = {role: Document.read(out / f'{role}-summary.json') for role in ('base', 'new')}
    for role, summary in summaries.items():
        head = config.text('base_head' if role == 'base' else 'candidate_head')
        require(summary.text('commit') == head, 'summary commit mismatch')
        rows = summary.rows('scenes')
        require(len(rows) == 6 and {row.text('label') for row in rows} == set(SCENES), 'summary scene coverage')
        require(summary.child('rss_outer').number('repeats') == 5, 'RSS outer bound needs five repeats')
        for scene in rows:
            label = scene.text('label')
            require(scene.number('repeats') == 3, 'summary needs three timed repeats')
            for metric in ('tick_mean_us', 'tick_p99_us'):
                median = statistics.median(selected[(False, label, repeat, role)].number(metric) for repeat in (1, 2, 3))
                require(abs(scene.number(metric) - median) < 1e-8, 'summary differs from paired raws')
        med_alloc = {scene: statistics.median(selected[(False, scene, repeat, role)].child('alloc_bytes').number('after_build') for repeat in (1, 2, 3)) for scene in RSS_SCENES}
        alloc_agent = (med_alloc['n2000'] - med_alloc['n200']) / 1800
        alloc_tile = (med_alloc['n200'] - med_alloc['n200_side64']) / (192**2 - 64**2)
        rss_agent = statistics.median((selected[(True, 'n2000', repeat, role)].child('rss_kib').number('after_build') - selected[(True, 'n200', repeat, role)].child('rss_kib').number('after_build')) * 1024 / 1800 for repeat in range(1, 6))
        rss_tile = statistics.median((selected[(True, 'n200', repeat, role)].child('rss_kib').number('after_build') - selected[(True, 'n200_side64', repeat, role)].child('rss_kib').number('after_build')) * 1024 / (192**2 - 64**2) for repeat in range(1, 6))
        require(summary.text('memory_ceiling_source') == 'alloc', 'standing C3 lacks allocator counter')
        for field, expected_value in [('alloc_per_agent', alloc_agent), ('alloc_per_tile', alloc_tile)]:
            require(abs(summary.number(field) - expected_value) < 1e-8, 'allocator summary differs from raw')
        for field, expected_value in [('per_agent_median', rss_agent), ('per_tile_median', rss_tile)]:
            require(abs(summary.child('rss_outer').number(field) - expected_value) < 1e-8, 'RSS summary differs from raw pairs')
        by_label = {row.text('label'): row for row in rows}
        expected_ceilings = dict(zip(CEILINGS, [by_label['n1000'].number('tick_mean_us') <= 5000,
            by_label['n2000'].number('tick_mean_us') <= 10000, alloc_agent <= 4096,
            alloc_tile <= 96, rss_agent <= 5120, rss_tile <= 128], strict=True))
        for field, passes in expected_ceilings.items():
            require(summary.child('ceilings').text(field) == ('PASS' if passes else 'MISS'), 'C3 flag differs from raw instrument')
    old_rows = {row.text('label'): row for row in summaries['base'].rows('scenes')}
    new_rows = {row.text('label'): row for row in summaries['new'].rows('scenes')}
    failures: list[str] = []
    standing: list[Json] = []
    for scene in SCENES:
        for metric in ('tick_mean_us', 'tick_p99_us'):
            old, new = old_rows[scene].number(metric), new_rows[scene].number(metric)
            require(old > 0, 'nonpositive standing denominator')
            ratio = new / old
            standing.append({'scene': scene, 'metric': metric, 'base': old, 'new': new, 'ratio': ratio, 'bar': 1.1})
            if ratio > 1.1:
                failures.append(f'{scene}/{metric}>1.1')
    old_alloc = summaries['base'].number('alloc_per_agent')
    new_alloc = summaries['new'].number('alloc_per_agent')
    require(old_alloc > 0 and new_alloc > 0, 'missing/nonpositive allocator slope')
    if new_alloc / old_alloc > 1.1:
        failures.append('alloc_per_agent>1.1')
    for ceiling in CEILINGS:
        if summaries['new'].child('ceilings').values.get(ceiling) != 'PASS':
            failures.append(f'standing C3 {ceiling}')
    comparisons: list[Json] = []
    for scene in ('n200', 'n1000', 'n2000'):
        old = [selected[(False, scene, repeat, 'base')] for repeat in (1, 2, 3)]
        new = [selected[(False, scene, repeat, 'new')] for repeat in (1, 2, 3)]
        decisions = [system_us(raw, 'AgentDecisionSystem') for raw in old]
        minds = [system_us(raw, 'ColonyMindSystem') for raw in new]
        old_ticks = [raw.number('tick_mean_us') for raw in old]
        new_ticks = [raw.number('tick_mean_us') for raw in new]
        mind_ratio, tick_ratio = mean_ratio(decisions, minds), mean_ratio(old_ticks, new_ticks)
        comparisons.append({'scene': scene, 'base_decision_mean_us': statistics.mean(decisions),
            'new_mind_mean_us': statistics.mean(minds), 'base_tick_mean_us': statistics.mean(old_ticks),
            'new_tick_mean_us': statistics.mean(new_ticks), 'mind_ratio': mind_ratio, 'tick_ratio': tick_ratio,
            'contemporary_base_share': statistics.mean(decisions) / statistics.mean(old_ticks),
            'pair_mind_ratios': [n / o for o, n in zip(decisions, minds, strict=True)],
            'pair_tick_ratios': [n / o for o, n in zip(old_ticks, new_ticks, strict=True)]})
        if scene == 'n1000':
            if mind_ratio > 0.5:
                failures.append('N1000 mind/old decision>0.5')
            if tick_ratio > TICK_BAR:
                failures.append('N1000 tick ratio exceeds accepted arithmetic bar')
    companion = saturation({scene: [selected[(False, scene, repeat, 'new')] for repeat in (1, 2, 3)] for scene in SCENES})
    companion_pending = Document.parse(companion).text('status') == 'PENDING'
    status = 'FAIL' if failures else ('PENDING' if companion_pending else 'PASS')
    files = [path for path in out.rglob('*') if path.is_file() and path.name != 'receipt.json']
    receipt: dict[str, Json] = {'schema': 1, 'status': status, 'candidate_head': config.text('candidate_head'),
        'measured_head': config.text('candidate_head'), 'expected_base': EXPECTED_BASE,
        'candidate_source_sha256': config.text('candidate_source_sha256'), 'config_sha256': sha(out / 'config.json'),
        'base_exe_sha256': config.child('base').text('sha256'), 'new_exe_sha256': config.child('new').text('sha256'),
        'accepted_audit_sha256': AUDIT_SHA, 'share_s': SHARE, 'mind_ratio_bar': 0.5, 'tick_ratio_bar': TICK_BAR,
        'comparisons': comparisons, 'standing_ratios': standing, 'alloc_per_agent_ratio': new_alloc / old_alloc,
        'standing_c3': dict(summaries['new'].child('ceilings').values), 'saturation_companion': companion,
        'failures': failures, 'accepted_pairs': 33, 'invalid_attempt_members': sum(row.text('status') == 'INVALID' for row in records),
        'files': [frozen_file(path) for path in sorted(files)],
        'historical_ledger_used_as_base': False,
        'gate_reuse': 'Re-run check against current HEAD/source/binaries; an existing PASS string alone is insufficient.',
        'standing_justification': 'No waiver applied; all six standing scenes use the same-session BASE.',
        'build_only_internal_load_note': 'Legacy build-only load_before is sampled after build; external admission is the true pre-execution sample; both are conservatively checked.'}
    destination = output if output is not None else out / 'receipt.json'
    write(destination, receipt, exclusive=output is not None)
    print(json.dumps({'status': status, 'receipt': str(destination), 'failures': failures,
                      'measured_head': config.text('candidate_head'), 'publication_head': git('rev-parse', 'HEAD')}))
    return 1 if failures else (75 if companion_pending else 0)


def main(args: Sequence[str]) -> int:
    """Small positional CLI avoids adding a packaging/runtime dependency."""
    match list(args):
        case ['freeze', base, candidate, audit, output]:
            freeze(Path(base), Path(candidate), Path(audit), Path(output))
            return 0
        case ['run', config, out]:
            return run(Path(config).resolve(), Path(out).resolve())
        case ['check', out]:
            return check(Path(out).resolve())
        case ['check', out, '--output', output]:
            return check(Path(out).resolve(), Path(output).resolve())
        case _:
            raise EvidenceError(__doc__ or 'see module usage')


if __name__ == '__main__':
    try:
        raise SystemExit(main(sys.argv[1:]))
    except (EvidenceError, OSError, ValueError, subprocess.SubprocessError) as error:
        print(f'core perf: INVALID/PENDING: {error}', file=sys.stderr)
        raise SystemExit(2) from error
