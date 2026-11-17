"""L2 same-model contact sheets; columns are BASE, AFTER, ILLUSTRATION TARGET."""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run (Blender supplies bpy/numpy; installed host python3 supplies Pillow):
# python3 tools/3d/blender/look2_contact.py --check
# python3 tools/v8/mux2_gate_lock.py -- blender -b --factory-startup -P tools/3d/blender/look2_contact.py -- OUT
from __future__ import annotations

import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import sys
from dataclasses import dataclass

ROOT = Path(__file__).resolve().parents[3]
DATA = Path('/Users/rexxa/work/art-img-data/look2')
BASE = DATA / 'baseline/repo'
TARGET = DATA.parent / 'look1/TARGET.png'
MANIFESTS = ('buildings_manifest.json', 'models/manifest.json',
             'models/asset_stations_manifest.json', 'models/w3d_stations_manifest.json')
STAGE_EXAMPLES = ('log_house', 'stone_house', 'mud_brick_house', 'cut_stone_house')
YARD_ITEMS = ('basket', 'firewood', 'log_pile_s', 'log_pile_m', 'log_pile_l',
              'pot', 'pot_stack', 'sack', 'tool', 'door')


@dataclass(frozen=True, slots=True)
class Model:
    identity: str
    family: str
    path: Path
    extent: float
    stage: str
    template: str


def models(root: Path) -> dict[str, Model]:
    """Parse checked-in fragments; later regenerated fragments carry current metadata."""
    result: dict[str, Model] = {}
    assets = root / 'assets/3d'
    paths = [assets / name for name in MANIFESTS]
    paths.extend(sorted(assets.glob('**/look2*manifest.json')))
    for path in paths:
        raw = json.loads(path.read_text())
        for identity, row in raw['models'].items():
            relative = Path(row['file'])
            assert not relative.is_absolute() and '..' not in relative.parts, identity
            extent = max(float(row['height']), *map(float, row['footprint']))
            assert math.isfinite(extent) and extent > 0, identity
            result[identity] = Model(identity, str(row['family']), assets / relative,
                                     extent, str(row.get('stage', '')), str(row.get('template', '')))
    return result


def selected(entries: dict[str, Model]) -> dict[str, list[Model]]:
    """Cover complete templates, era construction stages, stations and yard silhouettes."""
    groups: dict[str, list[Model]] = {}
    for identity, model in sorted(entries.items()):
        group = {'station': 'stations', 'prop': 'yard', 'yard': 'yard', 'wall': 'wall_stages'}.get(model.family, '')
        if identity in YARD_ITEMS:
            group = 'yard'
        if model.family == 'fence' and identity.endswith(('00', '03', '05', '0f')):
            group = 'fences'
        if model.family == 'building':
            group = 'buildings' if model.stage == 'complete' else 'stages' if model.template in STAGE_EXAMPLES else ''
        if group:
            groups.setdefault(group, []).append(model)
    return groups


def check(groups: dict[str, list[Model]], baseline: dict[str, Model]) -> None:
    """Fail closed on incomplete real coverage, invalid sources or an accidental reference swap."""
    complete = groups['buildings']
    assert len(complete) == 25 and len({m.template for m in complete}) == 25
    assert {m.stage for m in groups['stages']} >= {'foundation', 'half', 'frame'}
    assert {m.template for m in groups['stages']} == set(STAGE_EXAMPLES)
    assert len(groups['fences']) == 8
    for rows in groups.values():
        for model in rows:
            assert model.path.is_file(), model.path
            before = baseline.get(model.identity)
            if before:
                assert before.identity == model.identity and before.path.is_file(), model.identity
    print('LOOK2_CONTACT_CHECK ' + json.dumps({k: len(v) for k, v in groups.items()}))


