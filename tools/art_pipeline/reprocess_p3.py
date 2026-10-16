# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/reprocess_p3.py
"""Reprocess every unaccepted physical source, preserving overlap and review evidence."""
from __future__ import annotations

from collections import defaultdict
from copy import deepcopy
import hashlib
import json
from pathlib import Path
from typing import Final, NotRequired, TypedDict

import numpy as np
from PIL import Image, ImageDraw

from despeckle import despeckle
from palette_tools import rgb_to_lab
from pixel_normalize import outline
from pixel_normalize_v4 import fit_cell_v4
from regions import DISPLAY, classify as legacy_classify
from regions_v2 import ColorRegionTable, build_table, classify, table_to_data
from salvage_verdicts import candidate_fingerprint
from salvage_v4 import CandidateRecord, guard_data, guarded_candidate
from v3_semantics import semantic_counts
from v4_contracts import STANDING, guard_frame, guard_group

ROOT: Final = Path(__file__).resolve().parents[2]
OUTPUT: Final = ROOT / 'docs/art/production3/reprocess'


class Source(TypedDict):
    record: CandidateRecord
    ledger: str
    reasons: list[str]


class Reprocessed(CandidateRecord):
    ancestor_id: str
    ancestor_ledger: str
    ancestor_ledger_sha256: str
    ancestor_record: CandidateRecord
    coverage_reasons: list[str]
    raw_sha256: str
    target_slot: str
    despeckle: NotRequired[dict[str, float | int | bool]]
    image_sha256: NotRequired[dict[str, str]]


class Inventory(TypedDict):
    categories: dict[str, list[str]]
    sources: list[Source]
    original_candidate_count: int
    p1_n3_candidates: list[str]
    p1_ledger: str
    p1_ledger_sha256: str


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inventory(root: Path) -> Inventory:
    """Retain all attempts of unfilled slots and every uninstalled physical fill."""
    old = root / 'docs/art/production2'
    final = json.loads((old / 'final_selection.json').read_text())
    installed = json.loads((old / 'installation_selection.json').read_text())
    installed_ids = {row.get('candidate_id') for row in installed['slots'].values()}
    categories: dict[str, list[str]] = defaultdict(list)
    for slot, row in final['slots'].items():
        if row['disposition'] == 'unfilled':
            categories[row['reason']].append(slot)
    missing = {slot for slots in categories.values() for slot in slots}
    sources: list[Source] = []
    n3: list[str] = []
    physical = 0
    ledgers = [old / 'candidates_round2.json', *sorted((old / 'fills').glob('*attempt_1.json'))]
    for ledger in ledgers:
        is_original = ledger.name == 'candidates_round2.json'
        rows = json.loads(ledger.read_text())['candidates']
        for row in rows:
            if row['source_kind'] != 'generated':
                continue
            relevant = row['slot_id'] in missing if is_original else row['id'] not in installed_ids
            if not relevant:
                continue
            physical += int(is_original)
            if row['sheet'] == 'N3':
                n3.append(row['id'])
                continue
            reasons = [final['slots'][row['slot_id']]['reason']] if is_original else ['fill_not_installed']
            if not is_original:
                verdict_path = old / f"fills/review/{row['source_sheet']}/round2/verdicts.json"
                verdicts = json.loads(verdict_path.read_text())['verdicts']
                verdict = next(v for v in verdicts if v['sprite_id'] == row['slot_id'])
                if verdict['verdict'] == 'reject':
                    reasons.append('fill_rejected:' + verdict['reason'])
                    categories['fill_rejected'].append(row['id'])
            sources.append({'record': row, 'ledger': str(ledger.relative_to(root)), 'reasons': reasons})
    p1 = root / 'docs/art/production3/decals/candidates.json'
    return {'categories': dict(categories), 'sources': sources, 'original_candidate_count': physical,
            'p1_n3_candidates': n3, 'p1_ledger': str(p1.relative_to(root)), 'p1_ledger_sha256': digest(p1)}


def frozen_tables(root: Path, output: Path) -> dict[str, ColorRegionTable]:
    """Freeze original approved reference pixels, maps and majority-colour evidence."""
    old = root / 'docs/art/production2'
    final = json.loads((old / 'final_selection.json').read_text())
    rows = {r['id']: r for r in json.loads((old / 'candidates_round2.json').read_text())['candidates']}
    tables: dict[str, ColorRegionTable] = {}
    evidence = {}
    folder = output / 'references'
    folder.mkdir(parents=True, exist_ok=True)
    for body, identity in final['references'].items():
        row = rows[identity]
        with Image.open(row['images']['sprite']) as source:
            native = source.convert('RGBA')
        approved = legacy_classify(native, body, view='S', activity='idle')
        tables[body] = build_table(native, approved, body)
        paths = {'native': folder / f'{body}.png', 'regions': folder / f'{body}_regions.png'}
        native.save(paths['native'], optimize=True)
        approved.save(paths['regions'], optimize=True)
        evidence[body] = {'candidate_id': identity, 'ancestor_sprite': row['images']['sprite'],
            'ancestor_sprite_sha256': digest(Path(row['images']['sprite'])),
            'paths': {k: str(v.relative_to(root)) for k, v in paths.items()},
            'sha256': {k: digest(v) for k, v in paths.items()}, 'table': table_to_data(tables[body])}
    (folder / 'tables.json').write_text(json.dumps({'tables': {body: table_to_data(table) for body, table in tables.items()}, 'references': evidence}, indent=2) + '\n')
    return tables


