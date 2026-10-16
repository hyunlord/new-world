# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by measure_p8.py
"""Normalize actual founding markers without mixing logical/framebuffer pixels."""
from __future__ import annotations

import json
import math
from pathlib import Path

from ingame_qa_new_p8 import WayFrame


def way_frames(paths: list[Path]) -> list[WayFrame]:
    """Derive distance from independent FFI positions and use logical geometry only."""
    frames: list[WayFrame] = []
    for path in paths:
        data = json.loads(path.with_suffix('.entities.json').read_text())
        raw = data['society_snapshot']
        agents = data['agent_snapshot']
        positions = dict(zip(agents['agent_ids'], zip(agents['xs'], agents['ys'], strict=True), strict=True))
        fissions = set(raw['founder_fission_ids'])
        groups = data['society_presentation']['founding_groups']
        drawn = {group['fission_id'] for group in groups}
        for fid in sorted(fissions | drawn):
            matching = [group for group in groups if group['fission_id'] == fid]
            errors: list[str] = []
            if len(matching) != 1 or fid not in fissions:
                errors.append('drawn founding groups differ from raw fission membership')
            if not matching:
                frames.append({'frame': path.stem, 'walking': True, 'site_on_screen': False,
                    'stake_visible': False, 'arrows': [], 'screen_center': [0, 0], 'target_screen': [1, 0],
                    'distance_tiles': -1, 'expected_distance_tiles': 0, 'metadata_errors': errors})
                continue
            group = matching[0]
            target = group['target_screen_position']
            width, height = group['viewport_size']
            on_screen = 0 <= target[0] < width and 0 <= target[1] < height
            if on_screen != group['site_on_screen']:
                errors.append('site_on_screen differs from logical viewport geometry')
            arrows = [row['edge_screen_position'] for row in matching if row['edge_visible']]
            if any(not (0 <= point[0] < width and 0 <= point[1] < height) for point in arrows):
                errors.append('edge arrow outside logical viewport')
            leaders = [index for index, value in enumerate(raw['founder_fission_ids'])
                       if value == fid and raw['founder_leader'][index] == 1]
            distance = -100.0
            if len(leaders) != 1:
                errors.append('raw founding group lacks exactly one leader')
            else:
                index = leaders[0]
                ident = raw['founder_agent_ids'][index]
                if ident not in positions:
                    errors.append('raw leader position missing')
                else:
                    x, y = positions[ident]
                    distance = math.hypot(raw['founder_site_xs'][index] - x, raw['founder_site_ys'][index] - y)
            frames.append({'frame': path.stem, 'walking': fid in fissions,
                'site_on_screen': on_screen, 'stake_visible': group['stake_visible'], 'arrows': arrows,
                'screen_center': group['screen_center'], 'target_screen': target,
                'distance_tiles': group['displayed_distance_tiles'], 'expected_distance_tiles': distance,
                'metadata_errors': errors})
    return frames
