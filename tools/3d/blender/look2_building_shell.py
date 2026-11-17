"""Crafted walls, openings and footings for template-scale building envelopes."""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run through asset_buildings.py using Blender's bundled Python.
import math

import common as C
import look2_building_roofs as R


def doorway(m: C.Model, front: tuple[float, float], stage: str) -> None:
    y, base = front
    for x in (-0.48, 0.48):
        m.rod('wood_dark', (x, y, base), (x, y, 2.07), 0.085, segs=7, cap_mat='wood')
    m.rod('wood', (-0.62, y, 2.04), (0.62, y, 2.04), 0.10, segs=7, cap_mat='wood')
    if stage != 'complete':
        return
    for x in range(4):
        m.box('wood', (0.19, 0.065, 1.53), loc=(-0.3 + x * 0.20, y + 0.08, base + 0.79), jitter=0.005)
    for z in (base + 0.32, base + 1.24):
        m.box('wood_dark', (0.80, 0.075, 0.08), loc=(0, y + 0.025, z))
    m.rod('wood_dark', (-0.35, y - 0.02, base + 0.38), (0.35, y - 0.02, base + 1.2), 0.035, segs=5)
    m.box('stone', (1.12, 0.40, 0.14), loc=(0, y - 0.17, 0.07), jitter=0.008)


