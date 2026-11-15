"""World-directed shared motion avoids imported bone-roll dependent Euler swings."""
import math

import bpy
from mathutils import Quaternion, Vector


def swing(bone: bpy.types.PoseBone, axis: tuple[float, float, float], angle: float) -> None:
    """Rotate about a rest-world axis while retaining the parent's posed motion."""
    basis = bone.bone.matrix_local.to_quaternion()
    bone.rotation_quaternion = basis.inverted() @ Quaternion(Vector(axis), angle) @ basis


def direction(bone: bpy.types.PoseBone, target: Vector) -> None:
    """Aim the bone along a world direction, accounting for the posed parent."""
    rest = bone.bone.matrix_local.to_quaternion()
    desired = (bone.bone.tail_local-bone.bone.head_local).rotation_difference(target) @ rest
    parent = bone.parent
    base = parent.matrix.to_quaternion() @ parent.bone.matrix_local.to_quaternion().inverted() @ rest if parent else rest
    bone.rotation_quaternion = base.inverted() @ desired
    bpy.context.view_layer.update()


def hands(arm: bpy.types.Object, target: Vector, mesh: bpy.types.Object, forward: bool = False) -> None:
    """Solve actual hand-surface centroids, correcting asymmetric rest hand offsets."""
    root = arm.pose.bones["hips"]
    resting = root.rotation_quaternion.angle > .5
    body = root.matrix @ root.bone.matrix_local.inverted()
    for side, sign in (("L", 1), ("R", -1)):
        upper = arm.pose.bones[f"upper_arm_{side}"]
        lower = arm.pose.bones[f"lower_arm_{side}"]
        goal = target.copy()
        goal.x *= sign
        if resting:
            height = max(v.co.z for v in mesh.data.vertices)
            goal.x = 0
            goal = body @ goal
            goal.y += sign*.035*height
            goal.z += .06*height
        wrist = goal.copy()
        group = mesh.vertex_groups[f"hand_{side}"].index
        indices = [v.index for v in mesh.data.vertices if any(g.group == group and g.weight > .4 for g in v.groups)]
        if not indices:
            height = max(v.co.z for v in mesh.data.vertices)
            indices = [v.index for v in mesh.data.vertices
                       if sign*v.co.x > .25*height and .27*height < v.co.z < .43*height]
        assert indices, "No actual hand-surface vertices"
        for _ in range(4):
            start = upper.head.copy()
            delta = wrist-start
            length = min(delta.length, (upper.length+lower.length)*.985)
            axis = delta.normalized()
            along = (upper.length**2-lower.length**2+length**2)/(2*length)
            pole = Vector((-sign*.6, -1, 0)) if resting else Vector((sign, -.1, 0))
            if resting:
                pole = Vector((0, -1, .1)) if forward else Vector((0, -.5, 1))
            pole = (pole-axis*pole.dot(axis)).normalized()
            elbow = start+axis*along+pole*math.sqrt(max(0, upper.length**2-along**2))
            direction(upper, elbow-start)
            direction(lower, start+axis*length-elbow)
            palm = Vector((0, -1, -.15))
            direction(arm.pose.bones[f"hand_{side}"], body.to_3x3() @ palm if resting else palm)
            evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
            centre = sum((evaluated.data.vertices[i].co for i in indices), Vector())/len(indices)
            wrist += goal-centre


def ground(arm: bpy.types.Object, mesh: bpy.types.Object, torso: bool = False) -> None:
    """Place the lowest evaluated body vertex exactly on the floor."""
    bpy.context.view_layer.update()
    evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
    height = max(v.co.z for v in mesh.data.vertices)
    indices = [v.index for v in mesh.data.vertices
               if not torso or (.30*height < v.co.z < .72*height and abs(v.co.x) < .23*height)]
    floor = min((evaluated.matrix_world @ evaluated.data.vertices[i].co).z for i in indices)
    root = next(b for b in arm.pose.bones if b.parent is None)
    root.location += root.bone.matrix_local.to_3x3().inverted() @ Vector((0, 0, -floor))
    bpy.context.view_layer.update()


