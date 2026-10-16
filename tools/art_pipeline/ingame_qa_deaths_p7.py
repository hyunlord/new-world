"""Compare independently sampled death events, full feed history and drawn colors."""
from collections import Counter
import re
from typing import Final, TypedDict

DEATH_KEYS: Final = ('deaths_starvation','deaths_dehydration','deaths_combat','deaths_old_age','deaths_exposure')
FEED_KEYS: Final = ('HUD_FEED_DEATH_STARVATION','HUD_FEED_DEATH_DEHYDRATION','HUD_FEED_DEATH_COMBAT','HUD_FEED_DEATH_OLD_AGE','HUD_FEED_DEATH_EXPOSURE')
COLORS: Final = ((.45,.30,.15),(.20,.45,.95),(.95,.15,.10),(.70,.70,.70),(.70,.84,.97))

class DeathRow(TypedDict):
    tick: int
    x: int
    y: int
    reason: int


def death_identity(row: DeathRow) -> tuple[int,int,int,int]:
    """Preserve duplicates by using this key in a multiset, never a set."""
    return row['tick'],row['x'],row['y'],row['reason']


def check_death_frame(data) -> list[str]:
    """Return concrete inconsistencies; input is an original frame sidecar."""
    errors: list[str] = []
    tallies = data.get('tallies', {})
    if not all(key in tallies for key in DEATH_KEYS):
        errors.append('missing one or more of five death tallies')
    total = sum(tallies.get(key,0) for key in DEATH_KEYS)
    if data.get('hud_death_total') != total:
        errors.append(f'HUD sum {data.get("hud_death_total")} != {total}')
    labels = [row.get('text','') for row in data.get('labels',[]) if 'HudTopbar' in row.get('name','')]
    if not any(re.fullmatch(rf'(?:사망|Deaths?)\s+{total}', text) for text in labels):
        errors.append('actual HUD death label missing/mismatch')
    trace = data.get('death_truth_trace',{})
    if (trace.get('source') != 'independent FFI recent_deaths census' or
            not 0 < trace.get('max_sample_interval_ticks',0) < 120 or
            trace.get('window_end_tick') != data.get('tick') or trace.get('window_start_tick') != 0):
        errors.append('missing independent within-retention complete death census')
    truth = trace.get('deaths',[])
    feed = data.get('feed_log',[])
    reason_counts = Counter(row.get('reason') for row in truth)
    for reason,key in enumerate(DEATH_KEYS):
        if reason_counts[reason] != tallies.get(key,0):
            errors.append(f'census {reason_counts[reason]} != tally {key}={tallies.get(key)}')
    for row in truth:
        ident = row.get('death_id')
        matched = [event for event in feed if event.get('death_id') == ident]
        reason = row.get('reason',-1)
        if len(matched) != 1 or not 0 <= reason < 5 or matched[0].get('key') != FEED_KEYS[reason] or matched[0].get('tick') != row.get('tick'):
            errors.append(f'death feed not exactly once/right reason: {ident}')
    ids = {row.get('death_id') for row in truth}
    if any(event.get('death_id') not in ids for event in feed if event.get('key') in FEED_KEYS):
        errors.append('death feed has no matching independent event')
    drawn = data.get('death_drawn',[])
    expected = Counter(death_identity(row) for row in truth if 0 <= data.get('tick',0)-row['tick'] < 90)
    actual = Counter(death_identity(row) for row in drawn)
    if actual != expected:
        errors.append(f'death drawn multiset mismatch: expected={dict(expected)} actual={dict(actual)}')
    for row in drawn:
        reason = row.get('reason',-1)
        color = row.get('color',[])
        expected_color = COLORS[reason] if 0 <= reason < 5 else (.55,.55,.55)
        if len(color) != 3 or any(abs(a-b) > .001 for a,b in zip(color,expected_color)):
            errors.append(f'death color reason={reason} actual={color} expected={expected_color}')
    return errors
