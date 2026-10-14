# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by measure_p8.py
"""Audit feed completeness and food summaries without trusting visibility claims."""
from __future__ import annotations

from collections import Counter
import json
from typing import NotRequired, TypedDict

from ingame_qa_pixels import Check, result


class FeedEvent(TypedDict):
    event_id: str
    tick: int
    key: str
    event_class: str
    visible: bool


class Movement(TypedDict):
    tick: int
    village_id: int
    delta: int
    excluded: bool


class Summary(TypedDict):
    village_id: int
    start_tick: int
    end_tick: int
    stored: int
    eaten: int
    net: int


class NewsFrame(TypedDict):
    frame: str
    previous_tick: int
    tick: int
    expected_event_ids: list[str]
    events: list[FeedEvent]
    visible_event_ids: list[str]
    overflow_event_ids: list[str]
    more_count: int
    summaries: list[Summary]
    movements: list[Movement]
    departure_feed_frame: bool
    departure_event_id: str
    protected_since_previous: NotRequired[list[str]]
    coverage_complete: NotRequired[bool]
    oracle_errors: NotRequired[list[str]]


def news(frames: list[NewsFrame]) -> Check:
    """Missing events, invisible protected events and incorrect sums each fail."""
    errors: list[str] = []
    summary_count = 0
    departure_frames = 0
    for frame in frames:
        prefix = frame['frame'] + ': '
        errors.extend(prefix + value for value in frame.get('oracle_errors', []))
        if frame.get('coverage_complete') is False:
            errors.append(prefix + 'incomplete movement/summary sampling coverage')
        counts = Counter(event['event_id'] for event in frame['events'])
        for identity in frame['expected_event_ids']:
            if counts[identity] != 1:
                errors.append(prefix + 'expected event missing/duplicated: ' + identity)
        if any(count > 1 for count in counts.values()):
            errors.append(prefix + 'duplicate event identities')
        visible = set(frame['visible_event_ids'])
        overflow = set(frame['overflow_event_ids'])
        if not (visible | overflow).issubset(counts):
            errors.append(prefix + 'visible/overflow identity absent from audit log')
        if len(visible) > 4 or visible.intersection(overflow) or frame['more_count'] != len(overflow):
            errors.append(prefix + 'visible cap / overflow identity mismatch')
        protected = set(frame.get('protected_since_previous', [event['event_id'] for event in frame['events']
                        if event['event_class'] == 'protected' and frame['previous_tick'] < event['tick'] <= frame['tick']]))
        for event in frame['events']:
            identity = event['event_id']
            if event['event_class'] == 'routine' and (event['visible'] or identity in visible):
                errors.append(prefix + 'visible routine event: ' + identity)
            if identity in protected:
                if identity not in visible | overflow:
                    errors.append(prefix + 'new protected event absent from feed/+N: ' + identity)
        for summary in frame['summaries']:
            deltas = [movement['delta'] for movement in frame['movements']
                      if movement['village_id'] == summary['village_id'] and not movement['excluded']
                      and summary['start_tick'] < movement['tick'] <= summary['end_tick']]
            stored = sum(max(0, delta) for delta in deltas)
            eaten = sum(max(0, -delta) for delta in deltas)
            if (stored, eaten, stored - eaten) != (summary['stored'], summary['eaten'], summary['net']):
                errors.append(prefix + 'daily summary differs from movement audit')
            summary_count += 1
        if frame['departure_feed_frame']:
            departure_frames += 1
            if frame['departure_event_id'] not in visible:
                errors.append(prefix + 'departure +3h line not visible')
    if not departure_frames:
        errors.append('missing departure +3h feed frame')
    if not summary_count:
        errors.append('missing daily summary evidence')
    return result('A31', bool(frames) and not errors, json.dumps({
        'frames': len(frames), 'summaries': summary_count, 'departure_feed_frames': departure_frames,
        'errors': errors, 'event_oracle': 'expected_event_ids must derive from raw snapshot histories, not emitted feed'}))
