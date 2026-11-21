"""Retarget CMU FK to the S2 skeleton, then solve the spec's contact constraints."""

from collections.abc import Callable, Mapping
from dataclasses import dataclass
import math

import bpy
from mathutils import Matrix, Quaternion, Vector
import numpy as np

import common as C
from mocap_anatomy import Capsules, PENETRATION_LIMIT_M, forearm_penetration, joint_limits, torso_capsules
from mocap_balance import stationary_offset
from mocap_bvh import Motion
from mocap_gait import Gait
from mocap_kinematics import FloatArray, Track
from mocap_spec import Action, ArmLimits, BaseLayer, Hand, Take
import s2clips as clips
from s2clips_capture import BODY_SCALE
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


class Capture:
    def __init__(self, arm: bpy.types.Object, motion: Motion, spec: Action | BaseLayer, take: Take) -> None:
        self.motion = motion
        self.window = tuple(max(1., t * take.fps) for t in spec.trim)
        if not self.window[0] <= self.window[1] <= len(motion.positions) - 1:
            raise ValueError(f"{spec.key}: trim {spec.trim} is outside {take.id}")
        self.heading = Quaternion(C.Z, math.radians(spec.heading))
        self.indices = {bone: motion.joints[name] for bone, name in BONE_NAMES.items()}
        self.references = {name: Matrix(motion.rotation[0, i]).to_quaternion().inverted()
                           for name, i in self.indices.items()}
        self.rest = {bone.name: bone.bone.matrix_local.to_quaternion() for bone in arm.pose.bones}
        self.calibrate_arms = isinstance(spec, Action) and bool(spec.arm_limits or spec.base_layer)
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
        self.travel = Vector((0., 0., 0.))
        if isinstance(spec, Action) and spec.in_place:
            sample_times = np.arange(len(motion.positions))
            for axis in range(2):
                start, finish = np.interp(self.window, sample_times, motion.positions[:, hip, axis])
                self.travel[axis] = float(finish - start)
        stance = np.median(motion.positions[first:last + 1, foot_indices], axis=0)
        mid = Vector(stance.mean(axis=0))
        self.origin = Vector((mid.x, mid.y, motion.positions[0, hip, 2]))
        self.feet = {}
        foot_floor = stance[:, 2].min()
        for side, location, ankle in zip(SIDES, stance, rest_ankles):
            offset = self.heading @ ((Vector(location) - mid) * self.scale)
            elevation = (location[2] - foot_floor) * self.scale if isinstance(spec, BaseLayer) else 0.
            self.feet[side] = Vector((offset.x, offset.y, ankle + elevation))
        self.first = self._frame(self.window[0])
        self.last = self._frame(self.window[1])

    def _align(self, bone: bpy.types.PoseBone) -> Quaternion:
        neutral = self.rest[bone.name]
        name = bone.name
        if name.startswith(("upper_arm_", "lower_arm_", "hand_")):
            side = name[-1]
            outward = Vector((-1., 0., 0.)) if side == "R" else C.X
            if self.calibrate_arms:
                first = "upper_arm" if name.startswith("upper_arm_") else "lower_arm"
                second = "lower_arm" if first == "upper_arm" else "hand"
                source = self.motion.positions[0]
                outward = Vector(source[self.indices[f"{second}_{side}"]] - source[self.indices[f"{first}_{side}"]])
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
            if name == "hips" and self.travel.length_squared:
                fraction = (frame - self.window[0]) / (self.window[1] - self.window[0])
                position -= (fraction - .5) * self.travel
            matrices = self.motion.rotation
            rotation = Matrix(matrices[lo, index]).to_quaternion().slerp(
                Matrix(matrices[hi, index]).to_quaternion(), alpha)
            result[name] = JointPose(position, rotation)
        return result

    def sample(self, phase: float, loop: bool) -> tuple[dict[str, Quaternion], Vector]:
        lo, hi = self.window
        pose = self._frame(lo + (hi - lo) * phase)
        if loop and (phase < .1 or phase > .9):
            # Bring the first and last pose to the same midpoint with zero seam tangent.
            edge = min(phase, 1. - phase)
            weight = 1. - ease(edge / .1)
            pose = {name: value.mix(self.first[name].mix(self.last[name], .5), weight)
                    for name, value in pose.items()}
        rotations = {name: self.heading @ joint.rotation @ self.references[name] @ self.aligned[name]
                     for name, joint in pose.items()}
        return rotations, self.heading @ ((pose["hips"].position - self.origin) * self.scale)


