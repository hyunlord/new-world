# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pawn_lab.boards ROUTE_P LAB_OUTPUT
"""Exhaustive pawn boards and same-six historical A/B/P meadow comparison."""
from __future__ import annotations
import hashlib
import json
import shutil
import sys
from itertools import product
from pathlib import Path
from PIL import Image, ImageDraw
from art.pipeline3.layer_schema import load_layer
from art.pipeline4.pawn_gen import PawnLook, compose_layers
from .build import ROOT, stage

TOPS = ('none', 'hide_wrap', 'fur_cloak', 'woven_tunic')
HAIRS = ('none', 'short', 'long_tied', 'bun', 'braid', 'grey_thin')
STATES = ('idle', 'walk_a', 'walk_b', 'work_a', 'work_b', 'carry', 'sleep')


def sheet(route: Path, looks: list[PawnLook], target: Path) -> None:
    columns = 6 if len(looks) == 72 else 7
    board = Image.new('RGB', (columns * 104, ((len(looks) + columns - 1) // columns) * 128 + 24), '#e0bd8e')
    draw = ImageDraw.Draw(board)
    draw.text((4, 4), (target.parent.parent / 'DATASET_STATUS.txt').read_text().splitlines()[0], fill='#18110a')
    for index, appearance in enumerate(looks):
        x, y = index % columns * 104, index // columns * 128 + 24
        board.paste(stage(compose_layers(route, appearance), 'meadow'), (x, y))
        label = f'{appearance.facing} {appearance.top[:4]}/{appearance.hair_style[:5]}' if len(looks) == 72 else f'{appearance.facing} {appearance.state}'
        draw.text((x + 2, y + 112), label, fill='#18110a')
    for zoom in (1, 2):
        destination = target.with_name(f'{target.stem}_{zoom}x.png')
        board.resize((board.width * zoom, board.height * zoom), Image.Resampling.NEAREST).save(destination, optimize=True)


def comparison(route: Path, output: Path) -> None:
    historical = Path('/Users/rexxa/work/arttrial3-data')
    a_root = ROOT / 'art/character/route_a'
    b_root = historical / 'b/render'
    a_rows = json.loads((historical / 'gifs-final-source/answer_key.private.json').read_text())
    b_rows = json.loads((historical / 'b/lab/answer_key.private.json').read_text())
    records = []
    manifests = [a_root / 'manifest.json', b_root / 'manifest.json']
    indexes = [{layer.id: layer for layer in [load_layer(manifest.parent / path)
               for path in json.loads(manifest.read_text())['layers']]} for manifest in manifests]
    board = Image.new('RGB', (624, 6 * 244 + 32), '#e0bd8e')
    draw = ImageDraw.Draw(board)
    draw.text((8, 8), 'A/B rejected; P: ' + (output.parent / 'DATASET_STATUS.txt').read_text().splitlines()[0] + '; calm meadow 2x', fill='#18110a')
    for number in range(1, 7):
        a = next(row for row in a_rows if row['number'] == number and row['zoom'] == 1)
        b = next(row for row in b_rows if row['number'] == number and row['zoom'] == 1)
        sources = []
        for column, (row, index) in enumerate(zip((a, b), indexes, strict=True)):
            ids = row.get('ids', row.get('layer_ids'))
            selected = [index[key] for key in ids]
            if column == 0 and row['activity'] in ('walk', 'work', 'carry'):
                frame = 'work_a' if row['activity'] == 'work' else 'walk_a'
                base = selected[0]
                selected[0] = next(layer for layer in index.values() if layer.kind == 'body' and layer.body_type == base.body_type and layer.direction == base.direction and layer.frame == frame)
            with Image.open(selected[0].image) as original:
                pawn = Image.new('RGBA', original.size)
            for layer in selected:
                with Image.open(layer.image) as original:
                    offset = (0, 0)
                    if column == 0 and layer.kind in ('item', 'carry'):
                        anchor = layer.grip if layer.kind == 'item' else layer.shoulder
                        target = ((44, 64) if selected[0].direction == 'E' else (50, 66) if selected[0].direction == 'S' else (14, 66)) if layer.kind == 'item' else (32, 52)
                        if anchor is None:
                            raise ValueError(f'Historical anchor missing: {layer.id}')
                        offset = (target[0] - anchor[0], target[1] - anchor[1])
                    pawn.alpha_composite(original.convert('RGBA'), offset)
                sources.append({'path': str(layer.image), 'sha256': hashlib.sha256(layer.image.read_bytes()).hexdigest()})
            board.paste(stage(pawn, 'meadow').resize((208, 224), Image.Resampling.NEAREST), (column * 208, 32 + (number - 1) * 244))
        top = b['top']
        hair = b['hair']
        item = {'axe': 'stone_axe', 'logs': 'log_bundle', 'none': 'none'}[b['carried']]
        state = b['frames'][0]
        appearance = PawnLook(top=top, hair_style=hair, item=item, facing=b['direction'], state=state)
        board.paste(stage(compose_layers(route, appearance), 'meadow').resize((208, 224), Image.Resampling.NEAREST), (416, 32 + (number - 1) * 244))
        draw.text((4, 32 + (number - 1) * 244 + 224), f'{number:02d} {b["activity"]}: A | B | P', fill='#18110a')
        records.append({'number': number, 'original_sources': sources,
                        'pawn_attributes': {'top': top, 'hair': hair, 'item': item, 'facing': b['direction'], 'state': state}})
    board.save(output / 'compare_ABP_2x.png', optimize=True)
    (output / 'compare_provenance.json').write_text(json.dumps(records, indent=2) + '\n')


def build(route: Path, lab: Path) -> None:
    provenance = json.loads((lab / 'provenance.json').read_text())
    for filename, digest in provenance['input_hashes'].items():
        if hashlib.sha256(Path(filename).read_bytes()).hexdigest() != digest:
            raise ValueError(f'Pawn inputs changed since lab build: {filename}')
    output = lab / 'boards'
    output.mkdir(exist_ok=True)
    sheet(route, [PawnLook(top=top, hair_style=hair, facing=facing)
                 for facing, top, hair in product(('S', 'E', 'N'), TOPS, HAIRS)], output / 'pawn_sheet.png')
    sheet(route, [PawnLook(state=state, facing=facing, item='stone_axe' if state.startswith('work') else 'log_bundle' if state == 'carry' else 'none')
                 for facing, state in product(('S', 'E', 'N'), STATES)], output / 'pawn_states.png')
    comparison(route, output)
    for source in output.glob('*.png'):
        if source.stat().st_size > 1_500_000:
            raise ValueError(f'Committed board exceeds file ceiling: {source}')
        shutil.copyfile(source, ROOT / 'art/character/boards' / source.name)


if __name__ == '__main__':
    build(Path(sys.argv[1]), Path(sys.argv[2]))
