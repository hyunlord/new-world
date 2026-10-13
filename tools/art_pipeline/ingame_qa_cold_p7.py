# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: imported by measure_p7.py after windowed captures.
"""Check optional cold presentation against captured bridge and render observations."""
from __future__ import annotations

import json
from pathlib import Path

from ingame_qa_pixels import Check, result
from ingame_qa_types import Capture


def cold(captures: list[Capture], gate: Path) -> Check:
    """Require observed cold cycles and target labels, retaining no-cold N/A counts."""
    tests = ('test_cold_presentation', 'test_hud_wood', 'test_inspector_cold')
    results_path = gate / 'results.tsv'
    codes = {row[0]: int(row[1]) for line in results_path.read_text().splitlines()
             if len(row := line.split('\t')) >= 2} if results_path.exists() else {}
    test_status = {name: codes.get(name) for name in tests}
    errors: list[str] = []
    counts: dict[str, int] = {}
    visible_cold = 0
    labels_checked = 0
    cycles_checked = 0
    for capture in captures:
        data = json.loads(capture.path.with_suffix('.entities.json').read_text())
        snapshot = data.get('agent_snapshot', {})
        flags = dict(zip(snapshot.get('agent_ids', []), snapshot.get('colds', [])))
        agents = [row for row in data['entities'] if row['kind'] == 'agent' and row.get('visible')]
        cold_ids = {row['id'] for row in agents if flags.get(row['id'], 0) == 1}
        counts[capture.stem] = len(cold_ids)
        visible_cold += len(cold_ids)
        for row in agents:
            marked = flags.get(row['id'], 0) == 1
            if bool(row.get('cold', False)) != marked:
                errors.append(f'{capture.stem}:{row["id"]}: marker mismatch')
            if not marked and (float(row.get('puff_alpha', 0)) > 0 or float(row.get('shiver_offset', 0)) != 0):
                errors.append(f'{capture.stem}:{row["id"]}: non-cold effect')
            eligible = not row.get('moving', False) and row.get('cold_base_pose') in ('idle', 'sit', 'hop')
            if marked and eligible and (row.get('pose') != 'crossed' or row.get('cold_pose') not in ('crossed', 'sit', 'idle')):
                errors.append(f'{capture.stem}:{row["id"]}: cold pose decision mismatch')
            if marked and not eligible and row.get('cold_base_pose') and row.get('pose') != row['cold_base_pose']:
                errors.append(f'{capture.stem}:{row["id"]}: cold overrode an ineligible pose')
        if cold_ids:
            cycle_path = capture.path.with_suffix('.cold_cycle.json')
            if not cycle_path.exists():
                errors.append(f'{capture.stem}: missing observed cold cycle')
            else:
                cycle = json.loads(cycle_path.read_text())
                samples = cycle.get('samples', [])
                duration = (samples[-1]['real_time_usec'] - samples[0]['real_time_usec']) / 1e6 if samples else 0
                puffed = {row['id'] for sample in samples for row in sample['agents'] if row.get('cold') and row.get('puff_alpha', 0) > 0}
                unmarked = [row['id'] for sample in samples for row in sample['agents'] if not row.get('cold') and row.get('puff_alpha', 0) > 0]
                if duration < 2.4 or cold_ids - puffed or unmarked:
                    errors.append(f'{capture.stem}: cycle duration={duration:.3f}, missing={sorted(cold_ids-puffed)}, unmarked={unmarked}')
                cycles_checked += 1
        targets = data.get('settlement_snapshot', {}).get('wood_targets', [])
        if targets and max(targets) > 0:
            labels = [row for row in data['entities'] if row['kind'] == 'stockpile_wood_label' and row.get('visible')]
            for row in labels:
                labels_checked += 1
                target = int(row.get('wood_target', 0))
                wood = int(row.get('wood_stock', -1))
                if target <= 0 or f'{wood}/{target}' not in row.get('text', ''):
                    errors.append(f'{capture.stem}:{row["id"]}: missing wood/target label')
    details = {'tests': test_status, 'captured_visible_cold': visible_cold,
               'cold_counts_by_frame': counts, 'presentation': 'N/A' if visible_cold == 0 else 'observed',
               'cycles_checked': cycles_checked, 'target_labels_checked': labels_checked, 'errors': errors}
    return result('A19', bool(captures) and all(value == 0 for value in test_status.values()) and not errors,
                  json.dumps(details, ensure_ascii=False))
