"""Synthetic local fixture only; does not establish AI asset quality.

blender -b --factory-startup -P tools/3d/chars/blender/self_test.py -- EXTERNAL_OUTPUT
"""
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
import prepare

out = Path(sys.argv[sys.argv.index("--")+1]).resolve()
out.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
bpy.ops.mesh.primitive_uv_sphere_add(segments=48, ring_count=32, location=(0, 0, .85))
mesh = bpy.context.object
mesh.scale = (.3, .2, .85)
bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
material = bpy.data.materials.new("Synthetic ochre")
material.use_nodes = True
image = bpy.data.images.new("UV checker", width=64, height=64)
palette = [(1, .02, .02, 1), (.02, 1, .02, 1), (.02, .02, 1, 1), (1, 1, .02, 1)]
image.pixels[:] = [c for y in range(64) for x in range(64) for c in palette[(x//32)+2*(y//32)]]
texture = material.node_tree.nodes.new("ShaderNodeTexImage")
texture.image = image
texture.interpolation = "Closest"
material.node_tree.links.new(texture.outputs["Color"], material.node_tree.nodes.get("Principled BSDF").inputs["Base Color"])
mesh.data.materials.append(material)
source = out/"synthetic_raw.glb"
bpy.ops.export_scene.gltf(filepath=str(source), export_format="GLB", use_selection=True)
mesh = prepare.cleanup(source, "man")
old_uv = mesh.data.uv_layers.active
expected = []
for polygon in mesh.data.polygons:
    uv = sum((old_uv.data[i].uv for i in polygon.loop_indices), start=Vector((0, 0)))/len(polygon.loop_indices)
    expected.append((int(uv.x*64)%64//32)+2*(int(uv.y*64)%64//32))
prepare.atlas(mesh, out)
baked = bpy.data.images.get("Atlas512")
pixels = list(baked.pixels[:])
correct = 0
for polygon, colour in zip(mesh.data.polygons, expected):
    uv = sum((mesh.data.uv_layers.active.data[i].uv for i in polygon.loop_indices), start=Vector((0, 0)))/len(polygon.loop_indices)
    index = (min(511, int(uv.y*512))*512+min(511, int(uv.x*512)))*4
    rgb = pixels[index:index+3]
    correct += max(abs(rgb[i]-palette[colour][i]) for i in range(3)) < .15
assert correct/len(expected) > .85, f"Source UV correspondence lost: {correct}/{len(expected)}"
print("M3D2_CHECKER_UV_PASS", correct, len(expected))
sys.argv = ["blender", "--", str(source), str(out/"synthetic_man"), "man"]
prepare.main()
assert (out/"synthetic_man"/"man.glb").stat().st_size > 0
assert (out/"synthetic_man"/"contact_sheet.png").stat().st_size > 0
human_actions = [strip.action for track in bpy.data.objects["Humanoid"].animation_data.nla_tracks for strip in track.strips]
sys.argv = ["blender", "--", str(source), str(out/"synthetic_child"), "child"]
prepare.main()
arm = bpy.data.objects["Humanoid"]
mesh = bpy.data.objects["child"]
for action in human_actions:
    arm.animation_data.action = action
    for frame in (1, 8, 16, 23, 31):
        bpy.context.scene.frame_set(frame)
        evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
        assert all(math.isfinite(c) for v in evaluated.data.vertices for c in v.co)
        if action.name == "sleep":
            bottom = min((evaluated.matrix_world @ v.co).z for v in evaluated.data.vertices)
            assert abs(bottom) < .01, (action.name, frame, bottom)
print("M3D2_SHARED_HUMAN_ACTION_TRANSFER_PASS")
for track, action in zip(arm.animation_data.nla_tracks, human_actions):
    track.strips[0].action = action
shared = out/"shared_human_clips"
shared.mkdir(exist_ok=True)
prepare.capture(mesh, arm, shared)
sys.argv = ["blender", "--", str(source), str(out/"synthetic_goat"), "goat"]
prepare.main()
assert (out/"synthetic_goat"/"goat.glb").stat().st_size > 0
assert (out/"synthetic_goat"/"contact_sheet.png").stat().st_size > 0
collision = out/"collision"
collision.mkdir(exist_ok=True)
raw = collision/"man.glb"
original = source.read_bytes()
raw.write_bytes(original)
sys.argv = ["blender", "--", str(raw), str(collision), "man"]
try:
    prepare.main()
except AssertionError as error:
    assert str(error) == "Raw input and export destination must differ"
else:
    raise AssertionError("Raw collision was admitted")
assert raw.read_bytes() == original
print("M3D2_RAW_COLLISION_REJECTED_PASS")
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
bpy.ops.mesh.primitive_cube_add()
tiny = out/"tiny.glb"
bpy.ops.export_scene.gltf(filepath=str(tiny), export_format="GLB", use_selection=True)
try:
    prepare.cleanup(tiny, "man")
except AssertionError as error:
    assert "Triangle budget failed: 12 outside 1500..3000" == str(error)
else:
    raise AssertionError("Tiny mesh was admitted")
print("M3D2_TINY_MESH_REJECTED_PASS")
print("M3D2_SYNTHETIC_SELF_TEST_PASS")
