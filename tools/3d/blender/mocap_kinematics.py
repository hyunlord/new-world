"""Shared armature samples and the portable post-IK retarget export."""

from dataclasses import dataclass
from pathlib import Path
from typing import TypeAlias

import numpy as np
from numpy.typing import NDArray

FloatArray: TypeAlias = NDArray[np.float64]
BoolArray: TypeAlias = NDArray[np.bool_]
Windows: TypeAlias = tuple[tuple[float, float], ...]


@dataclass(frozen=True, slots=True)
class Check:
    """The measured extreme and its threshold, or None when inapplicable."""

    value: float | None
    frame: int | None
    limit: float | None
    passed: bool
    detail: str = ""


@dataclass(frozen=True, slots=True)
class Track:
    """Post-IK poses in armature metres, indexed by frame and bone."""

    names: tuple[str, ...]
    parents: tuple[int, ...]
    rest: FloatArray
    matrices: FloatArray
    lengths: FloatArray

    @property
    def heads(self) -> FloatArray:
        return self.matrices[..., :3, 3]

    @property
    def tails(self) -> FloatArray:
        return self.heads + self.matrices[..., :3, 1] * self.lengths[None, :, None]


def window_mask(phases: FloatArray, windows: Windows) -> BoolArray:
    mask = np.zeros(phases.shape, dtype=bool)
    for first, last in windows:
        mask |= (first <= phases) & (phases <= last)
    return mask


def check_prop(extent_m: float | None, height_m: float, limits: tuple[float, float] | None) -> Check:
    """Check the longest local mesh dimension as a fraction of standing rig height."""
    if extent_m is None:
        return Check(None, None, None, True, "no prop")
    ratio = extent_m / height_m
    if limits is None:
        return Check(ratio, None, None, False, "prop_ratio is required for a held prop")
    lo, hi = limits
    return Check(ratio, None, hi, lo <= ratio <= hi, f"allowed [{lo:g}, {hi:g}]")


def local_matrices(global_matrices: FloatArray, parents: tuple[int, ...]) -> FloatArray:
    """Convert the non-root bones from armature space to parent space."""
    local = global_matrices.copy()
    local[..., 1:, :, :] = np.linalg.inv(global_matrices[..., parents[1:], :, :]) @ global_matrices[..., 1:, :, :]
    return local


def rigid_rotations(matrices: FloatArray) -> FloatArray:
    """Remove Blender matrix roundoff before exporting joint rotations."""
    u, _, vt = np.linalg.svd(matrices)
    u[..., :, -1] *= np.where(np.linalg.det(u @ vt) < 0., -1., 1.)[..., None]
    rotations = u @ vt
    if float(np.abs(matrices - rotations).max()) > .001:
        raise ValueError("Retarget export contains a scaled or mirrored joint")
    return rotations


def write_track(path: Path, track: Track, phases: FloatArray, fps: int, body_scale: float) -> None:
    """Export the neutral metre scale, coordinate convention and exact sample clock."""
    pose = local_matrices(track.matrices, track.parents)
    rest = local_matrices(track.rest, track.parents)
    root = track.matrices[:, 0].copy()
    root[:, :3, :3] = rigid_rotations(root[:, :3, :3])
    path.parent.mkdir(parents=True, exist_ok=True)
    np.savez_compressed(
        path, version=np.int32(1), fps=np.float64(fps), body_scale=np.float64(body_scale),
        coordinate_system=np.str_("armature +Z up, -Y forward, metres before body_scale"),
        phases=phases, bone_names=np.asarray(track.names), parent_indices=np.asarray(track.parents, dtype=np.int32),
        local_rotations=rigid_rotations(pose[..., :3, :3]), local_offsets=pose[..., :3, 3],
        root_transforms=root, rest_rotations=rigid_rotations(rest[..., :3, :3]), rest_offsets=rest[..., :3, 3],
        bone_lengths=track.lengths,
    )
