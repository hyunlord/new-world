# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: blender -b --python-exit-code 1 -P tools/3d/blender/tripo_rig.py -- woman
"""Rig processed Tripo meshes, preserving shared bone names and procedural clips.

Run with Blender --background --python tools/3d/blender/tripo_rig.py -- woman 0
(or goat; omit the LOD to process all three). Inputs must already be normalised.
"""
import os
import sys

import bpy
from mathutils import Quaternion, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, '..', 'chars', 'blender'))
import common as C
import rig
import topology


def build(subject: str, lod: int) -> str:
    """Bind one processed mesh and overwrite its GLB with the shared animated rig."""
    assert subject in ('woman', 'goat') and lod in (0, 1, 2)
    stem = f'tripo_{subject}_lod{lod}'
    path = os.path.join(C.MODELS_DIR, stem + '.glb')
    C.reset_scene()
    bpy.ops.import_scene.gltf(filepath=path)
    meshes = [o for o in bpy.data.objects if o.type == 'MESH']
    assert len(meshes) == 1, f'{stem}: expected one baked body mesh'
    mesh = meshes[0]
    mesh.name = subject
    bpy.ops.object.select_all(action='DESELECT')
    mesh.select_set(True)
    bpy.context.view_layer.objects.active = mesh
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    # Re-running uses the original rest mesh, never a previously evaluated pose.
    for modifier in list(mesh.modifiers):
        if modifier.type == 'ARMATURE':
            mesh.modifiers.remove(modifier)
    mesh.parent = None
    mesh.vertex_groups.clear()
    topology.weld(mesh)
    topology.debris(mesh)
    height = max(v.co.z for v in mesh.data.vertices)
    quadruped = subject == 'goat'
    arm = rig.skeleton(topology.landmark(mesh, True).z / .75 if quadruped else height, quadruped)
    if not quadruped:
        bpy.context.view_layer.objects.active = arm
        bpy.ops.object.mode_set(mode='EDIT')
        # Tripo's T-pose requires a horizontal bind pose; names stay shared.
        for side, sign in (('L', 1), ('R', -1)):
            for bone_name, start, end in (('upper_arm', .12, .29), ('lower_arm', .29, .43), ('hand', .43, .49)):
                bone = arm.data.edit_bones[f'{bone_name}_{side}']
                bone.head = (sign * start * height, 0, .79 * height)
                bone.tail = (sign * end * height, 0, .79 * height)
        bpy.ops.object.mode_set(mode='OBJECT')
    rig.bind(mesh, arm)
    names = rig.clips(arm, quadruped, mesh)
    if not quadruped:
        # Lower T-pose arms in every active clip while retaining its shared motion.
        for track in arm.animation_data.nla_tracks:
            arm.animation_data.action = track.strips[0].action
            for frame in (1, 8, 16, 23, 31):
                bpy.context.scene.frame_set(frame)
                for side, sign in (('L', 1), ('R', -1)):
                    bone = arm.pose.bones[f'upper_arm_{side}']
                    basis = bone.bone.matrix_local.to_quaternion()
                    lower = basis.inverted() @ Quaternion(Vector((0, 1, 0)), sign * 1.15) @ basis
                    bone.rotation_euler = (lower @ bone.rotation_euler.to_quaternion()).to_euler('XYZ')
                    bone.keyframe_insert('rotation_euler', frame=frame)
    arm.animation_data.action = None
    bpy.context.scene.frame_set(1)
    assert len(arm.data.bones) == (11 if quadruped else 17)
    assert set(names) == ({'idle', 'walk', 'graze', 'lie_down'} if quadruped else {'idle', 'walk', 'work', 'carry', 'sleep'})
    bpy.ops.object.select_all(action='DESELECT')
    mesh.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    opts = dict(C.GLTF_COMMON)
    opts.update(export_image_format='AUTO', export_tangents=True,
                export_animations=True, export_skins=True, export_morph=False,
                export_animation_mode='ACTIONS', export_force_sampling=True,
                export_frame_step=1, export_reset_pose_bones=True,
                export_anim_single_armature=True)
    bpy.ops.export_scene.gltf(filepath=path, **opts)
    print(f'[tripo_rig] {stem}: bones={len(arm.data.bones)}, vertices={len(mesh.data.vertices)}, weights={mesh["weight_method"]}')
    return path


if __name__ == '__main__':
    args = C.script_args()
    assert args and args[0] in ('woman', 'goat'), 'Expected woman|goat [0|1|2]'
    for level in ([int(args[1])] if len(args) > 1 else range(3)):
        build(args[0], level)
