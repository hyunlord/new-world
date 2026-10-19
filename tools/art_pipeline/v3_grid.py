# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: imported by pixel_normalize_v3.py
"""Shared-period edge-minus-midpoint lattice recovery."""
from __future__ import annotations
from dataclasses import dataclass
import numpy as np
from numpy.typing import NDArray

@dataclass(frozen=True, slots=True)
class Grid:
    period_xy: tuple[float, float]
    phase_xy: tuple[float, float]
    detected_period_xy: tuple[float, float]
    separate_axes: bool
    improvement_xy: tuple[float, float]


def axis_scores(profile: NDArray[np.float64]) -> tuple[NDArray[np.float64], NDArray[np.float64]]:
    """Return best phase and literal rounded-sample score for each period."""
    scores, phases = [], []
    for s in np.linspace(4.0, 8.0, 201):
        phase = np.arange(0.0, s, 0.1)
        k = np.arange(-1, int(len(profile) / s) + 2)
        edges = np.rint(phase[:, None] + k * s).astype(int)
        mids = np.rint(phase[:, None] + (k + 0.5) * s).astype(int)
        valid_e = (edges >= 1) & (edges < len(profile))
        valid_m = (mids >= 1) & (mids < len(profile))
        energy_e = profile[np.clip(edges, 0, len(profile) - 1)] * valid_e
        energy_m = profile[np.clip(mids, 0, len(profile) - 1)] * valid_m
        values = energy_e.sum(1) / np.maximum(valid_e.sum(1), 1) - energy_m.sum(1) / np.maximum(valid_m.sum(1), 1)
        best = int(np.argmax(values))
        scores.append(float(values[best]))
        phases.append(float(phase[best]))
    return np.asarray(scores), np.asarray(phases)


def detect_grid(luma: NDArray[np.float64], foreground: NDArray[np.bool_]) -> Grid:
    """Share the optimum period unless both axes justify anisotropy by 15%."""
    xp = np.pad((np.abs(np.diff(luma, axis=1)) * (foreground[:, 1:] | foreground[:, :-1])).sum(0), (1, 0))
    yp = np.pad((np.abs(np.diff(luma, axis=0)) * (foreground[1:] | foreground[:-1])).sum(1), (1, 0))
    xs, xp_phase = axis_scores(xp)
    ys, yp_phase = axis_scores(yp)
    ix, iy, shared = int(np.argmax(xs)), int(np.argmax(ys)), int(np.argmax(xs + ys))
    periods = np.linspace(4.0, 8.0, 201)
    improvement = ((xs[ix] - xs[shared]) / max(abs(xs[shared]), 1e-12),
                   (ys[iy] - ys[shared]) / max(abs(ys[shared]), 1e-12))
    separate = abs(periods[ix] - periods[iy]) > 0.3 + 1e-9 and min(improvement) >= 0.15
    ax, ay = (ix, iy) if separate else (shared, shared)
    return Grid((float(periods[ax]), float(periods[ay])),
                (float(xp_phase[ax]), float(yp_phase[ay])),
                (float(periods[ix]), float(periods[iy])), bool(separate),
                (float(improvement[0]), float(improvement[1])))
