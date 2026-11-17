"""Check exported L2 building geometry independently of the Blender generator."""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/3d/blender/look2_building_test.py
import hashlib
import json
from pathlib import Path

from test_fence_directions import positions
from verify_assets import check_manifest
from verify_vat import accessor, read_glb

ROOT = Path(__file__).resolve().parents[3]
ASSETS = ROOT / 'assets/3d'


def check() -> None:
    path = ASSETS / 'buildings_manifest.json'
    models = json.loads(path.read_text())['models']
    templates = json.loads(Path(__file__).with_name('asset_buildings.json').read_text())['templates']
    stages = ('foundation', 'frame', 'half', 'complete')
    assert set(models) == {f"building_{t['id']}_{s}" for t in templates for s in stages}
    check_manifest(path)
    for template in templates:
        hashes = set()
        for stage in stages:
            name = f"building_{template['id']}_{stage}"
            entry = models[name]
            assert entry['footprint'] == template['footprint'], name
            assert entry['template'] == template['id'] and entry['stage'] == stage, name
            assert entry['source_status'] == ('connected' if template['connected'] else 'inert_concept'), name
            assert 0 < entry['triangles'] <= 6500 and len(entry['materials']) <= 9, name
            model_path = ASSETS / entry['file']
            hashes.add(hashlib.sha256(model_path.read_bytes()).hexdigest())
            vertices = positions(model_path)
            width, depth = entry['footprint']
            assert all(-width / 2 - .01 <= p[0] <= width / 2 + .01 and
                       -depth / 2 - .01 <= p[2] <= depth / 2 + .01 and p[1] >= -.00001
                       for p in vertices), name
            document, binary = read_glb(model_path)
            assert not document.get('skins') and not document.get('animations'), name
            if stage == 'complete' and template['id'] == 'well':
                assert 'thatch' in entry['materials'] and entry['height'] >= 2.35, name
            if stage == 'complete' and template['id'] == 'hut':
                for mesh in document['meshes']:
                    for primitive in mesh['primitives']:
                        verts = accessor(document, binary, primitive['attributes']['POSITION'])
                        indices = accessor(document, binary, primitive['indices'])
                        for i in range(0, len(indices), 3):
                            triangle = [verts[indices[i + n][0]] for n in range(3)]
                            if min(p[1] for p in triangle) < 1.6:
                                continue
                            signs = [triangle[n][0] * triangle[(n + 1) % 3][2] -
                                     triangle[n][2] * triangle[(n + 1) % 3][0] for n in range(3)]
                            assert not (abs(sum(signs)) > .00001 and
                                        (min(signs) >= -.00001 or max(signs) <= .00001)), 'blocked smoke throat'
            if stage == 'complete' and template['id'] == 'log_house':
                # The side window is an actual gap in log geometry, not a painted dark box.
                trunk = next(i for i, mat in enumerate(document['materials']) if mat['name'] == 'trunk')
                samples = [p for mesh in document['meshes'] for primitive in mesh['primitives']
                           if primitive['material'] == trunk
                           for p in accessor(document, binary, primitive['attributes']['POSITION'])]
                assert not any(abs(p[0] - 1.48) < .16 and abs(p[2]) < .30 and 1.39 < p[1] < 1.65 for p in samples)
        assert len(hashes) == 4, template['id']
    print(f'PASS L2: {len(templates)} families × 4 distinct stages; stable IDs, finite bounded meshes, budget and open window')


if __name__ == '__main__':
    check()
