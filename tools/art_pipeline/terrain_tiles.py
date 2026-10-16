"""D3 palette-only dual-grid tiles, with stable corner masks and native pixels."""
from __future__ import annotations
import json
from pathlib import Path
from typing import Final, TypedDict
import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw

LAYERS: Final = ('water', 'grass', 'forest_floor', 'camp_earth', 'hill', 'rock')
Color = tuple[int, int, int]

class Ramps(TypedDict):
    water: list[Color]
    grass: list[Color]
    forest_floor: list[Color]
    camp_earth: list[Color]
    hill: list[Color]
    rock: list[Color]
    shore: list[Color]


def read_ramps(path: Path) -> Ramps:
    source = json.loads(path.read_text())['rgb_ramps']['ground']
    def colors(name: str) -> list[Color]:
        return sorted([(int(c[0]), int(c[1]), int(c[2])) for c in source[name]],
                      key=lambda c: .2126*c[0]+.7152*c[1]+.0722*c[2])
    return Ramps(water=colors('water'), grass=colors('grass'),
                 forest_floor=colors('forest_floor'), camp_earth=colors('camp_earth'),
                 hill=colors('hill'), rock=colors('rock'), shore=colors('shore'))


def coverage(mask: int, variant: int) -> NDArray[np.bool_]:
    """Marching squares: rounded single corners, complementary three-corners."""
    y, x = np.mgrid[0:16, 0:16]
    if mask in (0, 15):
        return np.full((16, 16), mask == 15, dtype=bool)
    wobble = ((x * 13 + y * 7 + variant * 5) % 11 == 0).astype(int)
    wobble -= ((x * 3 + y * 11 + variant) % 13 == 0).astype(int)
    # Keep edge crossings stable between independent tile variants.
    wobble[(x == 0) | (y == 0) | (x == 15) | (y == 15)] = 0
    corners = ((0, 0), (15, 0), (0, 15), (15, 15))
    if mask.bit_count() in (1, 3):
        inverse = mask.bit_count() == 3
        bit = next(i for i in range(4) if bool(mask & (1 << i)) != inverse)
        cx, cy = corners[bit]
        disk = (x - cx) ** 2 + (y - cy) ** 2 <= (7.5 + wobble) ** 2
        return ~disk if inverse else disk
    adjacent = {3: y < 8 + wobble, 12: y >= 8 + wobble,
                5: x < 8 + wobble, 10: x >= 8 + wobble}
    if mask in adjacent:
        return adjacent[mask]
    result = np.zeros((16, 16), dtype=bool)
    for bit, (cx, cy) in enumerate(corners):
        if mask & (1 << bit):
            result |= (x - cx) ** 2 + (y - cy) ** 2 <= (7.5 + wobble) ** 2
    return result


def expand(mask: NDArray[np.bool_]) -> NDArray[np.bool_]:
    padded = np.pad(mask, 1, mode='edge')
    return mask | padded[:-2, 1:-1] | padded[2:, 1:-1] | padded[1:-1, :-2] | padded[1:-1, 2:]


