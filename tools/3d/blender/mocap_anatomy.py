"""Measure the body of a baked clip in armature space and decoded VAT space."""

from collections.abc import Mapping
from functools import cache
import json
import math
from pathlib import Path
from typing import Final

import numpy as np

from mocap_kinematics import Check, FloatArray, Track, Windows, window_mask
from mocap_spec import BODIES, Plausibility

RIG_LIMITS: Final = Path(__file__).parent / "mocap" / "plausibility_rig.ron"
SEGMENTS: Final = frozenset(("spine", "chest", "neck", "head", "upper_arm", "lower_arm", "hand",
                            "upper_leg", "lower_leg", "foot"))
PENETRATION_LIMIT_M: Final = .01
FACE_CLEARANCE_M: Final = .04
EPSILON: Final = 1.e-6
SIDES: Final = ("R", "L")
FOREARM_CAPSULE: Final = (.13, .85, .032)
Capsules = dict[str, tuple[FloatArray, float]]


@cache
def _rig_limits(path: Path) -> Mapping[str, Mapping[str, float]]:
    presets = json.loads(path.read_text())
    if not isinstance(presets, dict) or presets.keys() != {"default", *BODIES}:
        raise ValueError(f"{path}: expected default limits and the four S2 bodies")
    for kind, row in presets.items():
        if (not isinstance(row, dict) or row.keys() - SEGMENTS or (kind == "default" and row.keys() != SEGMENTS)
                or not all(type(value) in (float, int) and math.isfinite(value) and 0 < value <= 180
                           for value in row.values())):
            raise ValueError(f"{path}: invalid joint limits for {kind}")
    return {body: presets["default"] | presets[body] for body in BODIES}


def joint_limits(body: str) -> Mapping[str, float]:
    """Shared rig angles in degrees for the contact solve and bake gate."""
    return _rig_limits(RIG_LIMITS)[body]


def _angle_degrees(a: FloatArray, b: FloatArray) -> FloatArray:
    """Angle between nonzero vectors, in degrees."""
    cosine = np.sum(a * b, axis=-1) / (np.linalg.norm(a, axis=-1) * np.linalg.norm(b, axis=-1))
    return np.rad2deg(np.arccos(np.clip(cosine, -1., 1.)))


def _trunk_lean(hips: FloatArray, neck: FloatArray, phases: FloatArray,
                work: Windows, limit: float) -> Check:
    angles = _angle_degrees(neck - hips, np.array((0., 0., 1.)))
    all_worst = int(angles.argmax())
    swept = f"all frames max {angles[all_worst]:.3f}° at {all_worst}"
    frames = np.flatnonzero(~window_mask(phases, work))
    if not len(frames):
        return Check(None, None, limit, True, f"all frames inside work windows; {swept}")
    worst = int(frames[angles[frames].argmax()])
    value = float(angles[worst])
    return Check(value, worst, limit, value <= limit + EPSILON, swept)


def _joint_limits(track: Track, body: str) -> Check:
    """Measure bend at the elbow/knee and bind-relative rotation at other joints."""
    limits = joint_limits(body)
    worst: tuple[float, float, int, str] | None = None
    violations: list[str] = []
    for joint, parent in enumerate(track.parents):
        if parent < 0:
            continue
        name = track.names[joint]
        segment = name.removesuffix("_L").removesuffix("_R")
        cap = limits[segment]
        if segment in ("lower_arm", "lower_leg"):
            angles = _angle_degrees(track.matrices[:, parent, :3, 1], track.matrices[:, joint, :3, 1])
        else:
            neutral = track.rest[parent, :3, :3].T @ track.rest[joint, :3, :3]
            upstream = track.matrices[:, parent, :3, :3]
            local = upstream.transpose(0, 2, 1) @ track.matrices[:, joint, :3, :3]
            cosine = (np.sum(neutral * local, axis=(1, 2)) - 1.) / 2.
            angles = np.rad2deg(np.arccos(np.clip(cosine, -1., 1.)))
        frame = int(angles.argmax())
        value = float(angles[frame])
        if worst is None or value - cap > worst[0] - worst[1]:
            worst = (value, cap, frame, name)
        bad = np.flatnonzero(~np.isfinite(angles) | (angles > cap + EPSILON)).tolist()
        if bad:
            violations.append(f"{name} frames {bad}")
    if worst is None:
        raise ValueError("joint limits require at least one non-root bone")
    value, cap, frame, name = worst
    detail = name + (f"; {', '.join(violations)}" if violations else "")
    return Check(value, frame, cap, not violations, detail)


