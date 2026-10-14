# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by measure_p8.py
"""Normalize captured full feed audit while deriving event expectations separately."""
from __future__ import annotations

import json
from pathlib import Path

from ingame_qa_news_p8 import FeedEvent, Movement, NewsFrame, Summary
from event_oracle_p8 import event_errors


def feed_frames(paths: list[Path], departure_stem: str) -> list[NewsFrame]:
    """Use append-only audit cursors so same-tick casts remain new events."""
    frames: list[NewsFrame] = []
    previous_ids: set[str] = set()
    previous_tick = -1
    previous_seed = -1
    for path in paths:
        data = json.loads(path.with_suffix('.entities.json').read_text())
        audit = data['feed_audit']
        tick, seed = data['tick'], data['seed']
        if tick < previous_tick or seed != previous_seed:
            previous_ids = set()
            previous_tick = -1
        events: list[FeedEvent] = [{'event_id': row['event_id'], 'tick': row['tick'], 'key': row['key'],
                   'event_class': row['class'], 'visible': row['visible']} for row in audit['log']]
        current_ids = {row['event_id'] for row in events}
        protected = [row['event_id'] for row in events if row['event_class'] == 'protected' and row['event_id'] not in previous_ids]
        expected = [row['death_id'] for row in data['death_truth_trace']['deaths']]
        society = data['society_snapshot']
        for index, fid in enumerate(society['fission_ids']):
            for tick_key, identity in (('fission_depart_ticks', f'depart:{fid}'),
                                      ('fission_cancelled_ticks', f'cancel:{fid}'),
                                      ('fission_founded_ticks', f"founded:{society['fission_daughters'][index]}")):
                if 0 <= society[tick_key][index] <= tick:
                    expected.append(identity)
        expected.extend(f'cast:{identity}' for identity in data.get('intervention_state', {}).get('log_ids', []))
        food = audit['food']
        audit_errors = event_errors(path)
        for row in food['movements']:
            if row['raw_delta'] - row['excluded_delta'] != row['delta']:
                audit_errors.append('food correction arithmetic differs from raw delta')
        for row in food['summaries']:
            matching = [event for event in audit['log'] if event['key'] == 'HUD_FEED_DAY_SUMMARY_FMT'
                        and event.get('settlement_id') == row['settlement_id'] and event.get('period') == row['period']]
            if len(matching) != int(row['qualifies']):
                audit_errors.append('daily summary notification missing/duplicated or below threshold')
        movements: list[Movement] = [{'tick': row['tick'], 'village_id': row['settlement_id'], 'delta': row['delta'], 'excluded': False}
                     for row in food['movements']]
        summaries: list[Summary] = [{'village_id': row['settlement_id'], 'start_tick': row['period_start_tick'],
                      'end_tick': row['period_end_tick'], 'stored': row['stored'], 'eaten': row['eaten'], 'net': row['net']}
                     for row in food['summaries'] if row['qualifies']]
        frame: NewsFrame = {'frame': path.stem, 'previous_tick': previous_tick, 'tick': tick,
            'expected_event_ids': expected, 'events': events,
            'visible_event_ids': [row['event_id'] for row in audit['visible']],
            'overflow_event_ids': audit['overflow_ids'], 'more_count': audit['overflow_count'],
            'summaries': summaries, 'movements': movements, 'departure_feed_frame': path.stem == departure_stem,
            'departure_event_id': f"depart:{society['fission_ids'][0]}" if society['fission_ids'] else '',
            'protected_since_previous': protected, 'oracle_errors': audit_errors,
            'coverage_complete': bool(food['coverage_complete']) and all(row['coverage_complete'] and row['boundary_exact'] for row in food['summaries'])}
        frames.append(frame)
        previous_ids, previous_tick, previous_seed = current_ids, tick, seed
    return frames
