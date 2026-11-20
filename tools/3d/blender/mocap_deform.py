"""Measure stretch and shoulder/elbow collapse against the prepared A-pose mesh."""

from dataclasses import dataclass
from typing import Final

import numpy as np
from numpy.typing import NDArray

# The S2 rig is 1.60 m tall. Microedges below 2 mm amplify scanner and bake
# noise. Thin source slivers also amplify position error in area calculations.
MIN_EDGE_M: Final = .002
MIN_ALTITUDE_M: Final = .001
MIN_ALTITUDE_TO_EDGE: Final = .10
JOINTS: Final = ("shoulder_R", "elbow_R", "shoulder_L", "elbow_L")


class DeformationInputError(ValueError):
    """The rest mesh or its pose samples cannot support the deformation check."""


@dataclass(frozen=True, slots=True)
class FrameDeformation:
    max_edge_ratio: float
    max_edge_vertices: tuple[int, int]
    max_body_edge_ratio: float
    max_body_edge_vertices: tuple[int, int]
    min_area_ratio: float
    min_area_triangle: int
    min_area_joint: str


@dataclass(frozen=True, slots=True)
class DeformationReport:
    passed: bool
    max_stretch: float
    min_area: float
    min_rest_edge_m: float
    min_rest_altitude_m: float
    min_rest_altitude_to_edge: float
    edge_scope: str
    edge_sample_count: int
    body_edge_sample_count: int
    edge_total_count: int
    triangle_sample_count: int
    joint_triangle_counts: dict[str, int]
    max_edge_ratio: float
    max_edge_frame: int
    max_body_edge_ratio: float
    max_body_edge_frame: int
    min_area_ratio: float
    min_area_frame: int
    frames: tuple[FrameDeformation, ...]


def _edge_lengths(points: NDArray[np.float64], edges: NDArray[np.intp]) -> NDArray[np.float64]:
    return np.linalg.norm(points[edges[:, 1]] - points[edges[:, 0]], axis=1)


def _double_areas(points: NDArray[np.float64], triangles: NDArray[np.intp]) -> NDArray[np.float64]:
    origin, second, third = np.moveaxis(points[triangles], 1, 0)
    return np.linalg.norm(np.cross(second - origin, third - origin), axis=1)


def _joint_regions(corner_weights: NDArray[np.float64]) -> NDArray[np.bool_]:
    # Mean corner weights approximate the influence at each face's centroid.
    # Shoulder faces blend upper arm with other skin influences and have little
    # lower-arm weight. Elbow faces have substantial weight from both arm bones.
    weights = corner_weights.mean(axis=1)
    upper, lower = weights[:, ::2], weights[:, 1::2]
    shoulder = (upper >= .12) & (upper + lower <= .88) & (lower <= .15 * upper)
    elbow = (upper >= .12) & (lower >= .12) & (upper + lower >= .55)
    return np.column_stack((shoulder[:, 0], elbow[:, 0], shoulder[:, 1], elbow[:, 1]))


@dataclass(frozen=True, slots=True)
class Samples:
    edges: NDArray[np.intp]
    lengths: NDArray[np.float64]
    stable_edges: NDArray[np.bool_]
    joint_edges: NDArray[np.bool_]
    joint_faces: NDArray[np.bool_]
    rest_areas: NDArray[np.float64]


def _samples(rest: np.ndarray, triangles: np.ndarray, arm_weights: np.ndarray) -> Samples:
    sides = triangles[:, [[0, 1], [1, 2], [2, 0]]]
    flat = sides.reshape(-1, 2)
    edges, side_to_edge = np.unique(np.sort(flat, axis=1), axis=0, return_inverse=True)
    lengths = _edge_lengths(rest, edges)
    stable_edges = lengths >= MIN_EDGE_M
    side_lengths = lengths[side_to_edge].reshape(-1, 3)
    longest = side_lengths.max(axis=1)
    rest_areas = _double_areas(rest, triangles)
    area_ok = ((side_lengths.min(axis=1) >= MIN_EDGE_M)
               & (rest_areas >= longest * np.maximum(MIN_ALTITUDE_M, MIN_ALTITUDE_TO_EDGE * longest)))
    joint_regions = _joint_regions(arm_weights[triangles])
    joint_edges = np.zeros(len(edges), dtype=np.bool_)
    joint_edges[side_to_edge.reshape(-1, 3)[joint_regions.any(axis=1)]] = True
    return Samples(edges, lengths, stable_edges, joint_edges & stable_edges, joint_regions & area_ok[:, None],
                   rest_areas)


def joint_geometry(rest: np.ndarray, triangles: np.ndarray, arm_weights: np.ndarray) -> tuple[np.ndarray, ...]:
    """Return the same rest edge and area constraints used by the build guard."""
    samples = _samples(rest, triangles, arm_weights)
    faces = samples.joint_faces.any(axis=1)
    return (samples.edges, samples.lengths, samples.edges[samples.joint_edges], samples.lengths[samples.joint_edges],
            triangles[faces], samples.rest_areas[faces])


