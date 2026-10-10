"""Audit pass-2 files without conflating format success with art acceptance."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image

from palette_tools import load_palette

ROOT = Path(__file__).resolve().parents[2] / 'docs/art'


def main() -> None:
    directions: dict[str, object] = {}
    for direction in ('D1', 'D2', 'D3', 'H'):
        directory = ROOT / direction
        manifest = json.loads((directory / 'sprites_v2/manifest.json').read_text())
        palette = set(load_palette(directory / 'palette_v2.hex'))
        assert len(manifest['sprites']) == 16
        failures: list[str] = []
        for entry in manifest['sprites']:
            image = Image.open(directory / 'sprites_v2' / entry['file']).convert('RGBA')
            rgba = np.asarray(image)
            assert list(image.size) == entry['canvas_size'], entry['name']
            assert entry['canvas_size'] == [v + 2 for v in entry['native_size']], entry['name']
            assert entry['anchor'] == [image.width // 2, image.height - 1]
            assert np.isin(rgba[..., 3], [0, 255]).all()
            assert set(map(tuple, rgba[..., :3][rgba[..., 3] == 255])).issubset(palette)
            if entry['status'] == 'regenerate':
                failures.append(entry['name'])
        for name in ('ingame_noon.png', 'ingame_night.png', 'ingame_overview.png',
                     'ingame_noon_without_villagers.png', 'ingame_overview_without_villagers.png'):
            with Image.open(directory / name) as image:
                assert image.size == (1920, 1080) and image.format == 'PNG'
        readability = json.loads((directory / 'readability_v2.json').read_text())
        for key in ('zoom4', 'zoom1'):
            metric = readability[key]
            assert len(metric['villagers']) == 20
            assert metric['visible_fraction'] == sum(row['visible'] for row in metric['villagers']) / 20
            assert all(row['visible'] == (row['changed_fraction'] >= .2) for row in metric['villagers'])
        provenance = None
        if direction != 'H':
            source = Path(manifest['source'])
            provenance = hashlib.sha256(source.read_bytes()).hexdigest() == manifest['source_sha256']
            assert provenance
        directions[direction] = {'format_pass': True, 'sprites': 16, 'regenerate': failures,
                                 'source_sha256_verified': provenance,
                                 'semantic_pass': manifest.get('semantics', {}).get('passed', 'inherited D1/D3 failures'),
                                 'visibility4_pass': readability['zoom4']['target_pass'],
                                 'visibility1_pass': readability['zoom1']['target_pass']}
    report = {'format_pass': True, 'production_approved': False, 'new_image_generations': 0,
              'directions': directions, 'scope': 'Offline art only; no engine/runtime/device acceptance'}
    (ROOT / 'validation_v2.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
