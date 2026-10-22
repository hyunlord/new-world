# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by Blender's build_b_model.py.
"""Small explicit pose clips on the retained source rig (at most five key times)."""

from __future__ import annotations

import math

import bpy
from mathutils import Quaternion, Vector


def world_rotation(
    bone: bpy.types.PoseBone, axis: tuple[float, float, float], degrees: float
) -> None:
    """Express an armature-space rotation in the bone's local rest basis."""
    rest = bone.bone.matrix_local.to_quaternion()
    bone.rotation_mode = "QUATERNION"
    bone.rotation_quaternion = (
        rest.inverted() @ Quaternion(Vector(axis), math.radians(degrees)) @ rest
    )


def clips(rig: bpy.types.Object) -> list[tuple[str, list[int]]]:
    """Replace imported animations with deliberately bounded authored clips."""
    rig.animation_data_clear()
    for action in list(bpy.data.actions):
        bpy.data.actions.remove(action)
    result = []
    for name, times in [
        ("idle", [1, 13, 25]),
        ("walk", [1, 7, 13, 19, 25]),
        ("two_hand_chop", [1, 7, 13, 19, 25]),
        ("carry_walk", [1, 7, 13, 19, 25]),
        ("sleep", [1]),
    ]:
        action = bpy.data.actions.new(f"B_{name}")
        rig.animation_data_create()
        rig.animation_data.action = action
        for frame in times:
            for bone in rig.pose.bones:
                bone.rotation_mode = "QUATERNION"
                bone.rotation_quaternion = (1, 0, 0, 0)
                bone.location = (0, 0, 0)
                bone.scale = (1, 1, 1)
            world_rotation(rig.pose.bones["arm-left"], (0, 1, 0), 70)
            world_rotation(rig.pose.bones["arm-right"], (0, 1, 0), -70)
            phase = math.cos((frame - 1) / 24 * math.tau)
            if name in ("walk", "carry_walk"):
                world_rotation(rig.pose.bones["leg-left"], (1, 0, 0), phase * 24)
                world_rotation(rig.pose.bones["leg-right"], (1, 0, 0), -phase * 24)
            if name == "walk":
                for side, sign in [("left", 1), ("right", -1)]:
                    bone = rig.pose.bones[f"arm-{side}"]
                    base = bone.rotation_quaternion.copy()
                    rest = bone.bone.matrix_local.to_quaternion()
                    bone.rotation_quaternion = (
                        rest.inverted()
                        @ Quaternion(Vector((1, 0, 0)), math.radians(phase * sign * 14))
                        @ rest
                    ) @ base
            if name == "carry_walk":
                world_rotation(rig.pose.bones["arm-left"], (0, 1, 0), -45)
                world_rotation(rig.pose.bones["arm-right"], (0, 1, 0), 45)
            if name == "two_hand_chop":
                angle = -50 + 120 * (1 - phase) / 2
                world_rotation(rig.pose.bones["arm-left"], (0, 1, 0), angle)
                world_rotation(rig.pose.bones["arm-right"], (0, 1, 0), -angle)
            if name == "idle":
                world_rotation(rig.pose.bones["head"], (0, 0, 1), phase * 1.2)
            if name == "sleep":
                world_rotation(rig.pose.bones["root"], (0, 1, 0), -90)
                rig.pose.bones["root"].location = (0, 0, 0)
            for bone in rig.pose.bones:
                bone.keyframe_insert(
                    "rotation_quaternion", frame=frame, group=bone.name
                )
                bone.keyframe_insert("location", frame=frame, group=bone.name)
        action.use_fake_user = True
        result.append((action.name, times))
    rig.animation_data.action = bpy.data.actions["B_idle"]
    bpy.context.scene.frame_set(1)
    return result
