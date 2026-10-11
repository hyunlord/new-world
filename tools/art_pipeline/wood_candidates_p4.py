# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/wood_candidates_p4.py --sheet V_a4 --raw /path/attempt_1.png
"""Freeze v4 wood candidates and natural-colour boards without installing art."""
from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
from typing import TypedDict

import numpy as np
from PIL import Image, ImageDraw

from despeckle import despeckle
from fill_v4 import process
from hair_maps import HairPalette, classify
from natural_boards_p4 import BoardFrame, GRASS, atlas_frames, compose, paste, save_board
from palette_tools import rgb_to_lab
from pixel_normalize import outline
from pixel_normalize_v4 import fit_cell_v4
from salvage_v4 import CandidateRecord, guard_data, guarded_candidate
from salvage_verdicts import candidate_fingerprint
from v3_semantics import semantic_counts
from v4_contracts import guard_frame, guard_group

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / 'docs/art/production4/wood'
BODIES = {'V_a4': 'adult_a', 'V_b4': 'adult_b', 'V_c3': 'child', 'W1': ''}


class Cell(TypedDict):
    cell: int
    target_slot: str
    name: str
    activity: str
    view: str
    frame: int


def cells(sheet: str) -> list[Cell]:
    """Map the literal sixteen row-major cells to stable runtime activities."""
    result: list[Cell] = []
    for index in range(1, 17):
        activity, view, frame = '', '', 0
        if sheet != 'W1':
            activity = 'carry_wood' if index <= 12 else 'gather_wood'
            view = ('S', 'E', 'N')[(index - 1) // 4] if index <= 12 else ('S' if index <= 14 else 'E')
            frame = (index - 1) % (4 if index <= 12 else 2)
            name = f'{activity}_{view}_{frame}'
        elif index <= 8:
            name = f'deadwood_stage_{3 - (index - 1) // 2}_variant_{(index - 1) % 2}'
        else:
            phase = (index - 9) % 4
            name = 'wood_icon' if phase == 3 else f'woodpile_stage_{phase + 1}'
            name += f'_variant_{(index - 9) // 4}'
        result.append(Cell(cell=index, target_slot=f'{sheet}_{index:02}', name=name,
                           activity=activity, view=view, frame=frame))
    return result


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def configuration() -> Path:
    """Bind stable reviewed reference lineage without modifying old evidence."""
    path = OUTPUT / 'config/catalog.json'
    source = ROOT / 'docs/art/production2/final_selection.json'
    baseline = json.loads(source.read_text())
    catalog = {'schema_version': 1, 'source_selection': str(source.relative_to(ROOT)),
               'source_selection_sha256': digest(source), 'checkpoint_attempted_cells': 64,
               'sheets': [{'id': sheet, 'body': body, 'derive_missing': False, 'cells': cells(sheet),
                           'reference_candidate': baseline['references'].get(body, '')}
                          for sheet, body in BODIES.items()]}
    content = json.dumps(catalog, indent=2) + '\n'
    if path.exists() and path.read_text() != content:
        raise ValueError('Frozen wood configuration changed')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
    return path


def finish_candidate(record: CandidateRecord, folder: Path) -> None:
    """Apply the existing bounded despeckler and freeze exact outlined hair maps."""
    palette = [tuple(row['rgb']) for row in json.loads((ROOT / 'tools/art_pipeline/production_palette.json').read_text())['colors']]
    if record['factor'] is None:
        return
    with Image.open(record['images']['raw']) as source:
        fit = fit_cell_v4(source, palette, record['factor'],
                          icon_canvas=record['sheet'] == 'W1' and record['cell'] in (12, 16))
    cleaned = despeckle(fit.native)
    native = cleaned.image
    dark = palette[int(np.argmin(rgb_to_lab(np.array(palette, dtype=np.uint8))[:, 0]))]
    sprite = outline(native, dark)
    images = {'native': native, 'sprite': sprite}
    if record['body']:
        data = json.loads((ROOT / 'tools/art_pipeline/hair_palette_p4.json').read_text())['bodies'][record['body']]
        hair = HairPalette(record['body'], frozenset(tuple(c) for c in data['seen_rgb']), frozenset(tuple(c) for c in data['hair_rgb']))
        images.update(hair_mask=classify(native, hair, activity=record['activity']),
                      sprite_hair_mask=classify(sprite, hair, activity=record['activity']))
    for kind, image in images.items():
        path = folder / f'{kind}.png'
        image.save(path, optimize=True)
        record['images'][kind] = str(path)
    after, before = np.asarray(native), np.asarray(fit.median_sample)
    opaque = after[..., 3] == 255
    common = opaque & (before[..., 3] == 255)
    differences = np.linalg.norm(rgb_to_lab(before[common, :3]) - rgb_to_lab(after[common, :3]), axis=1)
    berry, fire = semantic_counts(native)
    record['metrics'] = dict(fit.metrics, mean_delta_e=float(differences.mean()) if len(differences) else 100.,
                             berry_texels=berry, fire_texels=fire, despeckle_change_fraction=cleaned.change_fraction)
    record['guards'] = guard_data(guard_frame(guarded_candidate(record)))
    record['review_status'] = 'pending_independent_visual_review'


def review_boards(records: list[CandidateRecord], folder: Path) -> list[dict[str, str | list[str]]]:
    """Show every converted physical cell, including failed technical guards."""
    index: list[dict[str, str | list[str]]] = []
    installed = atlas_frames()
    body = records[0]['body']
    for start in range(0, 16, 8):
        chunk = records[start:start + 8]
        if body:
            frames = []
            for row in chunk:
                if 'sprite_hair_mask' not in row['images']:
                    continue
                with Image.open(row['images']['sprite']) as image, Image.open(row['images']['sprite_hair_mask']) as mask:
                    peers = tuple(p.sprite for p in installed if p.body == body and p.activity == row['activity'] and p.view == row['view'])
                    frames.append(BoardFrame(row['id'], body, row['activity'], row['view'], row['frame'], image.convert('RGBA'), mask.convert('RGBA'), peers))
            with Image.open(ROOT / f'docs/art/production3/reprocess/references/{body}.png') as reference:
                board = compose(frames, reference.convert('RGBA')) if frames else Image.new('RGB', (600, 200), GRASS)
        else:
            board = Image.new('RGB', (1200, 600), GRASS)
            draw = ImageDraw.Draw(board)
            for offset, row in enumerate(chunk):
                x, y = (offset % 4) * 300, (offset // 4) * 300
                draw.text((x + 8, y + 8), row['id'], fill='white')
                draw.text((x + 8, y + 24), row['name'], fill='white')
                if 'sprite' in row['images']:
                    with Image.open(row['images']['sprite']) as image:
                        paste(board, image.convert('RGBA'), x + 8, y + 50, 6)
                        paste(board, image.convert('RGBA'), x + 210, y + 50, 2)
        path = save_board(board, folder / f'page_{start // 8 + 1:02}.png')
        index.append({'path': str(path.relative_to(ROOT)), 'sha256': digest(path),
                      'candidate_ids': [row['id'] for row in chunk],
                      'unavailable': [row['id'] for row in chunk if 'sprite' not in row['images']]})
    return index


def run(sheet: str, raw: Path, attempt: int = 1) -> Path:
    """Freeze one sheet; callers must supply a new attempt to preserve old review."""
    config = configuration()
    folder = OUTPUT / 'processed' / f'{sheet}_attempt_{attempt}'
    ledger_path = folder / 'candidates.json'
    if ledger_path.exists():
        raise ValueError('Refusing to overwrite frozen wood candidates')
    folder.mkdir(parents=True, exist_ok=True)
    ledger = process(ROOT, json.loads(config.read_text()), sheet, raw, attempt,
                     folder / 'source_candidates.json', folder / 'cells')
    records = [row for row in ledger['candidates'] if row['source_kind'] == 'generated' and row['sheet'] == sheet]
    raw_cells = raw.parent / f'cells_attempt_{attempt}'
    raw_cells.mkdir(parents=True, exist_ok=True)
    for row in records:
        old_raw = Path(row['images']['raw'])
        destination = raw_cells / f'{row["slot_id"]}.png'
        old_raw.rename(destination)
        row['images']['raw'] = str(destination)
    (folder / 'source_candidates.json').write_text(json.dumps(ledger, indent=2, allow_nan=False) + '\n')
    for row in records:
        try:
            finish_candidate(row, folder / 'cells' / row['id'].replace('@', '_'))
        except (ValueError, IndexError) as error:
            row['conversion_error'] = str(error)
            row['guards'] = {'passed': False, 'errors': ['conversion_failure'], 'warnings': []}
    groups: defaultdict[tuple[str, str], list[CandidateRecord]] = defaultdict(list)
    for row in records:
        if row['body'] and row['guards']['passed']:
            groups[(row['activity'], row['view'])].append(row)
    for group in groups.values():
        report = guard_data(guard_group([guarded_candidate(row) for row in group]))
        for row in group:
            row['group_guards'] = report
    ledger['candidates'] = records + [row for row in ledger['candidates'] if row['source_kind'] == 'reference']
    ledger['selected'] = {}
    ledger['boards'] = review_boards(records, OUTPUT / 'boards' / f'{sheet}_attempt_{attempt}')
    ledger['candidate_fingerprints'] = {row['slot_id']: candidate_fingerprint(row) for row in records}
    ledger_path.write_text(json.dumps(ledger, indent=2, allow_nan=False) + '\n')
    template = OUTPUT / 'templates' / f'{sheet}_attempt_{attempt}.json'
    template.parent.mkdir(parents=True, exist_ok=True)
    template.write_text(json.dumps({'verdicts': [{'sprite_id': row['slot_id'], 'candidate_id': row['id'],
        'verdict': '', 'reason': '', 'observation': '', 'unfitted_acceptable': None} for row in records]}, indent=2) + '\n')
    return ledger_path


def freeze_collection() -> Path:
    """Bind the first four sheets for the exact 64-cell checkpoint and installer."""
    combined = OUTPUT / 'processed/candidates.json'
    index_path = OUTPUT / 'processed/review_index.json'
    if combined.exists() or index_path.exists():
        raise ValueError('Refusing to overwrite the four-sheet review binding')
    candidates: list[CandidateRecord] = []
    context: dict[str, CandidateRecord] = {}
    sources: dict[str, str] = {}
    boards: dict[str, str] = {}
    for sheet in BODIES:
        path = OUTPUT / f'processed/{sheet}_attempt_1/candidates.json'
        ledger = json.loads(path.read_text())
        sources[str(path.relative_to(ROOT))] = digest(path)
        candidates.extend(row for row in ledger['candidates'] if row['sheet'] == sheet)
        context.update({row['id']: row for row in ledger['candidates'] if row['source_kind'] == 'reference'})
        boards.update({page['path']: page['sha256'] for page in ledger['boards']})
    if len(candidates) != 64:
        raise ValueError('First four sheets must preserve exactly 64 physical cells')
    combined.write_text(json.dumps({'schema_version': 1, 'scale': {'adult_height': 28, 'child_height': 21},
        'candidates': candidates, 'lineage_candidates': list(context.values()),
        'recovery_ids': [row['id'] for row in candidates], 'attempted_cells': 64}, indent=2) + '\n')
    index_path.write_text(json.dumps({'candidates_sha256': digest(combined),
        'candidate_fingerprints': {row['id']: candidate_fingerprint(row) for row in candidates},
        'images_sha256': boards, 'source_sha256': sources}, indent=2) + '\n')
    return index_path


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sheet', choices=tuple(BODIES))
    parser.add_argument('--raw', type=Path)
    parser.add_argument('--attempt', type=int, default=1)
    parser.add_argument('--freeze-collection', action='store_true')
    args = parser.parse_args()
    if args.freeze_collection:
        print(freeze_collection())
    elif args.sheet and args.raw:
        print(run(args.sheet, args.raw.resolve(), args.attempt))
    else:
        print(configuration())