def clips(arm: bpy.types.Object, mesh: bpy.types.Object) -> list[str]:
    """Generate looped human gestures and upright folded quadruped resting poses."""
    quad = "Root" in arm.pose.bones
    names = ["idle", "walk", "graze", "lie_down"] if quad else ["idle", "walk", "work", "carry", "sleep"]
    height = max(v.co.z for v in mesh.data.vertices) if not quad else arm.data.bones["Root"].head_local.z/.65
    root = arm.pose.bones["Root" if quad else "hips"]
    arm.animation_data_create()
    for name in names:
        action = bpy.data.actions.new(name)
        arm.animation_data.action = action
        for frame in range(1, 34):
            bpy.context.scene.frame_set(frame)
            t = (frame-1)/32
            wave = math.sin(t*math.tau)
            for bone in arm.pose.bones:
                bone.rotation_mode = "QUATERNION"
                bone.rotation_quaternion = (1, 0, 0, 0)
                bone.location = (0, 0, 0)
            if name == "sleep":
                basis = root.bone.matrix_local.to_quaternion()
                side = Quaternion(Vector((0, 1, 0)), math.pi/2) @ Quaternion(Vector((0, 0, 1)), math.radians(-35))
                root.rotation_quaternion = basis.inverted() @ side @ basis
                swing(arm.pose.bones["spine"], (0, 1, 0), math.radians(-12))
                swing(arm.pose.bones["neck"], (0, 1, 0), math.radians(20))
                swing(arm.pose.bones["head"], (0, 1, 0), math.radians(10))
            if name == "idle":
                swing(arm.pose.bones["Neck" if quad else "chest"], (1, 0, 0), wave*.018)
                root.location.x = wave*height*.007
            for side, sign in (("L", 1), ("R", -1)):
                phase = wave*sign
                if quad:
                    for end, gait in (("Front", 1), ("Back", -1)):
                        upper = arm.pose.bones[f"{end}Upper.{side}"]
                        lower = arm.pose.bones[f"{end}Lower.{side}"]
                        if name == "walk":
                            swing(upper, (1, 0, 0), phase*.38*gait)
                            swing(lower, (1, 0, 0), max(0, -phase*gait)*.65)
                        if name == "lie_down":
                            fold = 1 if end == "Front" else -1
                            swing(upper, (1, 0, 0), fold*1.45)
                            swing(lower, (1, 0, 0), -fold*2.70)
                else:
                    upper = arm.pose.bones[f"upper_leg_{side}"]
                    lower = arm.pose.bones[f"lower_leg_{side}"]
                    if name in ("walk", "carry"):
                        swing(upper, (1, 0, 0), phase*.38)
                        swing(lower, (1, 0, 0), max(0, -phase)*.6)
                    if name == "walk":
                        swing(arm.pose.bones[f"upper_arm_{side}"], (1, 0, 0), -phase*.3)
                        swing(arm.pose.bones[f"lower_arm_{side}"], (1, 0, 0), -.15)
                    if name == "sleep":
                        swing(upper, (1, 0, 0), -.4)
                        swing(lower, (1, 0, 0), .75)
                        swing(arm.pose.bones[f"upper_arm_{side}"], (1, 0, 0), -.35)
                        swing(arm.pose.bones[f"lower_arm_{side}"], (1, 0, 0), -.5)
            bpy.context.view_layer.update()
            if name in ("work", "carry"):
                z = .67 if name == "carry" else .61+.15*math.sin(math.tau*t)
                depth = -.16
                if name == "work":
                    group = mesh.vertex_groups["head"].index
                    bottom = min(v.co.z for v in mesh.data.vertices
                                 if any(g.group == group and g.weight > .7 for g in v.groups))/height
                    if bottom < .74:
                        depth = -.22  # Oversized-head morph: extend the forearms rather than fold tightly.
                        z = min(z, bottom-.04)
                hands(arm, Vector((.055, depth, z))*height, mesh)
            if name == "sleep":
                head_group = mesh.vertex_groups["head"].index
                head_bottom = min(v.co.z for v in mesh.data.vertices
                                  if any(g.group == head_group and g.weight > .7 for g in v.groups))/height
                hand_height = min(.70, head_bottom-.14) if head_bottom < .74 else .70
                hands(arm, Vector((.055, -.18, hand_height))*height, mesh, forward=head_bottom < .74)
            if quad:
                swing(arm.pose.bones["Tail"], (1, 0, 0), .6 if name == "lie_down" else wave*.08)
            if name == "graze":
                swing(arm.pose.bones["Neck"], (1, 0, 0), .9+wave*.04)
                swing(arm.pose.bones["Head"], (1, 0, 0), .2)
            ground(arm, mesh, torso=name == "sleep")
            if name == "sleep":
                # A larger target head needs less downward tilt while hips stay grounded.
                for _ in range(30):
                    evaluated = mesh.evaluated_get(bpy.context.evaluated_depsgraph_get())
                    head_floor = min(v.co.z for v in evaluated.data.vertices)
                    if head_floor >= -1e-5:
                        break
                    head = arm.pose.bones["head"]
                    direction(head, Quaternion(Vector((0, 1, 0)), math.radians(-1)) @ (head.tail-head.head))
                else:
                    raise AssertionError("Sleep head clearance did not converge")
            for bone in arm.pose.bones:
                bone.keyframe_insert("rotation_quaternion", frame=frame)
                bone.keyframe_insert("location", frame=frame)
        track = arm.animation_data.nla_tracks.new()
        track.name = name
        track.strips.new(name, 1, action)
        track.mute = True
    arm.animation_data.action = None
    return names
