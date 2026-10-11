# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/key_candidates_p5.py SHEET RAW ATTEMPT REFERENCE_MANIFEST
"""Freeze key-quantised fill candidates before independent review and installation."""
from __future__ import annotations

from collections import defaultdict
import hashlib
import json
from pathlib import Path
import sys
from typing import Final

import numpy as np
from PIL import Image

from fill_catalog import load_catalog
from fill_v4 import process
from key_boards_p5 import review_boards
from hair_masks_v5 import Reference, remask
from key_hair_p5 import KEYS, convert_key, delta_e, outlined_mask, protected_despeckle
from key_plan_p5 import OUTPUT, ROOT
from palette_tools import rgb_to_lab
from pixel_normalize import outline
from pixel_normalize_v4 import fit_cell_v4
from salvage_v4 import CandidateRecord, guard_data, guarded_candidate
from salvage_verdicts import candidate_fingerprint
from v3_semantics import semantic_counts
from v4_contracts import guard_frame, guard_group

KEY_ERRORS: Final = {'key_hair_missing', 'key_hair_outside_head', 'key_fringe'}


def run(sheet: str, raw: Path, attempt: int, reference_manifest: Path, *, natural_fallback: bool = False) -> Path:
    """Keep every physical cell and raw crop; never install or accept a candidate."""
    catalog = load_catalog(OUTPUT / 'fill_catalog.json')
    spec = next(s for s in catalog['sheets'] if s['id'] == sheet)
    reference = json.loads(reference_manifest.read_text())
    body = spec['body']
    body_reference = reference['bodies'][body]
    use_key = bool(spec['key_hair'] and body_reference['key_approved'] and not natural_fallback)
    if spec['key_hair'] and not use_key and not natural_fallback:
        message = 'Key-sheet processing requires approved key reference; use explicit natural fallback catalog'
        raise ValueError(message)
    if use_key:
        for field, digest_field in [('key_path', 'key_sha256'), ('review_path', 'review_sha256')]:
            bound_path = ROOT / body_reference[field]
            if hashlib.sha256(bound_path.read_bytes()).hexdigest() != body_reference[digest_field]:
                message = f'Approved reference evidence changed: {bound_path}'
                raise ValueError(message)
    with Image.open(ROOT / body_reference['natural_path']) as opened:
        if hashlib.sha256(opened.convert('RGBA').tobytes()).hexdigest() != body_reference['natural_rgba_sha256']:
            message = 'Natural identity reference changed'
            raise ValueError(message)
    palette = [tuple(row['rgb']) for row in json.loads((ROOT / 'tools/art_pipeline/production_palette.json').read_text())['colors']]
    if set(palette) & set(KEYS):
        message = 'Reserved hair key is in production palette'
        raise ValueError(message)
    references: list[Reference] = []
    if natural_fallback and body.startswith('adult'):
        pool = json.loads((ROOT / reference['pool_path']).read_text())
        for row in pool['references']:
            with Image.open(ROOT / row['sprite_path']) as opened, Image.open(ROOT / row['mask_path']) as source:
                references.append(Reference(row['id'], row['body'], row['view'], row['activity'], opened.convert('RGBA'), source.convert('RGBA')))
    folder = OUTPUT / 'keys/processed' / f'{sheet}_attempt_{attempt}'
    ledger_path = folder / 'candidates.json'
    if ledger_path.exists():
        message = f'Refusing to overwrite frozen candidate ledger: {ledger_path}'
        raise ValueError(message)
    folder.mkdir(parents=True, exist_ok=True)
    ledger = process(ROOT, catalog, sheet, raw, attempt, folder / 'source_candidates.json', folder / 'cells')
    records = [r for r in ledger['candidates'] if r['source_kind'] == 'generated' and r['sheet'] == sheet]
    dark = palette[int(np.argmin(rgb_to_lab(np.asarray(palette, dtype=np.uint8))[:, 0]))]
    raw_cells = raw.parent / f'cells_attempt_{attempt}'
    raw_cells.mkdir(parents=True, exist_ok=True)
    for row in records:
        old_raw = Path(row['images']['raw'])
        destination = raw_cells / f'{row["slot_id"]}.png'
        old_raw.rename(destination); row['images']['raw'] = str(destination)
    (folder / 'source_candidates.json').write_text(json.dumps(ledger, indent=2, allow_nan=False) + '\n')
    for row in records:
        destination = Path(row['images']['raw'])
        if row['factor'] is None:
            continue
        cell_folder = folder / 'cells' / row['id'].replace('@', '_')
        try:
            with Image.open(destination) as opened:
                fit = fit_cell_v4(opened, palette + list(KEYS) if use_key else palette, row['factor'])
            errors: list[str] = []
            extra = {}
            if use_key:
                colors = body_reference['browns']
                browns = (tuple(colors[0]), tuple(colors[1]), tuple(colors[2]))
                if not set(browns) <= set(palette):
                    message = 'Reference hair browns must be production colours'
                    raise ValueError(message)
                result = convert_key(fit.native, fit.median_sample, browns, asleep=row['activity'] == 'asleep')
                native, mask, cleaned = result.natural, result.mask, result.cleaned
                ranking_error = result.delta_before
                errors.extend(result.errors)
                extra = {'mean_delta_e_key_quantised': result.delta_before,
                         'mean_delta_e_after_substitution': result.delta_after, 'key_texels': result.key_count,
                         'key_outside_component': result.outside_count, 'key_fringe_texels': result.fringe_count,
                         'mask_method': 'key_colour'}
                row['hair_mask_source'] = 'hair_key'
                key_path = cell_folder / 'key_quantised.png'; result.keyed.save(key_path)
                row['images']['key_quantised'] = str(key_path)
            else:
                cleaned = protected_despeckle(fit.native)
                native = cleaned.image
                ranking_error = delta_e(fit.median_sample, native)
                pixels = np.zeros_like(np.asarray(native)); pixels[..., 3] = np.asarray(native)[..., 3]
                mask = Image.fromarray(pixels)
            sprite = outline(native, dark)
            sprite_mask = outlined_mask(mask, sprite)
            if natural_fallback and body.startswith('adult'):
                transferred = remask(sprite, native, body, row['view'], row['activity'], references)
                if transferred.mask is None:
                    errors.append('template_hair_unavailable')
                else:
                    sprite_mask = transferred.mask
                    mask = sprite_mask.crop((1, 1, native.width + 1, native.height + 1))
                    row['hair_mask_source'] = 'template_v5'
            images = {'native': native, 'sprite': sprite, 'hair_mask': mask, 'sprite_hair_mask': sprite_mask}
            for kind, image in images.items():
                path = cell_folder / f'{kind}.png'; image.save(path, optimize=True); row['images'][kind] = str(path)
            berry, fire = semantic_counts(native)
            warning = [w for w in fit.metrics['warnings'] if w != 'colour_drift']
            if ranking_error > 6:
                warning.append('colour_drift')
            row['metrics'] = dict(fit.metrics, **extra, mean_delta_e=ranking_error, berry_texels=berry,
                                  fire_texels=fire, palette_match_percent=100., warnings=warning,
                                  despeckle_change_fraction=cleaned.change_fraction)
            row['conversion_error'] = None
            row['guards'] = {'passed': not errors, 'errors': list(errors), 'warnings': []}
            row['guards'] = guard_data(guard_frame(guarded_candidate(row)))
            row['guards']['errors'] = list(dict.fromkeys(row['guards']['errors'] + errors))
            row['guards']['warnings'] = list(dict.fromkeys(row['guards']['warnings'] + warning))
            row['guards']['passed'] = not row['guards']['errors']
            row['review_status'] = 'pending_independent_visual_review'
            row['conversion_error'] = None
        except (ValueError, IndexError) as error:
            row['conversion_error'] = str(error)
            row['guards'] = {'passed': False, 'errors': ['conversion_failure'], 'warnings': []}
    groups: defaultdict[tuple[str, str], list[CandidateRecord]] = defaultdict(list)
    for row in records:
        groups[row['activity'], row['view']].append(row)
    for group in groups.values():
        usable = [guarded_candidate(r) for r in group if r['guards']['passed']]
        report = guard_data(guard_group(usable))
        for row in group:
            row['group_guards'] = report
    with Image.open(ROOT / body_reference['natural_path']) as idle, Image.open(ROOT / 'assets/art/villager_ramps.png') as ramps:
        boards = review_boards(records, OUTPUT / 'keys/boards' / f'{sheet}_attempt_{attempt}', idle.convert('RGBA'), ramps)
    ledger.update(candidates=records, selected={}, groups=[], boards=boards,
                  reference_manifest=str(reference_manifest), reference_sha256=hashlib.sha256(reference_manifest.read_bytes()).hexdigest(),
                  key_hair=use_key, key_error_free_cells=sum(not KEY_ERRORS.intersection(r['guards']['errors']) and r['conversion_error'] is None for r in records),
                  candidate_fingerprints={r['id']: candidate_fingerprint(r) for r in records})
    ledger_path.write_text(json.dumps(ledger, indent=2, allow_nan=False) + '\n')
    index = {'candidates_sha256': hashlib.sha256(ledger_path.read_bytes()).hexdigest(),
             'candidate_fingerprints': ledger['candidate_fingerprints'],
             'images_sha256': {page['path']: page['sha256'] for page in boards},
             'source_sha256': {str(raw): hashlib.sha256(raw.read_bytes()).hexdigest()}}
    (folder / 'review_index.json').write_text(json.dumps(index, indent=2) + '\n')
    return ledger_path


if __name__ == '__main__':
    if len(sys.argv) not in (5, 6):
        raise SystemExit('Usage: key_candidates_p5.py SHEET RAW ATTEMPT REFERENCE_MANIFEST [--natural]')
    print(run(sys.argv[1], Path(sys.argv[2]).resolve(), int(sys.argv[3]), Path(sys.argv[4]), natural_fallback='--natural' in sys.argv[5:]))
