# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
"""Production route contact sheets; absent looks stay visibly absent."""
from __future__ import annotations

import itertools
import json
from pathlib import Path

from PIL import Image, ImageDraw

from .build import ROOT, sha
from .prod1 import FACINGS, accepted, load, stage
from .samples import STATES, Person


def boards(route: Path, output: Path) -> None:
    """Render all 36 identities and all ten states of four accepted looks."""
    output.mkdir(parents=True, exist_ok=True)
    people = accepted(route)
    good = {p.look for p in people}
    identities = [Person(sex, age, top, hair, 'idle') for age, sex, top, hair in itertools.product(
        ('adult', 'child'), ('M', 'F'), ('loincloth', 'hide_wrap', 'fur_cloak'),
        ('short', 'long_tied', 'bun'))]
    sheet = Image.new('RGB', (224 * 6, 136 * 6), '#e0bd8e')
    draw = ImageDraw.Draw(sheet)
    for index, person in enumerate(identities):
        x, y = index % 6 * 224, index // 6 * 136
        draw.text((x + 2, y + 114), person.look, fill='#18110a')
        if person.look not in good:
            draw.text((x + 12, y + 48), 'MISSING / NOT ACCEPTED', fill='#18110a')
            continue
        for column, facing in enumerate(FACINGS):
            sheet.paste(stage(load(route, person.look, facing, 'idle'), 'meadow'), (x + column * 112, y))
    states = Image.new('RGB', (1120, 136 * 4), '#e0bd8e')
    for row, person in enumerate(people[:4]):
        for column, (facing, state) in enumerate(itertools.product(FACINGS, STATES)):
            states.paste(stage(load(route, person.look, facing, state), 'meadow'), (column * 112, row * 136))
        ImageDraw.Draw(states).text((2, row * 136 + 114), f'{person.look} | S idle/walk/work/carry/sleep | E same order', fill='#18110a')
    for name, image in (('route_i_sheet', sheet), ('route_i_states', states)):
        for zoom in (1, 2):
            image.resize((image.width * zoom, image.height * zoom), Image.Resampling.NEAREST).save(
                output / f'{name}_{zoom}x.png', optimize=True)


def comparison(route: Path, output: Path, records_path: Path) -> None:
    """Append PROD1 to historical A/B/P six cases; disclose absent N and looks."""
    historical = ROOT / 'art/character/boards/compare_ABP_2x.png'
    rows = json.loads(records_path.read_text())
    with Image.open(historical) as source:
        board = Image.new('RGB', (source.width + 224, source.height), '#e0bd8e')
        board.paste(source.convert('RGB'), (0, 0))
        left = source.width
    draw = ImageDraw.Draw(board)
    draw.text((left + 4, 8), 'PROD1; N uses S', fill='#18110a')
    records = []
    for index, row in enumerate(rows):
        attrs = row['pawn_attributes']
        facing = 'S' if attrs['facing'] == 'N' else attrs['facing']
        top = attrs['top'] if attrs['top'] != 'none' else 'loincloth'
        hair = attrs['hair'] if attrs['hair'] != 'none' else 'short'
        state = 'carry' if attrs['item'] == 'log_bundle' else attrs['state']
        look = f'adult_M_{top}_{hair}'
        source = route / look / facing / f'{state}.png'
        try:
            sprite = load(route, look, facing, state)
        except (FileNotFoundError, KeyError, ValueError):
            draw.text((left + 4, 48 + index * 244), 'MISSING / NOT ACCEPTED', fill='#18110a')
            records.append({'case': index + 1, 'status': 'MISSING', 'look': look})
            continue
        board.paste(stage(sprite, 'meadow').resize((224, 224), Image.Resampling.NEAREST), (left, 32 + index * 244))
        note = 'N absent: S proxy' if attrs['facing'] == 'N' else 'Same facing/top/hair'
        draw.text((left + 4, 256 + index * 244), note, fill='#18110a')
        records.append({'case': index + 1, 'status': 'ACCEPTED', 'source': str(source), 'sha256': sha(source), 'note': note})
    output.mkdir(parents=True, exist_ok=True)
    board.save(output / 'compare_ABPT2_2x.png', optimize=True)
    (output / 'compare_ABPT2_provenance.json').write_text(json.dumps({
        'historical_board_sha256': sha(historical), 'historical_records_sha256': sha(records_path),
        'limitation': 'PROD1 adult male; historical cases not sex/age matched', 'rows': records}, indent=2) + '\n')
