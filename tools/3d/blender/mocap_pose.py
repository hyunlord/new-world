"""Retarget CMU FK to the S2 skeleton, then solve the spec's contact constraints."""

from dataclasses import dataclass
import math

import bpy
from mathutils import Matrix, Quaternion, Vector
import numpy as np

import common as C
from mocap_bvh import Motion
from mocap_spec import Action, Hand, Take
import s2clips as clips
import s2clips_pose as old

SIDES = ("R", "L")
TORSO = (("hips", "Hips"), ("spine", "Spine"), ("chest", "Spine1"),
         ("neck", "Neck1"), ("head", "Head"))
LIMBS = (("upper_arm", "Arm"), ("lower_arm", "ForeArm"), ("hand", "Hand"),
         ("upper_leg", "UpLeg"), ("lower_leg", "Leg"), ("foot", "Foot"))
BONE_NAMES = dict(TORSO + tuple((f"{target}_{side}", prefix + source)
                              for side, prefix in zip(SIDES, ("Right", "Left")) for target, source in LIMBS))


def ease(value: float) -> float:
    value = max(0., min(1., value))
    return value * value * (3 - 2 * value)


@dataclass(frozen=True, slots=True)
class JointPose:
    position: Vector
    rotation: Quaternion

    def mix(self, other: 'JointPose', amount: float) -> 'JointPose':
        return JointPose(self.position.lerp(other.position, amount), self.rotation.slerp(other.rotation, amount))