def check_deformation(
    rest: NDArray[np.float64], frames: NDArray[np.float64], triangles: NDArray[np.intp],
    arm_weights: NDArray[np.float64], *, max_stretch: float = 1.35, min_area: float = .35,
    local_edges: bool = False,
) -> DeformationReport:
    """Check all sampled frames and return the extreme rest-relative ratios.

    The rest (V, 3) and posed (F, V, 3) points share metre scale and vertex
    order. Faces (T, 3) use stable vertex indices. Arm weights (V, 4) are in
    upper_arm_R, lower_arm_R, upper_arm_L, lower_arm_L order.

    Stretch uses unique body edges. With local_edges, the stretch limit
    applies to edges at the four arm joints and whole-body maxima are reported
    separately. Collapse uses unsigned triangle area at those joints. The
    area floor allows a 41% uniform shrink in linear dimensions. Rest-only
    masks exclude source degeneracies, so eligible triangles may still
    collapse to zero and fail. Frame and mesh indices are zero-based. Equal
    values select the first occurrence.
    """
    rest = np.asarray(rest, dtype=np.float64)
    frames = np.asarray(frames, dtype=np.float64)
    arm_weights = np.asarray(arm_weights, dtype=np.float64)
    if rest.ndim != 2 or rest.shape[1] != 3 or not len(rest):
        raise DeformationInputError("rest vertices must have shape (V, 3) and be nonempty")
    if frames.ndim != 3 or not len(frames) or frames.shape[1:] != rest.shape:
        raise DeformationInputError("posed frames must have shape (F, V, 3) and be nonempty")
    if not np.isfinite(rest).all() or not np.isfinite(frames).all():
        raise DeformationInputError("rest vertices and posed frames must be finite")
    if (triangles.ndim != 2 or triangles.shape[1] != 3 or not len(triangles)
            or not np.issubdtype(triangles.dtype, np.integer)
            or triangles.min() < 0 or triangles.max() >= len(rest)):
        raise DeformationInputError("triangles must have shape (T, 3) and index the nonempty rest mesh")
    if (arm_weights.shape != (len(rest), 4) or not np.isfinite(arm_weights).all()
            or arm_weights.min() < 0. or arm_weights.sum(axis=1).max() > 1.00001):
        raise DeformationInputError("arm weights must be four finite partial skin weights per vertex")
    if not np.isfinite(max_stretch) or max_stretch < 1. or not 0. < min_area <= 1.:
        raise DeformationInputError("stretch limit must be at least 1 and area floor in (0, 1]")

    samples = _samples(rest, triangles, arm_weights)
    body_edges, body_lengths = samples.edges[samples.stable_edges], samples.lengths[samples.stable_edges]
    if not len(body_edges):
        raise DeformationInputError("mesh has no edges at or above the rest length floor")

    joint_mask = samples.joint_faces
    joint_counts = {joint: int(count) for joint, count in zip(JOINTS, joint_mask.sum(axis=0))}
    if missing := [joint for joint, count in joint_counts.items() if not count]:
        raise DeformationInputError(f"no eligible rest triangles at {' '.join(missing)}")
    triangle_ids = np.flatnonzero(joint_mask.any(axis=1))
    sample_triangles, sample_areas = triangles[triangle_ids], samples.rest_areas[triangle_ids]
    joint_by_triangle = joint_mask[triangle_ids].argmax(axis=1)
    edge_ids = np.arange(len(body_edges))
    if local_edges:
        # Edge stability depends on its own length. Keep an eligible edge
        # when a neighboring joint face is too thin for the area check.
        edge_ids = np.flatnonzero(samples.joint_edges[samples.stable_edges])

    metrics: list[FrameDeformation] = []
    for posed in frames:
        edge_ratios = _edge_lengths(posed, body_edges) / body_lengths
        area_ratios = _double_areas(posed, sample_triangles) / sample_areas
        edge_index = int(edge_ids[edge_ratios[edge_ids].argmax()])
        body_index = int(edge_ratios.argmax())
        area_index = int(area_ratios.argmin())
        edge_a, edge_b = body_edges[edge_index]
        body_a, body_b = body_edges[body_index]
        metrics.append(FrameDeformation(
            max_edge_ratio=float(edge_ratios[edge_index]), max_edge_vertices=(int(edge_a), int(edge_b)),
            max_body_edge_ratio=float(edge_ratios[body_index]), max_body_edge_vertices=(int(body_a), int(body_b)),
            min_area_ratio=float(area_ratios[area_index]), min_area_triangle=int(triangle_ids[area_index]),
            min_area_joint=JOINTS[joint_by_triangle[area_index]],
        ))
    max_frame = max(range(len(metrics)), key=lambda frame: metrics[frame].max_edge_ratio)
    max_body_frame = max(range(len(metrics)), key=lambda frame: metrics[frame].max_body_edge_ratio)
    min_frame = min(range(len(metrics)), key=lambda frame: metrics[frame].min_area_ratio)
    max_ratio, min_ratio = metrics[max_frame].max_edge_ratio, metrics[min_frame].min_area_ratio
    return DeformationReport(
        passed=max_ratio <= max_stretch and min_ratio >= min_area,
        max_stretch=max_stretch, min_area=min_area, min_rest_edge_m=MIN_EDGE_M,
        min_rest_altitude_m=MIN_ALTITUDE_M, min_rest_altitude_to_edge=MIN_ALTITUDE_TO_EDGE,
        edge_scope="shoulder_elbow" if local_edges else "body",
        edge_sample_count=len(edge_ids), body_edge_sample_count=len(body_edges), edge_total_count=len(samples.edges),
        triangle_sample_count=len(sample_triangles), joint_triangle_counts=joint_counts,
        max_edge_ratio=max_ratio, max_edge_frame=max_frame, min_area_ratio=min_ratio, min_area_frame=min_frame,
        max_body_edge_ratio=metrics[max_body_frame].max_body_edge_ratio, max_body_edge_frame=max_body_frame,
        frames=tuple(metrics),
    )
