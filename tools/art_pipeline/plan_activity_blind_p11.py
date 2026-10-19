#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
# How to run: python3 plan_activity_blind_p11.py CHECKOUT NEWDIR CAPTURE_JSON ...
"""Freeze P11 exclusions and selection before extracting original-size blind crops."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
import time
from pathlib import Path
from typing import Final

from PIL import Image

sys.path.insert(0, str(Path(__file__).parent / 'p11_activity_oracle'))
from activity_rules import EvidenceError, load_atlas
from check_activity_sidecars import check_capture
from run_activity_captures_p11 import Capture, validate_capture

NAMES: Final = ('TendFire', 'Craft', 'Repair', 'HearthCircle', 'Explore', 'Play', 'SitByFire')


def covered_fraction(rect: list[float], blockers: list[list[float]]) -> float:
    """Union rectangle area, so overlapping foreground sprites never double count."""
    x, y, width, height = rect
    clips = [(max(x, a), max(y, b), min(x + width, a + w), min(y + height, b + h))
             for a, b, w, h in blockers if a < x + width and a + w > x and b < y + height and b + h > y]
    xs = sorted({edge for a, _, c, _ in clips for edge in (a, c)})
    area = 0.0
    for left, right in zip(xs, xs[1:]):
        intervals = sorted((b, d) for a, b, c, d in clips if a <= left and c >= right)
        end = -math.inf
        length = 0.0
        for low, high in intervals:
            length += max(0.0, high - max(low, end))
            end = max(end, high)
        area += (right - left) * length
    return area / (width * height)


def terminal_shortage(path: Path, paths: list[Path], counts: list[int]) -> dict[str, object]:
    """Require the leader's frozen terminal decision to match this exact selection."""
    data = json.loads(path.read_text())
    expected = [str(value) for value in paths]
    if (data.get('schema') != 1 or data.get('terminal_shortage') is not True
            or data.get('authorized_by') != 'leader'
            or data.get('remaining_capture_work') is not False
            or not isinstance(data.get('reason'), str) or not data['reason'].strip()
            or data.get('capture_order') != expected or data.get('counts') != counts
            or data.get('required_count') != 14 or not any(value < 2 for value in counts)):
        raise EvidenceError('terminal shortage record must authorize this exact incomplete selection')
    hashes = data.get('source_hashes', {})
    for source in paths:
        for item in (source, source.with_name('capture.png')):
            if hashes.get(str(item)) != hashlib.sha256(item.read_bytes()).hexdigest():
                raise EvidenceError('terminal shortage record source hash mismatch')
    return {'path': str(path.resolve()), 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'reason': data['reason']}


