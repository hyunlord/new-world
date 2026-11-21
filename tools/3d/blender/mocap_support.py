"""Evaluate whole-body balance and foot contacts on the decoded VAT surface."""

from collections.abc import Mapping
from typing import Final

import numpy as np
from numpy.typing import NDArray

from mocap_kinematics import BoolArray, Check, FloatArray, Windows, window_mask
from mocap_spec import Plausibility


# Adult male reference masses from de Leva (1996), table 4
# (doi:10.1016/0021-9290(95)00178-6), with the PHYS1 head/neck split.
# Fractions put each segment's centre along its rig bone
# from head to tail, so the cranial-origin torso/head fractions are reversed.
# The rig's foot ankle-to-toe axis approximates the anatomical foot axis.
LIMB_MASS_AND_CENTRE: Final = {
    "upper_arm": (.0271, .5772),
    "lower_arm": (.0162, .4574),
    "hand": (.0061, .3624),
    "upper_leg": (.1416, .4095),
    "lower_leg": (.0433, .4395),
    "foot": (.0137, .4415),
}
MASS_AND_CENTRE: Final = {
    "hips": (.1117, .3885),
    "spine": (.1633, .5498),
    "chest": (.1596, .7001),
    "neck": (.0094, .5),
    "head": (.0600, .4024),
    **{f"{part}_{side}": values for part, values in LIMB_MASS_AND_CENTRE.items() for side in ("R", "L")},
}
DYNAMIC_MARGIN_M: Final = .03
FOOT_LIFT_M: Final = .05
FOOT_SLIDE_M: Final = .01
# Fixed contact sampling allowance for discretized boot soles at z=0.
# Only vertices within this height can contribute to the support polygon.
GROUND_CONTACT_M: Final = .005
NUMERICAL_EPSILON: Final = 1e-12


def centre_of_mass(names: tuple[str, ...], heads: FloatArray, tails: FloatArray) -> FloatArray:
    """Estimate per-frame whole-body centres from Z-up joint coordinates."""
    masses, fractions = np.array([MASS_AND_CENTRE[name] for name in names]).T
    centres = heads + (tails - heads) * fractions[:, None]
    return np.average(centres, axis=-2, weights=masses)


def _cross(a: tuple[float, float], b: tuple[float, float], c: tuple[float, float]) -> float:
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])


def _nearest_support(point: FloatArray, contacts: FloatArray) -> FloatArray:
    points = sorted(set(map(tuple, contacts)))
    if len(points) == 1:
        return np.asarray(points[0])
    halves = []
    for order in (points, reversed(points)):
        chain: list[tuple[float, float]] = []
        for vertex in order:
            while len(chain) >= 2 and _cross(chain[-2], chain[-1], vertex) <= 0.:
                chain.pop()
            chain.append(vertex)
        halves.extend(chain[:-1])
    hull = np.array(halves)
    edges = np.roll(hull, -1, axis=0) - hull
    delta = point - hull
    cross = edges[:, 0] * delta[:, 1] - edges[:, 1] * delta[:, 0]
    if len(hull) >= 3 and np.all(cross >= -NUMERICAL_EPSILON):
        return point
    along = ((edges * delta).sum(axis=1) / (edges * edges).sum(axis=1)).clip(0., 1.)
    nearest = hull + edges * along[:, None]
    return nearest[np.linalg.norm(point - nearest, axis=1).argmin()]


def support_shift(point: FloatArray, contacts: FloatArray, margin: float) -> FloatArray:
    """Return the shortest XY correction into the supported region and its margin.

    Inputs share one length unit. An unsupported frame has no correction target.
    """
    if len(contacts) == 0:
        return np.zeros_like(point)
    offset = _nearest_support(point, contacts) - point
    distance = float(np.linalg.norm(offset))
    return offset * (1. - margin / distance) if distance > margin else np.zeros_like(point)


def contact_candidates(
    phases: FloatArray,
    names: tuple[str, ...],
    heads: FloatArray,
    tails: FloatArray,
    soles: Mapping[str, FloatArray],
    settings: Plausibility,
) -> tuple[FloatArray, FloatArray]:
    """Return frame-major XY candidates and signed heights above their contact planes."""
    samples_or_joints = list(soles.values())
    heights = [sole[..., 2] for sole in samples_or_joints]
    for support, windows in settings.support.items():
        segment, side = support.split("_")
        if segment == "knee":
            knee = names.index(f"lower_leg_{side}")
            thigh = names.index(f"upper_leg_{side}")
            # The knee contact sphere has a radius of 12% of thigh length.
            radius = .12 * np.linalg.norm(tails[:, thigh] - heads[:, thigh], axis=1)
            samples = heads[:, knee:knee + 1]
        else:
            hand = names.index(f"hand_{side}")
            # The hand contact capsule covers the wrist-to-fingertip bone.
            radius = .25 * np.linalg.norm(tails[:, hand] - heads[:, hand], axis=1)
            samples = np.stack((heads[:, hand], tails[:, hand]), axis=1)
        height = samples[:, :, 2] - radius[:, None]
        active = window_mask(phases, windows)
        samples_or_joints.append(samples)
        heights.append(np.where(active[:, None], height, np.inf))
    return np.concatenate([points[..., :2] for points in samples_or_joints], axis=1), np.concatenate(heights, axis=1)