class Retarget:
    def __init__(self, arm: bpy.types.Object, motion: Motion, action: Action, take: Take, body: str,
                 base: tuple[Take, Motion] | None = None,
                 rest_soles: Mapping[str, FloatArray] | None = None) -> None:
        self.arm = arm
        self.action = action
        self.capture = Capture(arm, motion, action, take)
        self.spine_limit = math.radians(joint_limits(body)["spine"] - .01)
        self.stoop = Quaternion(C.X, math.radians(action.stoop[body]))
        self.limits = action.arm_limits.get(body, action.arm_limits.get("default"))
        self.rest = self.capture.rest
        self.feet = self.capture.feet.copy()
        self.rest_soles = rest_soles
        self.body_scale = BODY_SCALE[body]
        self.floor = {}
        self._sole_local = {}
        self.base = None
        self.mask = {}
        if base is not None and action.base_layer is not None:
            base_take, base_motion = base
            self.base = Capture(arm, base_motion, action.base_layer, base_take)
            self.mask = {name: weight * action.base_layer.weight for name, weight in action.base_layer.mask.items()}
            for side in SIDES:
                self.feet[side] = self.feet[side].lerp(self.base.feet[side], self.mask.get(f"foot_{side}", 0.))
        if action.foot_lock:
            self._locked_stance()
        self.contact = action.contacts[0] if action.contacts else .5
        self.stride = self.capture.heading @ (self.capture.travel * self.capture.scale)
        if action.reverse:
            self.stride.negate()
        self._gait: Gait | None = None
        if action.in_place and action.plausibility.plants:
            if rest_soles is None:
                raise RuntimeError(f"{action.key}: locomotion needs the rendered rest soles")
            self._gait = Gait(arm, self.stride, action.plausibility.plants, rest_soles, self.pose, BODY_SCALE[body])

    def _locked_stance(self) -> None:
        """Freeze the loaded sole orientation across the stationary window."""
        layered, _ = self.base.sample(.5, self.action.loop) if self.base is not None else ({}, Vector())
        for side, goal in self.feet.items():
            x, y = self.action.stance_offsets.get(side, (0., 0.))
            goal.x += x
            goal.y += y
            name = f"foot_{side}"
            bone = self.arm.pose.bones[name].bone
            floor = bone.matrix_local.copy()
            weight = self.mask.get(name, 0.)
            if weight:
                floor = floor.to_quaternion().slerp(layered[name], weight).to_matrix().to_4x4()
            if self.rest_soles is not None:
                local = (self.rest_soles[side] - np.asarray(bone.head_local)) @ np.asarray(bone.matrix_local.to_3x3())
                self._sole_local[side] = local
                contact = local @ np.asarray(floor.to_3x3()).T
                if side in self.action.stance_offsets:
                    # Rest scans can stand on the toes alone. Level the sole
                    # between the lowest real toe and heel samples for seated support.
                    fore, back = float(contact[:, 1].min()), float(contact[:, 1].max())
                    quarter = (back - fore) / 4.
                    front = contact[contact[:, 1] <= fore + quarter]
                    rear = contact[contact[:, 1] >= back - quarter]
                    toe, heel = front[front[:, 2].argmin()], rear[rear[:, 2].argmin()]
                    pitch = -math.atan2(float(heel[2] - toe[2]), float(heel[1] - toe[1]))
                    floor = (Quaternion(C.X, pitch) @ floor.to_quaternion()).to_matrix().to_4x4()
                    contact = local @ np.asarray(floor.to_3x3()).T
                bottom = float(contact[:, 2].min()) + goal.z
                # Leave small normalization discrepancies in established, already planted poses.
                if weight or side in self.action.stance_offsets or not -.002 <= bottom * self.body_scale <= .002:
                    goal.z -= bottom
            self.floor[side] = floor

    def _pin_locked_feet(self, delta: Vector, rotations: dict[str, Quaternion]) -> None:
        bones = self.arm.pose.bones
        hips = bones["hips"]
        posed = delta.copy()
        poles = {}
        for side, foot in self.feet.items():
            thigh = bones[f"upper_leg_{side}"]
            shin = bones[f"lower_leg_{side}"]
            root = hips.bone.head_local + posed + rotations["hips"] @ (
                self.rest["hips"].inverted() @ (thigh.bone.head_local - hips.bone.head_local))
            reach = thigh.bone.length + shin.bone.length - .012
            flat = (root.x - foot.x) ** 2 + (root.y - foot.y) ** 2
            posed.z -= max(0., root.z - foot.z - math.sqrt(max(.01, reach ** 2 - flat)))
            # Keep the captured knee's bend plane through every pelvis correction.
            upper_offset = self.rest[thigh.name].inverted() @ (shin.bone.head_local - thigh.bone.head_local)
            poles[side] = rotations[thigh.name] @ upper_offset + Vector((0., -.15, 0.))
        hips.location = self.rest["hips"].inverted() @ posed
        bpy.context.view_layer.update()
        for side, goal in self.feet.items():
            thigh, shin, foot = (bones[f"{name}_{side}"] for name in ("upper_leg", "lower_leg", "foot"))
            old.solve(thigh, shin, goal, poles[side])
            floor = self.floor[side].copy()
            floor.translation = foot.head
            foot.matrix = floor
        bpy.context.view_layer.update()

    def _balance_locked_stance(self, phase: float, delta: Vector, rotations: dict[str, Quaternion]) -> None:
        bones = self.arm.pose.bones
        for _ in range(4):
            if not self._sole_local:
                return
            soles = {}
            for side, local in self._sole_local.items():
                matrix = bones[f"foot_{side}"].matrix
                soles[side] = local @ np.asarray(matrix.to_3x3()).T + np.asarray(matrix.translation)
            adjustment = self._support_adjustment(phase, soles)
            if float(np.linalg.norm(adjustment)) <= 1e-7:
                return
            delta.x += float(adjustment[0])
            delta.y += float(adjustment[1])
            self._pin_locked_feet(delta, rotations)

    def _support_adjustment(self, phase: float, soles: Mapping[str, FloatArray]) -> FloatArray:
        bones = self.arm.pose.bones
        adjustment = stationary_offset(
            tuple(bone.name for bone in bones), np.asarray([tuple(bone.head) for bone in bones]),
            np.asarray([tuple(bone.tail) for bone in bones]), soles, self.action.plausibility,
            phase=phase, scale=self.body_scale,
        )
        if self.action.torso_lean:
            hip = np.asarray(bones["hips"].head)[:2]
            stance = np.concatenate(tuple(soles.values()))[:, :2]
            adjustment = (hip + adjustment).clip(stance.min(axis=0), stance.max(axis=0)) - hip
        return adjustment

    def constrain(self, phase: float, shoes: Mapping[str, list[int]], solve: Callable[[], "ActionPose"]) -> "ActionPose":
        """Recheck support after reaching for the tool, then re-solve from the captured arm poses."""
        if not self.action.foot_lock or not self.action.hands:
            return solve()
        bones = self.arm.pose.bones
        arms = tuple((bones[f"{segment}_{side}"], bones[f"{segment}_{side}"].matrix_basis.copy())
                     for side in SIDES for segment in ("upper_arm", "lower_arm", "hand"))
        frame = solve()
        delta, rotations = self._locked_frame
        for _ in range(4):
            adjustment = self._support_adjustment(phase, {side: frame.positions[i] for side, i in shoes.items()})
            if float(np.linalg.norm(adjustment)) <= .00015:
                break
            delta.x += float(adjustment[0])
            delta.y += float(adjustment[1])
            for bone, pose in arms:
                bone.matrix_basis = pose
            self._pin_locked_feet(delta, rotations)
            frame = solve()
        return frame

    def pose(self, phase: float) -> tuple[float, float]:
        amount = 1. - phase if self.action.reverse else phase
        if self.action.retime:
            times, samples = zip(*self.action.retime)
            amount = float(np.interp(amount, times, samples))
        source, delta = self.capture.sample(amount, self.action.loop)
        bones = self.arm.pose.bones
        rotations = source.copy()
        if self.base is not None:
            layered, base_delta = self.base.sample(phase, self.action.loop)
            delta = delta.lerp(base_delta, self.mask.get("root", 0.))
            for bone in bones:
                name = bone.name
                weight = self.mask.get(name, 0.)
                if bone.parent is None:
                    rotations[name] = source[name].slerp(layered[name], weight)
                else:
                    parent = bone.parent.name
                    local = source[parent].inverted() @ source[name]
                    lower = layered[parent].inverted() @ layered[name]
                    rotations[name] = rotations[parent] @ local.slerp(lower, weight)
        for name in ("chest", "neck", "head"):
            rotations[name] = self.stoop @ rotations[name]
        if self.limits is not None and self.limits.spine_extension:
            elevation = max((rotations[f"upper_arm_{side}"] @ C.Y).z for side in SIDES)
            extension = Quaternion(C.X, -math.radians(self.limits.spine_extension) * ease((elevation - .55) / .4))
            for name in ("spine", "chest", "neck", "head",
                         *(f"{segment}_{side}" for side in SIDES for segment in ("upper_arm", "lower_arm", "hand"))):
                rotations[name] = extension @ rotations[name]
        spine_basis = rotations["hips"] @ self.rest["hips"].inverted() @ self.rest["spine"]
        spine_delta = spine_basis.inverted() @ rotations["spine"]
        if spine_delta.w < 0.:
            spine_delta.negate()
        if spine_delta.angle > self.spine_limit:
            turn = (spine_basis @ Quaternion().slerp(spine_delta, self.spine_limit / spine_delta.angle)
                    @ rotations["spine"].inverted())
            for name in ("spine", "chest", "neck", "head",
                         *(f"{segment}_{side}" for side in SIDES for segment in ("upper_arm", "lower_arm", "hand"))):
                rotations[name] = turn @ rotations[name]
        work = any(first <= phase <= last for first, last in self.action.plausibility.work)
        if self.action.torso_lean or not work:
            torso = Vector()
            for name in ("spine", "chest", "neck"):
                bone = bones[name]
                parent = bone.parent.name
                offset = bone.bone.head_local - bone.parent.bone.head_local
                torso += rotations[parent] @ (self.rest[parent].inverted() @ offset)
            limit = math.pi if work else math.radians(self.action.plausibility.lean_limit) * .999
            if self.action.torso_lean:
                phases, caps = zip(*self.action.torso_lean)
                limit = min(limit, math.radians(np.interp(phase, phases, caps)))
            angle = torso.angle(C.Z)
            if angle > limit:
                turn = Quaternion().slerp(torso.rotation_difference(C.Z), 1. - limit / angle)
                for name in ("hips", "spine", "chest", "neck", "head",
                             *(f"{segment}_{side}" for side in SIDES for segment in ("upper_arm", "lower_arm", "hand"))):
                    rotations[name] = turn @ rotations[name]
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
        delta += Vector(self.action.posture)
        if self.action.foot_lock:
            self._pin_locked_feet(delta, rotations)
            self._balance_locked_stance(phase, delta, rotations)
            self._locked_frame = (delta, rotations)
            return tuple((bones[f"foot_{side}"].head - self.feet[side]).length for side in SIDES)
        hips.location = self.rest["hips"].inverted() @ delta
        bpy.context.view_layer.update()
        if self._gait is not None:
            self._gait.pose(phase)
        return 0., 0.


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
    distances = [phase - frame for frame in action.contacts]
    if action.loop:
        distances = [(dist + .5) % 1. - .5 for dist in distances]
    approach, release = action.impact_window
    impact = ease(1. - min((abs(dist) / (approach if dist < 0. else release) for dist in distances),
                          default=1.))
    main = palms[primary_side]
    match target.kind:
        case "swing":
            bottom, top = wrists[primary_side], wrists[1 - primary_side]
            if sum(hand.target == "prop" for hand in action.hands.values()) == 2:
                radial = bottom.lerp(top, .5) - shoulder
                captured = (top - bottom).lerp(radial, .65).normalized()
            else:
                captured = (palms[primary_side] - bottom).lerp(bottom - shoulder, .35).normalized()
            contact_axis = tool_frame(axis).to_quaternion()
            tip_goal = anchor.copy()
            if (any(target.release_offset) or any(target.release_arc) or target.release_pitch) and action.contacts and phase > action.contacts[0]:
                elapsed = phase - action.contacts[0]
                ramp = ease(elapsed / release)
                impact = 1. - ease((elapsed - release) / (1. - action.contacts[0] - release))
                contact_axis = Quaternion(C.X, -math.radians(target.release_pitch) * ramp) @ contact_axis
                tip_goal += ramp * Vector(target.release_offset)
                if elapsed < release:
                    tip_goal += math.sin(math.pi * elapsed / release) ** 2 * Vector(target.release_arc)
            rotation = tool_frame(captured).to_quaternion().slerp(contact_axis, impact).to_matrix()
            if action.prop is None:
                return anchor, rotation, impact, impact
            centers = (palm - rotation @ offset(action.hands[side], phase)
                       for side, palm in zip(SIDES, palms)
                       if side in action.hands and action.hands[side].target == "prop")
            primary = sum(centers, Vector()) / sum(hand.target == "prop" for hand in action.hands.values())
            if target.windup is not None:
                apex, shift = target.windup
                lift_amount = ease(min(phase / apex, (action.contacts[0] - phase) / (action.contacts[0] - apex)))
                primary += lift_amount * Vector(shift)
            primary = primary.lerp(tip_goal - rotation @ Vector(target.tip), impact)
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
            held = phase <= at if action.reverse or target.release else phase >= at
            moved = lift if held and lift is not None else Vector((0, 0, 0))
            return anchor + moved, Matrix.Identity(3), 1. if held else impact, impact
        case "carry":
            lateral = palms[1] - palms[0]
            lateral.z = 0.
            lateral = lateral.normalized() if lateral.length > .01 else C.X
            depth = C.Z.cross(lateral)
            rotation = Matrix((lateral, depth, C.Z)).transposed()
            centers = [palm - rotation @ offset(action.hands[side], phase) for side, palm in zip(SIDES, palms)]
            return centers[0].lerp(centers[1], .5), rotation, 1., 0.
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
             palm: Vector, goal: Vector, min_reach: float = 0.) -> bool:
    """Turn an out-of-reach palm only enough to bring the wrist within range."""
    lever = palm - hand.head
    reach = upper.bone.length + lower.bone.length - .013
    aim = goal - upper.head
    distance = (aim - lever).length
    if min_reach <= distance <= reach or min(aim.length, lever.length) < .005:
        return False
    angle = lever.angle(aim)
    bound = reach if distance > reach else min_reach + .002
    cosine = (aim.length_squared + lever.length_squared - bound * bound) / (2 * aim.length * lever.length)
    enough = math.acos(max(-1., min(1., cosine)))
    if distance > reach:
        if angle <= enough or angle < .001:
            return False
        turn = Quaternion().slerp(lever.rotation_difference(aim), 1. - enough / angle)
    else:
        axis = aim.cross(lever)
        if axis.length < 1e-6:
            axis = lever.orthogonal()
        turn = Quaternion(axis, enough - angle)
    pose = (turn @ hand.matrix.to_quaternion()).to_matrix().to_4x4()
    pose.translation = hand.head
    hand.matrix = pose
    bpy.context.view_layer.update()
    return True


