"""Cross-check fresh Godot getter observations against the completed v19 probe."""
from __future__ import annotations

import gzip
import hashlib
import json
import sys
from pathlib import Path

from measure_days_p8 import calendar, samples
from probe_models_p8 import Fields


def main(source: Path) -> None:
    output = Path('docs/art/production8')
    measured_path = output / 'measurements/probe_calendar_p8.json'
    measured = Fields.parse(measured_path.read_text())
    society = measured.optional_integer('society_seed')
    seeds = [20261010] + ([society] if society is not None else [])
    results = []
    additions = ['\n## 완료된 headless getter 교차검증\n']
    for seed in seeds:
        path = output / f'measurements/headless_{seed}.json'
        raw = path.read_bytes()
        observed = Fields.parse(raw.decode())
        assert observed.integer('seed') == seed and observed.string('scene') == 'game_v19'
        rows = observed.records('rows')
        indexed = {row.integer('tick'): row for row in rows}
        probe_rows = samples(source / f'game_v19_s{seed}.jsonl')
        probe = {row.tick: row for row in probe_rows}
        clock = calendar(probe_rows)
        mismatches = []
        overlap = 0
        for row in rows:
            tick = row.integer('tick')
            if tick not in probe:
                continue
            overlap += 1
            cold = len(row.integers('cold_ids'))
            alive = sum(n > 0 for n in row.nested('settlements').integers('member_counts'))
            if cold != probe[tick].cold_members or alive != probe[tick].settlements_alive:
                mismatches.append({'tick': tick, 'cold': cold, 'probe_cold': probe[tick].cold_members,
                                   'alive': alive, 'probe_alive': probe[tick].settlements_alive})
        warm = []
        for row in rows:
            tick = row.integer('tick')
            previous = indexed.get(tick - 720)
            night = indexed.get(tick - 315)
            if row.integer('minute') == 435 and row.integer('display_day') <= 60 and previous and night:
                equal = row.nested('tallies').integer('cold_member_hours') == previous.nested('tallies').integer('cold_member_hours')
                fire = Fields.parse('{"values":' + night.nested('settlements').string('fire_lit') + '}')
                lit = any(n > 0 for n in fire.integers('values'))
                if equal and lit and not row.integers('cold_ids'):
                    warm.append(tick)
        foundings = [{'tick': row.integer('tick'), 'settlements_alive': row.integer('settlements_alive')}
                     for row in observed.records('founding_checks')]
        first_three = observed.integer('first_three_alive_tick')
        result = {'seed': seed, 'headless_rows': len(rows), 'probe_overlap_rows': overlap,
                  'mismatches': mismatches, 'warm_control_ticks': warm,
                  'first_three_alive_tick': first_three, 'founding_checks': foundings,
                  'source_sha256': hashlib.sha256(raw).hexdigest(), 'source': str(path)}
        results.append(result)
        additions += [f'### seed {seed}\n',
                      f'getter {len(rows)}행, probe와 동일 tick {overlap}행, cold·생존마을 불일치 {len(mismatches)}건.\n',
                      f'정확한 창립 직후 확인: `{foundings}`; 최초 ≥3마을 tick `{first_three}`.\n',
                      f'누적 cold 12시간 증가0 + 02:00 점화 + 아침 cold0 대조: `{[clock.label(t) for t in warm]}`.\n',
                      '| 가을 화면일 15:00 | 실제 목재 운반 ID |\n|---|---|']
        for row in rows:
            day = row.integer('display_day')
            if row.integer('minute') == 900 and ((day - 1) % 28) // 7 == 2 and day <= 84:
                additions.append(f"| {day} | {row.integers('wood_carrier_ids')} |")
        external = source.parent / f'headless_{seed}.json'
        external.write_bytes(raw)
        path.with_suffix('.json.gz').write_bytes(gzip.compress(raw, mtime=0))
        path.unlink()
    report = {'probe_source': str(source), 'runs': results,
              'status': 'PASS' if all(not r['mismatches'] for r in results) else 'FAIL'}
    (output / 'measurements/headless_crosscheck_p8.json').write_text(json.dumps(report, indent=2) + '\n')
    with (output / 'MEASURED_DAYS.md').open('a') as stream:
        stream.write('\n'.join(additions) + '\n\nheadless 원자료는 measurements/headless_<seed>.json.gz에 무손실 압축했고 외부 evidence 원본도 보존했다.\n')
    print(json.dumps(report, indent=2))
    assert report['status'] == 'PASS', 'Headless/probe mismatch; retained evidence'


if __name__ == '__main__':
    main(Path(sys.argv[1]))
