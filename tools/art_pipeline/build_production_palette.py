"""Build the single D3-derived production palette and exact villager lookup texture."""
from __future__ import annotations

import json
import math
from pathlib import Path
from typing import TypedDict

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw

from palette_tools import Color, load_palette, rgb_to_lab
from palette_v2 import opaque, skin_mask
from scene_ground import GroundRoles, ground_image, ground_roles
from scene_layout import Placement, layout, load_sprites, render

ROOT = Path(__file__).resolve().parents[2]


class Entry(TypedDict):
    id: str
    rgb: list[int]
    hex: str
    roles: list[str]
    ramp_positions: list[dict[str, str | int]]


def lab_to_rgb(lab: NDArray[np.generic]) -> NDArray[np.uint8]:
    """Inverse of palette_tools.rgb_to_lab: D65, sRGB gamut clipping and rounding."""
    values = np.asarray(lab, dtype=np.float64)
    light, aa, bb = np.moveaxis(values, -1, 0)
    fy = (light + 16) / 116
    f = np.stack((fy + aa / 500, fy, fy - bb / 200), axis=-1)
    delta = 6 / 29
    xyz = np.where(f > delta, f**3, 3 * delta**2 * (f - 4 / 29))
    xyz *= np.array((0.95047, 1.0, 1.08883))
    matrix = np.array(((.4124564,.3575761,.1804375),(.2126729,.7151522,.0721750),(.0193339,.1191920,.9503041)))
    linear = np.clip(xyz @ np.linalg.inv(matrix).T, 0, 1)
    srgb = np.where(linear <= .0031308, linear * 12.92, 1.055 * linear**(1 / 2.4) - .055)
    return np.clip(np.rint(srgb * 255), 0, 255).astype(np.uint8)


def transform(color: Color, lightness: float = 0, chroma: float = 1, hue: float = 0) -> Color:
    lab = rgb_to_lab(np.array(color))
    angle = math.atan2(float(lab[2]), float(lab[1])) + math.radians(hue)
    radius = math.hypot(float(lab[1]), float(lab[2])) * chroma
    rgb = lab_to_rgb(np.array((lab[0] + lightness, radius * math.cos(angle), radius * math.sin(angle))))
    return tuple(int(v) for v in rgb)


def sampled_ramp(colors: list[Color], count: int, descending: bool = False) -> list[Color]:
    unique = list(dict.fromkeys(colors))
    unique.sort(key=lambda c: float(rgb_to_lab(np.array(c))[0]), reverse=descending)
    return [unique[int(i)] for i in np.rint(np.linspace(0, len(unique)-1, count))]


class Registry:
    def __init__(self) -> None:
        self.entries: list[Entry] = []

    def add(self, color: Color, role: str, ramp: str, position: int) -> str:
        distances = np.linalg.norm(rgb_to_lab(np.array([e['rgb'] for e in self.entries])) - rgb_to_lab(np.array(color)), axis=-1) if self.entries else np.array([])
        if len(distances) and float(distances.min()) < 2:
            index = int(distances.argmin())
        else:
            index = len(self.entries)
            self.entries.append({'id': f'c{index:03d}', 'rgb': list(color), 'hex': '#%02X%02X%02X' % color, 'roles': [], 'ramp_positions': []})
        entry = self.entries[index]
        if role not in entry['roles']:
            entry['roles'].append(role)
        entry['ramp_positions'].append({'ramp': ramp, 'index': position})
        return entry['id']

    def ramp(self, colors: list[Color], role: str, name: str) -> list[str]:
        return [self.add(c, role, name, i) for i, c in enumerate(colors)]

    def rgb(self, ids: list[str]) -> list[list[int]]:
        return [self.entries[int(i[1:])]['rgb'] for i in ids]