class Retarget:
    def __init__(self, arm: bpy.types.Object, motion: Motion, action: Action, take: Take, body: str) -> None:
        self.arm = arm
        self.action = action
        self.motion = motion
        self.window = (max(1., action.trim[0] * take.fps), action.trim[1] * take.fps)
        if not self.window[0] < self.window[1] <= len(motion.positions) - 1:
            raise ValueError(f"{action.key}: trim {action.trim} is outside {take.id}")
        self.heading = Quaternion(C.Z, math.radians(action.heading))
        self.stoop = Quaternion(C.X, math.radians(action.stoop[body]))
        self.indices = {bone: motion.joints[name] for bone, name in BONE_NAMES.items()}
        self.references = {name: Matrix(motion.rotation[0, i]).to_quaternion().inverted()
                           for name, i in self.indices.items()}
        self.rest = {bone.name: bone.bone.matrix_local.to_quaternion() for bone in arm.pose.bones}
        self.aligned = {bone.name: self._align(bone) for bone in arm.pose.bones}
        foot_indices = [self.indices[f"foot_{side}"] for side in SIDES]
        hip = self.indices["hips"]
        source_height = motion.positions[0, hip, 2] - motion.positions[0, foot_indices, 2].mean()
        rest_hip = arm.pose.bones["hips"].bone.head_local
        rest_ankles = [arm.pose.bones[f"foot_{side}"].bone.head_local.z for side in SIDES]
        self.scale = (rest_hip.z - sum(rest_ankles) / 2) / source_height
        if not .005 < self.scale < 1.:
            raise ValueError(f"{take.id}: invalid BVH to S2 scale {self.scale}")
        first, last = round(self.window[0]), round(self.window[1])
        stance = np.median(motion.positions[first:last + 1, foot_indices], axis=0)
        mid = Vector(stance.mean(axis=0))
        self.origin = Vector((mid.x, mid.y, motion.positions[0, hip, 2]))
        self.feet = {}
        for side, location, ankle in zip(SIDES, stance, rest_ankles):
            offset = self.heading @ ((Vector(location) - mid) * self.scale)
            self.feet[side] = Vector((offset.x, offset.y, ankle))
        self.posture = Vector(action.posture)
        self.contact = action.contacts[0] if action.contacts else .5
        self.first = self._frame(self.window[0])
        self.last = self._frame(self.window[1])

    def _align(self, bone: bpy.types.PoseBone) -> Quaternion:
        neutral = self.rest[bone.name]
        name = bone.name
        if name.startswith(("upper_arm_", "lower_arm_", "hand_")):
            side = name[-1]
            outward = Vector((-1., 0., 0.)) if side == "R" else C.X
            neutral = (neutral @ C.Y).rotation_difference(outward) @ neutral
        return neutral

    def _frame(self, frame: float) -> dict[str, JointPose]:
        lo = int(frame)
        hi = min(lo + 1, len(self.motion.positions) - 1)
        alpha = frame - lo
        result = {}
        for name, index in self.indices.items():
            samples = self.motion.positions
            position = Vector(samples[lo, index]).lerp(Vector(samples[hi, index]), alpha)
            matrices = self.motion.rotation
            rotation = Matrix(matrices[lo, index]).to_quaternion().slerp(
                Matrix(matrices[hi, index]).to_quaternion(), alpha)
            result[name] = JointPose(position, rotation)
        return result

    def sample(self, phase: float) -> dict[str, JointPose]:
        amount = 1. - phase if self.action.reverse else phase
        lo, hi = self.window
        pose = self._frame(lo + (hi - lo) * amount)
        if self.action.loop and (amount < .1 or amount > .9):
            # Bring the first and last pose to the same midpoint with zero seam tangent.
            edge = min(amount, 1. - amount)
            weight = 1. - ease(edge / .1)
            return {name: value.mix(self.first[name].mix(self.last[name], .5), weight)
                    for name, value in pose.items()}
        return pose

    def pose(self, phase: float) -> tuple[float, float]:
        pose = self.sample(phase)
        bones = self.arm.pose.bones
        rotations = {}
        for bone in bones:
            name = bone.name
            rotation = self.heading @ pose[name].rotation @ self.references[name] @ self.aligned[name]
            if name in ("chest", "neck", "head"):
                rotation = self.stoop @ rotation
            rotations[name] = rotation
        for bone in bones:
            name = bone.name
            base = self.rest[name]
            if bone.parent is not None:
                parent = bone.parent.name
                base = rotations[parent] @ self.rest[parent].inverted() @ base
            bone.location = (0., 0., 0.)
            bone.scale = (1., 1., 1.)
            bone.rotation_mode = "QUATERNION"
            bone.rotation_quaternion = base.inverted() @ rotations[name]
        hips = bones["hips"]
        delta = (self.heading @ ((pose["hips"].position - self.origin) * self.scale)) + self.posture
        if self.action.foot_lock:
            # The fixed feet must remain within reach even when the source straightens its knees.
            for side, foot in self.feet.items():
                thigh = bones[f"upper_leg_{side}"]
                shin = bones[f"lower_leg_{side}"]
                root = hips.bone.head_local + delta + rotations["hips"] @ (
                    self.rest["hips"].inverted() @ (thigh.bone.head_local - hips.bone.head_local))
                reach = thigh.bone.length + shin.bone.length - .012
                flat = (root.x - foot.x) ** 2 + (root.y - foot.y) ** 2
                delta.z -= max(0., root.z - foot.z - math.sqrt(max(.01, reach ** 2 - flat)))
        hips.location = self.rest["hips"].inverted() @ delta
        bpy.context.view_layer.update()
        if not self.action.foot_lock:
            return 0., 0.
        for side, goal in self.feet.items():
            thigh, shin, foot = (bones[f"{name}_{side}"] for name in ("upper_leg", "lower_leg", "foot"))
            pole = shin.head - thigh.head + Vector((0., -.15, 0.))
            old.solve(thigh, shin, goal, pole)
            floor = foot.bone.matrix_local.copy()
            floor.translation = foot.head
            foot.matrix = floor
        bpy.context.view_layer.update()
        return tuple((bones[f"foot_{side}"].head - self.feet[side]).length for side in SIDES)


def offset(hand: Hand, phase: float) -> Vector:
    result = Vector(hand.offset)
    if hand.slide:
        times, slides = zip(*hand.slide)
        result.z += float(np.interp(phase, times, slides))
    return result


def tool_frame(axis: Vector) -> Matrix:
    """Set tool +Z to its working axis and keep its blade in the swing plane."""
    if abs(axis.x) < .98:
        return old.tool_basis(axis)
    return axis.to_track_quat("Z", "Y").to_matrix()