def make_tile(layer: str, mask: int, variant: int, frame: int, ramps: Ramps) -> Image.Image:
    pixels = np.zeros((16, 16, 4), dtype=np.uint8)
    if mask == 0:
        return Image.fromarray(pixels)
    land = coverage(mask, variant) if layer != 'water' else np.ones((16, 16), dtype=bool)
    ramp = ramps[layer]
    base = ramp[len(ramp)//2]
    pixels[land] = (*base, 255)
    y, x = np.mgrid[0:16, 0:16]
    interior = (x > 1) & (x < 14) & (y > 1) & (y < 14) & land
    pattern = (x*73856093 ^ y*19349663 ^ variant*83492791) & 0x7fffffff
    features = interior & (pattern % 41 < 2)
    pixels[features] = (*ramp[-1], 255)
    if layer == 'hill':
        pixels[interior & (pattern % 97 == 0)] = (*ramps['camp_earth'][1], 255)
    if layer == 'water':
        pixels[features] = (*base, 255)
        for n in range(3):
            px = 2 + ((variant*3 + n*5 + frame) % 10)
            py = 3 + ((variant + n*4) % 10)
            pixels[py, px:px+2] = (*ramp[-1], 255)
    if mask != 15 and layer != 'water':
        inner = land & expand(~land)
        pixels[inner] = (*ramp[0], 255)
        outer = expand(land) & ~land
        if layer == 'grass':
            shore = expand(expand(land)) & ~land
            wet = expand(shore | land) & ~(shore | land)
            pixels[wet] = (*ramps['water'][0], 255)
            pixels[shore] = (*ramps['shore'][0], 255)
        if layer == 'forest_floor':
            blend = outer & ((x + y + variant) % 2 == 0)
            pixels[blend] = (*ramp[1], 255)
            pixels[inner & ((x+y) % 2 == 0)] = (*base, 255)
        if layer == 'hill':
            pixels[inner & (y < 8)] = (*ramp[-1], 255)
            pixels[outer & (y >= 8)] = (*ramp[0], 255)
        if layer == 'camp_earth':
            pixels[outer & (pattern % 5 == 0)] = (*ramp[-1], 255)
    return Image.fromarray(pixels)


def build(root: Path) -> None:
    ramps = read_ramps(root / 'tools/art_pipeline/production_palette.json')
    output = root / 'assets/art/terrain'
    output.mkdir(parents=True, exist_ok=True)
    layers = []
    provenance = []
    contact = Image.new('RGB', (1024, 6*144), '#232827')
    draw = ImageDraw.Draw(contact)
    for index, layer in enumerate(LAYERS):
        water = layer == 'water'
        atlas = Image.new('RGBA', (256 if water else 128, 256))
        tiles = []
        for mask in range(1, 16):
            variants = 4 if water else (8 if mask == 15 else 2)
            for variant in range(variants):
                frames = 4 if water else 1
                column = variant * frames
                for frame in range(frames):
                    atlas.paste(make_tile(layer, mask, variant, frame, ramps), ((column+frame)*16, mask*16))
                tiles.append({'mask': mask, 'variant': variant, 'atlas_coords': [column, mask],
                              'rect': [column*16, mask*16, 16, 16], 'frames': frames,
                              'fps': 3 if water else 0, 'animation_columns': frames})
        path = output / f'{layer}.png'
        atlas.save(path)
        layers.append({'name': layer, 'path': f'res://assets/art/terrain/{layer}.png', 'tiles': tiles})
        provenance.append({'path': str(path.relative_to(root)), 'source': 'procedural:tools/art_pipeline/terrain_tiles.py',
                           'native_size': list(atlas.size), 'anchor': [0, 0], 'qa_status': 'passed'})
        draw.text((4, index*144+4), layer, fill='white')
        for mask in range(16):
            tile = make_tile(layer, mask, 0, 0, ramps).resize((64, 64), Image.Resampling.NEAREST)
            contact.paste(tile, (mask*64, index*144+24), tile)
        for variant in range(8 if not water else 4):
            tile = make_tile(layer, 15, variant, 0, ramps).resize((64, 64), Image.Resampling.NEAREST)
            contact.paste(tile, (variant*64, index*144+80), tile)
    (output/'terrain_manifest.json').write_text(json.dumps({'version': 1, 'tile_size': 16, 'offset': [-8,-8],
        'corner_bits': {'TL': 1, 'TR': 2, 'BL': 4, 'BR': 8}, 'layers': layers}, indent=2)+'\n')
    (output/'provenance.json').write_text(json.dumps(provenance, indent=2)+'\n')
    contact.save(root/'docs/art/production/terrain_tiles.png')


if __name__ == '__main__':
    build(Path(__file__).resolve().parents[2])