def main() -> None:
    source = ROOT / 'docs/art/D3'
    palette = load_palette(source / 'palette_v2.hex')
    original = ground_roles(palette)
    boulder = Image.open(source / 'sprites_v2/13_boulder.png')
    rock = sampled_ramp([tuple(int(v) for v in c) for c in np.unique(opaque(boulder), axis=0)], 4)
    raw_ground = {'grass': list(original.grass), 'earth': list(original.earth),
        'forest_floor': [transform(c, -8, 1, -6) for c in original.grass],
        'hill': [transform(c, 5, .85) for c in original.grass], 'rock': rock,
        'camp_earth': list(original.earth), 'shore': [original.shore], 'water': list(original.water)}
    ground = {name: [transform(c, 0, .85) if name == 'water' else transform(c, -3, .72, 4) for c in colors] for name, colors in raw_ground.items()}
    skin_colors = [c for c in palette if bool(skin_mask(np.array(c)))]
    skin_base = [min(skin_colors, key=lambda c: abs(float(rgb_to_lab(np.array(c))[0])-target)) for target in (78,66,54)]
    # D3 hide seed is the middle warm skin/hide shade, all variants are Lab offsets.
    hide = skin_base[1]
    villagers: dict[str, list[list[Color]]] = {
        'skin': [[transform(c, -12*k, 1+.05*k) for c in skin_base] for k in range(4)],
        'hair': [], 'clothing': []}
    hair_specs = ((-50,.2,0),(-37,.65,0),(-23,1.15,-15),(0,.08,0),(20,.04,0))
    cloth_specs = ((0,1,0),(-20,.8,0),(0,.08,0),(3,1.5,15),(-3,1.6,-25),(-9,.65,65))
    for family, specs in (('hair', hair_specs), ('clothing', cloth_specs)):
        for dl, cc, hh in specs:
            base = transform(hide, dl, cc, hh)
            villagers[family].append([transform(base, shift) for shift in (10,0,-12)])
    registry = Registry()
    actors = registry.ramp(palette, 'actor', 'source_d3')
    # Attach conservative role hints to source colors; region overrides refine anatomy.
    for i,c in enumerate(palette):
        lab = rgb_to_lab(np.array(c))
        role = 'skin' if bool(skin_mask(np.array(c))) else ('hair' if lab[0] < 28 else 'other')
        registry.add(c, role, 'source_roles', i)
    ground_ids = {name: registry.ramp(colors, 'ground', name) for name,colors in ground.items()}
    villager_ids = {name: [registry.ramp(colors, name, f'{name}_{index}') for index,colors in enumerate(ramps)] for name,ramps in villagers.items()}
    rgb_ground = {name: registry.rgb(ids) for name,ids in ground_ids.items()}
    rgb_villagers = {name: [registry.rgb(ids) for ids in ramps] for name,ramps in villager_ids.items()}
    document = {'version': 1, 'source': 'docs/art/D3/palette_v2.hex', 'count': len(registry.entries),
        'merge_delta_e76': 2, 'merge_rule': 'stable first existing color retained; ground and villager roles preserve references',
        'colors': registry.entries, 'actor_palette': actors, 'ground_ramps': ground_ids, 'villager_ramps': villager_ids,
        'rgb_ramps': {'ground': rgb_ground, 'villager': rgb_villagers},
        'unmerged_ground_rgb': ground, 'source_ground_rgb': raw_ground,
        'ground_transform': {'chroma_factor': .72, 'lightness_offset': -3, 'hue_degrees': 4, 'water_chroma_factor': .85},
        'villager_derivation': {'skin_base_rgb': skin_base, 'hide_seed_rgb': hide, 'hair_lab_specs': hair_specs, 'clothing_lab_specs': cloth_specs, 'shade_lightness_offsets': [10,0,-12]},
        'variant_names': {'skin':['light','medium','tan','deep'], 'hair':['black','dark_brown','auburn','grey','white'], 'clothing':['tan_hide','dark_hide','grey_fur','ochre_dyed','red_ochre','green_brown_fibre']}}
    (ROOT/'tools/art_pipeline/production_palette.json').write_text(json.dumps(document, indent=2)+'\n')
    out = ROOT/'docs/art/production'
    out.mkdir(parents=True, exist_ok=True)
    swatch = Image.new('RGB',(16*24, math.ceil(len(registry.entries)/16)*24), tuple(registry.entries[0]['rgb']))
    draw = ImageDraw.Draw(swatch)
    for i,entry in enumerate(registry.entries):
        x,y = i%16*24,i//16*24
        draw.rectangle((x,y,x+23,y+23),fill=tuple(entry['rgb']))
    swatch.save(out/'palette.png')
    lookup = np.array([row for family in ('skin','hair','clothing') for row in rgb_villagers[family]],dtype=np.uint8)
    (ROOT/'assets/art').mkdir(parents=True,exist_ok=True)
    Image.fromarray(lookup).save(ROOT/'assets/art/villager_ramps.png')
    muted = GroundRoles(tuple(tuple(c) for c in rgb_ground['grass']),tuple(tuple(c) for c in rgb_ground['earth']),tuple(tuple(c) for c in rgb_ground['water']),tuple(rgb_ground['shore'][0]))
    sprites = load_sprites(source)
    placements = layout(sprites,'D3')
    # Undo comparison compositor clothing jitter: actors identical, palette unchanged.
    placements = [Placement(sprites[p.sprite.name],p.x,p.y,p.villager_id) for p in placements]
    comparison = Image.new('RGB',(1920,1080))
    for index,roles in enumerate((original,muted)):
        world = render(ground_image(roles,'D3'),placements)
        crop = world.crop((392,377,632,647)).resize((960,1080),Image.Resampling.NEAREST)
        comparison.paste(crop,(index*960,0))
    comparison.save(out/'ground_compare.png')
    print(f'Production palette: {len(registry.entries)} colors; lookup {lookup.shape[1]}x{lookup.shape[0]}; comparison original left, muted right')


if __name__ == '__main__':
    main()
