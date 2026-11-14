"""Regress imported seam welding and selected-to-active remesh colour transfer.

blender -b --factory-startup -P tools/3d/chars/blender/test_topology_bake.py -- EXTERNAL_OUTPUT
"""
import sys
from pathlib import Path

import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree
from mathutils.geometry import barycentric_transform

sys.path.insert(0, str(Path(__file__).resolve().parent))
from prepare import atlas
from topology import weld

out = Path(sys.argv[sys.argv.index("--")+1]).resolve()
out.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
data = bpy.data.meshes.new("Seam split fixture")
data.from_pydata([(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 0), (1, 0, 0), (0, -1, 0), (9, 9, 9)], [], [(0, 1, 2), (4, 3, 5)])
mesh = bpy.data.objects.new("Seam split fixture", data)
bpy.context.collection.objects.link(mesh)
uv = data.uv_layers.new()
for index, loop in enumerate(uv.data):
    loop.uv = (index/8, index/9)
before = sorted(tuple(loop.uv) for loop in uv.data)
weld(mesh)
assert len(mesh.data.vertices) == 4
assert sorted(tuple(loop.uv) for loop in mesh.data.uv_layers.active.data) == before
assert all(any(v.index in polygon.vertices for polygon in mesh.data.polygons) for v in mesh.data.vertices)
print("M3D2_SEAM_WELD_LOOSE_GEOMETRY_PASS")
bpy.data.objects.remove(mesh, do_unlink=True)
bpy.ops.mesh.primitive_uv_sphere_add(segments=64, ring_count=32)
source = bpy.context.object
source.name = "Checker bake source"
source_data = source.data
source_uvs = [tuple(loop.uv) for loop in source_data.uv_layers.active.data]
material = bpy.data.materials.new("Source checker")
material.use_nodes = True
palette = [(1, .02, .02, 1), (.02, 1, .02, 1), (.02, .02, 1, 1), (1, 1, .02, 1)]
image = bpy.data.images.new("Source checker", width=64, height=64)
image.pixels[:] = [c for y in range(64) for x in range(64) for c in palette[x//32+2*(y//32)]]
texture = material.node_tree.nodes.new("ShaderNodeTexImage")
texture.image = image
texture.interpolation = "Closest"
material.node_tree.links.new(texture.outputs["Color"], material.node_tree.nodes.get("Principled BSDF").inputs["Base Color"])
source.data.materials.append(material)
target = source.copy()
target.data = source.data.copy()
bpy.context.collection.objects.link(target)
bpy.ops.object.select_all(action="DESELECT")
target.select_set(True)
bpy.context.view_layer.objects.active = target
target.data.remesh_voxel_size = .06
bpy.ops.object.voxel_remesh()
target["bake_source"] = source.name
source_data.calc_loop_triangles()
triangles = list(source_data.loop_triangles)
tree = BVHTree.FromPolygons([v.co for v in source_data.vertices], [triangle.vertices for triangle in triangles], all_triangles=True)
source.select_set(True)  # Reproduces the copied-source multi-object edit hazard.
atlas(target, out)
assert [tuple(loop.uv) for loop in source_data.uv_layers.active.data] == source_uvs, "Target unwrap mutated preserved source UV"
pixels = list(bpy.data.images.get("Atlas512").pixels[:])
correct, checked = 0, 0
for polygon in target.data.polygons:
    centre = sum((target.data.vertices[i].co for i in polygon.vertices), start=Vector())/len(polygon.vertices)
    point, normal, index, distance = tree.find_nearest(centre)
    triangle = triangles[index]
    positions = [source_data.vertices[i].co for i in triangle.vertices]
    coords = [Vector((*source_uvs[i], 0)) for i in triangle.loops]
    reference_uv = barycentric_transform(point, *positions, *coords)
    if min(abs(reference_uv.x-.5), abs(reference_uv.y-.5)) < .03:
        continue
    expected = palette[(int(reference_uv.x*64)%64//32)+2*(int(reference_uv.y*64)%64//32)]
    uv = sum((target.data.uv_layers.active.data[i].uv for i in polygon.loop_indices), start=Vector((0, 0)))/len(polygon.loop_indices)
    pixel = (min(511, int(uv.y*512))*512+min(511, int(uv.x*512)))*4
    checked += 1
    correct += max(abs(pixels[pixel+i]-expected[i]) for i in range(3)) < .15
assert checked > 100 and correct/checked > .85, (correct, checked)
print("M3D2_REMESH_SOURCE_UV_PRESERVED_PASS")
print("M3D2_REMESH_SPATIAL_CHECKER_PASS", correct, checked)
