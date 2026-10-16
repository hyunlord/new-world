"""Build lossless, labeled pass-2 comparison artifacts from committed outputs."""
from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

from palette_tools import load_palette
from presentation import FONTS

ROOT = Path(__file__).resolve().parents[2] / 'docs/art'


def sprite_sheet(directory: Path) -> Image.Image:
    entries = json.loads((directory / 'sprites_v2/manifest.json').read_text())['sprites']
    cell_w = max(item['canvas_size'][0] for item in entries) + 4
    cell_h = max(item['canvas_size'][1] for item in entries) + 10
    result = Image.new('RGB', (cell_w * 4, cell_h * 4), '#25262b')
    draw = ImageDraw.Draw(result)
    for i, entry in enumerate(entries):
        sprite = Image.open(directory / 'sprites_v2' / entry['file']).convert('RGBA')
        x, y = i % 4 * cell_w, i // 4 * cell_h
        result.paste(sprite, (x + (cell_w - sprite.width) // 2, y + cell_h - sprite.height), sprite)
        draw.text((x + 2, y), f'{i + 1:02d}', fill='white', font_size=8)
    result.save(directory / 'corrected_sheet_v2.png')
    return result


def swatch(directory: Path) -> Image.Image:
    palette = load_palette(directory / 'palette_v2.hex')
    result = Image.new('RGB', (448, 360), '#25262b')
    draw = ImageDraw.Draw(result)
    for i, color in enumerate(palette):
        x, y = i % 8 * 56, i // 8 * 24
        draw.rectangle((x + 2, y + 2, x + 53, y + 14), fill=color)
        draw.text((x + 2, y + 15), '#%02X%02X%02X' % color, fill='white', font_size=8)
    result.save(directory / 'palette_v2.png')
    return result


def main() -> None:
    width, height = 480, 510
    board = Image.new('RGB', (width * 5, height * 4 + 42), '#181c22')
    draw = ImageDraw.Draw(board)
    font = ImageFont.truetype(str(FONTS / 'Galmuri11-Bold.ttf'), 12)
    draw.fontmode = '1'
    labels = ('낮 4× · HUD', '밤 4×', '전체 지도 1×', '복원 스프라이트 2×', '마스터 팔레트')
    for col, label in enumerate(labels):
        draw.text((col * width + 12, 12), label, font=font, fill='white')
    for row, direction in enumerate(('D1', 'D2', 'D3', 'H')):
        directory = ROOT / direction
        sheet, palette = sprite_sheet(directory), swatch(directory)
        sources = [Image.open(directory / name).convert('RGB') for name in
                   ('ingame_noon.png', 'ingame_night.png', 'ingame_overview.png')] + [sheet, palette]
        for col, source in enumerate(sources):
            # Scene thumbnails use exact reciprocal4; sprite atlas integer zoom only.
            if col < 3:
                source = source.resize((480, 270), Image.Resampling.NEAREST)
            elif col == 3:
                source = source.resize((source.width * 2, source.height * 2), Image.Resampling.NEAREST)
            x, y = col * width, 42 + row * height
            board.paste(source, (x + (width - source.width) // 2, y + 28 + (height - 42 - source.height) // 2))
            draw.text((x + 10, y + 5), f'{direction} / {labels[col]}', font=font, fill='white')
    board.save(ROOT / 'direction_compare_v2.png')


if __name__ == '__main__':
    main()