def _point_segment_sq(point: FloatArray, a: FloatArray, b: FloatArray) -> FloatArray:
    line = b - a
    length_sq = np.sum(line * line, axis=1)
    phase = np.sum((point - a) * line, axis=1)
    np.divide(phase, length_sq, out=phase, where=length_sq > 0.)
    offset = point - a - np.clip(phase, 0., 1.)[:, None] * line
    return np.sum(offset * offset, axis=1)


def _capsule_penetration(first: FloatArray, second: FloatArray, r1: float, r2: float) -> FloatArray:
    """Pairwise capsule overlap using the exact closest points on each finite axis."""
    a, b = first[:, 0], first[:, 1]
    c, d = second[:, 0], second[:, 1]
    closest_sq = np.minimum.reduce((_point_segment_sq(a, c, d), _point_segment_sq(b, c, d),
                                    _point_segment_sq(c, a, b), _point_segment_sq(d, a, b)))
    u, v, delta = b - a, d - c, a - c
    uu, vv = np.sum(u * u, axis=1), np.sum(v * v, axis=1)
    uv = np.sum(u * v, axis=1)
    ud, vd = np.sum(u * delta, axis=1), np.sum(v * delta, axis=1)
    divisor = uu * vv - uv * uv
    independent = divisor > 1.e-14 * uu * vv
    s = np.divide(uv * vd - vv * ud, divisor, out=np.zeros_like(divisor), where=independent)
    t = np.divide(uu * vd - uv * ud, divisor, out=np.zeros_like(divisor), where=independent)
    interior = independent & (0. < s) & (s < 1.) & (0. < t) & (t < 1.)
    offset = delta[interior] + u[interior] * s[interior, None] - v[interior] * t[interior, None]
    closest_sq[interior] = np.minimum(closest_sq[interior], np.sum(offset * offset, axis=1))
    return np.maximum(0., r1 + r2 - np.sqrt(closest_sq))


def _bind_capsule(track: Track, name: str, offsets: FloatArray, radius: float) -> tuple[FloatArray, float]:
    """Attach a rest-world cap profile to the sampled global bone matrix."""
    index = track.names.index(name)
    local = offsets @ track.rest[index, :3, :3]
    pose = track.matrices[:, index]
    return local @ pose[:, :3, :3].transpose(0, 2, 1) + pose[:, None, :3, 3], radius


def _bone_capsule(track: Track, name: str, from_y: float, to_y: float,
                  radius: float) -> tuple[FloatArray, float]:
    index = track.names.index(name)
    pose = track.matrices[:, index]
    axis = pose[:, None, :3, 1] * track.lengths[index]
    return pose[:, None, :3, 3] + axis * np.asarray((from_y, to_y))[None, :, None], radius


def torso_capsules(track: Track) -> Capsules:
    # Cap profiles start from PHYS1's S2 bind approximation. The axes stop short of
    # attachment points, where the flesh of adjoining segments is continuous.
    return {
        "hips": _bind_capsule(track, "hips", np.asarray(((-.045, 0., .018), (.045, 0., .018))), .103),
        "spine": _bone_capsule(track, "spine", .14, .76, .103),
        "chest": _bone_capsule(track, "chest", .10, .72, .111),
        "head": _bind_capsule(track, "head", np.asarray(((0., -.023, .065), (0., -.023, .138))), .081),
    }


