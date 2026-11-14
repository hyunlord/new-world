"""Reweight and animate an existing B GLB without rebuilding its mesh or atlas.

blender -b --factory-startup -P reanimate.py -- INPUT.glb NEW_OUTPUT SUBJECT
"""
import hashlib
import json
import sys
from pathlib import Path

import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from motion import clips
from topology import weld
from weights import fit, heat


def geometry(mesh: bpy.types.Object) -> str:
    """Bind the source vertex positions, polygon topology and loop UV coordinates."""
    values = ([tuple(v.co) for v in mesh.data.vertices],
              [tuple(p.vertices) for p in mesh.data.polygons],
              [[tuple(uv.uv) for uv in layer.data] for layer in mesh.data.uv_layers])
    return hashlib.sha256(repr(values).encode()).hexdigest()


def load_tracks(load: bpy.types.Object, arm: bpy.types.Object) -> None:
    """Keep the original package geometry, following both hands only during carry."""
    load.animation_data_create()
    for arm_track in arm.animation_data.nla_tracks:
        name = arm_track.name
        action = bpy.data.actions.new(name+"_load")
        load.animation_data.action = action
        arm.animation_data.action = arm_track.strips[0].action
        for frame in range(1, 34):
            bpy.context.scene.frame_set(frame)
            bpy.context.view_layer.update()
            load.location = (arm.pose.bones["hand_L"].tail+arm.pose.bones["hand_R"].tail)/2
            load.scale = (1, 1, 1) if name == "carry" else (1e-6, 1e-6, 1e-6)
            load.keyframe_insert("location", frame=frame)
            load.keyframe_insert("scale", frame=frame)
        track = load.animation_data.nla_tracks.new()
        track.name = name
        track.strips.new(name, 1, action)
        track.mute = True
    load.animation_data.action = None
    load.scale = (1e-6, 1e-6, 1e-6)
    arm.animation_data.action = None


def main() -> None:
    """Preserve source geometry and emit a new independently reviewable attempt."""
    source, destination, subject = sys.argv[sys.argv.index("--")+1:]
    out = Path(destination).resolve()
    assert not out.exists(), "Output directory must be new"
    out.mkdir(parents=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    bpy.ops.import_scene.gltf(filepath=str(Path(source).resolve()))
    mesh = bpy.data.objects[subject]
    arm = next(o for o in bpy.context.scene.objects if o.type == "ARMATURE")
    load = bpy.data.objects.get("CarryLoad")
    objects = [mesh, arm]+([load] if load else [])
    source_hash = geometry(mesh)
    source_vertices = len(mesh.data.vertices)
    weld(mesh)
    before = geometry(mesh)
    for obj in objects:
        obj.animation_data_clear()
    for bone in arm.pose.bones:
        bone.matrix_basis.identity()
    bpy.context.view_layer.update()
    fit(arm, max(v.co.z for v in mesh.data.vertices))
    heat(mesh, arm)
    names = clips(arm, mesh)
    if load:
        load_tracks(load, arm)
    assert before == geometry(mesh), "Geometry or atlas UV changed"
    for obj in objects:
        if obj.animation_data:
            obj.animation_data.action = None
    for bone in arm.pose.bones:
        bone.matrix_basis.identity()
    bpy.context.view_layer.update()
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
        if obj.animation_data:
            for track in obj.animation_data.nla_tracks:
                track.mute = False
    bpy.ops.export_scene.gltf(filepath=str(out/f"{subject}.glb"), export_format="GLB", use_selection=True,
                              export_animation_mode="NLA_TRACKS", export_force_sampling=True,
                              export_optimize_animation_size=False, export_optimize_animation_keep_anim_object=True)
    receipt = {"subject": subject, "source": str(Path(source).resolve()), "clips": names,
               "welded_geometry_uv_sha256": before,
               "max_influences": max(len(v.groups) for v in mesh.data.vertices),
               "weights": "fitted-rig heat, max3 normalized same-side",
               "source_geometry_uv_sha256": source_hash, "seam_weld": True,
               "source_vertices": source_vertices, "working_vertices": len(mesh.data.vertices),
               "motion": "world-directed analytic two-link arms, side sleep, folded-under-body quadruped rest"}
    (out/"receipt.json").write_text(json.dumps(receipt, indent=2)+"\n")
    print("M3D2B_REANIMATE_DONE", subject)


if __name__ == "__main__":
    main()
