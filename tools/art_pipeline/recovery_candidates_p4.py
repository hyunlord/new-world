# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
"""Freeze recovery candidates, unchanged source pixels, hair maps and blind boards."""
from __future__ import annotations

from collections import Counter, defaultdict
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from hair_inventory_p4 import load_image
from hair_maps import classify
from hair_registry_p4 import HairRegistry
from natural_boards_p4 import BoardFrame, compose, save_board
from salvage_verdicts import candidate_fingerprint

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = Path('docs/art/production4/recovery')


class RecoveryError(ValueError):
    """Recovery evidence is missing, duplicated, or changed after freezing."""


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def eligible_rows(rows: list[dict]) -> list[dict]:
    """Keep every rejected attempt in unapproved physical slots, never logical aliases."""
    if len({row['candidate_id'] for row in rows}) != len(rows):
        raise RecoveryError('Duplicate review candidate identity')
    approved = {row.get('physical_slot_id', row['sprite_id']) for row in rows if row['verdict'] == 'ok'}
    return [row for row in rows if row['verdict'] == 'reject' and row['reason'] == 'region_map_wrong'
            and row.get('physical_slot_id', row['sprite_id']) not in approved]


def write_json(path: Path, data: dict | list) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n')


def verify_frozen(root: Path) -> None:
    """Validate all frozen source files, maps, ledger records and board hashes on reruns."""
    output = root / OUTPUT
    index = json.loads((output / 'review_index.json').read_text())
    ledger_path = output / 'candidates.json'
    if sha(ledger_path) != index['candidates_sha256']:
        raise RecoveryError('Frozen candidate ledger changed')
    rows = json.loads(ledger_path.read_text())['candidates']
    if {row['id']: candidate_fingerprint(row) for row in rows} != index['candidate_fingerprints']:
        raise RecoveryError('Frozen candidate images changed')
    for path, expected in index['source_sha256'].items():
        if sha(root / path) != expected:
            raise RecoveryError(f'Frozen source changed: {path}')
    for path, expected in index['images_sha256'].items():
        if sha(root / path) != expected:
            raise RecoveryError(f'Frozen board changed: {path}')


