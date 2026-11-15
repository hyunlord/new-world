"""Assemble frozen authored templates; construction previews never alter simulation sites."""
import json
import math
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import asset_building_concepts as K
import asset_stations as A
import common as C
import stations as S

DATA = Path(__file__).with_suffix('.json')
STAGES = {'foundation': 0.18, 'half': 1.1, 'complete': 2.2}
ALIASES = {'hearth_fire': 'hearth', 'bee_skep_body': 'bee_skep',
           'food_drying_rack': 'drying_rack', 'smoking_rack': 'drying_rack'}


def component(m, placement, stage):
    """Place one authored blueprint, fitting its primitive to its declared tile bounds."""
    kind, x, y, size, mask = placement
    before = set(m.bm.verts)
    height = STAGES[stage]
    base = ALIASES.get(kind, kind)
    if kind.endswith('_wall'):
        style = 'log' if kind == 'log_wall' else 'mud_brick' if kind == 'mud_brick_wall' else 'stone'
        A.masonry(m, style, height)
        wall = [v for v in m.bm.verts if v not in before]
        depth = max(v.co.y for v in wall) - min(v.co.y for v in wall)
        for v in wall:
            v.co.y *= 0.98 / depth
    elif kind == 'fence':
        A.fence(m, mask, False)
    elif kind == 'packed_floor':
        m.box('earth', (0.98, 0.98, 0.06), loc=(0, 0, 0.03))
    elif kind == 'door':
        m.box('wood', (0.82, 0.12, height), loc=(0, 0, height / 2))
    elif base in S.STATIONS:
        getattr(S, base)(m, S.STATIONS[base])
    elif kind == 'hut':
        m.cyl('wood', 1.15, height, segs=12)
        if stage == 'complete':
            m.cyl('thatch', 1.45, 1.25, loc=(0, 0, height), segs=12, r2=0.12)
        m.box('wood_dark', (0.65, 0.05, min(height, 1.5)), loc=(0, -1.16, min(height, 1.5) / 2))
    elif kind == 'bloomery':
        m.cyl('clay', 0.42, min(height, 1.7), segs=10, r2=0.30)
        m.cyl('earth', 0.22, 0.04, loc=(0, 0, min(height, 1.7)), segs=10)
    elif kind == 'forge':
        m.box('stone', (0.95, 0.85, min(height, 0.7)), loc=(0, 0, min(height, 0.7) / 2))
        m.cyl('earth', 0.30, 0.05, loc=(0, 0, min(height, 0.7)), segs=8)
    else:
        raise AssertionError('Unresolved authored component: ' + kind)
    verts = [v for v in m.bm.verts if v not in before]
    assert verts, kind
    xmin, xmax = min(v.co.x for v in verts), max(v.co.x for v in verts)
    ymin, ymax = min(v.co.y for v in verts), max(v.co.y for v in verts)
    sx, sy = min(1, size[0] / max(xmax - xmin, 0.01)), min(1, size[1] / max(ymax - ymin, 0.01))
    zmax = max(v.co.z for v in verts)
    sz = min(1, height / zmax) if stage != 'complete' else 1
    for v in verts:
        v.co.x = (v.co.x - (xmin + xmax) / 2) * sx + x
        v.co.y = (v.co.y - (ymin + ymax) / 2) * sy + y
        v.co.z *= sz


