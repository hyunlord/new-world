"""Create H from D1 environment and D3 corrected actors, without generation."""
from __future__ import annotations

import json
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from palette_tools import load_palette, save_palette, rgb_to_lab
from palette_v2 import merge_palette, quantize

ROOT = Path(__file__).resolve().parents[2] / 'docs/art'
ENVIRONMENT = ('13_boulder', '14_young_tree', '15_reed_tuft')


def main() -> None:
    output = ROOT / 'H'
    sprites = output / 'sprites_v2'
    sprites.mkdir(parents=True, exist_ok=True)
    d1 = json.loads((ROOT / 'D1/sprites_v2/manifest.json').read_text())
    d3 = json.loads((ROOT / 'D3/sprites_v2/manifest.json').read_text())
    # Pass-1 D1 scene colors are exactly the scene component of its master.
    ground = load_palette(ROOT / 'D1/palette.hex')
    actor_colors: list[tuple[int, int, int]] = []
    environment_colors: list[tuple[int, int, int]] = []
    images: dict[str, Image.Image] = {}
    entries: list[dict[str, object]] = []
    for original in d3['sprites']:
        name = original['name']
        source = 'D1' if name in ENVIRONMENT else 'D3'
        entry = dict(next(item for item in d1['sprites'] if item['name'] == name) if source == 'D1' else original)
        image = Image.open(ROOT / source / 'sprites_v2' / f'{name}.png').convert('RGBA')
        rgba = np.asarray(image)
        colors = [tuple(int(v) for v in color) for color in np.unique(rgba[..., :3][rgba[..., 3] == 255], axis=0)]
        (environment_colors if source == 'D1' else actor_colors).extend(colors)
        images[name] = image
        entry['source_direction'] = source
        entries.append(entry)
    palette = merge_palette(ground + environment_colors + actor_colors)
    save_palette(palette, output / 'palette_v2.hex')
    save_palette(ground, output / 'ground_palette_v2.hex')
    for name, image in images.items():
        quantize(image, palette).save(sprites / f'{name}.png')
    manifest = {'version': 2, 'direction': 'H', 'composition': 'D1 environment, D3 actors', 'sprites': entries,
                'palette_count': len(palette), 'merge': 'stable actual input color ΔE76<3',
                'ground_source': 'D1 pass-1 scene subset of D1 master plus D1 environment sprite colors',
                'actor_source': 'opaque colors used by corrected D3 actor sprites'}
    (sprites / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    (output / 'normalize_v2_report.md').write_text('# H composition\n\nD1 ground, boulder, young tree and reeds; D3 remaining corrected sprites. No rescaling. Source size/semantic failures remain applicable. Palette merge may change input colors by less than ΔE76 3.\n')
    board = Image.new('RGB', (1024, 1280), '#25262b')
    draw = ImageDraw.Draw(board)
    for i, (name, image) in enumerate(images.items()):
        image = quantize(image, palette)
        x, y = (i % 4) * 256, (i // 4) * 320
        draw.text((x + 4, y + 4), name, fill='white')
        scaled = image.resize((image.width * 8, image.height * 8), Image.Resampling.NEAREST)
        board.paste(scaled, (x + 8, y + 312 - scaled.height), scaled)
    board.save(output / 'sprites_v2_contact.png')
    print(json.dumps({'palette_count': len(palette), 'sprites': len(entries)}))


if __name__ == '__main__':
    main()
