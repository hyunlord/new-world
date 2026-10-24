#!/usr/bin/env python3
"""Score the declared M-UX A8 bars without changing thresholds or missing-data semantics."""
from __future__ import annotations
import argparse
import json
from pathlib import Path


def score(report: dict, first: int, count: int) -> dict:
    if report.get('scene') != 'colony' or report.get('days') != 48:
        raise ValueError('Expected colony, exactly 48 days')
    rows = report['seeds']
    if len(rows) != count or sorted(row['seed'] for row in rows) != list(range(first, first + count)):
        raise ValueError('Wrong, missing, or duplicate seeds')
    results = []
    for row in rows:
        m = row['mux']
        day = m['ticks_per_day']
        if not isinstance(day, int) or day <= 0:
            raise ValueError('Invalid ticks_per_day')
        flags = {
            'stockpile_day1': m['stockpile_first_tick'] is not None and m['stockpile_first_tick'] <= day,
            'walled_beds_day10': m['walled_beds_for_all_first_tick'] is not None and m['walled_beds_for_all_first_tick'] <= 10 * day,
            'zero_cold_deaths': m['first_winter_observed'] and m['first_winter_complete'] and m['first_winter_cold_deaths'] == 0,
            'firewood_during_snap': m['cold_snap_observed_ticks'] > 0 and m['cold_snap_firewood_zero_ticks'] == 0,
            'alive_day48': m['alive_at_day48'] is True and m['alive_end'] > 0,
            'exact_ledger': row['ledger_violation_ticks'] == 0 and row['reservation_violation_ticks'] == 0,
        }
        results.append({'seed': row['seed'], 'bars': flags, 'measurements': m})
    thresholds = {'stockpile_day1': count, 'walled_beds_day10': count,
                  'zero_cold_deaths': (count * 28 + 31) // 32,
                  'firewood_during_snap': (count * 24 + 31) // 32,
                  'alive_day48': count, 'exact_ledger': count}
    bars = {key: {'passed': sum(bool(row['bars'][key]) for row in results), 'required': needed,
                  'total': count} for key, needed in thresholds.items()}
    return {'bars': bars, 'passed': all(bar['passed'] >= bar['required'] for bar in bars.values()), 'seeds': results}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('summary', type=Path)
    parser.add_argument('--calibration', action='store_true')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = score(json.loads(args.summary.read_text()), 0 if args.calibration else 1000, 8 if args.calibration else 32)
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    for name, bar in result['bars'].items():
        print(f"{name}: {bar['passed']}/{bar['total']} required={bar['required']}")
    print('MUX_EVAL ' + ('PASS' if result['passed'] else 'FAIL'))
    return 0 if result['passed'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