def limit_wrist(lower: bpy.types.PoseBone, hand: bpy.types.PoseBone, limit: float) -> bool:
    """Keep the captured aim inside the wrist cone after rotating the forearm."""
    bind = lower.bone.matrix_local.to_quaternion().inverted() @ hand.bone.matrix_local.to_quaternion()
    neutral = lower.matrix.to_quaternion() @ bind
    rotation = hand.matrix.to_quaternion()
    angle = 2. * math.acos(min(1., abs(neutral.dot(rotation))))
    if angle <= limit:
        return False
    pose = neutral.slerp(rotation, limit / angle).to_matrix().to_4x4()
    pose.translation = hand.head
    hand.matrix = pose
    bpy.context.view_layer.update()
    return True


def axial_roll(first: Quaternion, second: Quaternion) -> float:
    swing = (first @ C.Y).rotation_difference(second @ C.Y) @ first
    twist = swing.inverted() @ second
    angle = 2. * math.atan2(twist.y, twist.w)
    return (angle + math.pi) % math.tau - math.pi


def min_arm_reach(a: float, b: float, flexion: float) -> float:
    return math.sqrt(a * a + b * b + 2 * a * b * math.cos(math.radians(flexion)))


def arm_obstacles(arm: bpy.types.Object) -> Capsules:
    bones = tuple(arm.pose.bones[name] for name in ("hips", "spine", "chest", "neck", "head"))
    track = Track(tuple(bone.name for bone in bones), (-1, 0, 1, 2, 3),
                  np.array([np.asarray(bone.bone.matrix_local) for bone in bones]),
                  np.array([np.asarray(bone.matrix) for bone in bones])[None],
                  np.array([bone.bone.length for bone in bones]))
    return torso_capsules(track)


