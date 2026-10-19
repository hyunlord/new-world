#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
# How to run: python3 plan_activity_blind_p12.py CHECKOUT NEWDIR CAPTURE_JSON ...
"""Freeze P12 exclusions and selection before extracting original-size blind crops."""
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

sys.path.insert(0, str(Path(__file__).parent / 'p12_activity_oracle'))
from p12_activity_oracle.activity_rules import EvidenceError, load_atlas
from p12_activity_oracle.check_activity_sidecars import check_capture
from run_activity_captures_p13 import Capture, validate_capture

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


def holder_geometry(body: list[float], icon: list[float], blockers: list[list[float]],
                    icon_blockers: list[list[float]] | None = None) -> dict:
    """Union both footprints, grow eight screen pixels, square without rescaling."""
    for rect in [body, icon, *blockers, *(icon_blockers or [])]:
        if len(rect) != 4 or not all(math.isfinite(value) for value in rect) or rect[2] <= 0 or rect[3] <= 0:
            raise EvidenceError('finite positive body/icon/blocker rectangles required')
    left = min(body[0], icon[0]) - 8
    top = min(body[1], icon[1]) - 8
    right = max(body[0] + body[2], icon[0] + icon[2]) + 8
    bottom = max(body[1] + body[3], icon[1] + icon[3]) + 8
    side = max(right-left, bottom-top)
    cx, cy = (left+right)/2, (top+bottom)/2
    # Outward integer rounding preserves every native pixel in the exact square.
    left, top = math.floor(cx-side/2), math.floor(cy-side/2)
    extent = max(math.ceil(cx+side/2)-left, math.ceil(cy+side/2)-top)
    body_covered = covered_fraction(body, blockers)
    icon_covered = covered_fraction(icon, blockers if icon_blockers is None else icon_blockers)
    return {'box': [left, top, left+extent, top+extent], 'upscale': 1,
            'resampling': 'none', 'body_covered_fraction': body_covered,
            'icon_covered_fraction': icon_covered,
            'excluded': body_covered > .5 or icon_covered > .5}


def row_geometry(row: dict, draw_rects: list[dict]) -> dict:
    """Use separate actual draw ranks; a holder's attached art is not foreign cover."""
    owner = row['node_path']
    foreign = [item for item in draw_rects if item['node_path'] != owner
               and not item['node_path'].startswith(owner + '/')]
    body_blockers = [item['rect'] for item in foreign if item['draw_rank'] > row['draw_rank']]
    icon_blockers = [item['rect'] for item in foreign if item['draw_rank'] > row['icon_draw_rank']]
    return holder_geometry(row['viewport_rect'], row['icon_viewport_rect'], body_blockers, icon_blockers)


def crowd_proof(data: dict) -> dict:
    """Report crowd and coverage independently; missing evidence cannot pass H9."""
    centre = data['crowd_centre_tile']
    agents = data['raw_agent_snapshot']
    nearby = [int(entity) for entity, x, y in zip(agents['ids'], agents['xs'], agents['ys'], strict=True)
              if (x-centre[0])**2 + (y-centre[1])**2 <= 100]
    selected = data['selected_entity_bits']
    holders = [row for row in data['activity_holders'] if row['entity_bits'] == selected]
    geometry = row_geometry(holders[0], data['draw_rects']) if len(holders) == 1 else None
    visible = len(holders) == 1 and holders[0]['visible_in_tree'] and holders[0]['fully_inside_viewport']
    valid = (data['complete'] and not data['errors'] and data['draw_order_schema'] == 3
             and data['draw_order_complete'] and data['zoom'] == 4 and len(nearby) >= 25
             and selected in nearby and visible and geometry is not None
             and geometry['body_covered_fraction'] == 0 and geometry['icon_covered_fraction'] == 0)
    return {'selected_entity_bits': selected, 'nearby_entity_bits': nearby, 'within_10_tiles': len(nearby),
            'centre_tile': centre, 'geometry': geometry, 'zero_coverage_crowd_proven': bool(valid),
            'activity_reading': 'NOT_VERIFIED'}


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
        if capture_index == 2 and case.seed != 20261018:
            raise EvidenceError('day60 base capture must use seed20261018')
        if capture_index < 3:
            base_worlds.add((case.seed, case.day))
        if capture_index >= 3 and ((case.seed, case.day) not in base_worlds or data.get('center_kind') not in range(7)):
            raise EvidenceError('added captures require same seed/day and a centered missing kind')
        with Image.open(path.with_name('capture.png')) as image:
            if list(image.size) != data['viewport']:
                raise EvidenceError('image dimensions differ from sidecar')
        if data.get('draw_order_schema') != 3 or data.get('draw_order_complete') is not True:
            raise EvidenceError('complete all-world-sprite draw ordering schema3 required')
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
            geometry = row_geometry(row, data['draw_rects'])
            left, top, right, bottom = geometry['box']
            if geometry['excluded']:
                reason = 'body_or_icon_more_than_50_percent_covered'
            if not (row['visible'] and row['visible_in_tree'] and row['fully_inside_viewport']):
                reason = 'not_visible_fully_inside_viewport'
            if left < 0 or top < 0 or right > data["viewport"][0] or bottom > data["viewport"][1]:
                reason = 'original_crop_outside_frame'
            if reason:
                exclusions.append({'source': str(path), 'agent_id': row['agent_id'], 'kind': kind,
                                   **geometry, 'reason': reason})
                continue
            if counts[kind] < 2 and (case.seed, row["agent_id"], kind) not in chosen_holders:
                selected.append({'source': str(path), 'agent_id': row['agent_id'], 'kind': kind,
                                 'entity_bits': row['entity_bits'], **geometry,
                                 'body_rect': rect, 'icon_rect': row['icon_viewport_rect'],
                                 'body_draw_rank': row['draw_rank'], 'icon_draw_rank': row['icon_draw_rank'],
                                 'icon_rect_source': row['icon_rect_source']})
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
            crop.save(blind / (row['blind_id'] + '.png'))
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