def prop_pose(action: Action, phase: float, palms: tuple[Vector, Vector],
              wrists: tuple[Vector, Vector], shoulder: Vector,
              lift: Vector | None, primary_side: int) -> tuple[Vector, Matrix, float, float]:
    target = action.target
    anchor = Vector(target.anchor)
    axis = Vector(target.orientation).normalized()
    distances = [abs(phase - frame) for frame in action.contacts]
    if action.loop:
        distances = [min(dist, 1. - dist) for dist in distances]
    impact = ease(1. - min(distances, default=1.) / .14)
    main = palms[primary_side]
    match target.kind:
        case "swing":
            bottom, top = wrists[primary_side], wrists[1 - primary_side]
            radial = bottom.lerp(top, .5) - shoulder
            captured = (top - bottom).lerp(radial, .65).normalized()
            rotation = tool_frame(captured).to_quaternion().slerp(tool_frame(axis).to_quaternion(), impact).to_matrix()
            if action.prop is None:
                return anchor, rotation, impact, impact
            centers = (palm - rotation @ offset(action.hands[side], phase)
                       for side, palm in zip(SIDES, palms)
                       if side in action.hands and action.hands[side].target == "prop")
            primary = sum(centers, Vector()) / sum(hand.target == "prop" for hand in action.hands.values())
            primary = primary.lerp(anchor - rotation @ Vector(target.tip), impact)
            return primary, rotation, 1., impact
        case "circle":
            x, y = math.cos(math.tau * phase), math.sin(math.tau * phase)
            tip = anchor + Vector((target.radius * x, target.radius * y, -target.depth))
            rotation = tool_frame(axis) @ Matrix.Rotation(.35 * math.sin(math.tau * phase), 3, "Z")
            return tip - rotation @ Vector(target.tip), rotation, 1., 1.
        case "reach":
            if action.prop is None:
                return anchor, Matrix.Identity(3), impact, impact
            at = action.contacts[0]
            held = phase <= at if action.reverse else phase >= at
            moved = lift if held and lift is not None else Vector((0, 0, 0))
            return anchor + moved, Matrix.Identity(3), 1. if held else impact, impact
        case "free":
            return main, Matrix.Identity(3), 1., 0.
        case _:
            raise ValueError(f"Unimplemented target {target.kind}")


def follow_hand(arm: bpy.types.Object, side: str, across_grip: Vector) -> None:
    hand = arm.pose.bones[f"hand_{side}"]
    aim = hand.matrix.to_quaternion() @ C.Y
    across = hand.matrix.to_quaternion() @ C.X
    target = across_grip - aim * across_grip.dot(aim)
    if target.length > .12:
        twist = across.rotation_difference(target.normalized())
        pose = (Quaternion().slerp(twist, .48) @ hand.matrix.to_quaternion()).to_matrix().to_4x4()
        pose.translation = hand.head
        hand.matrix = pose
        bpy.context.view_layer.update()


def aim_palm(upper: bpy.types.PoseBone, lower: bpy.types.PoseBone, hand: bpy.types.PoseBone,
             palm: Vector, goal: Vector) -> bool:
    """Turn an out-of-reach palm only enough to bring the wrist within range."""
    lever = palm - hand.head
    reach = upper.bone.length + lower.bone.length - .013
    aim = goal - upper.head
    if (aim - lever).length <= reach or min(aim.length, lever.length) < .005:
        return False
    angle = lever.angle(aim)
    cosine = (aim.length_squared + lever.length_squared - reach * reach) / (2 * aim.length * lever.length)
    enough = math.acos(max(-1., min(1., cosine)))
    if angle <= enough or angle < .001:
        return False
    turn = Quaternion().slerp(lever.rotation_difference(aim), 1. - enough / angle)
    pose = (turn @ hand.matrix.to_quaternion()).to_matrix().to_4x4()
    pose.translation = hand.head
    hand.matrix = pose
    bpy.context.view_layer.update()
    return True


@dataclass(frozen=True, slots=True)
class ActionPose:
    positions: np.ndarray
    sockets: tuple[Matrix, Matrix]
    prop: Matrix | None
    errors: dict[str, float]


def prop_hand(action: Action) -> int:
    return next((i for i, side in enumerate(SIDES)
                 if side in action.hands and action.hands[side].target == "prop"), 0)


def wrist_center(arm: bpy.types.Object, action: Action) -> Vector:
    wrists = [arm.pose.bones[f"hand_{side}"].head for side, hand in action.hands.items()
              if hand.target == "prop"]
    return sum(wrists, Vector()) / len(wrists)


def solid_clearance(points: np.ndarray, origin: Vector, orientation: Matrix, action: Action) -> float:
    """Return the prop's penetration into the authored cylindrical working surface."""
    anchor = action.target.anchor
    posed = points @ np.asarray(orientation).T + np.asarray(origin) - np.asarray(anchor)
    in_footprint = np.sum(posed[:, :2] ** 2, axis=1) <= action.target.fixture_radius ** 2
    return -min(0., float(posed[in_footprint, 2].min(initial=0.)))


