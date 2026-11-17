"""Layered roof coverings and exposed rafters for authored building previews."""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run through asset_buildings.py using Blender's bundled Python.
import math
from typing import TypedDict

import common as C


class Piece(TypedDict):
    kind: str
    at: str
    count: int | None


class BuildingTemplate(TypedDict):
    id: str
    footprint: list[int]
    era: str
    pieces: list[Piece]
    stations: list[str]
    roof: str
    warmth: str
    tint: str
    connected: bool


def roof(m: C.Model, template: BuildingTemplate, stage: str) -> None:
    """Keep roof overhang inside the authored tile footprint."""
    w, h = template['footprint']
    early = template['era'] in ('paleolithic', 'neolithic')
    cover = 'thatch' if early else 'wood'
    ridge, eave = 3.35, 2.12
    half_width, half_depth = w / 2 - 0.12, h / 2 - 0.12
    angle = math.atan2(ridge - eave, half_width)
    length = math.hypot(half_width, ridge - eave)
    for y in (-half_depth + 0.14, 0, half_depth - 0.14):
        for side in (-1, 1):
            m.rod('wood_dark', (0, y, ridge), (side * half_width, y, eave), 0.075, segs=7, cap_mat='wood')
    m.rod('trunk', (0, -half_depth, ridge), (0, half_depth, ridge), 0.12, segs=8, cap_mat='wood')
    if stage == 'frame':
        return
    rows = 4 if early else 6
    columns = max(8, round(h * 3))
    for side in (-1, 1):
        for row in range(rows):
            if stage == 'half' and (side > 0 or row > 1):
                continue
            t = (row + 0.5) / rows
            for col in range(columns):
                y = -half_depth + (col + 0.5) * 2 * half_depth / columns
                x = side * half_width * t
                z = ridge + (eave - ridge) * t + (rows - row) * 0.025
                thickness = 0.17 if early else 0.075
                m.box(cover, (length / rows + 0.18, 2 * half_depth / columns - 0.015,
                             thickness), loc=(x, y, z), rot=(0, side * angle, 0),
                      taper=0.9, jitter=0.012 if early else 0.005)
        if early and stage == 'complete':
            for t in (0.3, 0.68):
                x = side * half_width * t
                z = ridge + (eave - ridge) * t + 0.15
                m.rod('wood_dark', (x, -half_depth + 0.04, z),
                      (x, half_depth - 0.04, z), 0.035, segs=5)
    if stage == 'complete':
        for y in (-half_depth + 0.28, 0, half_depth - 0.28):
            m.rod('leather', (-0.13, y, ridge + 0.04), (0.13, y, ridge + 0.04), 0.035, segs=6)
        # A framed dark gable opening stays legible below the overhang at game zoom.
        for y in (-h / 2 + 0.47, h / 2 - 0.47):
            for n in range(7):
                z = 2.23 + n * 0.115
                span = max(0.12, (ridge - z) / (ridge - eave) * (w / 2 - 0.5))
                m.rod('wood', (-span, y, z), (span, y, z), 0.025, segs=4)
            for x in (-0.35, 0.35):
                m.rod('wood_dark', (x, y - 0.035, 2.22), (x, y - 0.035, 2.86), 0.035, segs=5)


def round_roof(m: C.Model, stage: str) -> None:
    """Overlapping circular reed skirts leave a real smoke throat."""
    for n in range(12):
        a = n * math.tau / 12
        m.rod('wood_dark', (0.18 * math.cos(a), 0.18 * math.sin(a), 3.05),
              (1.32 * math.cos(a), 1.32 * math.sin(a), 1.78), 0.045, segs=5)
    if stage == 'frame':
        return
    for row in range(4):
        if stage == 'half' and row > 1:
            continue
        bottom = 1.40 - row * 0.29
        top = bottom - 0.37
        z = 1.73 + row * 0.32
        rings = [[m.bm.verts.new((radius * math.cos(n * math.tau / 24),
                                 radius * math.sin(n * math.tau / 24), height)) for n in range(24)]
                 for radius, height in ((bottom, z), (bottom, z + 0.14),
                                        (top, z + 0.52), (top - 0.025, z + 0.43))]
        for a, b in zip(rings, rings[1:] + rings[:1]):
            for n in range(24):
                m.bm.faces.new((a[n], a[(n + 1) % 24], b[(n + 1) % 24], b[n]))
        # Fixed-angle rings avoid the generic tube's reversal on descending profiles.
        m._finish_part([v for ring in rings for v in ring], 'thatch', None)
    if stage == 'complete':
        m.cyl('wood_dark', 0.21, 0.14, loc=(0, 0, 3.08), segs=12, caps=False)
