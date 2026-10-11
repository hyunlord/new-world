"""Audit observed Pass 5 wood columns against captured renderer and label state."""
from __future__ import annotations
from dataclasses import replace
import json
import math
from pathlib import Path
import re
import numpy as np
from ingame_qa_pixels import Check, night, result
from ingame_qa_types import Capture, box, luma


def fire_night(noon: Capture, dark: Capture) -> Check:
    """Keep the old lit-hearth luminance gate and independently audit unlit glow."""
    data = json.loads(dark.path.with_suffix('.entities.json').read_text())
    hearths = [row for row in data['entities'] if row['kind'] == 'building' and row.get('building_kind') == 0 and row.get('visible')]
    unknown = [row['id'] for row in hearths if 'fire_lit' not in row]
    lit = {row['id'] for row in hearths if row.get('fire_lit') == 1}
    filtered = replace(dark, entities=tuple(e for e in dark.entities if e.kind != 'building' or e.building_kind != 0 or e.ident in lit))
    old = night(noon, filtered)
    violations = []
    for row in hearths:
        if row.get('fire_lit') == 0:
            glows = [e for e in data['entities'] if e['kind'] == 'hearth_glow' and e.get('id') == row['id']]
            if len(glows) != 1 or any(float(e.get('glow_alpha', 1)) > .01 for e in glows):
                violations.append(row['id'])
    means = []
    for frame in (noon,dark):
        pixels = luma(frame.image())
        world = np.ones(pixels.shape,dtype=bool)
        for panel in frame.entities:
            if panel.kind == 'panel' and panel.visible:
                x0,y0,x1,y1 = box(panel,frame.scale)
                world[max(0,y0):min(1080,y1),max(0,x0):min(1920,x1)] = False
        means.append(float(pixels[world].mean()))
    passed = means[1] <= .65*means[0] and bool(hearths) and not unknown and not violations and (old.status == 'PASS' if lit else True)
    return result('A6', passed, json.dumps({'world_means': means, 'lit_hearths': len(lit), 'lit_check': old.detail, 'unknown': unknown, 'unlit_glow_violations': violations}))


