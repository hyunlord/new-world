# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/3d/blender/test_tripo.py
"""Independent checks of processed GLB budgets, ground, textures and shared rigs."""
import json
import math
import struct
from pathlib import Path
from typing import Final

from test_fence_directions import positions

ROOT: Final = Path(__file__).resolve().parents[3] / 'assets/3d'
BUDGETS: Final = {'woman': (4000, 1500, 500), 'goat': (4000, 1500, 500),
                 'house': (8000, 3000, 1000), 'bloomery': (4000, 1500, 500)}


def check() -> None:
    """Given built files, independently parse outputs and enforce export contracts."""
    for subject, budgets in BUDGETS.items():
        for level, budget in enumerate(budgets):
            stem = f'tripo_{subject}_lod{level}'
            path = ROOT / 'models' / (stem+'.glb')
            raw = path.read_bytes()
            magic, version, length = struct.unpack_from('<4sII', raw)
            assert (magic, version, length) == (b'glTF', 2, len(raw)), stem
            size = struct.unpack_from('<I', raw, 12)[0]
            gltf = json.loads(raw[20:20+size])
            triangles = sum(gltf['accessors'][p['indices']]['count']//3
                            for mesh in gltf['meshes'] for p in mesh['primitives'])
            assert budget*.85 <= triangles <= budget*1.05, (stem, triangles)
            points = positions(path)
            assert all(math.isfinite(v) for point in points for v in point), stem
            height = max(p[1] for p in points)-min(p[1] for p in points)
            assert abs(min(p[1] for p in points)) < .03, stem
            if subject in ('woman', 'bloomery'):
                target = 1.6 if subject == 'woman' else 1.4
                assert abs(height-target) < .04, (stem, height)
            if subject == 'house':
                assert max(max(p[i] for p in points)-min(p[i] for p in points) for i in (0, 2)) <= 4.02
            for texture in ('albedo', 'normal'):
                png = (ROOT / 'models' / (stem+'_'+texture+'.png')).read_bytes()
                expected = 512 if level == 2 else (2048 if subject == 'house' else 1024)
                assert png[:8] == b'\x89PNG\r\n\x1a\n'
                assert struct.unpack_from('>II', png, 16) == (expected, expected), stem
            materials = gltf['materials']
            assert all('normalTexture' in mat and 'baseColorTexture' in mat['pbrMetallicRoughness'] for mat in materials), stem
            if subject in ('woman', 'goat'):
                assert len(gltf['skins']) == 1 and len(gltf['skins'][0]['joints']) == (17 if subject == 'woman' else 11)
                assert {a['name'] for a in gltf['animations']} >= ({'idle', 'walk', 'work', 'carry', 'sleep'} if subject == 'woman' else {'idle', 'walk', 'graze', 'lie_down'})
    print('PASS: 12 Tripo LODs, budgets, metre scale, ground pivots, albedo/normal textures and 17/11-bone rigs')


if __name__ == '__main__':
    check()
