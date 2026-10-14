# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/prepare_blind_p8.py RAW_ART RAW_LONG RAW_SOCIETY RAW_GOD
"""Freeze independent image packets before any observer answers are solicited."""
from __future__ import annotations

import hashlib
import json
import math
import os
import shutil
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from ingame_qa_pixels_p7 import crowd
from ingame_qa_types import load_capture
from make_blind_grid_p6 import create
from PIL import Image, ImageDraw
from review_boards_p7 import cycle_contacts

OUTPUT: Final = Path(os.environ.get('P8_REVIEW_OUTPUT', 'docs/art/production8/review'))


@dataclass(frozen=True, slots=True)
class PacketError(Exception):
    reason: str

    def __str__(self) -> str:
        return self.reason


def freeze(path: Path, payload) -> None:
    """Save one immutable answer/provenance document with a digest companion."""
    if path.exists():
        raise FileExistsError('Refusing to overwrite frozen packet: ' + str(path))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + '\n')
    path.with_suffix('.sha256').write_text(hashlib.sha256(path.read_bytes()).hexdigest() + '\n')


def source(path: Path, destination: Path):
    """Copy exact bytes under an opaque name and bind both paths."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(path, destination)
    return {'source': str(path.resolve()), 'image': str(destination.resolve()),
            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def rows(directory: Path):
    """Read only completed manifest entries, retaining original order."""
    return [(directory / row['file'], json.loads((directory / row['file']).with_suffix('.entities.json').read_text()))
            for row in json.loads((directory / 'manifest.json').read_text())['captures']]


def society_packets(society) -> None:
    """Freeze four unselected overviews and first-departure walking questions."""
    overviews = [(path, data) for path, data in society if data.get('capture_plan_row', [None, None, None])[2] == 'FV'
                 and data.get('selected_settlement_id', -1) == -1]
    walks = [(path, data) for path, data in society if data.get('capture_plan_row', [None, None, None])[2] == 'FD'
             and data['society_snapshot']['founder_agent_ids']]
    if len(overviews) < 4 or len(walks) < 2:
        raise PacketError('Inherited blind protocol requires four unselected FV and two FD frames')
    # Evenly sample chronology so the packet includes the late three-village overview.
    chosen = [overviews[round(index * (len(overviews) - 1) / 3)] for index in range(4)]
    created_ns = time.time_ns()
    sources = {}
    village_answers = {}
    walk_answers = {}
    for index, (path, data) in enumerate(chosen, 1):
        sources[f'A27_{index}'] = source(path, OUTPUT / 'packets/society' / f'V{index:02}.png')
        raw = data['society_snapshot']
        numbers = {row['id']: row['number'] for row in data['society_presentation']['settlements']}
        village_answers[str(index)] = {'count': len(raw['settlement_ids']), 'parents': {
            str(numbers[sid]): numbers[parent] for sid, parent in zip(raw['settlement_ids'], raw['parent_ids'], strict=True) if parent >= 0}}
    for index, (path, _) in enumerate(walks[:2], 1):
        sources[f'A28_{index}'] = source(path, OUTPUT / 'packets/society' / f'W{index:02}.png')
        walk_answers[str(index)] = 'moving away to settle'
    freeze(OUTPUT.parent / 'blind_society/frozen_answers.json',
           {'created_ns': created_ns, 'A27': village_answers, 'A28': walk_answers, 'sources': sources})
    direction_created_ns = time.time_ns()
    direction_frames = {}
    names = ('E', 'SE', 'S', 'SW', 'W', 'NW', 'N', 'NE')
    for index, (path, data) in enumerate(walks, 1):
        groups = data['society_presentation']['founding_groups']
        if len(groups) != 1:
            raise PacketError('Direction packet requires an explicit group label for multi-group frames: ' + str(path))
        group = groups[0]
        target, center = group['target_screen_position'], group['screen_center']
        angle = math.atan2(target[1] - center[1], target[0] - center[0])
        direction = names[math.floor(angle / (math.pi / 4) + .5) % 8]
        binding = source(path, OUTPUT / 'packets/direction' / f'D{index:02}.png')
        direction_frames[f'D{index:02}'] = dict(binding, expected=direction,
            vector=[target[0] - center[0], target[1] - center[1]], fission_id=group['fission_id'])
    freeze(OUTPUT / 'keys/direction.json', {'created_ns': direction_created_ns, 'frames': direction_frames})


def crowd_packet(natural) -> None:
    """Rank by unchanged opaque-mask A18, preserving every selection score."""
    scored = []
    for path, _ in natural:
        check = crowd(load_capture(path), Path('.'))
        detail = json.loads(check.detail)
        scored.append((detail['half_occluded'], detail['visible'], path))
    scored.sort(key=lambda row: (-row[0], -row[1], str(row[2])))
    if len(scored) < 12:
        raise PacketError('Twelve natural crowd frames are required')
    created_ns = time.time_ns()
    crowd_frames = {}
    for index, (count, visible, path) in enumerate(scored[:12], 1):
        crowd_frames[f'C{index:02}'] = dict(source(path, OUTPUT / 'packets/countability' / f'C{index:02}.png'), half_occluded=count, visible=visible)
    freeze(OUTPUT / 'keys/countability.json', {'created_ns': created_ns,
           'selection': 'top12 half-occluded count, then visible count, then source path', 'frames': crowd_frames})


def observation_packets(frames) -> None:
    """Number every visible object equally; retain unaltered pixels alongside it."""
    bindings = {}
    hud = []
    cycles = []
    for index, (path, data) in enumerate(frames, 1):
        label = f'O{index:03}'
        batch = f'batch{(index - 1) // 8 + 1:03}'
        original = OUTPUT / 'packets/objects' / batch / f'{label}.png'
        binding = source(path, original)
        with Image.open(path) as opened:
            annotated = opened.convert('RGB')
        draw = ImageDraw.Draw(annotated)
        identities = []
        for entity in data['entities']:
            if not entity.get('visible') or entity['kind'] not in ('agent', 'building'):
                continue
            x, y, width, height = entity['rect']
            ident = ('A' if entity['kind'] == 'agent' else 'B') + str(entity['id'])
            draw.rectangle((x, y, x + width, y + height), outline='#FFFF00', width=1)
            draw.text((x, max(0, y - 12)), ident, fill='#FFFF00', stroke_width=1, stroke_fill='black')
            identities.append({'label': ident, 'id': entity['id'], 'kind': entity['kind']})
        marked = original.with_name(label + '_ids.png')
        annotated.save(marked, optimize=True)
        bindings[label] = dict(binding, annotated=str(marked), annotated_sha256=hashlib.sha256(marked.read_bytes()).hexdigest(), objects=identities)
        if data.get('scenario') == 'NATURAL SCENARIO':
            labels = [row for row in data.get('labels', []) if 'HudTopbar' in row.get('name', '')]
            bottom = max((row['rect'][1] + row['rect'][3] for row in labels), default=160)
            with Image.open(path) as opened:
                hud.append((label, opened.convert('RGB').crop((0, 0, opened.width, min(opened.height, round(bottom) + 8)))))
        cycle = path.with_suffix('.cold_cycle.json')
        if cycle.exists():
            # Pure helper writes to supplied P8 directory. Its original-name captions
            # stay coordinator-only; neutral copies are provided to observers below.
            record = cycle_contacts(cycle, OUTPUT / 'cycle_sources')
            neutral = original.with_name(label + '_sequence.png')
            contact_path = record['contact']
            if not isinstance(contact_path, str):
                raise PacketError('Cycle contact path must be a string')
            with Image.open(contact_path) as opened:
                contact = opened.convert('RGB')
            ImageDraw.Draw(contact).rectangle((0, 0, contact.width, 26), fill='#191c1c')
            ImageDraw.Draw(contact).text((8, 6), label + ' actual sampled sequence', fill='white')
            contact.save(neutral, optimize=True)
            cycles.append(dict(record, observer_image=str(neutral), observer_sha256=hashlib.sha256(neutral.read_bytes()).hexdigest()))
    for offset in range(0, len(hud), 4):
        batch = hud[offset:offset + 4]
        canvas = Image.new('RGB', (max(image.width for _, image in batch), sum(image.height + 24 for _, image in batch)), '#191c1c')
        y = 0
        for label, image in batch:
            ImageDraw.Draw(canvas).text((8, y + 4), label, fill='white')
            canvas.paste(image, (0, y + 24))
            y += image.height + 24
        destination = OUTPUT / 'packets/hud' / f'H{offset // 4 + 1:03}.png'
        destination.parent.mkdir(parents=True, exist_ok=True)
        canvas.save(destination, optimize=True)
    freeze(OUTPUT / 'keys/objects.json', {'created_ns': time.time_ns(), 'frames': bindings, 'cycles': cycles})


def main(directories: list[Path]) -> None:
    """Build all packets once; this command never runs or grades observers."""
    if (OUTPUT / 'packet_sources.json').exists() or (OUTPUT / 'keys').exists():
        raise FileExistsError('Frozen P8 packet exists; preserve it and use an explicitly reviewed new round')
    art, long_raw, society_raw, god_raw = directories
    natural = [(path, data) for directory in (art, long_raw) for path, data in rows(directory)
               if not path.stem.startswith(('a_calib', 'a_lineup'))]
    society, god = rows(society_raw), rows(god_raw)
    overview_count = sum(data.get('capture_plan_row', [None, None, None])[2] == 'FV' and data.get('selected_settlement_id', -1) == -1 for _, data in society)
    walk_count = sum(data.get('capture_plan_row', [None, None, None])[2] == 'FD' and bool(data['society_snapshot']['founder_agent_ids']) for _, data in society)
    if overview_count < 4 or walk_count < 2 or len(natural) < 12:
        raise PacketError('Incomplete final capture coverage: need4FV,2FD,12natural before freezing')
    create(art, OUTPUT / 'packets/activity', OUTPUT / 'keys/activity.json', Path('.'))
    key = OUTPUT / 'keys/activity.json'
    key.with_suffix('.sha256').write_text(hashlib.sha256(key.read_bytes()).hexdigest() + '\n')
    society_packets(society)
    crowd_packet(natural)
    observation_packets(natural + society + god)
    freeze(OUTPUT / 'packet_sources.json', {'created_ns': time.time_ns(), 'raw_directories': [str(path.resolve()) for path in directories],
        'counts': {'natural': len(natural), 'society': len(society), 'god': len(god)},
        'packet_sha256': {str(path.relative_to(OUTPUT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted((OUTPUT / 'packets').rglob('*.png'))},
        'observer_root': str((OUTPUT / 'packets').resolve()), 'protocol': 'docs/art/production8/BLIND_PROTOCOL.md'})
    (OUTPUT / 'packets/INSTRUCTIONS.txt').write_text('Use only assigned packet images. Never open keys, source sidecars, GOAL, code or coordinator files. Activity: label1..24 at4x/2x using walk/carry/gather/build/eat/drink/sit/talk/child/unsure. Society V: count villages and daughter-number to parent-number relation; W: working/wandering/moving away to settle/fleeing/unsure. Direction D: N/NE/E/SE/S/SW/W/NW/unsure. Countability C: yes/roughly/no. Objects O: transcribe HUD/toast and describe visible markings, particles, breath, outlines and fire by neutral A/B labels; uncertain remains uncertain. Save original JSON before any grading.\n')
    (OUTPUT / 'packets/RESPONSE_SCHEMA.json').write_text(json.dumps({
        'observer_id': 'unique independent session ID',
        'activity': {'4x': {'1..24': 'one activity label'}, '2x': {'1..24': 'one activity label'}},
        'A27': {'1..4': {'count': 'integer or unknown', 'parents': 'daughter number to parent number mapping'}},
        'A28': {'1..2': 'one allowed activity choice'},
        'direction': {'Dxx': 'N/NE/E/SE/S/SW/W/NW/unsure'},
        'countability': {'C01..C12': 'yes/roughly/no'},
        'objects': {'Oxxx': {'hud_text': 'literal transcription', 'toast_text': 'literal transcription',
            'observations': 'free description with A/B object labels; include absent/uncertain',
            'sequence_observations': 'observed changes over shown times'}},
        'note': 'Return only assigned packet tasks; these are field descriptions, never expected answers.'}, indent=2) + '\n')
    print(json.dumps({'status': 'frozen, observers not launched', 'output': str(OUTPUT), 'natural_frames': len(natural)}))


if __name__ == '__main__':
    main([Path(value) for value in sys.argv[1:]])