def wood(captures: list[Capture], repo: Path) -> Check:
    """Every in-view resource is checked independently of the rendered sprite list."""
    atlas = json.loads((repo/'assets/art/atlas/world_atlas.json').read_text())['sprites']
    errors = []
    totals = {'hearths': 0, 'deadwood': 0, 'hud': 0, 'carrying': 0, 'real_carry': 0, 'fallback_carry': 0, 'displaced_wood_carriers': 0, 'displaced_now_stationary': 0}
    carry_seen = False
    def accepted(primary: str, alternate: str, fallback: str) -> str:
        for ident in (primary, alternate):
            entry = atlas.get(ident, {})
            if entry.get('qa_status') == 'accepted' and entry.get('drawable', True):
                return ident
        return fallback
    for capture in captures:
        data = json.loads(capture.path.with_suffix('.entities.json').read_text())
        entities = data['entities']
        settlement = data.get('settlement_snapshot', {})
        resources = data.get('resource_snapshot', {})
        required = ('settlement_ids','formation_xs','formation_ys','wood_stocks','fire_lit')
        if not all(key in settlement for key in required):
            errors.append(capture.stem+': missing settlement columns'); continue
        lengths = {len(settlement[key]) for key in required}
        if len(lengths) != 1 or not settlement['settlement_ids']:
            errors.append(capture.stem+': inconsistent settlement columns'); continue
        for row in entities:
            if row['kind'] != 'building' or row.get('building_kind') != 0 or not row.get('visible'):
                continue
            totals['hearths'] += 1
            x,y = row['tile_x'],row['tile_y']
            index = min(range(len(settlement['settlement_ids'])), key=lambda i: (max(abs(x-settlement['formation_xs'][i]),abs(y-settlement['formation_ys'][i])),settlement['settlement_ids'][i]))
            stock,fire = settlement['wood_stocks'][index],settlement['fire_lit'][index]
            valid = {'S1_02','S1_03','S1_04','S1_05'} if fire == 1 else {'S1_01' if stock > 0 else 'S1_16'}
            stage = 0 if stock <= 0 else (1 if stock < 40 else (2 if stock < 150 else 3))
            if row.get('sprite_id') not in valid or row.get('settlement_id') != settlement['settlement_ids'][index] or row.get('woodpile_stage') != stage:
                errors.append(f'{capture.stem}: hearth {row["id"]} state/frame/stage')
            piles = [e for e in entities if e['kind'] == 'camp_prop' and e.get('id') == row['id']*100+1 and e.get('woodpile_stage',0) > 0 and e.get('shown')]
            expected_pile = accepted(f'W1_{8+stage:02d}',f'W1_{12+stage:02d}','S2_15' if stage == 1 else 'S1_14')
            if (stage == 0 and piles) or (stage > 0 and (len(piles) != 1 or piles[0].get('sprite_id') != expected_pile)):
                errors.append(f'{capture.stem}: hearth {row["id"]} pile sprite/count')
        split_labels = [row for row in entities if row['kind'] == 'stockpile_wood_label']
        labels = split_labels or [row for row in entities if row['kind'] == 'stockpile_label']
        stockpiles = [row for row in entities if row['kind'] == 'building' and row.get('building_kind') == 2]
        assigned = {}
        for i in range(len(settlement['settlement_ids'])):
            if stockpiles:
                nearest = min(stockpiles, key=lambda row: max(abs(row['tile_x']-settlement['formation_xs'][i]),abs(row['tile_y']-settlement['formation_ys'][i])))
                ident = (int(nearest['tile_x']) << 32) | int(nearest['tile_y'])
                assigned[ident] = assigned.get(ident,0)+settlement['wood_stocks'][i]
        for ident,expected_stock in assigned.items():
            matches = [label for label in labels if label.get('id')==ident]
            if len(matches)!=1:
                errors.append(f'{capture.stem}: missing/duplicate assigned stockpile label {ident}')
                continue
            label = matches[0]
            if bool(label.get('shown')) != (capture.zoom>=2):
                errors.append(f'{capture.stem}: stockpile label {ident} zoom visibility')
            rect = label.get('rect',[])
            viewport = data.get('viewport_size',[])
            if len(rect)!=4 or len(viewport)!=2:
                errors.append(f'{capture.stem}: stockpile label {ident} missing footprint')
            else:
                x,y,w,h = rect
                visible = capture.zoom>=2 and w>0 and h>0 and x<viewport[0] and y<viewport[1] and x+w>0 and y+h>0
                if bool(label.get('visible')) != visible:
                    errors.append(f'{capture.stem}: stockpile label {ident} footprint visibility')
            target = 0
            targets = settlement.get('wood_targets', [])
            if targets:
                anchor = next(row for row in stockpiles if (int(row['tile_x']) << 32) | int(row['tile_y']) == ident)
                index = min(range(len(settlement['settlement_ids'])), key=lambda i: max(abs(anchor['tile_x']-settlement['formation_xs'][i]), abs(anchor['tile_y']-settlement['formation_ys'][i])))
                target = targets[index] if index < len(targets) else 0
            suffix = f'{expected_stock}/{target}' if target > 0 else str(expected_stock)
            pattern = r'(?:장작|Firewood)\s+' + re.escape(suffix) + r'(?![\d/])'
            if label.get('wood_stock') != expected_stock or not re.search(pattern, label.get('text','')):
                errors.append(f'{capture.stem}: stockpile {ident} wood label')
            if target > 0 and (label.get('wood_target') != target or not split_labels):
                errors.append(f'{capture.stem}: stockpile {ident} missing split target label')
            if target > 0 and expected_stock < target:
                tint = label.get('modulate', [])
                if len(tint) < 3 or any(abs(a-b) > 0.001 for a,b in zip(tint[:3], (0.96,0.62,0.32), strict=True)):
                    errors.append(f'{capture.stem}: stockpile {ident} shortage tint')
        woods = sum(settlement['wood_stocks'])
        hud = [label['text'] for label in data.get('labels',[]) if 'HudTopbar' in label.get('name','')]
        totals['hud'] += 1
        if not any(re.fullmatch(r'(?:장작|Firewood)\s+'+str(woods), text) for text in hud):
            errors.append(capture.stem+': HUD wood sum')
        if not all(key in resources for key in ('xs','ys','kinds','amounts','maxes')):
            errors.append(capture.stem+': missing resource columns'); continue
        camera = data.get('camera_position',[])
        viewport = data.get('viewport_size',[])
        zoom = data.get('zoom',[])
        if len(camera) != 2 or len(viewport) != 2 or len(zoom) != 2:
            errors.append(capture.stem+': missing view transform'); continue
        for x,y,kind,amount,maximum in zip(*(resources[key] for key in ('xs','ys','kinds','amounts','maxes')),strict=True):
            if kind != 3 or abs(448+x*16+8-camera[0]) > viewport[0]/float(data.get('framebuffer_scale',1))/zoom[0]/2 or abs(28+y*16+15-camera[1]) > viewport[1]/float(data.get('framebuffer_scale',1))/zoom[1]/2:
                continue
            totals['deadwood'] += 1
            stage = min(3,max(0,math.ceil(3*amount/max(maximum,1))))
            rows = [e for e in entities if e['kind']=='deadwood' and e.get('tile_x')==x and e.get('tile_y')==y and e.get('shown')]
            frame = accepted(f'W1_{7-2*stage:02d}',f'W1_{8-2*stage:02d}','N1_15' if stage >= 2 else 'S2_15')
            if len(rows)!=1 or rows[0].get('deadwood_stage')!=stage or rows[0].get('sprite_id')!=frame:
                errors.append(f'{capture.stem}: deadwood {x},{y} count/stage/frame')
            elif stage == 0 and frame == 'S2_15' and abs(rows[0].get('modulate_alpha',1)-.35)>.001:
                errors.append(f'{capture.stem}: deadwood {x},{y} empty alpha')
        if capture.stem == 'l_d10_1500_F2':
            carry_seen = True
            snap = data.get('agent_snapshot',{})
            motion = data.get('agent_motion_evidence', {})
            before = motion.get('before', {})
            columns = ('agent_ids','carried_woods','xs','ys','states')
            before_columns = ('agent_ids','xs','ys')
            if not all(key in snap for key in columns) or len({len(snap[key]) for key in columns}) != 1 or not all(key in before for key in before_columns) or len({len(before[key]) for key in before_columns}) != 1:
                errors.append(capture.stem+': missing/inconsistent carrier columns'); continue
            visible_ids = {e['id'] for e in entities if e['kind']=='agent' and e.get('visible')}
            if not visible_ids.issubset(set(snap['agent_ids'])) or not visible_ids.issubset(set(before['agent_ids'])):
                errors.append(capture.stem+': visible carriers absent from movement snapshots')
            positions = {ident:(x,y) for ident,x,y in zip(*(before.get(key,[]) for key in ('agent_ids','xs','ys')),strict=True)}
            if not positions or motion.get('end_tick') != data.get('tick') or not 0 < motion.get('end_tick',0)-motion.get('start_tick',0) <= 8:
                errors.append(capture.stem+': missing motion observation')
            for ident,stock,x,y,state in zip(*(snap[key] for key in columns),strict=True):
                rows = [e for e in entities if e['kind']=='agent' and e.get('id')==ident and e.get('visible')]
                if stock <= 0 or ident not in positions or positions[ident] == (x,y) or not rows:
                    continue
                totals['displaced_wood_carriers'] += 1
                if state != 1:
                    totals['displaced_now_stationary'] += 1
                    continue
                totals['carrying'] += 1
                row = rows[0]
                frame = atlas.get(row.get('frame_id',''), {})
                safe = frame.get('drawable',True) and frame.get('body') == row.get('body') and not frame.get('contains_basket',False) and not frame.get('contains_bush',False) and frame.get('source_sprite') != 'direction_09_villager_gatherer'
                real = safe and frame.get('activity') == 'carry_wood' and row.get('activity') == 'carry_wood'
                icons = [e for e in entities if e['kind']=='activity_icon' and e.get('id')==ident and e.get('shown')]
                expected_icon = accepted('W1_12','W1_16','S2_15')
                icon_valid = len(icons)==1 and icons[0].get('sprite_id')==expected_icon if capture.zoom>=2 else not icons
                fallback = safe and frame.get('activity') in ('walk','idle') and row.get('activity')=='carry_wood' and row.get('fallback_kind')=='wood_icon' and icon_valid
                totals['real_carry'] += int(real)
                totals['fallback_carry'] += int(not real and fallback)
                if not real and not fallback:
                    errors.append(f'{capture.stem}: wood carrier {ident} unrecorded fallback')
    if not carry_seen:
        errors.append('missing l_d10_1500_F2 carry observation')
    return result('A17', bool(captures) and totals['hearths']>0 and totals['deadwood']>0 and not errors,
                  json.dumps({'counts':totals,'errors':errors,'carry_status':'N/A' if not totals['carrying'] else 'observed','moving_criterion':'current snapshot state == 1 (walking) AND observed displacement during final synchronous advance of <=8 ticks','label_scope':'all settlement-assigned stockpiles require captured nodes; shown follows zoom>=2 independently of viewport; visible follows rectangle intersection'}))
