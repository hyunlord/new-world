"""Focused exported skin regression: <=3 local weights, finite grounded loops.

blender -b --factory-startup -P test_skin.py -- CANDIDATE.glb SUBJECT
"""
import math
import sys
from pathlib import Path

import bpy

source, subject = sys.argv[sys.argv.index("--")+1:]
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
bpy.ops.import_scene.gltf(filepath=str(Path(source).resolve()))
mesh = bpy.data.objects[subject]
arm = next(o for o in bpy.context.scene.objects if o.type == "ARMATURE")
load = bpy.data.objects.get("CarryLoad")
if "Root" in arm.data.bones:
    assert len(arm.data.bones) == 12
    assert arm.data.bones["Tail"].parent.name == "Root"
height = max(v.co.z for v in mesh.data.vertices)
assert max(len(v.groups) for v in mesh.data.vertices) <= 3, "Skin has more than3 influences"
for vertex in mesh.data.vertices:
    assert abs(sum(g.weight for g in vertex.groups)-1) < 1e-6
    for group in vertex.groups:
        name = mesh.vertex_groups[group.group].name
        if "Root" in arm.data.bones and vertex.co.z < .25*(arm.data.bones["Root"].head_local.z/.65):
            assert name not in ("Root", "Neck", "Head", "Tail"), (vertex.index, name, "hoof leakage")
        if "hips" in arm.data.bones and vertex.co.z < .25*height:
            assert "arm_" not in name and not name.startswith("hand_"), (vertex.index, name, "low limb leakage")
        if abs(vertex.co.x) > .025*height:
            opposite = name.endswith(("_L", ".L")) if vertex.co.x < 0 else name.endswith(("_R", ".R"))
            assert not opposite, (vertex.index, name)
objects = [arm]+([load] if load else [])
tracks = {track.name: track.strips[0].action for track in arm.animation_data.nla_tracks}
for obj in objects:
    for track in obj.animation_data.nla_tracks:
        track.mute = True
for name in tracks:
    for obj in objects:
        own = next(track for track in obj.animation_data.nla_tracks if track.name == name)
        obj.animation_data.action = own.strips[0].action
    frames = [1, 5, 9, 13, 17, 21, 25, 29, 33]
    poses = []
    for frame in frames:
        bpy.context.scene.frame_set(frame)
        bpy.context.view_layer.update()
        if load:
            expected = 1 if name == "carry" else 1e-6
            assert all(abs(value-expected) < 1e-7 for value in load.scale), (name, frame, "load visibility")
        evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
        points = [evaluated.matrix_world @ v.co for v in evaluated.data.vertices]
        assert all(math.isfinite(c) for point in points for c in point)
        floor = min(point.z for point in points)
        assert abs(floor) < .003, (name, frame, "floor", floor)
        poses.append(points)
    drift = max((a-b).length for a, b in zip(poses[0], poses[-1]))
    assert drift < 1e-4, (name, "loop seam", drift)
    print("M3D2B_CLIP_FINITE_GROUNDED_LOOP_PASS", subject, name, drift)
print("M3D2B_EXPORTED_SKIN_PASS", subject)