def expand(template, footprints):
    """Mirror unrotated row-major Ring/Interior/Edge/Anchor selectors from geometry.rs."""
    w, h = template['footprint']
    door = (w // 2, h - 1)
    overrides = {door for p in template['pieces'] if p['at'] == 'Edge(South)'}
    plan = []
    for piece in template['pieces']:
        at = piece['at']
        positions = []
        match at:
            case 'Anchor': positions = [(0, 0)]
            case 'Edge(South)': positions = [door]
            case 'Ring' | 'Interior':
                for y in range(h):
                    for x in range(w):
                        border = x in (0, w - 1) or y in (0, h - 1)
                        if (at == 'Ring' and border and (x, y) not in overrides) or (at == 'Interior' and not border):
                            positions.append((x, y))
            case _: raise AssertionError(at)
        size = footprints.get(piece['kind'], [1, 1])
        for x, y in positions[:piece['count']]:
            ring = {(a, b) for b in range(h) for a in range(w) if a in (0, w - 1) or b in (0, h - 1)}
            mask = sum(1 << bit for bit, (dx, dy) in enumerate(((0, -1), (1, 0), (0, 1), (-1, 0))) if (x + dx, y + dy) in ring)
            plan.append((piece['kind'], x + size[0] / 2 - w / 2, h / 2 - y - size[1] / 2, size, mask))
    if template['stations']:
        station = template['stations'][0]
        size = footprints[station]
        plan.append((station, 1 + size[0] / 2 - w / 2, h / 2 - 1 - size[1] / 2, size, 0))
    return plan


def roof(m, template):
    w, h = template['footprint']
    cover = 'thatch' if template['era'] in ('paleolithic', 'neolithic') else 'wood'
    if template['roof'] == 'Round':
        before = set(m.bm.verts)
        m.cyl(cover, w / 2, 1.05, loc=(0, 0, 2.2), segs=12, r2=0.1)
        for v in m.bm.verts:
            if v not in before:
                v.co.y *= h / w
    else:
        angle = math.atan2(1.05, h / 2)
        length = math.hypot(h / 2, 1.05)
        for side in (-1, 1):
            for strip in range(max(4, w * 3)):
                sx = -w / 2 + (strip + 0.5) * w / max(4, w * 3)
                m.box(cover, (w / max(4, w * 3) - 0.012, length, 0.10), loc=(sx, side * h / 4, 2.725), rot=(-side * angle, 0, 0))


def build():
    data = json.loads(DATA.read_text())
    assert len(data['templates']) == 25
    entries = {}
    for template in data['templates']:
        plan = expand(template, data['blueprint_footprints'])
        assert plan
        if template['id'] == 'log_house':
            assert len(plan) == 18 and sum(p[0] == 'door' for p in plan) == 1
        if template['id'] == 'pen':
            assert len(plan) == 16 and all(p[4] in (3, 5, 6, 9, 10, 12) for p in plan if p[0] == 'fence')
        ring = any(p['at'] == 'Ring' and p['kind'].endswith('_wall') for p in template['pieces'])
        for stage in STAGES:
            C.reset_scene()
            mid = 'building_' + template['id'] + '_' + stage
            m = C.Model(mid)
            tints = json.loads((Path(C.REPO) / 'assets/3d/bindings3d.json').read_text())['materials']['by_id']
            tint = tints.get(template['tint'])
            if tint:
                channels = {'oak': ('trunk', 'wood'), 'pine': ('trunk', 'wood'), 'granite': ('stone',), 'clay': ('clay',), 'mud_brick': ('clay',)}
                m.colors.update({channel: tuple(tint) for channel in channels.get(template['tint'], ())})
            if template['id'] in K.KINDS:
                K.concept(m, template, stage)
            else:
                for placement in plan:
                    component(m, placement, stage)
            if stage == 'complete' and (ring or template['id'] in K.KINDS) and template['warmth'] == 'Roofed':
                roof(m, template)
            C.save_static(m, template['footprint'], entries, {
                'family': 'building', 'template': template['id'], 'stage': stage,
                'source_sha': data['source_sha'], 'source_status': 'connected' if template['connected'] else 'inert_concept',
                'representation': 'inert_art_concept' if template['id'] in K.KINDS else 'authored_piece_assembly',
                'roof_cover': 'thatch' if template['era'] in ('paleolithic', 'neolithic') else 'plank',
                'roof_style': template['roof'], 'tint_material': template['tint'],
                'piece_count': len(plan), 'selected_station': next(iter(template['stations']), None),
                'stage_note': 'visual progress preview; not a discrete simulation stage',
            })
    out = Path(C.REPO) / 'assets/3d/buildings_manifest.json'
    out.write_text(json.dumps({'models': entries}, indent=2, sort_keys=True) + '\n')
    assert len(entries) == 75
    return entries


if __name__ == '__main__':
    build()
