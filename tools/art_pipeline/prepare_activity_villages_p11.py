#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
# How to run: python3 prepare_activity_villages_p11.py FROZEN_C_ANSWERS NEWDIR Z1_JSON ...
"""Prepare six summary questions only after blind activity answers exist."""
from __future__ import annotations
import hashlib
import json
import sys
import time
from pathlib import Path
from PIL import Image
from plan_activity_blind_p11 import NAMES


def main(args: list[str]) -> int:
    answers, output = Path(args[0]), Path(args[1])
    if not answers.is_file() or not answers.read_text().strip():
        raise RuntimeError('freeze c answers before preparing d questions')
    answer_hash = hashlib.sha256(answers.read_bytes()).hexdigest()
    paths = [Path(name).resolve() for name in args[2:]]
    if len(paths) != len(set(paths)):
        raise RuntimeError('duplicate capture paths would repeat village/capture pairs')
    output.mkdir(parents=True, exist_ok=False)
    selected = []
    counts = {}
    excluded = []
    source_hashes = {}
    for path in paths:
        name = str(path)
        data = json.loads(path.read_text())
        if data['zoom'] != 1 or not data['complete'] or data['errors']:
            raise RuntimeError('village task requires complete Z1 captures')
        source_hashes[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        source_hashes[str(path.with_name('capture.png'))] = hashlib.sha256(path.with_name('capture.png').read_bytes()).hexdigest()
        for row in sorted(data['village_labels']['activity_summaries'], key=lambda row: row['id']):
            activities = row['activities']
            village = (data['seed'], row['id'])
            if not row['visible'] or not activities or counts.get(village, 0) >= 2:
                continue
            if len(activities) >= 2 and activities[0]['count'] == activities[1]['count']:
                excluded.append({'source': name, 'seed': data['seed'], 'village': row['id'], 'reason': 'top_two_tie'})
                continue
            x, y, w, h = row['screen_rect']
            if w <= 0 or h <= 0 or x < 0 or y < 0 or x + w > 1920 or y + h > 1080:
                excluded.append({'source': name, 'seed': data['seed'], 'village': row['id'], 'reason': 'summary_outside_frame'})
                continue
            selected.append({'source': name, 'seed': data['seed'], 'village': row['id'], 'box': [x, y, x + w, y + h], 'kind': activities[0]['kind']})
            counts[village] = counts.get(village, 0) + 1
            if len(selected) == 6:
                break
        if len(selected) == 6:
            break
    evidence = {'prepared_ns': time.time_ns(), 'c_answers_sha256': answer_hash, 'source_hashes': source_hashes,
                'pairs': selected, 'exclusions': excluded, 'complete': len(selected) == 6}
    (output / 'key.json').write_text(json.dumps(evidence, indent=2) + '\n')
    if len(selected) != 6:
        return 1
    if hashlib.sha256(answers.read_bytes()).hexdigest() != answer_hash:
        raise RuntimeError('c answers changed during d preparation')
    blind = output / 'blind'
    blind.mkdir()
    (blind / 'names.json').write_text(json.dumps({'choices': NAMES, 'instructions': 'Name the most common activity for each numbered village summary; freeze all answers before opening either key.'}, indent=2) + '\n')
    for index, row in enumerate(selected):
        with Image.open(Path(row['source']).with_name('capture.png')) as image:
            image.crop(row['box']).save(blind / f'village_{index+1:02d}.png')
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
