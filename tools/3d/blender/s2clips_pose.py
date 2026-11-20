"""Pose targets for the S2 rig in metres, Blender Z up and forward -Y."""

from dataclasses import dataclass
import math

import bpy
from mathutils import Matrix, Quaternion, Vector

import anchor2_woman as anchor
import common as C


@dataclass(frozen=True, slots=True)
class Pose:
    drop: float
    lean: float
    twist: float
    right: tuple[float, float, float]
    left: tuple[float, float, float]
    tool: tuple[float, float, float]


@dataclass(frozen=True, slots=True)
class Clip:
    duration: float
    keys: tuple[Pose, ...]
    loop: bool = True
    times: tuple[float, ...] = ()
    feet: tuple[tuple[float, float], tuple[float, float]] = ((0., 0.), (0., 0.))
    hips_back: float | None = None


CHOP_CONTACT = Vector((-.22, -.67, .57))
AXE_EDGE = Vector((.142, 0, .50))
POT_CENTRE = (-.25, -.67)
LADLE_TIP = Vector((0, 0, .51))
DIG_CUT = Vector((-.25, -.60, .025))
DIG_TIP = .68
BUILD_CONTACT = Vector((-.19, -.54, .60))
HAMMER_FACE = Vector((0., 0., .338))
BASKET_FLOOR_GRIP = Vector((-.235, -.53, .315))
HARVEST_PATCH = (.25, -.68)
SEAT_FEET = ((-.035, -.407), (.045, -.432))
SEAT_BACK = .12
SEAT_TOP = .285
LIFT_FEET = ((-.060, -.040), (.065, -.065))
LIFT = (
    Pose(.38, 1.04, -.08, (-.24, -.49, .38), (.09, -.52, .44), (0., 0., 1.)),
    Pose(.33, .84, -.06, (-.24, -.45, .46), (.08, -.47, .50), (0., 0., 1.)),
    Pose(.16, .36, -.04, (-.24, -.38, .71), (.08, -.37, .73), (0., 0., 1.)),
    Pose(.045, .08, 0., (-.24, -.33, .98), (.13, -.30, .90), (0., 0., 1.)),
)
CLIPS = {
    "chop": Clip(1.4, (
        Pose(.10, .08, -.12, (-.34, -.24, 1.20), (.19, -.25, 1.01), (.0, -.73, .68)),
        Pose(.075, .03, -.20, (-.33, -.09, 1.42), (.19, -.24, 1.06), (.0, .35, .93)),
        Pose(.076, .04, -.28, (-.31, -.065, 1.48), (.18, -.24, 1.09), (0., .57, .82)),
        Pose(.18, .37, .08, (-.22, -.59, 1.00), (.16, -.36, .91), (0., -.60, -.80)),
        Pose(.18, .37, .08, (-.22, -.59, 1.00), (.16, -.36, .91), (0., -.60, -.80)),
        Pose(.15, .27, .01, (-.26, -.51, 1.06), (.18, -.31, .95), (0., -.69, -.72)),
    ), times=(0., .19, .38, .57, .66, .79)),
    "carry": Clip(1.0, (
        Pose(0., .05, 0., (-.245, -.275, 1.015), (.245, -.275, 1.015), (0., -1., 0.)),
    )),
    "cook": Clip(1.6, (
        Pose(.324, .40, 0., (-.25, -.34, .65), (.22, -.24, .73), (0., -.69, -.724)),
    )),
    "dig": Clip(1.4, (
        Pose(.13, .20, -.10, (-.31, -.30, 1.05), (.20, -.31, .91), (0., -.40, -.92)),
        Pose(.10, .13, -.12, (-.33, -.24, 1.25), (.18, -.30, .99), (0., -.45, -.89)),
        Pose(.30, .61, -.02, (-.25, -.41, .69), (.11, -.43, .72), (0., -.29, -.96)),
        Pose(.35, .67, .02, (-.25, -.34, .61), (.09, -.44, .64), (0., -.49, -.87)),
        Pose(.24, .37, -.03, (-.28, -.24, .76), (.15, -.38, .82), (0., -.77, -.64)),
    ), times=(0., .21, .47, .62, .79), feet=((-0.065, -.11), (.046, .090))),
    "build": Clip(1.2, (
        Pose(.18, .41, -.06, (-.30, -.30, .90), (.13, -.42, .67), (.12, -.44, -.89)),
        Pose(.15, .30, -.19, (-.31, -.17, 1.14), (.14, -.42, .67), (.04, -.48, -.88)),
        Pose(.22, .52, .09, (-.22, -.40, .86), (.12, -.47, .66), (.07, -.49, -.87)),
        Pose(.22, .50, .07, (-.22, -.40, .86), (.12, -.46, .67), (.07, -.49, -.87)),
        Pose(.18, .43, .03, (-.26, -.32, .98), (.14, -.43, .68), (.08, -.39, -.92)),
    ), times=(0., .25, .48, .59, .77), feet=((-0.047, .065), (.035, -.11))),
    "craft": Clip(1.6, (
        Pose(.38, .40, -.12, (-.23, -.42, .57), (.10, -.36, .60), (0., 0., 1.)),
        Pose(.38, .45, -.02, (-.11, -.43, .58), (.17, -.42, .58), (0., 0., 1.)),
        Pose(.38, .43, .12, (.09, -.44, .56), (.23, -.40, .60), (0., 0., 1.)),
        Pose(.38, .32, .02, (-.06, -.39, .67), (.11, -.35, .62), (0., 0., 1.)),
    ), feet=SEAT_FEET, hips_back=SEAT_BACK),
    "sit": Clip(2.0, (
        Pose(.38, .19, -.06, (-.18, -.28, .47), (.20, -.31, .49), (0., 0., 1.)),
        Pose(.38, .25, .03, (-.17, -.28, .48), (.19, -.29, .51), (0., 0., 1.)),
        Pose(.38, .20, .09, (-.18, -.28, .47), (.18, -.29, .49), (0., 0., 1.)),
        Pose(.38, .14, .02, (-.19, -.28, .47), (.18, -.28, .50), (0., 0., 1.)),
    ), feet=SEAT_FEET, hips_back=SEAT_BACK),
    "talk": Clip(1.6, (
        Pose(.045, .08, .06, (-.25, -.20, .87), (.235, -.16, .92), (0., 0., 1.)),
        Pose(.040, .13, -.14, (-.36, -.28, 1.24), (.12, -.18, .98), (0., 0., 1.)),
        Pose(.047, .21, .12, (-.17, -.42, 1.28), (.32, -.28, 1.13), (0., 0., 1.)),
        Pose(.052, .08, .18, (-.23, -.28, .96), (.39, -.31, 1.24), (0., 0., 1.)),
        Pose(.044, .08, .11, (-.245, -.22, .89), (.29, -.22, 1.02), (0., 0., 1.)),
    ), times=(0., .20, .48, .67, .83)),
    "pick_up": Clip(.94, LIFT, loop=False, times=(0., .22, .68, 1.), feet=LIFT_FEET),
    "set_down": Clip(.94, LIFT[::-1], loop=False, times=(0., .32, .78, 1.), feet=LIFT_FEET),
    "harvest": Clip(1.6, (
        Pose(.11, .23, .11, (-.31, -.29, .90), (.21, -.33, .87), (0., 0., 1.)),
        Pose(.30, .83, -.10, (-.34, -.32, .65), (.24, -.57, .42), (0., 0., 1.)),
        Pose(.30, .84, -.13, (-.34, -.36, .66), (.25, -.58, .45), (0., 0., 1.)),
        Pose(.21, .52, .22, (-.31, -.40, .77), (.03, -.52, .69), (0., 0., 1.)),
        Pose(.16, .34, .32, (-.28, -.42, .81), (-.15, -.46, .80), (0., 0., 1.)),
    ), times=(0., .27, .41, .64, .79), feet=((-0.060, .07), (.065, -.10))),
}


