"""Regression: placing interior models must never transform an existing building shell."""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: blender -b --factory-startup -P tools/3d/blender/look2_building_component_test.py
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import asset_buildings as B
import common as C
import look2_building_shell as L


def check() -> None:
    templates = json.loads(B.DATA.read_text())['templates']
    template = next(t for t in templates if t['id'] == 'log_house')
    for kind in ('bedroll', 'forge', 'clay_oven', 'wood_bench', 'stone_wall', 'fence', 'hut'):
        C.reset_scene()
        model = C.Model('component_regression_' + kind)
        L.shell(model, template, 'complete')
        identity = model.bm.verts.layers.int.new('regression_identity')
        expected = {}
        for i, vertex in enumerate(model.bm.verts, 1):
            vertex[identity] = i
            expected[i] = tuple(vertex.co)
        B.component(model, (kind, -.5, .5, (1, 1), 3), 'complete')
        actual = {vertex[identity]: tuple(vertex.co) for vertex in model.bm.verts if vertex[identity]}
        assert actual == expected, f'{kind} transformed pre-existing shell vertices'
        model.bm.free()
    print('PASS: seven component builders preserve every pre-existing shell coordinate')


if __name__ == '__main__':
    check()
