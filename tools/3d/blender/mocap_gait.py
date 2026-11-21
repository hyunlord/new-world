"""Lock alternating mocap stances in the moving character's ground frame."""

from collections.abc import Callable, Mapping
from dataclasses import dataclass
import math
from typing import Final

import bpy
from mathutils import Quaternion, Vector
import numpy as np

from mocap_kinematics import FloatArray, Windows
from mocap_support import GROUND_CONTACT_M, centre_of_mass, support_shift
from s2clips_pose import solve

STANCE_BLEND: Final = .07
FLOOR_M: Final = .001
CORRECTION_MARGIN_M: Final = .012
# With a pinned ankle, the knee and thigh follow only part of the pelvis shift.
PELVIS_COM_FRACTION: Final = .80


@dataclass(frozen=True, slots=True)
class Plant:
    first: float
    last: float
    anchor: Vector

    def sample(self, phase: float) -> tuple[float, float, float]:
        """Return the phase in this foot's cycle, its edge distance and blend."""
        centre = (self.first + self.last) / 2
        local = phase - math.floor(phase - centre + .5)
        depth = min(local - self.first, self.last - local)
        t = min(1., max(0., 1. + depth / STANCE_BLEND))
        return local, depth, t * t * (3. - 2. * t)


def _joined(windows: Windows) -> Windows:
    """Unwrap a plant that continues across the animation seam."""
    if len(windows) > 1 and windows[0][0] == 0. and windows[-1][1] == 1.:
        return (*windows[1:-1], (windows[-1][0], 1. + windows[0][1]))
    return windows


@dataclass(frozen=True, slots=True)
class Foot:
    upper: bpy.types.PoseBone
    lower: bpy.types.PoseBone
    ankle: bpy.types.PoseBone
    flat: Quaternion
    sole: FloatArray
    plants: tuple[Plant, ...]

    def floor(self, rotation: Quaternion) -> FloatArray:
        return self.sole @ np.asarray(rotation.to_matrix()).T


@dataclass(frozen=True, slots=True)
class Goal:
    foot: Foot
    anchor: Vector
    head: Vector
    rotation: Quaternion
    depth: float
    blend: float


class Gait:
    def __init__(self, arm: bpy.types.Object, stride: Vector, plants: Mapping[str, Windows],
                 rest_soles: Mapping[str, FloatArray], sample: Callable[[float], tuple[float, float]],
                 scale: float) -> None:
        self.arm = arm
        self.stride = stride
        self.scale = scale
        bones = arm.pose.bones
        feet = []
        for side, points in rest_soles.items():
            upper, lower, ankle = (bones[f"{part}_{side}"] for part in ("upper_leg", "lower_leg", "foot"))
            rest = ankle.bone.matrix_local
            flat = rest.to_quaternion()
            sole = (points - np.asarray(rest.translation)) @ np.asarray(flat.to_matrix())
            anchors = []
            for first, last in _joined(plants.get(side, ())):
                positions = []
                for phase in np.linspace(first + .25 * (last - first), last - .25 * (last - first), 5):
                    sample(float(phase % 1.))
                    positions.append(np.asarray(ankle.head + stride * phase))
                point = Vector(np.median(positions, axis=0))
                point.z = float(rest.translation.z - points[:, 2].min() + FLOOR_M / scale)
                anchors.append(Plant(first, last, point))
            feet.append(Foot(upper, lower, ankle, flat, sole, tuple(anchors)))
        self.feet = tuple(feet)

    def pose(self, phase: float) -> None:
        """Solve the stance anchors and weight transfer before hand and prop IK."""
        phase %= 1.
        goals = []
        for foot in self.feet:
            candidates = [(plant, *plant.sample(phase)) for plant in foot.plants]
            if not candidates:
                continue
            plant, local, depth, blend = max(candidates, key=lambda row: row[2])
            head = foot.ankle.head.copy()
            rotation = foot.ankle.matrix.to_quaternion().slerp(foot.flat, blend)
            goals.append(Goal(foot, plant.anchor - self.stride * local, head, rotation, depth, blend))

        bones = self.arm.pose.bones
        names = tuple(bone.name for bone in bones)
        heads = np.asarray([tuple(bone.head) for bone in bones])
        tails = np.asarray([tuple(bone.tail) for bone in bones])
        centre = centre_of_mass(names, heads, tails)[:2]
        shifts, weights = [], []
        for goal in goals:
            if goal.depth >= -1.e-10:
                sole = goal.foot.floor(goal.foot.flat) + np.asarray(goal.anchor)
                contacts = sole[sole[:, 2] * self.scale <= GROUND_CONTACT_M, :2]
                shifts.append(support_shift(centre, contacts, CORRECTION_MARGIN_M / self.scale))
                weights.append(max(goal.depth, 1.e-9))
        xy = np.average(shifts, weights=weights, axis=0) / PELVIS_COM_FRACTION if shifts else (0., 0.)
        shift = Vector((float(xy[0]), float(xy[1]), 0.))

        targets = []
        for goal in goals:
            foot = goal.foot
            anchor = (goal.head + shift).lerp(goal.anchor, goal.blend)
            floor = FLOOR_M / self.scale - float(foot.floor(goal.rotation)[:, 2].min())
            anchor.z = max(anchor.z, floor)
            targets.append((goal, anchor, foot.lower.head - foot.upper.head))

        for goal, anchor, _ in targets:
            foot = goal.foot
            # Only a load-bearing sole can demand pelvis lowering. The IK
            # reach clamp eases a rolling or swinging foot out of a long step.
            if goal.depth >= -1.e-10:
                hip = foot.upper.head
                horizontal = (hip.x + shift.x - anchor.x) ** 2 + (hip.y + shift.y - anchor.y) ** 2
                reach = foot.upper.bone.length + foot.lower.bone.length - .006
                shift.z = min(shift.z, anchor.z - hip.z + math.sqrt(max(.01, reach ** 2 - horizontal)))
        hips = bones["hips"]
        hips.location += hips.bone.matrix_local.to_quaternion().inverted() @ shift
        bpy.context.view_layer.update()

        for goal, anchor, pole in targets:
            foot = goal.foot
            # Preserve the captured swing height when the support leg lowers
            # the pelvis. Otherwise the lifted foot can scrub the floor.
            if goal.blend == 0. and (foot.ankle.head - anchor).length < 1.e-5:
                continue
            if goal.depth < -1.e-10:
                # Shorten an unreachable swing horizontally before the
                # chain clamp can lift the entire foot above its arc.
                hip = foot.upper.head
                offset = anchor - hip
                reach = foot.upper.bone.length + foot.lower.bone.length - .006
                width = math.sqrt(max(0., reach ** 2 - offset.z ** 2))
                length = math.hypot(offset.x, offset.y)
                if length > width:
                    anchor.x = hip.x + offset.x * width / length
                    anchor.y = hip.y + offset.y * width / length
            solve(foot.upper, foot.lower, anchor, pole + Vector((0., -.15, 0.)))
            matrix = goal.rotation.to_matrix().to_4x4()
            matrix.translation = foot.ankle.head
            foot.ankle.matrix = matrix
            bpy.context.view_layer.update()