def forearm_penetration(elbows: FloatArray, wrist: FloatArray, torso: Capsules) -> FloatArray:
    """Measure candidate elbow poles against the same volumes as the bake gate."""
    start, end, radius = FOREARM_CAPSULE
    forearms = elbows[:, None, :] + np.asarray((start, end))[None, :, None] * (wrist - elbows)[:, None, :]
    return np.maximum.reduce([_capsule_penetration(forearms, np.broadcast_to(body, forearms.shape), radius, other)
                              for body, other in torso.values()])


def _self_penetration(track: Track, scale: float) -> Check:
    caps = torso_capsules(track)
    pairs: list[tuple[str, str]] = []
    for side in SIDES:
        hand, forearm, thigh = (f"{segment}_{side}" for segment in ("hand", "lower_arm", "upper_leg"))
        other_hip = f"opposite_hip_{side}"
        other_x = .060 if side == "R" else -.060
        caps[other_hip] = _bind_capsule(track, "hips", np.asarray(((other_x, 0., .018), (other_x, 0., .072))),
                                        .070)
        caps[hand] = _bone_capsule(track, hand, .20, .80, .032)
        caps[forearm] = _bone_capsule(track, forearm, *FOREARM_CAPSULE)
        caps[thigh] = _bone_capsule(track, thigh, .22, .82, .063)
        pairs.extend((limb, core) for limb in (hand, forearm) for core in ("hips", "spine", "chest", "head"))
        # The same-side hip and thigh are contiguous. Test the other half of
        # the pelvis and the torso above the attachment instead.
        pairs.extend((thigh, core) for core in (other_hip, "spine", "chest"))
    worst_value, worst_frame, worst_pair = 0., 0, ""
    for first, second in pairs:
        a, r1 = caps[first]
        b, r2 = caps[second]
        depths = _capsule_penetration(a, b, r1, r2) * scale
        frame = int(depths.argmax())
        value = float(depths[frame])
        if value > worst_value:
            worst_value, worst_frame, worst_pair = value, frame, f"{first}/{second}"
    return Check(worst_value, worst_frame, PENETRATION_LIMIT_M, worst_value <= PENETRATION_LIMIT_M + EPSILON,
                 worst_pair)


def _face_clearance(rest: FloatArray, posed: FloatArray, grips: tuple[int, int], phases: FloatArray,
                    exempt: Windows, scale: float) -> Check:
    """Reuse the apex-close-up masks to scan the visible front head across the clip."""
    frames = np.flatnonzero(~window_mask(phases, exempt))
    if not len(frames):
        return Check(None, None, FACE_CLEARANCE_M, True, "all frames inside hand–face windows")
    height = float(np.ptp(rest[:, 2]))
    head = rest[:, 2] > rest[:, 2].min() + .84 * height
    face = head & (rest[:, 1] <= np.median(rest[head, 1]))
    hands = [np.linalg.norm(rest - rest[grip], axis=1) < .055 for grip in grips]
    closest = min(
        (float(np.linalg.norm(posed[frame][hand, None] - posed[frame][None, face], axis=2).min()) * scale,
         int(frame), side)
        for frame in frames
        for side, hand in zip(SIDES, hands, strict=True)
    )
    value, frame, side = closest
    return Check(value, frame, FACE_CLEARANCE_M, value >= FACE_CLEARANCE_M - EPSILON, f"hand_{side}")


def evaluate_anatomy(track: Track, body: str, scale: float, phases: FloatArray, limits: Plausibility, *,
                     rest_vertices: FloatArray, posed_vertices: FloatArray, grips: tuple[int, int]) -> dict[str, Check]:
    """Apply the four anatomy checks without modifying authored motion."""
    hips, neck = (track.names.index(name) for name in ("hips", "neck"))
    return {
        "trunk": _trunk_lean(track.matrices[:, hips, :3, 3], track.matrices[:, neck, :3, 3], phases,
                             limits.work, limits.lean_limit),
        "joint_limits": _joint_limits(track, body),
        "penetration": _self_penetration(track, scale),
        "face_clearance": _face_clearance(rest_vertices, posed_vertices, grips, phases, limits.face, scale),
    }
