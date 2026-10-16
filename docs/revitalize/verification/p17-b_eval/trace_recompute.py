# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 trace_recompute_corrected.py TRACE_DIR SEED,SEED
import json
import sys
from collections import defaultdict
from pathlib import Path

BASE = Path(sys.argv[1])
SEEDS = [int(value) for value in sys.argv[2].split(',')]
KINDS = ['TendFire', 'Craft', 'Repair', 'HearthCircle', 'Explore', 'Play', 'SitByFire']

def blank():
    return dict(activities_started=[0]*7, activity_minutes=[0]*7, completed_activities=0,
                completed_minutes=0, no_place_redraws=0, no_place_failures=0)

for seed in SEEDS:
    def load(suffix):
        return [json.loads(line) for line in (BASE/f'game_v22_s{seed}_{suffix}.jsonl').read_text().splitlines()]
    rows, people, adults, hours = (load(s) for s in ('routine', 'people', 'agents', 'hours'))
    offset = (hours[0]['minute'] - hours[0]['tick']) % 1440
    all_counts, adult_counts, intervals, ends = defaultdict(blank), defaultdict(blank), defaultdict(list), {}
    for row in rows:
        agent = row['id']
        match row['event']:
            case 'started' | 'no_place':
                targets = [all_counts[agent]] + ([adult_counts[agent]] if row['life_stage'] == 'adult' else [])
                for counts in targets:
                    if row['event'] == 'started':
                        counts['activities_started'][KINDS.index(row['kind'])] += 1
                    counts['no_place_redraws'] += int(row['redrawn'])
                    counts['no_place_failures'] += int(row['event'] == 'no_place')
            case 'consuming_interval':
                duration = row['end_tick'] - row['start_tick']
                assert duration == row['minutes'], ('interval', seed, row)
                intervals[(agent, row['chosen_tick'])].append(row)
                all_counts[agent]['activity_minutes'][KINDS.index(row['kind'])] += duration
                if row['life_stage'] == 'adult':
                    adult_counts[agent]['activity_minutes'][KINDS.index(row['kind'])] += duration
            case 'ended':
                key = (agent, row['chosen_tick'])
                assert key not in ends, ('duplicate_end', seed, row)
                ends[key] = row
            case 'arrived' | 'end_observation':
                pass
            case unknown:
                raise AssertionError(('unknown_event', unknown))
    for key, row in ends.items():
        spans = intervals[key]
        duration = sum(r['end_tick'] - r['start_tick'] for r in spans)
        adult_duration = sum(r['end_tick'] - r['start_tick'] for r in spans if r['life_stage'] == 'adult')
        assert (duration, adult_duration) == (row['consuming_minutes'], row['adult_consuming_minutes']), ('episode', seed, key)
        if row['completed']:
            assert row['arrived_tick'] is not None, ('arrival', seed, row)
            assert row['tick'] >= row['until_tick'] or (row['tick'] + offset) % 1440 >= 1080, ('clock', seed, row, offset)
            all_counts[row['id']]['completed_activities'] += 1
            all_counts[row['id']]['completed_minutes'] += duration
            if adult_duration:
                adult_counts[row['id']]['completed_activities'] += 1
                adult_counts[row['id']]['completed_minutes'] += adult_duration
    for label, records, counts in [('people', people, all_counts), ('adults', adults, adult_counts)]:
        for row in records:
            assert row['routine'] == counts[row['id']], (label, seed, row['id'], row['routine'], counts[row['id']])
    meta = json.loads((BASE/f'game_v22_s{seed}_trace_meta.json').read_text())
    assert meta['pass17']['observation_misses'] == 0
    false360 = sum(r['completed'] and not (r['tick'] >= r['until_tick'] or (r['tick'] + 360) % 1440 >= 1080) for r in ends.values())
    print(f'PASS seed={seed} people={len(people)} adults={len(adults)} offset={offset} completed={sum(r["completed"] for r in ends.values())} invalid_360_rejections={false360} discrepancies=0 misses=0')