def _balance(
    phases: FloatArray,
    names: tuple[str, ...],
    heads: FloatArray,
    tails: FloatArray,
    soles: Mapping[str, FloatArray],
    settings: Plausibility,
    airborne: BoolArray,
) -> Check:
    centre = centre_of_mass(names, heads, tails)
    dynamic = window_mask(phases, settings.dynamic)
    points, heights = contact_candidates(phases, names, heads, tails, soles, settings)
    worst = Check(None, None, None, True, "all frames declared airborne")
    # Missing support ranks above any distance breach. Within a category,
    # retain the greatest gap, overshoot or passing distance.
    severity = (0, -1.)
    for frame, point in enumerate(centre[:, :2]):
        contacts = points[frame, heights[frame] <= GROUND_CONTACT_M]
        limit = DYNAMIC_MARGIN_M if dynamic[frame] else 0.
        if len(contacts) == 0:
            if not airborne[frame]:
                clearance = float(heights[frame].min())
                key = (2, clearance)
                if key > severity:
                    severity = key
                    worst = Check(None, frame, limit, False,
                                  f"no ground support outside an airborne window, lowest point {clearance:.4f} m")
            continue
        distance = float(np.linalg.norm(support_shift(point, contacts, 0.)))
        passed = distance <= limit + NUMERICAL_EPSILON
        key = (0, distance) if passed else (1, distance - limit)
        if key > severity:
            severity = key
            worst = Check(distance, frame, limit, passed)
    return worst


def _lift(
    phases: FloatArray, soles: Mapping[str, FloatArray], settings: Plausibility, airborne: BoolArray,
) -> Check:
    worst = Check(None, None, FOOT_LIFT_M, True, "all foot lifts inside declared step or airborne windows")
    for side, sole in soles.items():
        ordinary = ~(window_mask(phases, settings.steps.get(side, ())) | airborne)
        frames = np.flatnonzero(ordinary)
        if len(frames) == 0:
            continue
        heights = sole[frames, :, 2].min(axis=1).clip(0., None)
        peak = int(heights.argmax())
        value = float(heights[peak])
        if worst.value is None or value > worst.value:
            worst = Check(value, int(frames[peak]), FOOT_LIFT_M, value <= FOOT_LIFT_M, f"foot {side}")
    return worst


def _plant_runs(phases: FloatArray, plants: Windows, loop: bool) -> list[NDArray[np.intp]]:
    runs = [np.flatnonzero(window_mask(phases, (window,))) for window in plants]
    if loop and len(runs) > 1 and plants[0][0] == 0. and plants[-1][1] == 1.:
        return [np.concatenate((runs[-1], runs[0])), *runs[1:-1]]
    return runs


def _slide(
    phases: FloatArray, soles: Mapping[str, FloatArray], settings: Plausibility,
    foot_lock: bool, loop: bool, stride: FloatArray | None,
) -> Check:
    worst = Check(None, None, FOOT_SLIDE_M, True, "no sampled plant windows")
    for side, sole in soles.items():
        plants = ((0., 1.),) if foot_lock else settings.plants.get(side, ())
        for frames in _plant_runs(phases, plants, loop):
            if len(frames) == 0:
                continue
            if worst.value is None:
                worst = Check(0., int(frames[0]), FOOT_SLIDE_M, True, f"foot {side}")
            positions = sole[frames]
            if stride is not None:
                cycles = np.cumsum(np.r_[False, np.diff(frames) < 0])
                positions = positions + (phases[frames] + cycles)[:, None, None] * stride
            for first, start in enumerate(positions[:-1]):
                # Exact diameter of the sampled trajectory for each sole vertex,
                # including excursions in opposite directions from the first pose.
                delta = positions[first + 1:] - start
                distance_sq = (delta * delta).sum(axis=2).max(axis=1)
                peak = int(distance_sq.argmax())
                value = float(np.sqrt(distance_sq[peak]))
                if value > worst.value:
                    end = int(frames[first + 1 + peak])
                    worst = Check(value, end, FOOT_SLIDE_M, value <= FOOT_SLIDE_M + NUMERICAL_EPSILON,
                                  f"foot {side}, compared with frame {frames[first]}")
    return worst


def evaluate_support(
    phases: FloatArray,
    names: tuple[str, ...],
    heads: FloatArray,
    tails: FloatArray,
    soles: Mapping[str, FloatArray],
    settings: Plausibility,
    *,
    foot_lock: bool,
    loop: bool,
    stride: FloatArray | None = None,
) -> dict[str, Check]:
    """Check scaled VAT metres, restoring one cycle of stride for world-space slide."""
    airborne = window_mask(phases, settings.airborne)
    return {
        "balance": _balance(phases, names, heads, tails, soles, settings, airborne),
        "foot_lift": _lift(phases, soles, settings, airborne),
        "foot_slide": _slide(phases, soles, settings, foot_lock, loop, stride),
    }
