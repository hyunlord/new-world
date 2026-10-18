#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/revitalize/evaluate_perf_pass18.py --stage a --input RAW_DIR --out REPORT_DIR
"""Evaluate Pass 18 timing only (Pass 17's evaluator; owner string and stage c added) from complete, hash-bound quiet-hold evidence."""

import argparse
from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import re
import statistics
import sys
from typing import Final

from compare_identity_pass17 import JsonValue, reject_constant
from perf_p15 import loglog

OWNER: Final = 'pass18-sim (Claude Code, new-world-pass4)'
JsonObject = dict[str, JsonValue]


@dataclass(frozen=True, slots=True)
class EvidenceError(ValueError):
    detail: str

    def __str__(self) -> str:
        return self.detail


@dataclass(frozen=True, slots=True)
class Batch:
    role: str
    directory: str
    mode: str
    version: str
    sizes: tuple[int, ...]
    days: int


def require(condition: bool, detail: str) -> None:
    if not condition:
        raise EvidenceError(detail)


def record(value: JsonValue) -> JsonObject:
    match value:
        case dict() as result:
            return result
        case _:
            raise EvidenceError('expected JSON object')


def number(value: JsonValue) -> float:
    match value:
        case bool():
            raise EvidenceError('boolean supplied as a measurement')
        case int() | float():
            require(math.isfinite(value), 'nonfinite measurement')
            return float(value)
        case _:
            raise EvidenceError('expected finite number')


def digest(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path: Path) -> JsonObject:
    return record(json.loads(path.read_text(), parse_constant=reject_constant))


def batches(stage: str) -> list[Batch]:
    require(stage in ('a', 'b', 'c'), 'stage must be a, b or c')
    old, new = {'a': ('20', '21'), 'b': ('21', '22'), 'c': ('22', '23')}[stage]
    return ([Batch('final', f'a/r{r}', 'stress', version, (500,), 1) for r in (1, 2, 3) for version in (old, new)]
            + [Batch('baseline', 'quick_base', 'quick', '19', (500,), 1), Batch('final', 'quick_final', 'quick', '19', (500,), 1)]
            + [Batch('final', f'b_game_v{version}', 'stress', version, (100, 500, 1000), 2) for version in (new, old)])


def command(root: Path, batch: Batch, binary: str) -> list[str]:
    args = [binary, batch.mode]
    if batch.mode == 'stress':
        args += ['--seeds', '15100..15101', '--agents', ','.join(map(str, batch.sizes)), '--side', '64', '--days', str(batch.days), '--feed', 'fed', '--rules', f'game_v{batch.version}']
    return args + ['--out', str(root / batch.directory)]