@dataclass(frozen=True, slots=True)
class ArmChain:
    upper: Quaternion
    upper_roll: float
    lower_roll: float
    elevation: float
    min_reach: float
    up: Vector
    obstacles: Capsules | None
    overlap_limit: float

    @classmethod
    def from_capture(cls, upper: bpy.types.PoseBone, lower: bpy.types.PoseBone, chest: bpy.types.PoseBone,
                     body: str, elbow_limit: float, limits: ArmLimits | None,
                     obstacles: Capsules | None) -> 'ArmChain':
        side = -1 if upper.name.endswith("_R") else 1
        torso = chest.matrix.to_quaternion() @ chest.bone.matrix_local.to_quaternion().inverted()
        neutral = torso @ upper.bone.matrix_local.to_quaternion()
        if not upper.id_data.data.get("scan_arm_rest"):
            bind_angle = .97 if body == "p4" and side == -1 else 1.14
            neutral = torso @ Quaternion(C.Y, side * bind_angle) @ upper.bone.matrix_local.to_quaternion()
        source_upper, source_lower = upper.matrix.to_quaternion(), lower.matrix.to_quaternion()
        twist = axial_roll(neutral, source_upper)

        def cap(angle: float, maximum: float) -> float:
            limit = math.radians(maximum)
            return max(-limit, min(limit, angle))

        shared = .4 * twist if limits is None else cap(.4 * twist, limits.upper_roll)
        forearm = axial_roll(source_upper, source_lower) + twist - shared
        if limits is not None:
            forearm = cap(forearm, limits.lower_roll)
        up = torso @ C.Z
        reach = min_arm_reach(upper.bone.length, lower.bone.length, elbow_limit)
        elevation = 1. if limits is None else math.sin(math.radians(limits.elevation))
        clearance = .8 * PENETRATION_LIMIT_M / BODY_SCALE[body or "p1"]
        return cls(neutral, shared, forearm, elevation, reach, up, obstacles, clearance)

    def clear_elbow(self, center: Vector, offset: Vector, axis: Vector, wrist: Vector, height: float) -> Vector:
        elbow = center + offset
        if self.obstacles is None or forearm_penetration(np.asarray([elbow]), np.asarray(wrist),
                                                      self.obstacles)[0] <= self.overlap_limit:
            return elbow
        # Rotate the elbow around the wrist–shoulder axis while keeping
        # the wrist fixed. Test the nearest poles within the elevation cap.
        arc = axis.cross(offset)
        steps = np.arange(10, 181, 10)
        angles = np.deg2rad(np.column_stack((steps, -steps)).ravel())
        candidates = (np.asarray(center) + np.cos(angles)[:, None] * np.asarray(offset)
                      + np.sin(angles)[:, None] * np.asarray(arc))
        candidates = np.vstack((np.asarray(elbow), candidates[candidates @ np.asarray(self.up) <= height]))
        depths = forearm_penetration(candidates, np.asarray(wrist), self.obstacles)
        clear = np.flatnonzero(depths <= self.overlap_limit)
        return Vector(candidates[clear[0] if len(clear) else int(depths.argmin())])

    def correct(self, upper: bpy.types.PoseBone, lower: bpy.types.PoseBone, hand: bpy.types.PoseBone,
                correction: Vector, pole: Vector) -> None:
        pose = hand.matrix.copy()
        origin = upper.head.copy()
        goal = hand.head + correction
        delta = goal - origin
        axis = delta.normalized()
        a, b = upper.bone.length, lower.bone.length
        reach = max(self.min_reach, min(a + b - .004, delta.length))
        along = (a * a - b * b + reach * reach) / (2. * reach)
        radius = math.sqrt(max(0., a * a - along * along))
        bend = pole - axis * pole.dot(axis)
        if bend.length < .0001:
            bend = axis.orthogonal()
        bend.normalize()
        plane = self.up - axis * self.up.dot(axis)
        if plane.length > .0001 and radius > .0001:
            limit = (a * self.elevation - along * self.up.dot(axis)) / (radius * plane.length)
            plane.normalize()
            if bend.dot(plane) > limit:
                tangent = axis.cross(plane)
                limit = max(-1., min(1., limit))
                bend = plane * limit + tangent * math.copysign(math.sqrt(1. - limit * limit), bend.dot(tangent))
        wrist = origin + reach * axis
        elbow = self.clear_elbow(origin + along * axis, radius * bend, axis, wrist,
                                 origin.dot(self.up) + a * self.elevation)
        arm_vector = elbow - origin
        old.orient(upper, arm_vector, self.upper @ Quaternion(C.Y, self.upper_roll))
        old.orient(lower, reach * axis - arm_vector,
                   upper.matrix.to_quaternion() @ Quaternion(C.Y, self.lower_roll))
        pose.translation = hand.head
        hand.matrix = pose
        bpy.context.view_layer.update()


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
              origin: Vector, palms: tuple[Vector, Vector], impact: float, elbow_limit: float) -> Vector:
    """Project the shared haft position into the overlapping palm reach of both arms."""
    limits = []
    for side, palm in zip(SIDES, palms):
        grip = action.hands.get(side)
        if grip is not None and grip.target == "prop":
            upper, lower, hand = (arm.pose.bones[f"{segment}_{side}"]
                                  for segment in ("upper_arm", "lower_arm", "hand"))
            a, b = upper.bone.length, lower.bone.length
            lever = palm - hand.head
            reach = a + b - .02 + .6 * lever.length
            minimum = min_arm_reach(a, b, elbow_limit) + .45 * lever.length
            limits.append((upper.head - orientation @ offset(grip, phase), minimum, reach))
    fitted = origin.copy()
    for _ in range(4):
        for center, minimum, radius in limits:
            delta = fitted - center
            if delta.length > radius:
                fitted = center + radius * delta.normalized()
            elif .0001 < delta.length < minimum:
                fitted = center + minimum * delta.normalized()
    return fitted.lerp(origin, impact)


