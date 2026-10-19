#!/usr/bin/env python3
"""Offline report: AFTER_FOLDER OFFICIAL_MATRIX_SUMMARY OUTPUT_FOLDER.
No validators, calibration probes, measurements or acceptance-rule changes.
"""
from __future__ import annotations
import hashlib
import json
import math
import runpy
import sys
from pathlib import Path
from typing import Any

ROOT = Path('/Users/rexxa/work/new-world-art-prod')
HELPER = Path('/Users/rexxa/work/art-production13b-evidence/after-aggregation-helper/aggregate_after.py')
H = runpy.run_path(str(HELPER))

def read(path: Path) -> Any:
    return json.loads(path.read_text())

def source(path: Path) -> dict[str, str]:
    return {'path': str(path.resolve()), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}

def admitted(path: Path) -> bool:
    side = Path(str(path) + '.validity.json')
    return side.is_file() and read(side).get('valid') is True and read(side).get('verdict') == 'VALID'

def fmt(value: Any) -> str:
    if value is None: return 'PENDING'
    if isinstance(value, float): return f'{value:.6f}'
    return str(value)

def table(headers: list[str], rows: list[list[Any]]) -> list[str]:
    return ['| ' + ' | '.join(headers) + ' |', '| ' + ' | '.join('---' for _ in headers) + ' |'] + ['| ' + ' | '.join(fmt(v) for v in row) + ' |' for row in rows]