def verify_provenance(root: Path, stage: str) -> JsonObject:
    manifest = read(root / 'provenance.json')
    require(type(manifest['schema_version']) is int and manifest['schema_version'] == 1 and manifest['stage'] == stage and type(manifest['status']) is int and manifest['status'] == 0, 'incomplete or wrong-stage provenance')
    builds = record(manifest['builds'])
    for role in ('final', 'baseline'):
        build = record(builds[role])
        require(bool(re.fullmatch('[0-9a-f]{40}', str(build['commit']))), f'{role} commit missing')
        require(digest(Path(str(build['path']))) == build['sha256'], f'{role} binary changed')
    quiet = record(manifest['quiet'])
    require(quiet['owner'] == OWNER and str(quiet['pid']).isdigit() and int(str(quiet['pid'])) > 1, 'wrong quiet owner')
    require(type(quiet['contended']) is bool, 'missing contention classification')
    log = Path(str(quiet['log'])).read_bytes()
    prefix_bytes = int(number(quiet['prefix_bytes']))
    require(type(quiet['prefix_bytes']) is int and 0 < prefix_bytes <= len(log) and hashlib.sha256(log[:prefix_bytes]).hexdigest() == quiet['prefix_sha256'], 'quiet-log prefix changed')
    purpose = str(quiet['purpose'])
    starts = re.findall(r'HOLD start: ' + re.escape(purpose) + r'; load=([0-9.]+); contended=([01]); waited=([0-9.]+)s', log[:prefix_bytes].decode())
    require(len(starts) == 1 and (starts[0][1] == '1') is quiet['contended'], 'quiet hold start missing or contention differs')
    ends = re.findall(r'HOLD end: ' + re.escape(purpose) + r'; exit=0; held=([0-9.]+)s', log[prefix_bytes:].decode())
    require(len(ends) == 1 and 0 <= float(ends[0]) <= 3600, 'successful quiet hold end missing or over 60 minutes')
    started = datetime.fromisoformat(str(quiet['started']).replace('Z', '+00:00')).timestamp()
    for key in ('commands', 'runlog'):
        item = record(manifest[key])
        require(digest(root / str(item['file'])) == item['sha256'], f'{key} log changed')
    ledger = root / str(record(manifest['commands'])['file'])
    rows = [record(json.loads(line, parse_constant=reject_constant)) for line in ledger.read_text().splitlines()]
    expected = batches(stage)
    require(len(rows) == len(expected), 'expected exactly ten command records')
    indexes = record(manifest['indexes'])
    require(indexes.keys() == {f'{batch.directory}/runs.json' for batch in expected}, 'run-index inventory differs')
    for relative, sha in indexes.items():
        require(digest(root / relative) == sha, f'run index changed: {relative}')
    previous_end = started
    for index, (row, batch) in enumerate(zip(rows, expected)):
        build = record(builds[batch.role])
        require(type(row['index']) is int and row['index'] == index and row['build'] == batch.role and type(row['exit_code']) is int and row['exit_code'] == 0, f'command {index} failed or out of order')
        require(row['argv'] == command(root, batch, str(build['path'])), f'command {index} workload changed')
        require(row['binary_sha256_before'] == row['binary_sha256_after'] == build['sha256'], f'command {index} binary changed')
        begin, end = number(row['started']), number(row['finished'])
        require(previous_end <= begin <= end <= started + float(ends[0]) + 1, f'command {index} outside recorded quiet hold')
        previous_end = end
        paths = {f'{batch.directory}/{batch.mode}_stress_v{batch.version}_s15100_n{size}_side64_fed.json' for size in batch.sizes}
        outputs = record(row['outputs'])
        require(outputs.keys() == paths, f'command {index} output inventory differs')
        for relative, sha in outputs.items():
            require(digest(root / relative) == sha, f'run changed: {relative}')
    manifest['manifest_sha256'] = digest(root / 'provenance.json')
    manifest['quiet_log_sha256'] = hashlib.sha256(log).hexdigest()
    manifest['input_directory'] = str(root)
    manifest['command_records'] = rows
    return manifest


def load_batch(root: Path, batch: Batch, provenance: JsonObject) -> list[JsonObject]:
    build = record(record(provenance['builds'])[batch.role])
    quiet = record(provenance['quiet'])
    index = read(root / batch.directory / 'runs.json')
    require(type(index['runs']) is list, 'run index must contain a list')
    expected_names = {f'{item.mode}_stress_v{item.version}_s15100_n{size}_side64_fed.json' for item in batches(str(provenance['stage'])) if item.directory == batch.directory for size in item.sizes}
    require(len(index['runs']) == len(expected_names) and {record(item).get('file') for item in index['runs']} == expected_names, 'run index contains missing or extra records')
    results: list[JsonObject] = []
    for size in batch.sizes:
        name = f'{batch.mode}_stress_v{batch.version}_s15100_n{size}_side64_fed.json'
        entries = [record(entry) for entry in index['runs'] if record(entry).get('file') == name]
        require(len(entries) == 1 and entries[0].get('status') == 'ok', f'run index missing, duplicate, or failed: {name}')
        path = root / batch.directory / name
        run = read(path)
        expected: JsonObject = {'mode': batch.mode, 'scene': f'stress_v{batch.version}', 'seed': 15100, 'agents_requested': size, 'founders': size, 'side': 64, 'feed': 'fed', 'build_profile': 'release', 'ticks_per_day': 1440, 'ticks': batch.days * 1440, 'days': float(batch.days), 'git_sha': build['commit'], 'contended': quiet['contended']}
        require(all(run.get(key) == value and type(run.get(key)) is type(value) for key, value in expected.items()), f'wrong run configuration: {name}')
        tick = record(run['tick_summary'])
        require(record(run['warmup'])['ticks'] == 60 and tick['ticks'] == batch.days * 1440 - 60, f'wrong warmup: {name}')
        require(type(run['systems']) is list, f'missing systems: {name}')
        systems = [record(item) for item in run['systems'] if record(item).get('name') == 'AgentDecisionSystem']
        require(len(systems) == 1 and number(systems[0]['calls']) > 0, f'missing or duplicate AgentDecisionSystem: {name}')
        require(bool(record(run['machine'])) and isinstance(run['fingerprint'], str) and bool(run['fingerprint']), f'missing machine or fingerprint: {name}')
        measured: JsonObject = {'file': str(path), 'sha256': digest(path), 'mean_ms': number(tick['mean_ms']), 'p999_ms': number(tick['p999_ms']), 'max_ms': number(tick['max_ms']), 'alive': number(run['mean_alive_0600']), 'decision_us': number(systems[0]['mean_us_per_call']), 'decision_share': number(systems[0]['share_of_tick_time']), 'fingerprint': run['fingerprint'], 'machine': run['machine']}
        require(all(number(measured[key]) > 0 for key in ('mean_ms', 'p999_ms', 'max_ms', 'alive', 'decision_us')), f'nonpositive timing/population: {name}')
        results.append(measured)
    return results


