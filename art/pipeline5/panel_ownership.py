# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy"]
# ///
# How to run: imported by python -m art.pipeline5.sheet_slice
"""Lossless connected-figure ownership with deterministic Euclidean ties."""
import hashlib
from dataclasses import dataclass

import numpy as np
from PIL import Image
from scipy.ndimage import distance_transform_edt

from art.pipeline3.structure import Point


@dataclass(frozen=True, slots=True)
class FragmentOwnership:
    area: int
    box: tuple[int, int, int, int]
    panel_indices: tuple[int, ...]
    distance: float
    tied_panels: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class OwnershipEvidence:
    source_rgba_sha256: str
    ownership_sha256: str
    visible_pixels: int
    tied_visible_pixels: int
    reconstructed_exactly: bool
    foreign_major_pixels: int
    fragments: tuple[FragmentOwnership, ...]


@dataclass(frozen=True, slots=True)
class OwnedPanels:
    images: tuple[Image.Image, ...]
    boxes: tuple[tuple[int, int, int, int], ...]
    evidence: OwnershipEvidence


def extract_owned(image: Image.Image, groups: list[list[Point]], dust: list[list[Point]]) -> OwnedPanels:
    """Partition original RGBA; no seed deletion, recolouring, or shape repair."""
    rgba = np.asarray(image)
    visible = rgba[:, :, 3] > 0
    seeds = np.zeros(visible.shape, dtype=np.uint8)
    distances = []
    for index, group in enumerate(groups, 1):
        ys = np.array([y for _, y in group])
        xs = np.array([x for x, _ in group])
        seeds[ys, xs] = index
        target = np.zeros(visible.shape, dtype=bool)
        target[ys, xs] = True
        distances.append(distance_transform_edt(~target))
    stack = np.stack(distances)
    nearest = np.argmin(stack, axis=0)
    minimum = np.min(stack, axis=0)
    ties = ((stack == minimum).sum(axis=0) > 1) & visible
    owners = np.where(visible, nearest + 1, 0).astype(np.uint8)
    fragments: list[FragmentOwnership] = []
    for group in dust:
        ys = np.array([y for _, y in group])
        xs = np.array([x for x, _ in group])
        component_distances = stack[:, ys, xs].min(axis=1)
        winner = int(np.argmin(component_distances))
        tied = tuple(int(value) for value in np.flatnonzero(component_distances == component_distances[winner]))
        owners[ys, xs] = winner+1
        fragments.append(FragmentOwnership(len(group),
            (int(xs.min()), int(ys.min()), int(xs.max())+1, int(ys.max())+1),
            (winner,), float(component_distances[winner]), tied))
    foreign = int(np.count_nonzero((seeds > 0) & (owners != seeds)))
    images: list[Image.Image] = []
    boxes: list[tuple[int, int, int, int]] = []
    reconstructed = np.zeros_like(rgba)
    coverage = np.zeros(visible.shape, dtype=np.uint8)
    for index in range(1, len(groups)+1):
        owned = owners == index
        ys, xs = np.nonzero(owned)
        box = (int(xs.min()), int(ys.min()), int(xs.max())+1, int(ys.max())+1)
        x0, y0, x1, y1 = box
        crop = rgba[y0:y1, x0:x1].copy()
        crop[~owned[y0:y1, x0:x1]] = 0
        images.append(Image.fromarray(crop))
        boxes.append(box)
        reconstructed[owned] = rgba[owned]
        coverage[owned] += 1
    exact = bool(np.array_equal(reconstructed[visible], rgba[visible])
                 and np.all(coverage[visible] == 1) and np.all(coverage[~visible] == 0))
    if foreign or not exact:
        message = f'ownership invariant failed: foreign_major={foreign}, exact={exact}'
        raise ValueError(message)
    evidence = OwnershipEvidence(hashlib.sha256(rgba.tobytes()).hexdigest(),
        hashlib.sha256(owners.tobytes()).hexdigest(), int(visible.sum()), int(ties.sum()),
        exact, foreign, tuple(fragments))
    return OwnedPanels(tuple(images), tuple(boxes), evidence)
