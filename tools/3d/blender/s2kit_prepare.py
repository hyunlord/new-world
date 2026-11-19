"""Decimate the owner-supplied meshes while retaining the source UV map."""
from pathlib import Path
import sys

import bpy

root = Path(sys.argv[sys.argv.index('--') + 1])
sources = root / 'tripo-s2'
dest = root / 's2kit' / 'prepared'
dest.mkdir(exist_ok=True)

for source in sorted(sources.glob('*.glb')):
    key = source.stem[3:5]
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(source), merge_vertices=True)
    model, = [obj for obj in bpy.context.scene.objects if obj.type == 'MESH']
    bpy.context.view_layer.objects.active = model
    model.select_set(True)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    top = 8000 if key.startswith('p') else 6000 if key.startswith('h') else 2000 if key == 'x5' else 4000
    budgets = [top, 2500, 800] if key.startswith('p') else [top, 1500 if key.startswith('h') else int(top * .30)]
    for budget in budgets:
        mod = model.modifiers.new('Triangle budget', 'DECIMATE')
        mod.ratio = (budget - 12) / len(model.data.polygons)
        mod.use_collapse_triangulate = True
        bpy.ops.object.modifier_apply(modifier=mod.name)
        assert len(model.data.polygons) <= budget, (key, budget, len(model.data.polygons))
        print(f'S2_MESH {key} {len(model.data.polygons)}', flush=True)
        bpy.ops.export_scene.gltf(filepath=str(dest / f'{key}-{budget}.glb'), use_selection=True)