def summarize(rows: list[JsonObject]) -> JsonObject:
    return {key: statistics.median(number(row[key]) for row in rows) for key in ('mean_ms', 'p999_ms')}


def relative_change(candidate: JsonValue, reference: JsonValue) -> float:
    return float(Decimal(str(number(candidate))) / Decimal(str(number(reference))) - 1)


def evaluate(root: Path, stage: str) -> JsonObject:
    result: JsonObject = {'schema_version': 1, 'stage': stage, 'verdict': 'ERROR', 'passed': None, 'errors': [], 'checks': {}}
    try:
        root = root.resolve()
        provenance = verify_provenance(root, stage)
        loaded = [load_batch(root, batch, provenance) for batch in batches(stage)]
        all_runs = [run for batch in loaded for run in batch]
        require(all(run['machine'] == all_runs[0]['machine'] for run in all_runs), 'measurement machines differ')
        old, new = [loaded[i][0] for i in (0, 2, 4)], [loaded[i][0] for i in (1, 3, 5)]
        require(len({str(run['fingerprint']) for run in old}) == len({str(run['fingerprint']) for run in new}) == 1, 'repeat fingerprints differ')
        med_old, med_new = summarize(old), summarize(new)
        slope_new = record(loglog([number(row['alive']) for row in loaded[8]], [number(row['decision_us']) for row in loaded[8]]))
        slope_old = record(loglog([number(row['alive']) for row in loaded[9]], [number(row['decision_us']) for row in loaded[9]]))
        checks: JsonObject = {}
        for name, value, limit, strict in (
            ('mean_relative', relative_change(med_new['mean_ms'], med_old['mean_ms']), 0.15, False),
            ('mean_absolute', number(med_new['mean_ms']), 4.0, False),
            ('p999_relative', relative_change(med_new['p999_ms'], med_old['p999_ms']), 0.35, False),
            ('slope', number(slope_new['b']), 1.7, True),
        ):
            require(math.isfinite(value), f'nonfinite computed check: {name}')
            checks[name] = {'value': value, 'limit': limit, 'operator': '<' if strict else '<=', 'passed': value < limit if strict else value <= limit}
        passed = all(record(check)['passed'] is True for check in checks.values())
        result.update({'verdict': 'PASS' if passed else 'FAIL', 'passed': passed, 'checks': checks, 'repeats': {'old': old, 'new': new}, 'medians': {'old': med_old, 'new': med_new}, 'slopes': {'old': slope_old, 'new': slope_new, 'reference': 1.06, 'old_points': loaded[9], 'new_points': loaded[8]}, 'quick': {'base': loaded[6][0], 'final': loaded[7][0]}, 'provenance': provenance})
    except (OSError, KeyError, TypeError, ValueError, OverflowError) as error:
        result['errors'] = [str(error)]
    return result


def render(result: JsonObject) -> str:
    lines = [f'# Pass 18 stage {result["stage"]} performance: {result["verdict"]}', '', '| Check | Value | Limit | Pass |', '|---|---:|---:|---|']
    for name, value in record(result['checks']).items():
        check = record(value)
        lines.append(f'| {name} | {check["value"]} | {check["operator"]} {check["limit"]} | {check["passed"]} |')
    lines += ['', 'AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.']
    if result['verdict'] != 'ERROR':
        for field in ('repeats', 'medians', 'slopes', 'quick', 'provenance'):
            lines += ['', f'## {field}', '', '```json', json.dumps(result[field], indent=2, allow_nan=False), '```']
    else:
        lines += ['', 'Errors:', json.dumps(result['errors'], indent=2)]
    return '\n'.join(lines) + '\n'


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--stage', choices=('a', 'b', 'c'), required=True)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args(argv)
    result = evaluate(args.input, args.stage)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / 'perf.json').write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    (args.out / 'perf.md').write_text(render(result))
    print(f'{result["verdict"]}: {args.out / "perf.json"}')
    return {'PASS': 0, 'FAIL': 1, 'ERROR': 2}[str(result['verdict'])]


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
