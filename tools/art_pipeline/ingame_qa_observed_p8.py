# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: imported by measure_p7.py; observations frozen by independent screen reviewers.
"""Cross-check independent screenshot observations against raw bridge snapshots.

Renderer metadata is diagnostic, never sufficient to PASS visual gates. Review rows
are bound to exact lossless screenshot SHA256. Missing independent evidence fails.
"""
from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

from ingame_qa_pixels import Check, result
from ingame_qa_types import Capture, load_capture
from observation_rules_p8 import chill_window, intervention_visible


def observed_checks(captures: list[Capture], god_raw: Path, gate: Path, *, morning_stems: set[str], warm_control_stem: str | None) -> list[Check]:
    """Evaluate A21-A25, retaining every error and natural/intervention separation."""
    review_path = Path('docs/art/production8/visual_observations.json')
    reviews = json.loads(review_path.read_text()).get('frames', {}) if review_path.exists() else {}
    errors = {code: [] for code in ('A21', 'A22', 'A23', 'A24', 'A25')}
    counts = {code: 0 for code in errors}
    gate_path = gate / 'results.tsv'
    gate_codes = {parts[0]: int(parts[1]) for line in gate_path.read_text().splitlines() if len(parts := line.split('\t')) >= 2} if gate_path.exists() else {}
    for name in ('test_god_hand', 'test_interventions_real_p6'):
        if gate_codes.get(name) != 0:
            errors['A24'].append(name + ': missing or failed Godot test')
    god_manifest = god_raw / 'manifest.json'
    god_frames = [load_capture(god_raw / row['file']) for row in json.loads(god_manifest.read_text())['captures']
                  if json.loads((god_raw / row['file']).with_suffix('.entities.json').read_text()).get('seed') == 20261010] if god_manifest.exists() else []
    if not god_frames:
        errors['A24'].append('Missing ART_GOD capture set')
    for capture in captures + god_frames:
        data = json.loads(capture.path.with_suffix('.entities.json').read_text())
        review = reviews.get(capture.stem, {})
        bound = review.get('sha256') == hashlib.sha256(capture.path.read_bytes()).hexdigest()
        natural = data.get('scenario') == 'NATURAL SCENARIO' and data.get('seed') == 20261010
        snapshot = data.get('agent_snapshot', {})
        time = data.get('time_snapshot', {})
        settlement = data.get('settlement_snapshot', {})
        entities = data.get('entities', [])
        _ = int(time.get('day_index', -2)) + 1
        hour = float(time.get('time_of_day', -1))
        severity = int(time.get('winter_severity', -1))
        winter_night = severity >= 0 and (hour >= 18 or 0 <= hour < 7)
        flags = dict(zip(snapshot.get('agent_ids', []), snapshot.get('colds', [])))
        cold_count = sum(value == 1 for value in flags.values())
        if natural:
            counts['A23'] += 1
            shown = review.get('hud_text', '')
            if not bound or not re.search(r'(?:추위|Cold)\s*' + str(cold_count) + r'(?!\d)', shown):
                errors['A23'].append(capture.stem + ': cold HUD count absent/mismatch or unbound screenshot review')
            winter_words = ('온화한 겨울', '보통 겨울', '혹독한 겨울')
            english = ('Mild winter', 'Winter', 'Harsh winter')
            if severity >= 0 and not (winter_words[min(severity, 2)] in shown or english[min(severity, 2)] in shown):
                errors['A23'].append(capture.stem + ': severity label mismatch')
            if severity < 0 and review.get('severity_visible', True):
                errors['A23'].append(capture.stem + ': severity visible outside winter')
            if severity >= 0 and time.get('season') != 3 and hour < 7 and not ('겨울 7일째 밤' in shown or 'Winter, night of day 7' in shown):
                errors['A23'].append(capture.stem + ': booked winter night label missing')
            intervention = data.get('intervention_state', {})
            if (data.get('intervention_casts') or review.get('intervention_effects') or intervention.get('rain_settlement_ids') or intervention.get('cold_snap_tonight') or intervention.get('log_kinds')):
                errors['A24'].append(capture.stem + ': intervention contaminates natural evidence')
            if winter_night:
                counts['A21'] += 1
                cold_tiles = {}
                for x, y, state, target, flag in zip(*(snapshot.get(key, []) for key in ('xs', 'ys', 'states', 'target_kinds', 'colds')), strict=True):
                    if state == 3 and target == 3 and flag == 1:
                        cold_tiles[(x, y)] = cold_tiles.get((x, y), 0) + 1
                expected_frost = {row['id'] for row in entities if row['kind'] == 'building' and row.get('building_kind') == 1 and row.get('visible') and cold_tiles.get((row.get('tile_x'), row.get('tile_y')), 0)}
                observed_frost = set(review.get('frosted_shelter_ids', []))
                if not bound or observed_frost != expected_frost:
                    errors['A21'].append(capture.stem + ': independently observed frost does not match cold-sleeper snapshot')
                cycle_path = capture.path.with_suffix('.cold_cycle.json')
                cycle = json.loads(cycle_path.read_text()) if cycle_path.exists() else {}
                samples = cycle.get('samples', [])
                puffed = {row['id'] for sample in samples for row in sample.get('shelters', []) if row.get('doorway_puff_alpha', 0) > 0}
                duration = (samples[-1]['real_time_usec'] - samples[0]['real_time_usec']) / 1e6 if samples else 0
                if expected_frost and (duration < 2.4 or not expected_frost.issubset(puffed) or not review.get('doorway_cycle_visible', False)):
                    errors['A21'].append(capture.stem + ': doorway breath lacks observed full cycle')
                fire_by_id = dict(zip(settlement.get('settlement_ids', []), settlement.get('fire_lit', [])))
                expected_ash = {row['id'] for row in entities if row['kind'] == 'building' and row.get('building_kind') == 0 and row.get('visible') and fire_by_id.get(row.get('settlement_id')) == 0}
                if set(review.get('ash_hearth_ids', [])) != expected_ash:
                    errors['A21'].append(capture.stem + ': cold ash mismatch with snapshot fire state')
                if cold_count and 0 in fire_by_id.values():
                    counts['A25'] += 1
                    cues = set(review.get('visible_cues', [])) & {'frost', 'doorway_breath', 'cold_ash', 'tint', 'hud_count'}
                    if not bound or len(cues) < 2:
                        errors['A25'].append(capture.stem + ': fewer than two independently visible cold cues')
            if capture.stem in morning_stems or capture.stem == warm_control_stem:
                counts['A22'] += 1
                trace = data.get('cold_truth_trace', {})
                if trace.get('source') != 'FFI colds sampled independently of presentation' or not 0 < trace.get('max_sample_interval_ticks', 0) <= 8:
                    errors['A22'].append(capture.stem + ': missing independent FFI clear-time history')
                history = {row['id']: row for row in trace.get('agents', [])}
                state = dict(zip(snapshot.get('agent_ids', []), snapshot.get('states', [])))
                target = dict(zip(snapshot.get('agent_ids', []), snapshot.get('target_kinds', [])))
                required, possible = set(), set()
                duration = float(time.get('ticks_per_day', 1440)) / 1440 * 90
                rendered = set()
                for row in entities:
                    ident = row.get('id')
                    if row['kind'] != 'agent' or not row.get('visible') or flags.get(ident) != 0 or (state.get(ident) == 3 and target.get(ident) == 3):
                        continue
                    truth = history.get(ident, {})
                    definitely, possibly = chill_window(data['tick'], truth.get('last_cold_tick', -1), truth.get('first_clear_tick', -1), duration)
                    if definitely:
                        required.add(ident)
                    if possibly:
                        possible.add(ident)
                    if row.get('chill_alpha', 0) > 0:
                        rendered.add(ident)
                        if not truth.get('last_cold_tick', -1) <= row.get('chill_since_tick', -2) <= truth.get('first_clear_tick', -1):
                            errors['A22'].append(f'{capture.stem}:{ident}: chill timestamp outside independent clear interval')
                observed = set(review.get('chill_agent_ids', []))
                if (not bound or not required.issubset(observed) or not observed.issubset(possible)
                        or not required.issubset(rendered) or not rendered.issubset(possible)
                        or (capture.stem == warm_control_stem and (observed or rendered))):
                    errors['A22'].append(capture.stem + ': visible/rendered morning chill differs from independent FFI interval')

        else:
            counts['A24'] += 1
            casts = data.get('intervention_casts', [])
            if data.get('scenario') != 'INTERVENTION SCENARIO' or not casts or any(not cast.get('reply', {}).get('ok') for cast in casts):
                errors['A24'].append(capture.stem + ': missing intervention provenance or failed cast')
            expected_effect = casts[-1].get('kind') if casts else ''
            observed_effects = review.get('intervention_effects', [])
            if not capture.stem.endswith('_panel_log') and review.get('world_effect_visibly_identified') is False:
                errors['A24'].append(capture.stem + ': observer explicitly saw only panel confirmation, not the world effect')
            if not bound or not intervention_visible(capture.stem, expected_effect, float(data.get('seconds_since_last_cast', -1)), observed_effects):
                errors['A24'].append(capture.stem + ': effect not independently observed')
    for code in ('A21', 'A22', 'A25'):
        if not counts[code]:
            errors[code].append('No qualifying natural frame; N/A is not PASS')
    if warm_control_stem is None:
        errors['A22'].append('Warm control unavailable: no measured lit-night/no-cold preceding12h control')
    return [result(code, counts[code] > 0 and not failures, json.dumps({'frames_checked': counts[code], 'errors': failures, 'independent_observations': str(review_path)}, ensure_ascii=False)) for code, failures in errors.items()]