def fit_swing(arm: bpy.types.Object, action: Action, phase: float, orientation: Matrix,
              origin: Vector, palms: tuple[Vector, Vector], impact: float) -> Vector:
    """Project the shared haft position into the overlapping palm reach of both arms."""
    limits = []
    for side, palm in zip(SIDES, palms):
        grip = action.hands.get(side)
        if grip is not None and grip.target == "prop":
            upper, lower, hand = (arm.pose.bones[f"{segment}_{side}"]
                                  for segment in ("upper_arm", "lower_arm", "hand"))
            reach = upper.bone.length + lower.bone.length - .02 + .6 * (palm - hand.head).length
            limits.append((upper.head - orientation @ offset(grip, phase), reach))
    fitted = origin.copy()
    for _ in range(4):
        for center, radius in limits:
            delta = fitted - center
            if delta.length > radius:
                fitted = center + radius * delta.normalized()
    return fitted.lerp(origin, impact)


def constrain(arm: bpy.types.Object, mesh: bpy.types.Object, grips: tuple[int, int], action: Action,
              phase: float, at_contact: Vector | None, prop_points: np.ndarray | None) -> ActionPose:
    positions = clips.posed_positions(mesh)
    palms = tuple(Vector(positions[index]) for index in grips)
    wrists = tuple(arm.pose.bones[f"hand_{side}"].head for side in SIDES)
    shoulder = sum((arm.pose.bones[f"upper_arm_{side}"].head for side in SIDES), Vector()) / 2
    primary = prop_hand(action)
    lift = None if at_contact is None else wrist_center(arm, action) - at_contact
    origin, orientation, weight, impact = prop_pose(action, phase, palms, wrists, shoulder, lift, primary)
    if action.target.kind == "swing" and action.prop is not None:
        origin = fit_swing(arm, action, phase, orientation, origin, palms, impact)
    if prop_points is not None and action.target.solid:
        origin.z += solid_clearance(prop_points, origin, orientation, action)
    accuracy = {}
    sockets = []
    for n, (side, i, palm) in enumerate(zip(SIDES, grips, palms)):
        hand = action.hands.get(side)
        bone = arm.pose.bones[f"hand_{side}"]
        if hand is not None and weight > 0.:
            upper, lower = (arm.pose.bones[f"{segment}_arm_{side}"] for segment in ("upper", "lower"))
            pole = lower.head - upper.head
            grip = offset(hand, phase)
            goal = origin + orientation @ grip if hand.target == "prop" else Vector(action.target.anchor) + grip
            if hand.target == "prop" and action.target.kind in ("swing", "circle"):
                follow_hand(arm, side, orientation @ (C.X if action.target.kind == "circle" else C.Z))
                positions = clips.posed_positions(mesh)
            goal = palm.lerp(goal, weight)
            if aim_palm(upper, lower, bone, Vector(positions[i]), goal):
                positions = clips.posed_positions(mesh)
            for _ in range(25):
                error = goal - Vector(positions[i])
                if error.length < .0008:
                    break
                # The sampled palm is weighted to both wrist and forearm on the scans.
                old.adjust_grip(arm, side, .42 * error, pole)
                positions = clips.posed_positions(mesh)
            accuracy[side] = (Vector(positions[i]) - goal).length
        rotation = orientation if n == primary and action.prop is not None else None
        sockets.append(clips.socket(bone, positions[i], rotation))
    prop = None
    if action.prop is not None:
        prop = orientation.to_4x4()
        grip = offset(action.hands[SIDES[primary]], phase)
        prop.translation = (origin if weight < 1. or action.target.kind == "reach"
                            else sockets[primary].translation - orientation @ grip)
        accuracy["prop_origin"] = (prop.translation - origin).length
        accuracy["contact_weight"] = impact
        if prop_points is not None and action.target.solid:
            accuracy["penetration"] = solid_clearance(prop_points, prop.translation, orientation, action)
        if weight == 1.:
            for side, socket in zip(SIDES, sockets):
                hand = action.hands.get(side)
                if hand is not None and hand.target == "prop":
                    accuracy[f"grip_{side}"] = (prop @ offset(hand, phase) - socket.translation).length
        if action.target.kind == "swing" and impact > .9999:
            accuracy["tip"] = (prop @ Vector(action.target.tip) - Vector(action.target.anchor)).length
        if action.target.kind == "circle":
            tip = prop @ Vector(action.target.tip) - Vector(action.target.anchor)
            accuracy["tip"] = math.hypot(math.hypot(tip.x, tip.y) - action.target.radius, tip.z + action.target.depth)
    return ActionPose(positions, (sockets[0], sockets[1]), prop, accuracy)