def tool_basis(direction: Vector) -> Matrix:
    """Keep the blade in the swing plane, with its edge across the haft."""
    lateral = Vector((1, 0, 0))
    x = lateral.cross(direction).normalized()
    return Matrix((x, direction.cross(x), direction)).transposed()


def tool_goal(name: str, phase: float, direction: Vector) -> Vector | None:
    """Contact poses are solved from the visual tool's working edge."""
    if name == "chop" and .57 <= phase <= .66:
        return CHOP_CONTACT - tool_basis(direction) @ AXE_EDGE
    if name == "cook":
        angle = math.tau * phase
        tip = Vector((POT_CENTRE[0] + .085 * math.cos(angle),
                      POT_CENTRE[1] + .085 * math.sin(angle), .265))
        return tip - direction * LADLE_TIP.z
    if name == "dig" and .47 <= phase <= .62:
        push = (phase - .47) / .15
        return DIG_CUT + Vector((0, -.038, -.052)) * push - direction * DIG_TIP
    if name == "build" and .48 <= phase <= .59:
        return BUILD_CONTACT - direction * HAMMER_FACE.z
    if (name == "pick_up" and phase == 0.) or (name == "set_down" and phase == 1.):
        return BASKET_FLOOR_GRIP.copy()
    return None


def solve(upper: bpy.types.PoseBone, lower: bpy.types.PoseBone, goal: Vector, pole: Vector) -> None:
    """Place a two-bone chain with a stable bend plane and a reachable target."""
    origin = upper.head.copy()
    axis = (goal - origin).normalized()
    a, b = upper.bone.length, lower.bone.length
    reach = max(abs(a - b) + .002, min(a + b - .004, (goal - origin).length))
    bend = (pole - axis * pole.dot(axis)).normalized()
    along = (a * a - b * b + reach * reach) / (2 * reach)
    joint = origin + along * axis + math.sqrt(max(0, a * a - along * along)) * bend
    arm_side = upper.name[-1]
    if upper.name.startswith("upper_arm"):
        sign = -1 if arm_side == "R" else 1
        base = Quaternion(C.Y, sign * 1.14) @ upper.bone.matrix_local.to_quaternion()
        orient(upper, joint - origin, base)
        orient(lower, origin + reach * axis - joint, upper.matrix.to_quaternion())
    else:
        anchor._orient_at(upper, joint - origin)
        anchor._orient_at(lower, origin + reach * axis - joint)


