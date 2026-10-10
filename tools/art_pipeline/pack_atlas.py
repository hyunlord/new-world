# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/pack_atlas.py --root .
"""Stable shelf packing of manifest sprites, preserving provisional direction art."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
from PIL import Image


def pack(root: Path) -> dict:
    """Pack exact PNG pixels on2048² pages with1texel exterior gutters."""
    path = root/'assets/art/manifest.json'
    manifest = json.loads(path.read_text())
    entries = sorted((entry for entry in manifest['assets'] if entry.get('sprite_id') and entry.get('qa_status') in ('accepted','fallback','provisional')), key=lambda entry:entry['sprite_id'])
    output = root/'assets/art/atlas'; output.mkdir(parents=True,exist_ok=True)
    pages = [Image.new('RGBA',(2048,2048))]
    page,x,y,row_height = 0,1,1,0
    sprites = {}
    for entry in entries:
        with Image.open(root/entry['path']) as opened: image = opened.convert('RGBA')
        if image.width > 2046 or image.height > 2046:
            raise ValueError(f"Sprite exceeds atlas page: {entry['sprite_id']}")
        if x+image.width+1 > 2048:
            x,y,row_height = 1,y+row_height+2,0
        if y+image.height+1 > 2048:
            pages.append(Image.new('RGBA',(2048,2048)))
            page,x,y,row_height = page+1,1,1,0
        pages[page].paste(image,(x,y))
        rect = [x,y,image.width,image.height]
        sprites[entry['sprite_id']] = {'page':f'res://assets/art/atlas/world_{page}.png',
          'rect':rect,'anchor':entry['anchor'],'class':entry.get('class','direction'),
          'frames':entry.get('frames',[]),'region_rect':rect,'source_path':entry['path'],
          'body':entry.get('body',''),'view':entry.get('view',''),'activity':entry.get('activity',''),
          'frame':entry.get('frame',0)}
        x += image.width+2
        row_height = max(row_height,image.height)
    for i,image in enumerate(pages): image.save(output/f'world_{i}.png',optimize=True)
    atlas = {'version':1,'page_size':[2048,2048],'gutter':1,
             'pages':[f'res://assets/art/atlas/world_{i}.png' for i in range(len(pages))],'sprites':sprites}
    (output/'world_atlas.json').write_text(json.dumps(atlas,indent=2)+'\n')
    manifest['assets'] = [entry for entry in manifest['assets'] if not entry['path'].startswith('assets/art/atlas/world_') or entry['path'].endswith('_regions.png')]
    for i in range(len(pages)):
        manifest['assets'].append({'path':f'assets/art/atlas/world_{i}.png','source':'procedural:tools/art_pipeline/pack_atlas.py',
                                  'native_size':[2048,2048],'anchor':[0,0],'qa_status':'accepted','role':'atlas'})
    path.write_text(json.dumps(manifest,indent=2)+'\n')
    return atlas

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=Path.cwd())
    args = parser.parse_args()
    atlas = pack(args.root)
    print(json.dumps({'pages':len(atlas['pages']),'sprites':len(atlas['sprites'])}))
