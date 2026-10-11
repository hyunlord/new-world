"""Create anatomical index atlases sharing the exact production color-atlas UVs."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

import numpy as np
from PIL import Image, ImageDraw

from palette_tools import rgb_to_lab
from hair_registry_p4 import HairFrame, HairRegistry, ReviewedMapError

ROOT = Path(__file__).resolve().parents[2]
REGIONS = {'transparent': 0, 'outline': 1, 'skin': 2, 'hair': 3, 'clothing': 4, 'other': 5}
DISPLAY = np.array(((35,35,35),(5,5,5),(238,166,112),(80,122,225),(71,204,113),(219,101,207)),dtype=np.uint8)


def classify(image: Image.Image, body: str = 'default', *, palette_path: Path | None = None,
             overrides_path: Path | None = None, view: str = '', activity: str = '') -> Image.Image:
    """Encode region in R and luminance-rank shade in G, preserving binary alpha."""
    palette = json.loads((palette_path or ROOT/'tools/art_pipeline/production_palette.json').read_text())
    overrides = json.loads((overrides_path or ROOT/'tools/art_pipeline/region_overrides.json').read_text())
    rules = overrides['bodies'].get(body,overrides['bodies']['default'])
    rgba = np.asarray(image.convert('RGBA'))
    rgb = rgba[...,:3]
    alpha = rgba[...,3] > 0
    result = np.zeros_like(rgba)
    result[...,3] = np.where(alpha,255,0)
    regions = np.where(alpha,5,0).astype(np.uint8)
    palette_colors = np.array([entry['rgb'] for entry in palette['colors']])
    outline = palette_colors[np.argmin(rgb_to_lab(palette_colors)[:,0])]
    yy,xx = np.indices(alpha.shape)
    y = (yy+.5)/image.height
    x = (xx+.5)/image.width
    clothing = (y>=rules['clothing_y'][0]) & (y<rules['clothing_y'][1]) & (x>=rules['clothing_x'][0]) & (x<rules['clothing_x'][1])
    luma = rgb_to_lab(rgb)[...,0]
    for entry in palette['colors']:
        mask = alpha & np.all(rgb==entry['rgb'],axis=-1)
        roles = set(entry['roles'])
        if 'skin' in roles:
            regions[mask] = 2
            regions[mask & clothing] = 4
        elif 'clothing' in roles:
            regions[mask] = 4
        elif 'hair' in roles:
            regions[mask & (y<rules['hair_y_max'])] = 3
        if entry['hex'] in rules['colors']:
            regions[mask] = REGIONS[rules['colors'][entry['hex']]]
    if not (view or activity):
        regions[alpha & (y<rules['hair_y_max']) & (luma<38)] = 3
    else:
        regions[alpha & (y<rules['hair_y_max']) & (luma<38)] = 3
        context = overrides.get('context_rules', {})
        for color in context.get('warm_skin_colors', []):
            value = tuple(bytes.fromhex(color.removeprefix('#')))
            regions[alpha & (y >= rules['hair_y_max']) & np.all(rgb == value, axis=-1)] = 2
        # Recover warm shadow texels adjoining known exposed skin, not entire body boxes.
        seeds = regions == 2
        padded = np.pad(seeds, 1)
        adjacent = np.zeros_like(seeds)
        for dy, dx in ((0,1),(1,0),(1,2),(2,1)):
            adjacent |= padded[dy:dy+image.height, dx:dx+image.width]
        for color in context.get('skin_shadow_colors', []):
            value = tuple(bytes.fromhex(color.removeprefix('#')))
            regions[alpha & adjacent & (y >= rules['hair_y_max']) & np.all(rgb == value, axis=-1)] = 2
        digest = hashlib.sha256(rgba.tobytes()).hexdigest()
        for correction in overrides.get('reviewed_images', []):
            if correction['rgba_sha256'] != digest or correction['size'] != list(image.size):
                continue
            if correction['body'] != body or correction['view'] != view or correction['activity'] != activity:
                continue
            for patch in correction['patches']:
                for row, start, stop in patch['spans']:
                    mask = alpha[row, start:stop]
                    regions[row, start:stop][mask] = REGIONS[patch['region']]
    regions[alpha & np.all(rgb==outline,axis=-1)] = 1
    # Shade bins are ranked within each region, not ranked over unrelated objects.
    for region in (2,3,4):
        mask = regions==region
        if mask.any():
            unique = np.unique(luma[mask])
            rank = np.searchsorted(unique,luma[mask])
            shade = 2-np.rint(rank/max(1,len(unique)-1)*2).astype(np.uint8)
            result[...,1][mask] = shade
    result[...,0] = regions
    return Image.fromarray(result)


def build_atlases(manifest_path: Path, *, diagnostic_path: Path | None = None,
                  tables_path: Path | None = None,
                  reviewed_maps_path: Path | None = None) -> dict[str, Any]:
    """Consume world_atlas.json; preserve rects and write *_regions.png beside pages."""
    manifest = json.loads(manifest_path.read_text())
    # tables_path remains accepted for older diagnostic callers; runtime is hair-only.
    reviewed = HairRegistry.load(ROOT, reviewed_maps_path)
    pages: dict[str, Image.Image] = {}
    previews: list[tuple[str,Image.Image,Image.Image]] = []
    output: dict[str, Any] = {'version':1,'encoding':{'r':'hair 3, otherwise 0','g':'shade integer 0..2','a':'binary source alpha'},'pages':{},'sprites':{}}
    for name,entry in manifest['sprites'].items():
        if not (any(token in name for token in ('villager','adult','child')) or name.startswith(('V_a','V_b','V_c')) or entry.get('class') in ('adult','child','villager')):
            continue
        page_value = entry['page']
        page = str(manifest['pages'][page_value]) if isinstance(page_value,int) else str(page_value)
        path = ROOT/page.removeprefix('res://')
        if page not in pages:
            with Image.open(path) as source:
                pages[page] = Image.new('RGBA',source.size)
        rect = entry['rect']
        with Image.open(path) as source:
            sprite = source.convert('RGBA').crop((rect[0],rect[1],rect[0]+rect[2],rect[1]+rect[3]))
        body = str(entry.get('body',next((value for value in ('adult_a','adult_b','child') if value in name),'default')))
        if name.startswith(('V_a','V_b','V_c')):
            body = {'a':'adult_a','b':'adult_b','c':'child'}[name[2]]
        resolved = reviewed.resolve(HairFrame(body, str(entry.get('activity', '')), sprite, entry.get('frame')))
        indices = resolved.image
        classification_source = resolved.source
        evidence = {}
        if resolved.path:
            evidence = {'reviewed_mask_path': resolved.path,
                        'reviewed_mask_sha256': resolved.sha256}
        pages[page].paste(indices,(rect[0],rect[1]))
        region_page = 'res://'+str(path.with_name(path.stem+'_regions.png').relative_to(ROOT))
        output['sprites'][name] = {'page':region_page,'rect':rect,'body':body,
            'classification_source':classification_source, **evidence}
        if len(previews)<96:
            previews.append((name,sprite,indices))
    for page,indices in pages.items():
        path = ROOT/page.removeprefix('res://')
        target = path.with_name(path.stem+'_regions.png')
        indices.save(target)
        output['pages'][page] = 'res://'+str(target.relative_to(ROOT))
    manifest_path.with_name('region_atlas.json').write_text(json.dumps(output,indent=2)+'\n')
    contact = Image.new('RGB',(768,max(1,(len(previews)+3)//4)*176),(30,30,30))
    draw = ImageDraw.Draw(contact)
    for i,(name,sprite,indices) in enumerate(previews):
        x,y = (i%4)*192,(i//4)*176
        data = np.asarray(indices)
        diagnostic = Image.fromarray(DISPLAY[data[...,0]])
        contact.paste(sprite.resize((sprite.width*3,sprite.height*3),Image.Resampling.NEAREST),(x,y+18),sprite.resize((sprite.width*3,sprite.height*3),Image.Resampling.NEAREST))
        contact.paste(diagnostic.resize((sprite.width*3,sprite.height*3),Image.Resampling.NEAREST),(x+96,y+18))
        draw.text((x+2,y+2),name[-26:],fill='white')
    if diagnostic_path is not None:
        diagnostic_path.parent.mkdir(parents=True, exist_ok=True)
        contact.save(diagnostic_path)
    return output


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',type=Path,default=ROOT/'assets/art/atlas/world_atlas.json')
    parser.add_argument('--diagnostic', type=Path, help='Optional contact PNG; old production documents are never implicit outputs')
    parser.add_argument('--tables', type=Path, help='Legacy diagnostic argument; runtime pages always use hair masks')
    parser.add_argument('--reviewed-maps', type=Path, help='Exact-pixel frozen P4 hair-mask registry')
    args = parser.parse_args()
    result = build_atlases(args.manifest, diagnostic_path=args.diagnostic, tables_path=args.tables,
                          reviewed_maps_path=args.reviewed_maps)
    print(f"Regions: {len(result['sprites'])} frames, {len(result['pages'])} matching atlas pages")
