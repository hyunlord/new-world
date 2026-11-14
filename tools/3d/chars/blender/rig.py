"""Shared template skeletons and procedural clips; Blender Z-up/front -Y."""
import math

import bpy
from mathutils import Vector


def skeleton(height: float, quadruped: bool) -> bpy.types.Object:
    """Create identical named/proportioned templates within each skeleton family."""
    arm = bpy.data.objects.new("Quadruped" if quadruped else "Humanoid", bpy.data.armatures.new("Skeleton"))
    bpy.context.collection.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    arm.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    specs = [("hips", (0, 0, .50), (0, 0, .58), None),
             ("spine", (0, 0, .58), (0, 0, .76), "hips"),
             ("chest", (0, 0, .70), (0, 0, .80), "spine"),
             ("neck", (0, 0, .80), (0, 0, .86), "chest"),
             ("head", (0, 0, .86), (0, 0, 1), "neck")]
    for side, sign in (("L", 1), ("R", -1)):
        specs += [(f"upper_arm_{side}", (sign*.12, 0, .76), (sign*.25, 0, .62), "chest"),
                  (f"lower_arm_{side}", (sign*.25, 0, .62), (sign*.34, 0, .47), f"upper_arm_{side}"),
                  (f"hand_{side}", (sign*.34, 0, .47), (sign*.37, 0, .42), f"lower_arm_{side}"),
                  (f"upper_leg_{side}", (sign*.08, 0, .50), (sign*.08, 0, .26), "hips"),
                  (f"lower_leg_{side}", (sign*.08, 0, .26), (sign*.08, 0, .04), f"upper_leg_{side}"),
                  (f"foot_{side}", (sign*.08, 0, .04), (sign*.08, -.10, .02), f"lower_leg_{side}")]
    if quadruped:
        specs = [("Root", (0, .25, .65), (0, -.25, .75), None),
                 ("Neck", (0, -.25, .75), (0, -.47, .95), "Root"),
                 ("Head", (0, -.47, .95), (0, -.70, .95), "Neck")]
        for side, x in (("L", .18), ("R", -.18)):
            for end, y in (("Front", -.30), ("Back", .35)):
                specs += [(f"{end}Upper.{side}", (x, y, .70), (x, y, .35), "Root"),
                          (f"{end}Lower.{side}", (x, y, .35), (x, y, .02), f"{end}Upper.{side}")]
    for name, head, tail, parent in specs:
        bone = arm.data.edit_bones.new(name)
        bone.head, bone.tail = Vector(head), Vector(tail)
        if parent:
            bone.parent = arm.data.edit_bones[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    arm.scale = (height, height, height)
    bpy.context.view_layer.update()
    return arm


def bind(mesh: bpy.types.Object, arm: bpy.types.Object) -> None:
    """Use Blender automatic heat weights and reject any uncovered vertex."""
    bpy.ops.object.select_all(action="DESELECT")
    mesh.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    bpy.context.view_layer.objects.active = mesh
    bpy.ops.object.vertex_group_normalize_all(lock_active=False)
    assert all(sum(g.weight for g in v.groups) > .99 for v in mesh.data.vertices), "Automatic weights left uncovered vertices"


def clips(arm: bpy.types.Object, quadruped: bool, mesh: bpy.types.Object) -> list[str]:
    """Create reusable one-second loops; sleep/lie_down are held poses."""
    names = ["idle", "walk", "graze", "lie_down"] if quadruped else ["idle", "walk", "work", "carry", "sleep"]
    arm.animation_data_create()
    for name in names:
        action = bpy.data.actions.new(name)
        arm.animation_data.action = action
        for frame in (1, 8, 16, 23, 31):
            wave = math.sin((frame-1)*math.tau/30)
            for bone in arm.pose.bones:
                bone.rotation_mode = "XYZ"
                bone.rotation_euler = (0, 0, 0)
                bone.location = (0, 0, 0)
                limb = "leg" in bone.name or "Upper" in bone.name
                if name in ("walk", "carry") and limb:
                    bone.rotation_euler.x = wave*.42 * (-1 if bone.name.endswith(("_R", ".R")) else 1)
                if name in ("work", "carry") and "arm" in bone.name:
                    bone.rotation_euler.x = -.9 + (wave*.6 if name == "work" else 0)
                if name == "graze" and bone.name == "Neck":
                    bone.rotation_euler.x = .8 + wave*.08
                if name == "lie_down" and "Upper" in bone.name:
                    bone.rotation_euler.x = 1.1
                if name == "lie_down" and "Lower" in bone.name:
                    bone.rotation_euler.x = -1.8
                if name in ("sleep", "lie_down") and bone.parent is None:
                    bone.rotation_euler.x = math.pi/2 if name == "sleep" else 0
                if name == "idle" and ("spine" in bone.name or bone.name == "Neck"):
                    bone.rotation_euler.x = wave*.025
                bone.keyframe_insert("rotation_euler", frame=frame)
                bone.keyframe_insert("location", frame=frame)
            if name in ("sleep", "lie_down"):
                bpy.context.view_layer.update()
                evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
                ground = min((evaluated.matrix_world @ v.co).z for v in evaluated.data.vertices)
                root = next(b for b in arm.pose.bones if b.parent is None)
                offset = arm.matrix_world.to_3x3().inverted() @ Vector((0, 0, -ground))
                root.location += root.bone.matrix_local.to_3x3().inverted() @ offset
                root.keyframe_insert("location", frame=frame)
        track = arm.animation_data.nla_tracks.new()
        track.name = name
        track.strips.new(name, 1, action)
        track.mute = True
    return names