def shell(m: C.Model, template: R.BuildingTemplate, stage: str) -> None:
    """Wall courses interlock at alternating corner heights without tile-sized blocks."""
    w, h = template['footprint']
    x, y = w / 2 - 0.52, h / 2 - 0.52
    kinds = {piece['kind'] for piece in template['pieces']}
    logs = 'log_wall' in kinds
    brick = 'mud_brick_wall' in kinds
    mat = 'trunk' if logs else 'clay' if brick else 'stone'
    for side in (-1, 1):
        for n in range(max(3, round(w * 1.5))):
            px = -x + n * 2 * x / (max(3, round(w * 1.5)) - 1)
            m.box('stone', (0.48, 0.38, 0.25), loc=(px, side * y, 0.13), jitter=0.018)
        for n in range(1, max(3, round(h * 1.5)) - 1):
            py = -y + n * 2 * y / (max(3, round(h * 1.5)) - 1)
            m.box('stone', (0.38, 0.48, 0.25), loc=(side * x, py, 0.13), jitter=0.018)
    m.box('earth', (2 * x, 2 * y, 0.12), loc=(0, 0, 0.06))
    if stage == 'foundation':
        return
    for sx in (-1, 1):
        for sy in (-1, 1):
            m.rod('wood_dark', (sx * x, sy * y, 0.22), (sx * x, sy * y, 2.17), 0.09, segs=7)
    doorway(m, (-y - 0.06, 0.20), stage)
    R.roof(m, template, stage)
    if stage == 'frame':
        for sx in (-1, 1):
            m.rod('wood', (sx * x, -y, 0.3), (sx * x, y, 2.1), 0.045, segs=5)
        return
    count = 9 if stage == 'complete' else 4
    for row in range(count):
        z = 0.35 + row * 0.20
        # Split the front around the real door; side walls leave an open window.
        for side in (-1, 1):
            lo, hi = side * 0.57, side * (x + 0.12)
            if logs:
                m.rod(mat, (lo, -y, z), (hi, -y, z + 0.006), 0.105,
                      segs=8, cap_mat='wood', jitter=0.006)
            else:
                m.box(mat, (abs(hi - lo), 0.24, 0.19), loc=((lo + hi) / 2, -y, z), jitter=0.009)
        if logs:
            m.rod(mat, (-x - 0.12, y, z), (x + 0.12, y, z + 0.006), 0.105,
                  segs=8, cap_mat='wood', jitter=0.006)
        else:
            for n in range(6):
                m.box(mat, (2 * x / 6 - 0.015, 0.25, 0.19), loc=(-x + (n + 0.5) * 2 * x / 6, y, z), jitter=0.008)
        for side in (-1, 1):
            spans = [(-y - 0.12, y + 0.12)] if row not in (5, 6) else [(-y - 0.12, -0.40), (0.40, y + 0.12)]
            for start, end in spans:
                if logs:
                    notch_points = [start, start + 0.10, start + 0.18, end - 0.18, end - 0.10, end]
                    vertices = m.tube(mat, [(side * x, py, z + 0.085) for py in notch_points],
                           [0.105, 0.067, 0.105, 0.105, 0.067, 0.105], segs=6)
                    for face in {f for vertex in vertices for f in vertex.link_faces}:
                        if max(v.co.y for v in face.verts) - min(v.co.y for v in face.verts) < 0.00001:
                            face.material_index = m.mats.index('wood')
                else:
                    pieces = max(1, math.ceil((end - start) / 0.55))
                    for n in range(pieces):
                        m.box(mat, (0.25, (end - start) / pieces - 0.015, 0.19),
                              loc=(side * x, start + (n + 0.5) * (end - start) / pieces, z), jitter=0.008)
    if stage == 'complete':
        if template['era'] == 'neolithic' and template['stations']:
            for side in (-1, 1):
                for col in range(5):
                    px = side * (0.66 + col * (x - 0.80) / 4)
                    m.rod('wood_dark', (px, -y - 0.12, 0.38), (px, -y - 0.12, 1.82), 0.018, segs=4)
                for row in range(8):
                    z = 0.44 + row * 0.18
                    points = [(side * (0.62 + col * (x - 0.72) / 6),
                               -y - 0.13 + (0.022 if (col + row) % 2 else -0.022), z)
                              for col in range(7)]
                    m.tube('wood', points, [0.014] * 7, segs=4)
        for side in (-1, 1):
            for z in (1.26, 1.72):
                m.box('wood', (0.33, 0.93, 0.085), loc=(side * x, 0, z))
            for wy in (-0.42, 0.42):
                m.box('wood_dark', (0.30, 0.08, 0.48), loc=(side * x, wy, 1.49))
            m.rod('wood', (side * x, 0, 1.29), (side * x, 0, 1.69), 0.025, segs=5)
        # Low timber porch reads as a built threshold without increasing the footprint.
        for px in (-0.55, 0, 0.55):
            m.box('wood', (0.51, 0.40, 0.07), loc=(px, -h / 2 + 0.21, 0.22))
        if template['stations']:
            m.cyl('clay', 0.19, 0.47, loc=(0.63, y * 0.55, 2.89), segs=10, r2=0.16, caps=False)
            m.cyl('wood_dark', 0.14, 0.025, loc=(0.63, y * 0.55, 3.18), segs=10)


def hut(m: C.Model, stage: str) -> None:
    m.cyl('earth', 1.17, 0.13, segs=20)
    for n in range(12):
        a = n * math.tau / 12
        m.box('stone', (0.34, 0.27, 0.23), loc=(1.10 * math.cos(a), 1.10 * math.sin(a), 0.12), rot=(0, 0, a), jitter=0.008)
    if stage == 'foundation':
        return
    for n in range(16):
        a = n * math.tau / 16
        m.rod('wood_dark', (1.09 * math.cos(a), 1.09 * math.sin(a), 0.13),
              (1.04 * math.cos(a), 1.04 * math.sin(a), 1.90), 0.045, segs=5)
    doorway(m, (-1.03, 0.13), stage)
    R.round_roof(m, stage)
    if stage == 'frame':
        return
    for row in range(8 if stage == 'complete' else 4):
        for n in range(16):
            a, b = n * math.tau / 16, (n + 1) * math.tau / 16
            if n in (11, 12):
                continue
            z = 0.27 + row * 0.20
            m.rod('wood', (1.08 * math.cos(a), 1.08 * math.sin(a), z),
                  (1.08 * math.cos(b), 1.08 * math.sin(b), z + 0.035), 0.06, segs=5)
