"""Assemble frozen authored templates; construction previews never alter simulation sites."""
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import asset_building_concepts as K
import asset_stations as A
import common as C
import stations as S
import stations_w3d as W
import look2_props_details as D
import look2_building_shell as L
import look2_building_roofs as R

DATA = Path(__file__).with_suffix('.json')
STAGES = {'foundation': 0.18, 'frame': 2.2, 'half': 1.1, 'complete': 2.2}
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
        D.detail(m, base)
    elif kind == 'hut':
        m.cyl('wood', 1.15, height, segs=12)
        if stage == 'complete':
            m.cyl('thatch', 1.45, 1.25, loc=(0, 0, height), segs=12, r2=0.12)
        m.box('wood_dark', (0.65, 0.05, min(height, 1.5)), loc=(0, -1.16, min(height, 1.5) / 2))
    elif kind == 'bloomery':
        W.bloomery(m, False)
    elif kind == 'forge':
        W.forge(m, False)
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
            if ring:
                L.shell(m, template, stage)
                if stage == 'complete':
                    for placement in plan:
                        if placement[0] not in ('door', 'packed_floor') and not placement[0].endswith('_wall'):
                            component(m, placement, stage)
                    if template['id'] == 'granary':
                        for vertex in m.bm.verts:
                            vertex.co.z += 0.32
                        for x in (-1.35, 1.35):
                            for y in (-1.35, 1.35):
                                m.cyl('stone', 0.28, 0.35, loc=(x, y, 0), segs=8, r2=0.20)
                        m.box('wood', (1.05, 0.30, 0.16), loc=(0, -1.82, 0.08))
            elif template['id'] == 'hut':
                L.hut(m, stage)
            elif template['id'] in K.KINDS:
                K.concept(m, template, stage)
                if stage != 'foundation':
                    R.roof(m, template, stage)
            elif stage in ('foundation', 'frame'):
                w, h = template['footprint']
                for sx in (-1, 1):
                    for sy in (-1, 1):
                        x, y = sx * (w / 2 - 0.2), sy * (h / 2 - 0.2)
                        m.box('stone', (0.30, 0.30, 0.16), loc=(x, y, 0.08))
                        if stage == 'frame':
                            m.rod('wood', (x, y, 0.12), (x, y, 1.4), 0.045, segs=5)
                m.box('earth', (w - 0.12, h - 0.12, 0.07), loc=(0, 0, 0.035))
            else:
                for placement in plan:
                    component(m, placement, stage)
            for vertex in m.bm.verts:
                vertex.co.z = max(vertex.co.z, 0.0)
            triangles = sum(len(face.verts) - 2 for face in m.bm.faces)
            assert 0 < triangles <= 6500, (mid, triangles)
            C.save_static(m, template['footprint'], entries, {
                'family': 'building', 'template': template['id'], 'stage': stage,
                'source_sha': data['source_sha'], 'source_status': 'connected' if template['connected'] else 'inert_concept',
                'representation': 'inert_art_concept' if template['id'] in K.KINDS else 'authored_footprint_envelope' if ring else 'authored_piece_assembly',
                'look_revision': 'L2-crafted-v1',
                'roof_cover': 'thatch' if template['era'] in ('paleolithic', 'neolithic') else 'plank',
                'roof_style': template['roof'], 'tint_material': template['tint'],
                'piece_count': len(plan), 'selected_station': next(iter(template['stations']), None),
                'stage_note': 'visual progress preview; not a discrete simulation stage',
            })
    out = Path(C.REPO) / 'assets/3d/buildings_manifest.json'
    out.write_text(json.dumps({'models': entries}, indent=2, sort_keys=True) + '\n')
    assert len(entries) == 100
    return entries


if __name__ == '__main__':
    build()
