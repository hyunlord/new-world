"""Rebuild selected lossless sheets without resizing native sprites."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from native_grid import recover_cell
from palette_tools import load_palette, save_palette, rgb_to_lab
from palette_v2 import build_master, quantize
from pixel_normalize import outline
from semantics_v2 import evaluate

ROOT = Path(__file__).resolve().parents[2]


def limits(name: str) -> tuple[str, tuple[int, int]]:
    if 'child' in name:
        return 'child', (14, 22)
    if 'villager' in name:
        return 'villager', (18, 28)
    if any(word in name for word in ('lean_to', 'storage_pit', 'shelter_frame')):
        return 'building', (34, 30)
    if 'campfire' in name:
        return 'campfire', (24, 26)
    if 'tree' in name:
        return 'tall', (24, 34)
    return 'small object', (24, 22)


def contact_sheet(directory: Path, raw: dict[str, Image.Image], natives: dict[str, Image.Image], finals: dict[str, Image.Image]) -> None:
    """Raw generator cell alongside pass-1 and pass-2 sprites at 8x."""
    max_w = max(max(im.width for im in finals.values()) * 8, 320)
    max_h = max(max(im.height for im in finals.values()) * 8, 320)
    width, height = max_w * 3 + 32, max_h + 44
    board = Image.new('RGB', (width * 4, height * 4), '#25262b')
    draw = ImageDraw.Draw(board)
    for index, (name, final) in enumerate(finals.items()):
        x, y = index % 4 * width, index // 4 * height
        old = Image.open(directory / 'sprites' / f'{name}.png').convert('RGBA')
        raw_preview = raw[name].copy()
        raw_preview.thumbnail((max_w, max_h), Image.Resampling.NEAREST)
        images = (raw_preview, old.resize((old.width * 8, old.height * 8), Image.Resampling.NEAREST),
                  final.resize((final.width * 8, final.height * 8), Image.Resampling.NEAREST))
        draw.text((x + 4, y + 3), f'{name} | source / pass 1 8x / pass 2 8x', fill='white')
        for col, image in enumerate(images):
            board.paste(image, (x + col * max_w + 8, y + 32 + max_h - image.height), image)
    board.save(directory / 'sprites_v2_contact.png')


def run(direction: str, raw_root: Path, art_root: Path) -> None:
    directory = art_root / direction
    filename = f'{direction}_sheet' + ('' if direction == 'D1' else '_retry1') + '.png'
    source = raw_root / direction / filename
    fallback = not source.exists()
    if fallback:
        source = directory / f'{direction}_sheet.jpg'
    sheet = Image.open(source).convert('RGBA')
    spec = json.loads((ROOT / 'tools/art_pipeline/sheet_v1.json').read_text())
    natives: dict[str, Image.Image] = {}
    step3: dict[str, Image.Image] = {}
    raw: dict[str, Image.Image] = {}
    entries: list[dict[str, object]] = []
    for item in spec['cells']:
        index = item['index'] - 1
        x, y = index % 4, index // 4
        name = f"{index + 1:02d}_{item['name']}"
        cell = sheet.crop((x * sheet.width // 4, y * sheet.height // 4, (x + 1) * sheet.width // 4, (y + 1) * sheet.height // 4))
        native, metadata = recover_cell(cell)
        step3[name] = Image.fromarray(np.asarray(metadata.pop('sample_rgba'), dtype=np.uint8))
        category, maximum = limits(name)
        oversized = native.width > maximum[0] or native.height > maximum[1]
        natives[name], raw[name] = native, cell
        entries.append({'name': name, 'file': f'{name}.png', 'class': category, 'native_size': list(native.size),
                        'class_max': list(maximum), 'status': 'regenerate' if oversized else 'size_pass',
                        'regenerate_reasons': ['native_class_max'] if oversized else [], 'grid': metadata})
    palette, palette_meta = build_master(load_palette(directory / 'palette.hex'), natives)
    save_palette(palette, directory / 'palette_v2.hex')
    quantized = {name: quantize(image, palette) for name, image in natives.items()}
    semantics = evaluate(step3, quantized, palette, palette_meta)
    for entry in entries:
        name = str(entry['name'])
        reasons = list(entry['regenerate_reasons'])
        if semantics['sprites'][name]['mean_delta_e'] > 6:
            reasons.append('fidelity_delta_e_above_6')
        if name.endswith('berry_bush_bare') and semantics['metrics']['berry_red_counts']['bare'] != 0:
            reasons.append('bare_bush_contains_red')
        if entry['grid']['axis_periods_separate']:
            reasons.append('anisotropic_grid_requires_source_review')
        entry['regenerate_reasons'] = reasons
        entry['status'] = 'regenerate' if reasons else 'pass'
    darkest = palette[int(np.argmin(rgb_to_lab(np.asarray(palette))[:, 0]))]
    finals = {name: outline(image, darkest) for name, image in quantized.items()}
    output = directory / 'sprites_v2'
    output.mkdir(exist_ok=True)
    for entry in entries:
        name = str(entry['name'])
        final = finals[name]
        final.save(output / f'{name}.png')
        natives[name].save(output / f'{name}.native.png')
        quantized[name].save(output / f'{name}.quantized.png')
        step3[name].save(output / f'{name}.sample.png')
        entry.update({'canvas_size': list(final.size), 'anchor': [final.width // 2, final.height - 1],
                      'footprint_tiles': [2, 2] if entry['class'] == 'building' else [1, 1]})
    report = {'version': 2, 'direction': direction, 'source': str(source), 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
              'jpeg_fallback': fallback, 'palette': palette_meta, 'semantics': semantics, 'sprites': entries}
    (output / 'manifest.json').write_text(json.dumps(report, indent=2) + '\n')
    (directory / 'normalize_v2_report.json').write_text(json.dumps(report, indent=2) + '\n')
    lines = [f'# {direction} normalization v2', '', 'No native resampling; maxima exclude outside outline. Source/grid/palette/semantic details are in normalize_v2_report.json.', '',
             '|Sprite|Native|Canvas|Max|Status|', '|---|---|---|---|---|']
    for entry in entries:
        lines.append('|{}|{}|{}|{}|{}|'.format(entry['name'], entry['native_size'], entry['canvas_size'], entry['class_max'], entry['status']))
    lines += ['', '## Semantic evidence', '', '```json', json.dumps(semantics, indent=2), '```', '', '## Palette additions', '', '```json', json.dumps(palette_meta, indent=2), '```']
    (directory / 'normalize_v2_report.md').write_text('\n'.join(lines) + '\n')
    contact_sheet(directory, raw, natives, finals)
    print(direction, json.dumps(semantics))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--raw-root', type=Path, default=Path('/Users/rexxa/work/art-direction-raw'))
    parser.add_argument('--root', type=Path, default=ROOT / 'docs/art')
    parser.add_argument('--direction', choices=('D1', 'D2', 'D3'))
    args = parser.parse_args()
    for direction in (args.direction,) if args.direction else ('D1', 'D2', 'D3'):
        run(direction, args.raw_root, args.root)


if __name__ == '__main__':
    main()