def orient(bone: bpy.types.PoseBone, direction: Vector, base: Quaternion) -> None:
    matrix = ((base @ C.Y).rotation_difference(direction) @ base).to_matrix().to_4x4()
    matrix.translation = bone.head
    bone.matrix = matrix
    bpy.context.view_layer.update()


def pose(arm: bpy.types.Object, body: str, name: str, phase: float) -> Vector:
    """Pose one VAT sample and return the right-hand prop's long axis."""
    clip = CLIPS[name]
    size = len(clip.keys)
    span = size if clip.loop else max(1, size - 1)
    times = clip.times or tuple(index / span for index in range(size))
    segment = max(index for index, time in enumerate(times) if time <= phase)
    first, second = clip.keys[segment], clip.keys[(segment + 1) % size]
    stop = times[segment + 1] if segment + 1 < size else 1.0
    fraction = (phase - times[segment]) / max(1e-5, stop - times[segment])
    t = (1.0 - math.cos(math.pi * fraction)) * .5
    lean = first.lean + (second.lean - first.lean) * t
    twist = first.twist + (second.twist - first.twist) * t
    drop = first.drop + (second.drop - first.drop) * t
    elder = body == "p4"
    bones = arm.pose.bones
    if name != "carry":
        for bone in bones:
            bone.matrix_basis = Matrix.Identity(4)
            bone.rotation_mode = "QUATERNION"
        hips = bones["hips"]
        match name:
            case "cook":
                depth = -.06
            case "pick_up" | "set_down" | "harvest":
                depth = drop * -.15
            case _:
                depth = drop * .25 if clip.hips_back is None else clip.hips_back
        hips.location = hips.bone.matrix_local.to_quaternion().inverted() @ Vector((0, depth, -drop))
    if name == "cook":
        twist += .10 * math.sin(math.tau * phase)
        lean += .035 * math.cos(math.tau * phase)
    spine = bones["spine"]
    spine.rotation_quaternion = anchor._offset(spine, Quaternion(C.X, .52 * lean))
    chest = bones["chest"]
    chest.rotation_quaternion = anchor._offset(chest, Quaternion(C.Z, twist) @ Quaternion(C.X, .65 * lean + .13 * elder))
    bpy.context.view_layer.update()
    if name != "carry":
        for side, (spread, stride) in zip(("R", "L"), clip.feet):
            leg, shin, foot = (bones[f"{part}_{side}"] for part in ("upper_leg", "lower_leg", "foot"))
            foot_rest = foot.bone.matrix_local.copy()
            foot_rest.translation += (Vector((.040 if side == "L" else -.040,
                                               -.030 if side == "L" else .030, 0))
                                      if name == "chop" else Vector((spread, stride, 0)))
            solve(leg, shin, foot_rest.translation, Vector((0, -1, 0)))
            foot.matrix = foot_rest
            bpy.context.view_layer.update()
    axis = Vector(first.tool).lerp(Vector(second.tool), t).normalized()
    for side, sign, start, end in (("R", -1, first.right, second.right), ("L", 1, first.left, second.left)):
        upper, lower, hand = (bones[f"{part}_{side}"] for part in ("upper_arm", "lower_arm", "hand"))
        wrist = Vector(start).lerp(Vector(end), t)
        wrist.z -= .06 * elder
        if name == "carry":
            wrist.z += bones["hips"].head.z - .75
            wrist.y += bones["hips"].head.y
        solve(upper, lower, wrist, Vector((sign * .7, .25, -1.0)))
        match name:
            case "carry":
                direction = Vector((0, -1, -.12))
            case "talk":
                direction = Vector((sign * .5, -.8, .25)).lerp(
                    (hand.head - lower.head).normalized(), .6)
            case _:
                direction = (hand.head - lower.head).normalized()
        orient(hand, direction, lower.matrix.to_quaternion())
    return axis


def adjust_grip(arm: bpy.types.Object, side: str, correction: Vector, pole: Vector | None = None) -> None:
    upper, lower, hand = (arm.pose.bones[f"{part}_{side}"] for part in ("upper_arm", "lower_arm", "hand"))
    pose = hand.matrix.copy()
    target = hand.head + correction
    if pole is None or pole.cross(target - upper.head).length < .0001:
        pole = Vector((-.7 if side == "R" else .7, .25, -1.0))
    solve(upper, lower, target, pole)
    pose.translation = hand.head
    hand.matrix = pose
    bpy.context.view_layer.update()
