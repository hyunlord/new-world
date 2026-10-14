# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Normalize source-bound P8 QA evidence without changing acceptance thresholds."""
from __future__ import annotations

import gzip
import hashlib
import json
import math
import sys
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import TypeAlias

Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
Document: TypeAlias = dict[str, Json]


@dataclass(frozen=True, slots=True)
class EvidenceError(ValueError):
    detail: str


def mapping(value: Json) -> Document:
    """Parse one JSON object boundary."""
    if not isinstance(value, dict):
        raise EvidenceError('expected JSON object')
    return value


def array(value: Json) -> list[Json]:
    """Require an array before iterating a JSON union."""
    if not isinstance(value, list):
        raise EvidenceError('expected JSON array')
    return value


def rows(value: Json) -> list[Document]:
    """Parse object-array boundaries without coercing missing data."""
    if not isinstance(value, list):
        raise EvidenceError('expected JSON array')
    return [mapping(row) for row in value]


def number(value: Json) -> float:
    """Reject booleans, missing values and nonfinite evidence."""
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise EvidenceError('expected finite number')
    return float(value)


def json_decoder() -> Callable[[bytes], Json]:
    """Constrain the standard JSON decoder to its recursive JSON result domain."""
    return json.loads


def load(path: Path) -> Document:
    """Read JSON or gzip JSON without modifying original evidence."""
    raw = path.read_bytes()
    decode = json_decoder()
    return mapping(decode(gzip.decompress(raw) if path.suffix == '.gz' else raw))