def constrain(arm: bpy.types.Object, mesh: bpy.types.Object, grips: tuple[int, int], action: Action,
              phase: float, at_contact: Vector | None, prop_points: np.ndarray | None,
              body: str = "") -> ActionPose:
    arm_limits = action.arm_limits.get(body, action.arm_limits.get("default"))
    per_rig = joint_limits(body or "p1")
    # Leave a small reserve for Blender's float32 rotations at an exact joint limit.
    elbow_limit = min(per_rig["lower_arm"], arm_limits.elbow_flexion if arm_limits else 180.) - .05
    wrist_limit = math.radians(per_rig["hand"] - .05)
    if arm_limits is not None and arm_limits.clavicle_lift:
        bones = arm.pose.bones
        for side in SIDES:
            upper, lower = (bones[f"{segment}_arm_{side}"] for segment in ("upper", "lower"))
            lift_weight = ease(((lower.head - upper.head).normalized().z - .25) / .6)
            shift = C.Z * (lift_weight * arm_limits.clavicle_lift)
            shoulder = upper.matrix.copy()
            shoulder.translation += shift
            upper.matrix = shoulder
        bpy.context.view_layer.update()
    positions = clips.posed_positions(mesh)
    palms = tuple(Vector(positions[index]) for index in grips)
    wrists = tuple(arm.pose.bones[f"hand_{side}"].head for side in SIDES)
    shoulder = sum((arm.pose.bones[f"upper_arm_{side}"].head for side in SIDES), Vector()) / 2
    primary = prop_hand(action)
    lift = None if at_contact is None else wrist_center(arm, action) - at_contact
    origin, orientation, weight, impact = prop_pose(action, phase, palms, wrists, shoulder, lift, primary)
    if action.target.kind == "swing" and action.prop is not None:
        origin = fit_swing(arm, action, phase, orientation, origin, palms, impact, elbow_limit)
    obstacles = arm_obstacles(arm) if action.target.kind == "swing" else None
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
            chain = None
            if arm_limits is not None or action.target.kind in ("swing", "circle"):
                chain = ArmChain.from_capture(upper, lower, arm.pose.bones["chest"], body, elbow_limit, arm_limits,
                                              obstacles)
            grip = offset(hand, phase)
            goal = origin + orientation @ grip if hand.target == "prop" else Vector(action.target.anchor) + grip
            if chain is not None:
                chain.correct(upper, lower, bone, Vector(), pole)
                positions = clips.posed_positions(mesh)
            if hand.target == "prop" and action.target.kind in ("swing", "circle", "carry"):
                follow_hand(arm, side, orientation @ (C.Z if action.target.kind == "swing" else C.X))
                positions = clips.posed_positions(mesh)
            goal = palm.lerp(goal, weight)
            aimed = aim_palm(upper, lower, bone, Vector(positions[i]), goal, 0. if chain is None else chain.min_reach)
            if limit_wrist(lower, bone, wrist_limit) or aimed:
                positions = clips.posed_positions(mesh)
            for _ in range(25):
                error = goal - Vector(positions[i])
                if error.length < .0008:
                    break
                # The sampled palm is weighted to both wrist and forearm on the scans.
                if chain is None:
                    old.adjust_grip(arm, side, .42 * error, pole)
                else:
                    chain.correct(upper, lower, bone, .42 * error, pole)
                limit_wrist(lower, bone, wrist_limit)
                positions = clips.posed_positions(mesh)
            accuracy[side] = (Vector(positions[i]) - goal).length
            if chain is not None:
                arm_direction = (lower.head - upper.head).normalized()
                elevation = max(-1., min(1., arm_direction.dot(chain.up)))
                accuracy[f"elevation_{side}_deg"] = math.degrees(math.asin(elevation))
                accuracy[f"elbow_{side}_deg"] = math.degrees(arm_direction.angle(bone.head - lower.head))
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
        if action.target.kind == "swing" and phase in action.contacts:
            accuracy["tip"] = (prop @ Vector(action.target.tip) - Vector(action.target.anchor)).length
        if action.target.kind == "circle":
            tip = prop @ Vector(action.target.tip) - Vector(action.target.anchor)
            accuracy["tip"] = math.hypot(math.hypot(tip.x, tip.y) - action.target.radius, tip.z + action.target.depth)
    return ActionPose(positions, (sockets[0], sockets[1]), prop, accuracy)