def main(args: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--partial-diagnostic', type=Path, metavar='TERMINAL_SHORTAGE_JSON')
    parser.add_argument('checkout', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('captures', nargs='+', type=Path)
    options = parser.parse_args(args)
    checkout, output = options.checkout, options.output
    paths = [value.resolve() for value in options.captures]
    if len(paths) < 3 or len(set(paths)) != len(paths):
        raise EvidenceError('requires three distinct base captures, without duplicate paths')
    output.mkdir(parents=True, exist_ok=False)
    atlas = load_atlas(checkout)
    selected = []
    exclusions = []
    counts = [0] * 7
    source_hashes = {}
    instants = []
    base_worlds = set()
    chosen_holders = set()
    for capture_index, path in enumerate(paths):
        data = json.loads(path.read_text())
        case = Capture(data['seed'], data['display_day'], data['hour'], data['zoom'])
        validate_capture(data, case)
        if capture_index < 2 and case.seed != 20261010:
            raise EvidenceError('day30 base captures must use the live seed')
        if capture_index < 3:
            base_worlds.add((case.seed, case.day))
        if capture_index >= 3 and ((case.seed, case.day) not in base_worlds or data.get('center_kind') not in range(7)):
            raise EvidenceError('added captures require same seed/day and a centered missing kind')
        with Image.open(path.with_name('capture.png')) as image:
            if list(image.size) != data['viewport']:
                raise EvidenceError('image dimensions differ from sidecar')
        if data.get('draw_order_schema') != 2 or data.get('draw_order_complete') is not True:
            raise EvidenceError('complete all-world-sprite draw ordering schema2 required')
        if data['zoom'] != 4 or not check_capture(data, atlas)['pose_check_complete']:
            raise EvidenceError('only pose-verified Z4 captures are eligible')
        instants.append([case.day, case.hour])
        source_hashes[str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
        source_hashes[str(path.with_name('capture.png'))] = hashlib.sha256(path.with_name('capture.png').read_bytes()).hexdigest()
        for row in sorted(data['activity_holders'], key=lambda holder: holder['agent_id']):
            if row['until_tick'] < 0:
                continue
            kind = row['kind']
            reason = ''
            rect = row['viewport_rect']
            blockers = [item['rect'] for item in data['draw_rects']
                        if item['draw_rank'] > row['draw_rank']]
            covered = covered_fraction(rect, blockers)
            x, y, w, h = rect
            # Exact Art Pass 10 v2 crop geometry and nearest-neighbor display scale.
            side = math.ceil(max(w, h) + 16)
            left, top = math.floor(x + w / 2 - side / 2), math.floor(y + h / 2 - side / 2)
            if covered > .5:
                reason = 'more_than_50_percent_covered'
            if not (row['visible'] and row['visible_in_tree'] and row['fully_inside_viewport']):
                reason = 'not_visible_fully_inside_viewport'
            if left < 0 or top < 0 or left + side > 1920 or top + side > 1080:
                reason = 'original_crop_outside_frame'
            if reason:
                exclusions.append({'source': str(path), 'agent_id': row['agent_id'], 'kind': kind,
                                   'covered_fraction': covered, 'reason': reason})
                continue
            if counts[kind] < 2 and (case.seed, row["agent_id"], kind) not in chosen_holders:
                selected.append({'source': str(path), 'agent_id': row['agent_id'], 'kind': kind,
                                 'entity_bits': row['entity_bits'], 'covered_fraction': covered,
                                 'box': [left, top, left + side, top + side], 'upscale': 4, 'resampling': 'nearest'})
                counts[kind] += 1
                chosen_holders.add((case.seed, row["agent_id"], kind))
    if instants[:3] != [[30, 10], [30, 15], [60, 10]]:
        raise EvidenceError('first three captures must be d30 10, d30 15, d60 10 in that order')
    if any(hour not in (11, 14) for _, hour in instants[3:]):
        raise EvidenceError('additional captures must be at 11 or 14')
    frozen = {'frozen_ns': time.time_ns(), 'capture_order': [str(path) for path in paths],
              'source_hashes': source_hashes, 'exclusions': exclusions, 'counts': counts,
              'missing_kinds': [kind for kind, count in enumerate(counts) if count < 2]}
    # This immutable log is on disk before ANY crop is made, including failed selection.
    (output / 'exclusions-before-crops.json').write_text(json.dumps(frozen, indent=2) + '\n')
    diagnostic = options.partial_diagnostic is not None
    if frozen['missing_kinds'] and not diagnostic:
        return 1
    terminal = terminal_shortage(options.partial_diagnostic, paths, counts) if diagnostic else None
    status = {'mode': 'partial_diagnostic' if diagnostic else 'full_protocol',
              'actual_count': len(selected), 'required_count': 14,
              'full_protocol_complete': not frozen['missing_kinds'],
              'acceptance': 'NOT_VERIFIED' if diagnostic else 'AWAITING_BLIND_ANSWERS',
              'required_bar': 'at least 10/14 AND no activity scores 0/2',
              'terminal_shortage': terminal}
    (output / 'protocol-status.json').write_text(json.dumps(status, indent=2) + '\n')
    selected.sort(key=lambda row: hashlib.sha256(f"{row['source']}:{row['entity_bits']}".encode()).hexdigest())
    for index, row in enumerate(selected):
        row['blind_id'] = f'crop_{index + 1:02d}'
    (output / 'key.json').write_text(json.dumps({'crops': selected, 'frozen_ns': frozen['frozen_ns'], 'protocol_status': status}, indent=2) + '\n')
    blind = output / 'blind'
    blind.mkdir()
    (blind / 'names.json').write_text(json.dumps({'choices': NAMES, 'instructions': 'Identify each numbered crop. Freeze answers before the village task; do not open any key.'}, indent=2) + '\n')
    for row in selected:
        with Image.open(Path(row['source']).with_name('capture.png')) as image:
            crop = image.crop(row['box'])
            crop.resize((crop.width * 4, crop.height * 4), Image.Resampling.NEAREST).save(blind / (row['blind_id'] + '.png'))
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
