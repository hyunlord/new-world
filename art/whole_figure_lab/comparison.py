# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.whole_figure_lab.comparison ROUTE OUTPUT
"""Extend frozen six-row ABP comparison without pretending absent N is matched."""
from __future__ import annotations

import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw

from .build import ROOT, load, sha, source_directory, stage


def comparison(route: Path, output: Path, diagnostic: bool = False) -> None:
    """Retain historical pixels exactly; disclose I-only condition differences."""
    historical = ROOT / 'art/character/boards/compare_ABP_2x.png'
    records_path = Path('/Users/rexxa/work/arttrial4-data/round0/lab/boards/compare_provenance.json')
    rows = json.loads(records_path.read_text())
    with Image.open(historical) as source:
        board = Image.new('RGB', (832, source.height), '#e0bd8e')
        board.paste(source.convert('RGB'), (0, 0))
    draw = ImageDraw.Draw(board)
    draw.text((628, 8), 'DIAGNOSTIC NOT ACCEPTED' if diagnostic else 'I-1 route; differences below', fill='#18110a')
    records = []
    for index, row in enumerate(rows):
        attrs = row['pawn_attributes']
        facing = attrs['facing']
        top = attrs['top'] if attrs['top'] != 'none' else 'loincloth'
        hair = attrs['hair'] if attrs['hair'] != 'none' else 'short'
        state = 'carry' if attrs['item'] == 'log_bundle' else attrs['state']
        label = 'I: same facing/top/hair'
        if facing == 'N':
            facing = 'S'
            label = 'N absent; S alternative only'
        if attrs['state'] == 'sleep':
            label = 'I sleep: loincloth/short'
        look = f'adult_M_{top}_{hair}'
        sprite = load(route, look, facing, state, diagnostic)
        board.paste(stage(sprite, 'meadow').resize((208, 224), Image.Resampling.NEAREST), (624, 32 + index * 244))
        draw.text((628, 256 + index * 244), label, fill='#18110a')
        path = source_directory(route, look, facing, diagnostic) / f'{state}.png'
        records.append({'number': index + 1, 'historical_attributes': attrs,
                        'I_status': json.loads(path.with_suffix('.json').read_text()), 'I_source': str(path), 'I_sha256': sha(path), 'comparison_note': label,
                        'sex_age': 'I adult male; historical ABP not sex-age matched'})
    output.mkdir(parents=True, exist_ok=True)
    target = output / 'compare_ABPT_2x.png'
    board.save(target, optimize=True)
    (output / 'compare_provenance.json').write_text(json.dumps({'dataset_status': 'diagnostic_not_accepted' if diagnostic else 'STRUCTURAL_PASS_REVIEW_PENDING', 'historical_board': str(historical),
        'historical_sha256': sha(historical), 'historical_records_sha256': sha(records_path),
        'rows': records, 'board_sha256': sha(target)}, indent=2) + '\n')
    if target.stat().st_size > 1_500_000:
        message = f'Comparison exceeds file ceiling: {target}'
        raise ValueError(message)


if __name__ == '__main__':
    arguments = sys.argv[1:]
    diagnostic = '--diagnostic' in arguments
    if diagnostic:
        arguments.remove('--diagnostic')
    if len(arguments) != 2:
        raise SystemExit('Usage: python3 -m art.whole_figure_lab.comparison ROUTE_OR_SELECTION_JSON OUTPUT [--diagnostic]')
    comparison(Path(arguments[0]), Path(arguments[1]), diagnostic)
