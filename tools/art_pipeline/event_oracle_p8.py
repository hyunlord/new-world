# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by feed_adapter_p8.py
"""Compare independently observed source events with the emitted feed census."""
from __future__ import annotations

import json
from pathlib import Path


def event_errors(path: Path) -> list[str]:
    """Avoid deriving expected identities from the notification stream itself."""
    data = json.loads(path.with_suffix('.entities.json').read_text())
    trace = data.get('event_truth_trace', {})
    errors: list[str] = []
    if not trace.get('coverage_complete') or trace.get('window_end_tick') != data['tick'] or trace.get('window_start_tick') != 0 or trace.get('max_sample_interval_ticks', 2) > 1:
        errors.append('independent event census incomplete or not current')
    events = trace.get('events', [])
    log = data['feed_audit']['log']
    matched_ids: set[str] = set()
    for event in events:
        identity = event['event_id']
        candidates = [row for row in log if row['event_id'] == identity]
        if not candidates and event['key'] in ('HUD_FEED_SITE_STARTED', 'HUD_FEED_SITE_COMPLETED', 'HUD_FEED_AGENT_BORN'):
            candidates = [row for row in log if row['key'] == event['key']
                          and row['observed_tick'] == event['observed_tick'] and row['args'] == event['args_raw']]
        if len(candidates) != 1:
            errors.append(identity + ': independent event missing/duplicated')
            continue
        actual = candidates[0]
        if actual['key'] != event['key'] or actual['tick'] != event['tick'] or actual['class'] != event['class']:
            errors.append(identity + ': key/time/class mismatch')
        matched_ids.add(actual['event_id'])
    census_keys = {'HUD_FEED_SITE_STARTED', 'HUD_FEED_SITE_COMPLETED', 'HUD_FEED_AGENT_BORN',
                   'HUD_FEED_SETTLEMENT_FORMED', 'HUD_FEED_SETTLEMENT_ABANDONED', 'HUD_FEED_FISSION_DEPARTED',
                   'HUD_FEED_FISSION_CANCELLED', 'HUD_FEED_SETTLEMENT_FOUNDED', 'HUD_FEED_GOD_CAST_FMT'}
    raw = data.get('society_snapshot', {})
    initial_roots = {f'founded:{sid}' for sid, parent, tick in zip(raw.get('settlement_ids', []), raw.get('parent_ids', []), raw.get('founded_ticks', []), strict=True)
                     if parent < 0 and tick == 0}
    for row in log:
        initial_fact = (row['key'] == 'HUD_FEED_SETTLEMENT_FORMED' and row['event_id'] in initial_roots
                        and row['tick'] == 0 and 0 <= row['observed_tick'] <= 1 and row['class'] == 'protected')
        if initial_fact:
            continue
        if row['key'] in census_keys and row['event_id'] not in matched_ids:
            errors.append(row['event_id'] + ': feed event absent from independent census')
    return errors