def main() -> None:
    args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else sys.argv[1:]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('out', nargs='?', type=Path, default=DATA / 'contacts')
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--family', choices=('buildings', 'stages', 'stations', 'yard', 'fences', 'wall_stages'))
    options = parser.parse_args(args)
    baseline = models(BASE)
    groups = selected(models(ROOT))
    check(groups, baseline)
    if options.check:
        return
    assert TARGET.is_file(), 'L1 TARGET.png is required for final contact sheets'
    assert 'WORLDSIM_MUX2_LANE_FD' in os.environ, 'launch through mux2_gate_lock.py'
    descriptor = int(os.environ['WORLDSIM_MUX2_LANE_FD'])
    subprocess.run(['python3', str(ROOT / 'tools/v8/mux2_gate_lock.py'), '--check'],
                   pass_fds=(descriptor,), check=True)
    import bpy
    import numpy as np
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import render_assets_contact as contact
    out = options.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    receipts = []
    index_path = out / (f'{options.family}_index.json' if options.family else 'index.json')
    for family, rows in groups.items():
        if options.family and options.family != family:
            continue
        for start in range(0, len(rows), 2):
            batch = rows[start:start + 2]
            extent = max(max(m.extent, baseline[m.identity].extent if m.identity in baseline else 0) for m in batch)
            contact.CELL = max(512, math.ceil(extent * 1.7) * 64)
            assert contact.CELL <= 1024, f'oversized model needs a separate review: {extent}'
            contact.scene()
            bpy.context.scene.render.filepath = str(out / '_cell.png')
            cell = contact.CELL
            target_path = out / '_target.png'
            # Existing host Pillow composes the target without touching its source or invoking another renderer.
            subprocess.run(['python3', '-c', '''
from PIL import Image, ImageDraw, ImageOps
import sys
size=int(sys.argv[3]); target=Image.open(sys.argv[1]).convert('RGB')
canvas=Image.new('RGB',(size,size),(235,227,207))
fit=ImageOps.contain(target,(size,size-42))
canvas.paste(fit,((size-fit.width)//2,(size-42-fit.height)//2+42))
ImageDraw.Draw(canvas).text((14,14),'ILLUSTRATION TARGET - NOT GAME CAPTURE',fill=(38,31,22))
canvas.save(sys.argv[2])
''', str(TARGET), str(target_path), str(cell)], check=True)
            target_image = bpy.data.images.load(str(target_path))
            target_pixels = np.empty(cell * cell * 4, dtype=np.float32)
            target_image.pixels.foreach_get(target_pixels)
            target_pixels = target_pixels.reshape(cell, cell, 4)
            bpy.data.images.remove(target_image)
            pixels = np.ones((len(batch) * cell, 3 * cell, 4), dtype=np.float32)
            notes = []
            for index, model in enumerate(batch):
                after = contact.render_cell(model.path, 'AFTER ' + model.identity, 1)
                before_model = baseline.get(model.identity)
                before = contact.render_cell(before_model.path, 'BASE ' + model.identity, 1) if before_model else np.ones_like(after)
                row = len(batch) - index - 1
                pixels[row*cell:(row+1)*cell] = np.concatenate((before, after, target_pixels), axis=1)
                notes.append({'id': model.identity, 'baseline': str(before_model.path) if before_model else None,
                              'baseline_note': 'same model ID' if before_model else 'NEW: no baseline model; blank first column',
                              'candidate': str(model.path)})
            page = out / f'{family}_{start // 2 + 1:03}.png'
            image = bpy.data.images.new('look2_page', 3 * cell, len(batch) * cell, alpha=True)
            image.pixels.foreach_set(pixels.ravel())
            image.filepath_raw = str(page)
            image.file_format = 'PNG'
            image.save()
            bpy.data.images.remove(image)
            subprocess.run(['python3', '-c', '''
from PIL import Image, ImageDraw
import json, sys
path=sys.argv[1]; cell=int(sys.argv[2]); notes=json.loads(sys.argv[3])
image=Image.open(path); draw=ImageDraw.Draw(image)
for row, note in enumerate(notes):
    for col, title in enumerate(('BASE' if note['baseline'] else 'NEW - NO BASELINE MODEL','AFTER','ILLUSTRATION TARGET')):
        x=col*cell; y=row*cell
        draw.rectangle((x,y,x+cell,y+28),fill=(235,227,207,255))
        draw.text((x+12,y+8),title,fill=(38,31,22,255))
image.save(path)
''', str(page), str(cell), json.dumps(notes)], check=True)
            receipts.append({'sheet': page.name, 'cell': cell, 'models': notes})
            index_path.write_text(json.dumps({'target': str(TARGET), 'target_kind': 'illustration',
                'baseline': str(BASE), 'fov': 30, 'pitch': 50, 'px_per_metre': 64,
                'columns': ['same-ID baseline (blank if new)', 'candidate', 'illustration target'],
                'complete': False, 'sheets': receipts}, indent=2) + '\n')
    receipt = json.loads(index_path.read_text())
    receipt['complete'] = True
    index_path.write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'LOOK2_CONTACT_DONE sheets={len(receipts)}')


if __name__ == '__main__':
    main()
