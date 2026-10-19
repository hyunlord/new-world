# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/harvest_baseline_p5.py
"""Freeze original native, sprite and rendered-region content from the sealed P4 Git tree."""
from __future__ import annotations

import hashlib
from io import BytesIO
import json
from pathlib import Path
import subprocess
from typing import Final
from PIL import Image
from salvage_verdicts import atomic_json

BASE: Final = 'c6d88d625a86dfdc06cebe48923ae65ff1455cfb'


def blob(path: str) -> bytes:
    return subprocess.run(['git','show',f'{BASE}:{path}'], capture_output=True, check=True).stdout


def rgba_hash(image: Image.Image) -> str:
    return hashlib.sha256(image.convert('RGBA').tobytes()).hexdigest()


def freeze(root: Path = Path.cwd()) -> Path:
    """Read immutable Git PNG blobs, including masks as actually drawn in the P4 atlas."""
    destination = root / 'docs/art/production5/harvest/baseline_content.json'
    if destination.exists():
        return destination
    manifest = json.loads(blob('assets/art/manifest.json'))
    regions = json.loads(blob('assets/art/atlas/region_atlas.json'))
    pages = {}
    frames = {}
    for row in manifest['assets']:
        target = row.get('sprite_id')
        if not target or not row.get('body') or not row.get('drawable',True):
            continue
        values = {}
        for field in ('path','native_path'):
            with Image.open(BytesIO(blob(row[field]))) as image:
                values[field] = dict(size=list(image.size), rgba_sha256=rgba_hash(image))
        region = regions['sprites'].get(target)
        if region:
            page = region['page'].removeprefix('res://')
            if page not in pages:
                with Image.open(BytesIO(blob(page))) as image:
                    pages[page] = image.convert('RGBA')
            x,y,w,h = region['rect']
            values['mask'] = dict(size=[w,h], rgba_sha256=rgba_hash(pages[page].crop((x,y,x+w,y+h))))
        frames[target] = values
    atomic_json(destination, dict(source_commit=BASE, source='git blobs; actual P4 region atlas texels', frames=frames))
    return destination


if __name__ == '__main__':
    print(freeze())