def source(path: Path) -> Document:
    return {'path': str(path.resolve()), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def normalize(manifest: Path, performance: Path, calendar: Path, old_pacing: list[Path]) -> Document:
    """Return QA evidence with independently checked identities and source hashes."""
    raw, cpu, dates = load(manifest), load(performance), load(calendar)
    sources: list[Json] = [source(path) for path in [manifest, performance, calendar, *old_pacing]]
    seed = dates['society_seed']
    clock = mapping(dates['clock'])
    tpd, offset = number(clock['ticks_per_day']), number(clock['offset'])
    selected = [row for row in rows(dates['selection']) if row['seed'] == seed]
    if len(selected) != 1:
        raise EvidenceError('calendar must select exactly one society seed')
    expected_days = sorted(number(row['day']) for row in rows(selected[0]['cast_days']))
    if len(expected_days) != 3 or len(set(expected_days)) != 3:
        raise EvidenceError('calendar must contain three distinct measured cast days')
    def day_of(tick: Json) -> int:
        return int((number(tick) + offset) // tpd) + 1
    perf_errors: list[str] = []
    if cpu.get('valid') is not True or cpu.get('errors'):
        perf_errors.append('raw CPU collection invalid')
    if cpu.get('seed') != seed or cpu.get('tick') != selected[0]['first_three_alive_tick']:
        perf_errors.append('CPU seed/tick differs from measured first-three frame')
    if not cpu.get('git_sha') or not cpu.get('run_id'):
        perf_errors.append('CPU source code/run identity missing')
    before = mapping(cpu.get('load_before', {}))
    cpu['uptime'] = ''.join(str(item) for item in array(before.get('uptime', [])))
    cpu['processes'] = ''.join(str(item) for item in array(before.get('processes', [])))
    cpu['first_three_alive_tick'] = selected[0]['first_three_alive_tick']
    cpu['old_pacing_ms'] = [load(path)['median_ms'] for path in old_pacing]
    cpu['old_pacing_sources'] = [{'source': source(path), 'raw': load(path), 'metric': 'historical P7 wall-clock pacing; not CPU'} for path in old_pacing]
    cpu['normalization_errors'] = [error for error in perf_errors]
    paired: Document = {}
    pair_errors: list[str] = []
    identities: list[tuple[str, float]] = []
    cast_ticks: list[list[float]] = []
    final_days: list[bool] = []
    code_shas: set[str] = set()
    for label, kind in [('P', 'spoil_food'), ('G', 'gift_food')]:
        errors: list[str] = []
        casts = [row for row in rows(raw.get('intervention_casts', [])) if row.get('world_label') == label]
        frames = [row for row in rows(raw.get('captures', [])) if row.get('world_label') == label]
        identity = {(str(row.get('run_id', raw.get('run_id', ''))), number(row.get('world_epoch'))) for row in [*casts, *frames]}
        if len(identity) != 1 or not next(iter(identity), ('', 0))[0]:
            errors.append('world must have one nonempty run/epoch identity')
        else:
            identities.append(next(iter(identity)))
        if len(casts) != 3 or sorted(day_of(row['tick']) for row in casts) != expected_days:
            errors.append('requires exactly three casts on measured days')
        cast_ticks.append(sorted(number(row['tick']) for row in casts))
        code_shas.update(str(row.get('git_sha', raw.get('git_sha', ''))) for row in [*casts, *frames])
        originals: set[float] = set()
        for cast in casts:
            options = mapping(cast.get('capture_options', {}))
            pre = mapping(cast.get('precast_original', {}))
            reply = mapping(cast.get('reply', {}))
            if options.get('paired_world') != label or options.get('case_id') != 'a33':
                errors.append('cast label/options disagree')
            if cast.get('seed') != seed or cast.get('kind') != kind or cast.get('amount') != 60 or reply.get('ok') is not True:
                errors.append('cast seed/kind/amount/reply mismatch')
            if pre.get('found') is not True or pre.get('settlement_id') != cast.get('settlement_id'):
                errors.append('original target missing or mismatched')
                continue
            originals.add(number(pre['settlement_id']))
            need = mapping(pre.get('need_provenance', {}))
            line = max(1, math.floor(number(pre['members']) * number(pre['adult_need_per_day']) * number(pre['scarcity_days'])))
            if need.get('valid') is not True or not need.get('source_files') or pre.get('scarcity_line') != line or pre.get('below_scarcity_line') != (number(pre['food']) < line):
                errors.append('precast scarcity evidence inconsistent')
        if len(originals) != 1:
            errors.append('casts must target one actual original village')
        original = next(iter(originals), -1)
        observed: list[Document] = []
        for frame in frames:
            side_path = manifest.parent / str(frame['entities'])
            if not side_path.resolve().is_relative_to(manifest.parent.resolve()):
                raise EvidenceError('sidecar escapes manifest directory')
            side = load(side_path)
            sources.append(source(side_path))
            for field in ['world_epoch', 'world_label', 'seed', 'tick', 'capture_options']:
                if side.get(field) != frame.get(field):
                    errors.append('frame/sidecar mismatch: ' + field)
            if side.get('run_id', frame.get('run_id', raw.get('run_id'))) != frame.get('run_id', raw.get('run_id')):
                errors.append('frame/sidecar run ID mismatch')
            code_shas.add(str(side.get('git_sha', raw.get('git_sha', ''))))
            if side.get('seed') != seed:
                errors.append('sidecar wrong society seed')
            options = mapping(side.get('capture_options', {}))
            if options.get('paired_world') != label or options.get('case_id') != 'a33':
                errors.append('frame label/options disagree')
            side['_source'] = source(side_path)
            side['_run_id'] = frame.get('run_id', raw.get('run_id'))
            observed.append(side)
        mornings = sorted([row for row in observed if mapping(row['capture_options']).get('role') == 'morning'], key=lambda row: number(row['tick']))
        finals = [row for row in observed if mapping(row['capture_options']).get('role') == 'final']
        if len(mornings) != 3 or [day_of(row['tick']) for row in mornings] != [day + 1 for day in expected_days]:
            errors.append('requires morning after each measured cast day')
        for morning in mornings:
            if (number(morning['tick']) + offset) % tpd != 360:
                errors.append('morning observation is not 06:00')
        final_valid = len(finals) == 1 and day_of(finals[0]['tick']) == 120
        final_days.append(final_valid)
        if not final_valid:
            errors.append('requires one actual day120 final')
        meter = mapping(mornings[-1].get('original_settlement_observation', {})) if mornings else {}
        if meter.get('found') is not True or meter.get('settlement_id') != original:
            errors.append('last morning original meter missing/mismatched')
        pressure, threshold = meter.get('pressure'), meter.get('threshold')
        if pressure is None or threshold is None or number(pressure) < 0 or number(threshold) <= 0:
            errors.append('last morning P/theta unavailable')
            pressure, threshold = 0, 0
        departures: list[float] = []
        if finals:
            society = mapping(finals[0].get('society_snapshot', {}))
            if 'fission_parents' not in society or 'fission_depart_ticks' not in society:
                errors.append('final fission getter columns missing')
            parents, ticks = society.get('fission_parents', []), society.get('fission_depart_ticks', [])
            if not isinstance(parents, list) or not isinstance(ticks, list) or len(parents) != len(ticks):
                errors.append('fission getter columns misaligned')
            else:
                departures = [number(tick) for parent, tick in zip(parents, ticks, strict=True) if parent == original and number(tick) >= 0]
        paired[label] = {'pressure': pressure, 'threshold': threshold, 'departure_tick': min(departures) if departures else None,
                         'departure_day': day_of(min(departures)) if departures else None,
                         'casts': [cast for cast in casts], 'last_morning': mornings[-1] if mornings else None, 'final': finals[0] if finals else None}
        pair_errors.extend(label + ': ' + error for error in errors)
    if len(identities) != 2 or identities[0] == identities[1]:
        pair_errors.append('P/G require distinct reset worlds')
    if len(cast_ticks) != 2 or cast_ticks[0] != cast_ticks[1]:
        pair_errors.append('P/G cast ticks differ')
    if len(code_shas) != 1 or '' in code_shas:
        pair_errors.append('P/G source code SHA differs or is absent')
    if not raw.get('git_sha'):
        pair_errors.append('paired manifest source SHA missing')
    paired['same_seed_and_cast_days'] = not pair_errors
    paired['both_reached_day120'] = len(final_days) == 2 and all(final_days)
    paired['normalization_errors'] = [error for error in pair_errors]
    errors = perf_errors + pair_errors
    return {'performance': cpu, 'paired_worlds': paired, 'normalization': {'valid': not errors, 'errors': [error for error in errors], 'sources': sources, 'capture_git_sha': raw.get('git_sha'), 'capture_run_id': raw.get('run_id')}}


def write_evidence(output: Path, evidence: Document) -> None:
    """Only update production8 docs; preserve blind assessment and other fields."""
    allowed = Path(__file__).resolve().parents[2] / 'docs/art/production8'
    if not output.resolve().is_relative_to(allowed.resolve()):
        raise EvidenceError('output must remain under docs/art/production8')
    audit = mapping(evidence.get('normalization', {}))
    for item in rows(audit.get('sources', [])):
        if output.resolve() == Path(str(item['path'])).resolve():
            raise EvidenceError('output cannot overwrite an evidence source')
    prior = output if output.exists() or output.suffix != '.gz' else output.with_suffix('')
    existing = load(prior) if prior.exists() else {}
    existing.update(evidence)
    output.parent.mkdir(parents=True, exist_ok=True)
    encoded = (json.dumps(existing, indent=2, ensure_ascii=False) + '\n').encode('utf-8')
    _ = output.write_bytes(gzip.compress(encoded, mtime=0) if output.suffix == '.gz' else encoded)


def main() -> int:
    """CLI: MANIFEST A30 CALENDAR OUTPUT [P7_PACING ...]."""
    if len(sys.argv) < 5:
        print((__doc__ or '') + '\nUsage: normalize_evidence_p8.py MANIFEST A30 CALENDAR OUTPUT [P7_PACING ...]', file=sys.stderr)
        return 2
    manifest, performance, calendar, output, *old = map(Path, sys.argv[1:])
    evidence = normalize(manifest, performance, calendar, old)
    write_evidence(output, evidence)
    return 0 if mapping(evidence['normalization'])['valid'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