def reprocess(source: Source, tables: dict[str, ColorRegionTable], output: Path) -> Reprocessed:
    """Refit raw source at the prior factor; never repair by resampling old sprites."""
    ancestor = source['record']
    record: Reprocessed = deepcopy(ancestor)
    record.update(id=ancestor['id'] + ':p3', ancestor_id=ancestor['id'], source_candidate=ancestor['id'], ancestor_ledger=source['ledger'],
        ancestor_ledger_sha256=digest(ROOT / source['ledger']), ancestor_record=deepcopy(ancestor),
        coverage_reasons=source['reasons'], raw_sha256=digest(Path(ancestor['raw_path'])),
        target_slot=ancestor.get('target_slot', ancestor['slot_id']), source_kind='generated')
    folder = output / 'cells' / record['id'].replace('@', '_').replace(':', '_')
    folder.mkdir(parents=True, exist_ok=True)
    palette = [tuple(c['rgb']) for c in json.loads((ROOT / 'tools/art_pipeline/production_palette.json').read_text())['colors']]
    dark = palette[int(np.argmin(rgb_to_lab(np.array(palette, dtype=np.uint8))[:, 0]))]
    record['images'] = {}
    record['review_status'] = 'pending_independent_visual_review'
    record.pop('group_guards', None)
    record.pop('cross_guards', None)
    try:
        with Image.open(ancestor['raw_path']) as raw:
            crop = raw.convert('RGBA').crop(tuple(ancestor['raw_box']))
        fit = fit_cell_v4(crop, palette, float(ancestor['factor']), icon_canvas=ancestor['sheet'] == 'U1')
        cleaned = despeckle(fit.native)
        native = cleaned.image
        sprite = outline(native, dark)
        region = classify(sprite, tables[record['body']], standing=record['activity'] in STANDING) if record['body'] else legacy_classify(sprite)
        for kind, image in [('native', native), ('sprite', sprite), ('regions', region)]:
            path = folder / f'{kind}.png'
            image.save(path, optimize=True)
            record['images'][kind] = str(path.relative_to(ROOT))
        record['metrics'] = dict(fit.metrics)
        berry, fire = semantic_counts(native)
        after = np.asarray(native); before = np.asarray(fit.median_sample)
        opaque = after[..., 3] == 255; common = opaque & (before[..., 3] == 255)
        differences = np.linalg.norm(rgb_to_lab(before[common, :3])-rgb_to_lab(after[common, :3]), axis=1)
        post_delta = float(differences.mean()) if len(differences) else 100.0
        record['metrics'].update(native_after=list(native.size), outlined_canvas=list(sprite.size), berry_texels=berry, fire_texels=fire,
            original_fit_mean_delta_e=fit.metrics['mean_delta_e'], mean_delta_e=post_delta, common_opaque=int(common.sum()),
            keyed_out=int(((before[...,3]==255)&~opaque).sum()), binary_alpha=bool(np.isin(after[...,3],[0,255]).all()),
            palette_match_percent=100.*sum(tuple(map(int,c)) in set(palette) for c in after[opaque,:3])/max(1,int(opaque.sum())))
        record['despeckle'] = {'changed_count': cleaned.changed_count, 'opaque_count': cleaned.opaque_count,
            'change_fraction': cleaned.change_fraction, 'rejected': cleaned.rejected}
        report = guard_data(guard_frame(guarded_candidate(record)))
        record['guards'] = report
        record['conversion_error'] = None
        record['image_sha256'] = {k: digest(ROOT / v) for k, v in record['images'].items()}
    except (ValueError, IndexError) as error:
        record['conversion_error'] = f'{type(error).__name__}: {error}'
        record['guards'] = {'errors': ['conversion_failure'], 'warnings': [], 'passed': False}
    return record


