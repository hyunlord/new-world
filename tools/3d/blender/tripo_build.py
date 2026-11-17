# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: blender -b -t 4 -P tools/3d/blender/tripo_build.py -- RAW_DIR EVIDENCE_DIR [subject]
"""Normalize four licensed external Tripo sources, decimate and bake game LODs."""
import json
import sys
import time
from pathlib import Path
from typing import Final

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as C
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'chars/blender'))
from topology import landmark, weld

SOURCES: Final = {
    'woman': ('neolithic villager woman 3d model.glb', 1.6, (4000, 1500, 500)),
    'goat': ('domestic goat 3d model.glb', .75, (4000, 1500, 500)),
    'house': ('neolithic log house 3d model.glb', 4., (8000, 3000, 1000)),
    'bloomery': ('iron age bloomery furnace 3d model.glb', 1.4, (4000, 1500, 500)),
}


def bounds(mesh: bpy.types.Object) -> tuple[Vector, Vector]:
    """Local mesh bounds after applying imported transforms."""
    return tuple(Vector(tuple(fn(v.co[i] for v in mesh.data.vertices) for i in range(3)))
                 for fn in (min, max))


def bake(source: bpy.types.Object, mesh: bpy.types.Object, prefix: Path) -> None:
    """Project original high-poly colour and tangent normals onto new UVs."""
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = 1
    scene.render.bake.use_pass_direct = False
    scene.render.bake.use_pass_indirect = False
    scene.render.bake.use_pass_color = True
    scene.render.bake.use_selected_to_active = True
    scene.render.bake.use_cage = True
    scene.render.bake.cage_extrusion = .06
    scene.render.bake.max_ray_distance = .12
    bpy.ops.object.select_all(action='DESELECT')
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = mesh
    for layer in list(mesh.data.uv_layers):
        mesh.data.uv_layers.remove(layer)
    mesh.data.uv_layers.new(name='UVMap')
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.uv.smart_project(angle_limit=1.151917, island_margin=.006)
    bpy.ops.object.mode_set(mode='OBJECT')
    mat = bpy.data.materials.new(prefix.name)
    mat.use_nodes = True
    mesh.data.materials.clear()
    mesh.data.materials.append(mat)
    for polygon in mesh.data.polygons:
        polygon.material_index = 0
        polygon.use_smooth = True
    shader = mat.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Roughness'].default_value = .85
    textures = {}
    size = 512 if prefix.name.endswith('lod2') else (2048 if 'house' in prefix.name else 1024)
    source.select_set(True)
    for label, kind in (('albedo', 'DIFFUSE'), ('normal', 'NORMAL')):
        image = bpy.data.images.new(prefix.name+'_'+label, width=size, height=size, alpha=False)
        if label == 'normal':
            image.colorspace_settings.name = 'Non-Color'
        node = mat.node_tree.nodes.new('ShaderNodeTexImage')
        node.image = image
        mat.node_tree.nodes.active = node
        bpy.ops.object.bake(type=kind, margin=8)
        image.filepath_raw = str(prefix.with_name(prefix.name+'_'+label+'.png'))
        image.file_format = 'PNG'
        image.save()
        textures[label] = node
    mat.node_tree.links.new(textures['albedo'].outputs['Color'], shader.inputs['Base Color'])
    normal = mat.node_tree.nodes.new('ShaderNodeNormalMap')
    mat.node_tree.links.new(textures['normal'].outputs['Color'], normal.inputs['Color'])
    mat.node_tree.links.new(normal.outputs['Normal'], shader.inputs['Normal'])
    source.select_set(False)


def build(subject: str, paths: tuple[Path, Path]) -> None:
    """Process one source sequentially, retaining raw geometry only in memory."""
    started = time.monotonic()
    raw, evidence = paths
    filename, target_size, budgets = SOURCES[subject]
    C.reset_scene()
    bpy.ops.import_scene.gltf(filepath=str(raw / filename))
    meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    assert meshes, filename
    bpy.ops.object.select_all(action='DESELECT')
    for mesh in meshes:
        mesh.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.join()
    source = bpy.context.object
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    lo, hi = bounds(source)
    original = {'triangles': C.tri_count(source), 'bounds': [list(lo), list(hi)],
                'textures': [{'name':im.name, 'size':list(im.size)} for im in bpy.data.images]}
    offset = Vector(((lo.x+hi.x)/2, (lo.y+hi.y)/2, lo.z))
    reference = max(hi.x-lo.x, hi.y-lo.y) if subject == 'house' else hi.z-lo.z
    if subject == 'goat':
        reference = landmark(source, True).z-lo.z
    scale = target_size/reference
    for vertex in source.data.vertices:
        vertex.co = (vertex.co-offset)*scale
    source.name = 'HighPoly'
    source.select_set(False)
    rows = []
    for level, budget in enumerate(budgets):
        mesh = source.copy()
        mesh.data = source.data.copy()
        bpy.context.collection.objects.link(mesh)
        mesh.name = subject
        bpy.context.view_layer.objects.active = mesh
        mesh.select_set(True)
        weld(mesh)
        modifier = mesh.modifiers.new('Triangle budget', 'DECIMATE')
        modifier.ratio = budget/C.tri_count(mesh)
        bpy.ops.object.modifier_apply(modifier=modifier.name)
        triangles = C.tri_count(mesh)
        assert budget*.85 <= triangles <= budget*1.05, (subject, level, triangles, budget)
        prefix = Path(C.MODELS_DIR) / f'tripo_{subject}_lod{level}'
        bake(source, mesh, prefix)
        bpy.ops.object.select_all(action='DESELECT')
        mesh.select_set(True)
        bpy.context.view_layer.objects.active = mesh
        bpy.ops.export_scene.gltf(filepath=str(prefix.with_suffix('.glb')), export_format='GLB',
                                  use_selection=True, export_animations=False, export_tangents=True)
        low, high = bounds(mesh)
        rows.append({'id':prefix.name, 'file':f'models/{prefix.name}.glb', 'triangles':triangles,
                     'height':round(high.z-low.z, 4), 'bounds':[list(low), list(high)],
                     'footprint':[4,4] if subject == 'house' else ([2,2] if subject == 'bloomery' else [1,1]),
                     'materials':[], 'sockets':{}, 'family':'building' if subject == 'house' else 'station',
                     'texture_size':512 if level == 2 else (2048 if subject == 'house' else 1024)})
        bpy.data.objects.remove(mesh, do_unlink=True)
    evidence.mkdir(parents=True, exist_ok=True)
    (evidence / f'{subject}-processing.json').write_text(json.dumps({
        'subject':subject, 'raw':original, 'scale':scale, 'lods':rows,
        'elapsed_seconds':round(time.monotonic()-started, 2)}, indent=2)+'\n')
    if subject in ('house', 'bloomery'):
        manifest = Path(C.MODELS_DIR) / 'tripo_manifest.json'
        entries = json.loads(manifest.read_text())['models'] if manifest.exists() else {}
        for row in rows:
            entries[row['id']] = {key: value for key, value in row.items() if key != 'id'}
        manifest.write_text(json.dumps({'models': dict(sorted(entries.items()))}, indent=1)+'\n')
    print('TRIPO_BUILT', subject, rows, flush=True)


if __name__ == '__main__':
    arguments = sys.argv[sys.argv.index('--')+1:]
    raw_dir, evidence_dir = map(Path, arguments[:2])
    for subject in arguments[2:] or SOURCES:
        assert subject in SOURCES, subject
        build(subject, (raw_dir, evidence_dir))