def main() -> None:
    folder, official_path, out = map(Path, sys.argv[1:4])
    before_path = ROOT / 'docs/art/production13b/perf/before-reference.json'
    calibration_path = ROOT / 'docs/art/production13b/perf/calibration.json'
    before, calibration, official = read(before_path), read(calibration_path), read(official_path)
    sources = [source(p) for p in (before_path, calibration_path, official_path, HELPER)]
    matrix: dict[str, Any] = {}
    for raw_path, summary in zip(official['sources'], official['sessions'], strict=True):
        p = Path(raw_path)
        if not admitted(p) or summary.get('valid') is not True or summary.get('errors') != []:
            raise ValueError(f'Official summary contains non-admitted source: {p}')
        raw = read(p)
        if raw['world'] != summary['world'] or raw['world'] in matrix:
            raise ValueError('World/source ambiguity')
        detail = H['matrix'](raw)
        if detail['coverage'] != 'COMPLETE_RAW_SHAPE': raise ValueError('Incomplete admitted matrix')
        for group, stats in detail['eligible_pooled'].items():
            if stats['frames'] == 0: continue
            for metric, dist in stats['metrics'].items():
                for stat in ('median', 'p90', 'p99', 'max'):
                    if not math.isclose(dist[stat], summary['pooled'][group]['all_frames'][metric][stat], abs_tol=1e-12, rel_tol=0):
                        raise ValueError('Raw/official statistic mismatch')
        detail['source'] = source(p)
        matrix[raw['world']] = detail
        sources.append(source(p))
    criteria = []
    for old in before['H1_H11_before']:
        row = dict(old)
        row.update(after_value=None, after_before_ratio=None, after_status='PENDING')
        if old.get('world') in matrix:
            stats = matrix[old['world']]['eligible_pooled'].get(old['condition'])
            if stats and stats['frames']:
                value = stats['metrics']['primary_cpu_ms'][old['metric']]
                row.update(after_value=value, after_before_ratio=value / old['before_value_ms'], after_status='PASS' if value <= old['bar_ms'] else 'FAIL')
        criteria.append(row)
    sections: dict[str, Any] = {}
    for p in sorted(folder.glob('sections-*/frames.json')):
        verdict = p.parent / 'summary.json'
        if not admitted(p) or not verdict.exists() or read(verdict).get('valid') is not True: continue
        raw = read(p)
        if raw['world'] in sections: raise ValueError('Multiple admitted section attempts: require explicit selection')
        detail = H['sections'](raw)
        for name, stat in detail['exclusive_ms'].items():
            for key in ('median', 'p90', 'p99', 'max'):
                assert math.isclose(stat[key] * 1000, read(verdict)['section_statistics_usec'][name][key], abs_tol=1e-9)
        detail['source'] = source(p)
        sections[raw['world']] = detail
        sources += [source(p), source(verdict)]
    ablations = []
    for p in sorted(folder.rglob('*.json')):
        if p.name.endswith('.validity.json'): continue
        obj = read(p)
        if not isinstance(obj, dict) or not {'ranked', 'floor', 'source'} <= obj.keys(): continue
        raw_path = Path(obj['source'])
        if not admitted(raw_path): continue
        ablations.append({'summary': obj, 'source': source(p)})
        sources.append(source(p))
    probes = []
    for p in sorted(folder.rglob('*.validity.json')):
        d = read(p)
        raw_path = Path(str(p).removesuffix('.validity.json'))
        probes.append({'source': source(p), 'raw_exists': raw_path.is_file(), 'valid': d.get('valid'), 'verdict': d.get('verdict'), 'exit_code': d.get('exit_code'), 'reference_seconds': d.get('reference_seconds'), 'reference_sha256': d.get('reference_sha256'), **{k: {a: b for a, b in d.get(k, {}).items() if a != 'ps'} for k in ('start', 'end')}})
    report = {'status': 'DRAFT_PART4', 'sources': sources, 'historical_R': before['calibration']['reference_seconds'], 'current_R': calibration['reference_seconds'], 'calibration_runs': [{k: v for k, v in run.items() if k != 'ps'} for run in calibration['runs']], 'H1_H11': criteria, 'matrix': matrix, 'sections': sections, 'ablation': ablations, 'measurement_probes': probes, 'pending_matrix': [w for w in H['WORLDS'] if w not in matrix], 'pending_sections': [w for w in ('W200', 'W-multi', 'W500') if w not in sections], 'ablation_status': 'AVAILABLE' if ablations else 'PENDING'}
    lines = ['# Part4 비교 초안 — 승인된 측정만 반영', '', '전체 관문: 진행 중. PENDING은 미측정·미승인 상태이며 통과가 아니다. 비승인 raw 값은 after 표에 사용하지 않는다.', '', f"역사 R={report['historical_R']}; 현재 R={report['current_R']}. 시간 보정 없음. 아래 after/before는 세션 간 기술적 비교이며 후보의 인과 효과 판정은 동일 세션 paired 결과를 별도로 사용한다.", '', '출처 동등성: 전체 manifest 동일 주장은 하지 않는다. identity-source-equivalence.md에 따라 추적 runtime source, compiled imported-resource bytes, loaded bridge 동등성과 measurement-local snapshot 안정성만 구분한다.', '', '## H1–H11', '']
    lines += table(['항목', '조건/지표', 'before ms', 'after ms', 'after/before', '기존 기준 ms', '상태'], [[r['criterion'], str(r.get('world', '비수치')) + ' ' + str(r.get('metric', '')), r.get('before_value_ms', '별도 역사 기록'), r['after_value'], r['after_before_ratio'], r.get('bar_ms', '별도 증거'), r['after_status']] for r in criteria])
    lines += ['', 'H6–H11은 성능 숫자로 대체할 수 없는 보고/identity/가독성/입력 등 증거를 포함한다. 이 초안은 별도 증거를 추정하지 않는다.', '', '## Matrix', '']
    for world in H['WORLDS']:
        lines += ['', f'### {world}', '']
        if world not in matrix: lines.append('PENDING'); continue
        for condition, stats in matrix[world]['eligible_pooled'].items():
            lines += ['', condition, '']
            lines += table(['metric', 'median', 'p90', 'p99', 'max', 'mean'], [[m] + [d[k] for k in ('median', 'p90', 'p99', 'max', 'mean')] for m, d in stats['metrics'].items()])
            lines += ['', f"frames={stats['frames']}; ticks={stats['ticks_total']}; tick-carrying frames={stats['tick_carrying_frames']}"]
        lines += ['', *table(['block', 'frames', 'ticks', 'tick frames', 'pop start/end', 'tick start/end', 'excluded'], [[b['label'], b['statistics']['frames'], b['statistics']['ticks_total'], b['statistics']['tick_carrying_frames'], f"{b['population_start']} → {b['population_end']}", f"{b['start_state']['tick']} → {b['end_state']['tick']}", b['population_flagged']] for b in matrix[world]['blocks']])]
    lines += ['', '## Exclusive sections / getter means', '']
    for world in ('W200', 'W-multi', 'W500'):
        lines += ['', f'### {world}', '']
        if world not in sections: lines.append('PENDING'); continue
        s = sections[world]
        lines += table(['section ms', 'median', 'p90', 'p99', 'max'], [[name] + [d[k] for k in ('median', 'p90', 'p99', 'max')] for name, d in s['exclusive_ms'].items()])
        lines += ['', *table(['getter', 'mean calls/frame', 'mean ms/frame'], [[name, d['calls_per_frame']['mean'], d['ms_per_frame']['mean']] for name, d in s['getters'].items()])]
    lines += ['', '## W200 node ablation', '']
    if not ablations: lines.append('PENDING')
    for a in ablations:
        lines += table(['node', 'cost ms', 'off median', 'neighbor median', 'role'], [[r['node'], r['cost_ms'], r['off_statistics']['median'], r['neighbour_statistics']['median'], r['role']] for r in a['summary']['ranked']])
    lines += ['', '## Calibration — 15 runs', '', *table(['run', 'epoch', 'seconds', 'load1m', 'checksum'], [[i+1, r.get('epoch'), r.get('wall_seconds'), r.get('load_1m'), r.get('checksum')] for i, r in enumerate(report['calibration_runs'])])]
    lines += ['', '## 모든 measurement endpoint — 거부 포함', '']
    lines += table(['attempt', 'endpoint', 'epoch', 'seconds', 'load1m', 'verdict', 'exit', 'raw'], [[str(Path(p['source']['path']).relative_to(folder)), end, p[end].get('epoch'), p[end].get('wall_seconds'), p[end].get('load_1m'), p['verdict'], p['exit_code'], p['raw_exists']] for p in probes for end in ('start', 'end')])
    out.mkdir(parents=True, exist_ok=True)
    (out/'comparison.json').write_text(json.dumps(report, indent=2, allow_nan=False)+'\n')
    (out/'comparison.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps({'matrix': list(matrix), 'sections': list(sections), 'ablation': len(ablations), 'probes': len(probes), 'calibration_runs': len(calibration['runs'])}))

if __name__ == '__main__': main()