def boards(records: list[Reprocessed], output: Path) -> dict[str, list[str]]:
    """Every physical candidate appears once, including failed guards, without icons."""
    groups: dict[str, list[Reprocessed]] = defaultdict(list)
    for record in records:
        groups[record['body'] or 'objects'].append(record)
    indexes: dict[str, list[str]] = {}
    for body, items in groups.items():
        indexes[body] = []
        for start in range(0, len(items), 12):
            chunk = items[start:start+12]
            sizes = []
            for row in chunk:
                if 'sprite' in row['images']:
                    with Image.open(ROOT / row['images']['sprite']) as image:
                        sizes.append(image.size)
            width = max([34, *[size[0] for size in sizes]])
            height = max([45, *[size[1] for size in sizes]])
            card_w, card_h = max(480, width*14+60), max(370, height*6+90)
            board = Image.new('RGB', (card_w*3, card_h*((len(chunk)+2)//3)), '#aaa69d')
            draw = ImageDraw.Draw(board)
            for offset, record in enumerate(chunk):
                x, y = (offset % 3)*card_w, (offset//3)*card_h
                draw.text((x+8,y+5), record['id'], fill='black')
                draw.text((x+8,y+20), f"{record['activity']} {record['view']} frame {record['frame']} | {record['guards']['errors']}", fill='black')
                draw.text((x+8,y+35), 'sprite 6x       region 6x       sprite 2x | ref 2x', fill='black')
                if 'sprite' not in record['images']:
                    draw.text((x+8,y+60), record['conversion_error'] or 'no image', fill='red')
                    continue
                with Image.open(ROOT / record['images']['sprite']) as source:
                    sprite = source.convert('RGBA')
                with Image.open(ROOT / record['images']['regions']) as source:
                    codes = np.array(source.convert('RGBA'))
                pixels = codes.copy(); pixels[..., :3] = DISPLAY[codes[..., 0]]
                visuals = [(sprite,6,x+8), (Image.fromarray(pixels),6,x+width*6+24), (sprite,2,x+width*12+40)]
                for visual, scale, left in visuals:
                    enlarged = visual.resize((visual.width*scale,visual.height*scale), Image.Resampling.NEAREST)
                    board.paste(enlarged,(left,y+55),enlarged)
                if body != 'objects':
                    with Image.open(output / f'references/{body}.png') as source:
                        reference = source.convert('RGBA').resize((source.width*2,source.height*2),Image.Resampling.NEAREST)
                    board.paste(reference,(x+width*12+40,y+height*2+85),reference)
            path = output / f'boards/{body}_{start//12+1:02d}.png'
            path.parent.mkdir(parents=True,exist_ok=True); board.save(path,optimize=True)
            indexes[body].append(str(path.relative_to(ROOT)))
    return indexes


def freeze_review_index(output: Path) -> None:
    """Bind all physical candidates and boards before independent review starts."""
    path = output / 'candidates.json'
    ledger = json.loads(path.read_text())
    ids = set(ledger['reprocessed_ids'])
    fingerprints = {r['id']: candidate_fingerprint(r) for r in ledger['candidates'] if r['id'] in ids}
    images = {str((ROOT / board).resolve()): digest(ROOT / board) for paths in ledger['boards'].values() for board in paths}
    index = {'schema_version': 1, 'candidates_sha256': digest(path),
             'candidate_fingerprints': fingerprints, 'images_sha256': images}
    (output / 'review_index.json').write_text(json.dumps(index, indent=2) + '\n')


def main() -> None:
    """Publish complete source coverage before making independent review boards."""
    if any((OUTPUT / name).exists() for name in ['candidates.json', 'review_index.json']):
        raise ValueError('Refusing to change previously frozen review evidence')
    OUTPUT.mkdir(parents=True, exist_ok=True)
    coverage = inventory(ROOT)
    (OUTPUT / 'coverage.json').write_text(json.dumps(coverage, indent=2) + '\n')
    tables = frozen_tables(ROOT, OUTPUT)
    records = []
    for index, source in enumerate(coverage['sources']):
        records.append(reprocess(source, tables, OUTPUT))
        print(f"{index+1}/{len(coverage['sources'])} {records[-1]['id']}", flush=True)
    groups = defaultdict(list)
    for record in records:
        if record['body'] and record['guards']['passed']:
            groups[(record['body'],record['sheet'],record['attempt'],record['view'],record['activity'])].append(record)
    for members in groups.values():
        report = guard_data(guard_group([guarded_candidate(r) for r in members]))
        for record in members:
            record['group_guards'] = report
    ledger = {'schema_version':1,'scale':{'adult_height':28,'child_height':21},'candidates':records+[r['ancestor_record'] for r in records],
        'acceptance_status':'pending_independent_visual_review','reprocessed_ids':[r['id'] for r in records],
        'reference_tables':'docs/art/production3/reprocess/references/tables.json',
        'references':{'adult_a':'V_a1_01@1','adult_b':'V_b1_01@2','child':'V_c1_01@1'},'boards':boards(records,OUTPUT)}
    (OUTPUT / 'candidates.json').write_text(json.dumps(ledger,indent=2,allow_nan=False)+'\n')
    (OUTPUT / 'board_index.json').write_text(json.dumps(ledger['boards'],indent=2)+'\n')
    freeze_review_index(OUTPUT)


if __name__ == '__main__':
    main()