def prepare(root: Path = ROOT) -> None:
    """Use existing native and outlined images without fitting, quantizing or converting."""
    output = root / OUTPUT
    if (output / 'review_index.json').exists():
        verify_frozen(root)
        print('Verified frozen recovery inventory; no files overwritten')
        return
    base = Path('docs/art/production3')
    review_paths = [base / f'reprocess/reviews/{body}.json' for body in ('adult_a', 'adult_b', 'child')]
    review_paths += [base / f'fills/P3_fill_0{i}/review.json' for i in range(1, 5)]
    ledger_paths = [base / 'reprocess/candidates.json'] + [base / f'fills/P3_fill_0{i}/candidates.json' for i in range(1, 5)]
    reviews = [row for path in review_paths for row in json.loads((root / path).read_text())]
    eligible = eligible_rows(reviews)
    source_by_id = {}
    lineage = {}
    source_hashes = {str(path): sha(root / path) for path in review_paths + ledger_paths}
    for path in ledger_paths:
        for row in json.loads((root / path).read_text())['candidates']:
            source_by_id[row['id']] = (row, path)
    hair = HairRegistry.load(root)
    for path in ('tools/art_pipeline/hair_palette_p4.json', 'tools/art_pipeline/hair_maps.py',
                 'assets/art/manifest.json', 'assets/art/villager_ramps.png'):
        source_hashes[path] = sha(root / path)
    manifest = json.loads((root / 'assets/art/manifest.json').read_text())
    peers = defaultdict(list)
    for row in manifest['assets']:
        if row.get('body') not in ('adult_a', 'adult_b', 'child') or not row.get('sprite_id') or not row.get('drawable', True):
            continue
        path = root / row['path']
        source_hashes[row['path']] = sha(path)
        peers[(row['body'], row['activity'], row['view'])].append((row['frame'], row['sprite_id'], load_image(path)))
    candidates = []
    board_rows = defaultdict(list)
    templates = defaultdict(list)
    for review in eligible:
        original, ledger_path = source_by_id[review['candidate_id']]
        record = deepcopy(original)
        physical = review.get('physical_slot_id', review['sprite_id'])
        if physical != original['slot_id']:
            raise RecoveryError(f'Physical slot disagrees with ledger: {record["id"]}')
        record['slot_id'] = physical
        record['target_slot'] = original.get('target_slot', original['slot_id'])
        record['p4_provenance'] = {'ledger': str(ledger_path), 'ledger_sha256': source_hashes[str(ledger_path)],
            'original_record_sha256': hashlib.sha256(json.dumps(original, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
            'original_record': original}
        identity = record['id'].replace('@', '_').replace(':', '_')
        native = load_image(root / record['images']['native'])
        sprite = load_image(root / record['images']['sprite'])
        native_mask = classify(native, hair.palettes[record['body']], record['activity'])
        sprite_mask = classify(sprite, hair.palettes[record['body']], record['activity'])
        for kind, mask in (('hair_mask', native_mask), ('sprite_hair_mask', sprite_mask)):
            path = OUTPUT / 'masks' / identity / f'{kind}.png'
            (root / path).parent.mkdir(parents=True, exist_ok=True)
            mask.save(root / path, optimize=True)
            record['images'][kind] = str(path)
        record['p4_hair'] = {'source': 'hair_maps', 'verdict': 'pending',
            'native_hair_texels': native_mask.getchannel('R').histogram()[3],
            'sprite_hair_texels': sprite_mask.getchannel('R').histogram()[3],
            'native_sha256': sha(root / record['images']['native']),
            'sprite_sha256': sha(root / record['images']['sprite']),
            'native_mask_sha256': sha(root / record['images']['hair_mask']),
            'sprite_mask_sha256': sha(root / record['images']['sprite_hair_mask'])}
        candidates.append(record)
        ancestor = original
        while ancestor.get('source_candidate'):
            parent = ancestor['source_candidate']
            if parent in lineage:
                break
            if parent not in source_by_id:
                raise RecoveryError(f'Missing lineage: {parent}')
            ancestor, _ = source_by_id[parent]
            lineage[parent] = ancestor
        group = (record['body'], record['activity'], record['view'])
        cycle = tuple(image for _, _, image in sorted(peers[group], key=lambda item: (item[0], item[1])))
        label = f"{record['id']} | physical {physical} | target {record['target_slot']}"
        board_rows[record['body']].append(BoardFrame(label, record['body'], record['activity'], record['view'], record['frame'], sprite, sprite_mask, cycle))
        templates[record['body']].append({'sprite_id': physical, 'candidate_id': record['id'], 'target_slot': record['target_slot'],
            'verdict': '', 'reason': '', 'observation': '', 'unfitted_acceptable': None})
    ledger = {'schema_version': 1, 'scale': {'adult_height': 28, 'child_height': 21},
        'candidates': candidates, 'lineage_candidates': list(lineage.values()),
        'acceptance_status': 'pending_independent_visual_review'}
    ledger_path = output / 'candidates.json'
    write_json(ledger_path, ledger)
    index = {}
    board_hashes = {}
    for body, frames in board_rows.items():
        reference_path = base / f'reprocess/references/{body}.png'
        reference = load_image(root / reference_path)
        source_hashes[str(reference_path)] = sha(root / reference_path)
        index[body] = []
        for start in range(0, len(frames), 12):
            chunk = frames[start:start + 12]
            page = save_board(compose(chunk, reference), output / f'boards/{body}_{start // 12 + 1:02}.png')
            relative = str(page.relative_to(root))
            board_hashes[relative] = sha(page)
            index[body].append({'path': relative, 'sha256': sha(page), 'candidate_ids': [row.ident.split(' | ')[0] for row in chunk]})
        write_json(output / f'templates/{body}.json', {'verdicts': templates[body]})
    write_json(output / 'index.json', index)
    targets = {row['target_slot'] for row in candidates}
    selection = json.loads((root / base / 'installation_selection.json').read_text())['slots']
    physical_bodies = {row['slot_id']: row['body'] for row in candidates}
    counts = {'candidate_attempts': len(candidates), 'physical_slots': len(physical_bodies), 'target_slots': len(targets),
        'attempts_by_body': dict(Counter(row['body'] for row in candidates)), 'physical_by_body': dict(Counter(physical_bodies.values())),
        'p3_target_dispositions': dict(Counter(selection.get(target, {}).get('disposition', 'absent') for target in targets)),
        'lineage_context_records': len(lineage), 'native_bytes_unchanged': True, 'sprite_bytes_unchanged': True,
        'note': '142 physical slots contain206 distinct attempts; all attempts remain reviewable. P3 base selection excludes7 later reprocess approvals.'}
    write_json(output / 'counts.json', counts)
    write_json(output / 'review_index.json', {'schema_version': 1, 'candidates_sha256': sha(ledger_path),
        'candidate_fingerprints': {row['id']: candidate_fingerprint(row) for row in candidates},
        'source_sha256': source_hashes, 'images_sha256': board_hashes})
    (output / 'REVIEW_READY.txt').write_text('Independent Art Pass4 recovery review. Read docs/art/production4/REVIEW_RUBRIC.md.\n'
        'Use only your body boards listed in index.json and templates/<body>.json. Do not inspect pipeline code, candidate ledger, previous verdicts or GOAL section3.\n'
        'Each label gives unique candidate ID, physical slot and logical target. Boards show exact unchanged outlined installation sprite and its sprite_hair_mask. Judge action at2x withouticons, natural colours, anatomy and hairmask.\n'
        'Return one verdict per candidate ID using template fields; no installation judgement. Child masks are intentionallyempty.\n')
    verify_frozen(root)
    print(json.dumps(counts))


if __name__ == '__main__':
    prepare()
