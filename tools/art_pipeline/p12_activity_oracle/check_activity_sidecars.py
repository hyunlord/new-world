"""Run: python3 check_activity_sidecars.py CHECKOUT CAPTURE_JSON [...].

Reports only pose/visibility evidence, never overall H7 or blind-review acceptance.
No calls to production frame selection. Missing instrumentation fails closed.
"""
from __future__ import annotations
import json
import hashlib
import math
import sys
from pathlib import Path
from .activity_rules import EvidenceError, Frame, body_for, daily, direction, final_frame, load_atlas


def columns(snapshot: dict, names: tuple[str, ...]) -> list[dict]:
    sizes = {len(snapshot[k]) for k in names}
    if len(sizes) != 1:
        raise EvidenceError('unequal raw columns: '+str(names))
    return [dict(zip(names, values)) for values in zip(*(snapshot[k] for k in names))]


def required_bool(value: dict, key: str) -> bool:
    if type(value[key]) is not bool:
        raise EvidenceError('invalid boolean '+key)
    return value[key]


def numbers(value: list, length: int) -> tuple[float, ...]:
    if not isinstance(value, list) or len(value) != length or any(type(x) not in (int,float) or not math.isfinite(x) for x in value):
        raise EvidenceError('invalid numeric vector')
    return tuple(value)


def expected_facing(kind: int, place: tuple, previous: str, buildings: list[dict], home_tile: tuple | None) -> str:
    if kind == 5:
        return 'W' if previous == 'W' else 'E'
    building_kind = {1:2, 2:1}.get(kind,0)
    points = [(r['xs'],r['ys']) for r in buildings if r['kinds'] == building_kind]
    if building_kind == 0 and home_tile in points:
        points = [home_tile]
    nearest = min(points, key=lambda p: ((p[0]-place[0])**2+(p[1]-place[1])**2,p[1],p[0])) if points else place
    sign = -1 if kind == 4 else 1
    return direction((sign*(nearest[0]-place[0]), sign*(nearest[1]-place[1])), previous)


def fixed_animation_seconds(data: dict, serialized: float) -> float:
    """Recover IEEE754 additions lost by JSON decimal rounding, not from sprite output."""
    required = {"state_change_frame": 10, "capture_frame": 130, "fixed_fps": 60,
                "fixed_delta_valid": True}
    if any(type(data.get(k)) is not type(v) or data.get(k) != v for k,v in required.items()):
        raise EvidenceError('unsupported deterministic animation schedule')
    seconds = 0.0
    # Reset occurs after settled frame10; frames11..130 each add the fixed delta.
    # refresh_snapshot calls _process(0.0), which contributes no elapsed time.
    for _ in range(data['capture_frame'] - data['state_change_frame']):
        seconds += 1.0 / data['fixed_fps']
    if not math.isfinite(serialized) or abs(serialized - seconds) > 1e-12:
        raise EvidenceError('serialized animation time contradicts callback schedule')
    return seconds


def check_holder(row: dict, raw: dict, activity: dict, data: dict, atlas: tuple[Frame, ...], buildings: list[dict], homes: dict, formations: dict) -> dict:
    entity, agent, kind = raw['ids'], raw['agent_ids'], activity['kinds']
    if (row['entity_bits'],row['agent_id'],row['kind'],row['place'],row['until_tick'],row['state_tag']) != (entity,agent,kind,[activity['place_x'],activity['place_y']],activity['until_tick'],raw['states']):
        raise EvidenceError('raw holder identity mismatch')
    if row['renderer_present'] is not True or row['missing_metadata'] != []:
        raise EvidenceError('missing renderer or metadata')
    meta = row['metadata']; inputs = meta['activity_oracle_inputs']
    if inputs['schema'] != 1:
        raise EvidenceError('unsupported input schema')
    seconds = fixed_animation_seconds(data, numbers([inputs['animation_seconds']],1)[0])
    if seconds < 0 or inputs['previous_facing'] not in ('E','W','N','S'):
        raise EvidenceError('invalid time/facing')
    move = numbers(inputs['movement'],2); moving = sum(v*v for v in move) > 0.000001
    pickup, meal = (required_bool(inputs,k) for k in ('pickup','meal'))
    chill = numbers([inputs['morning_chill_alpha']],1)[0] > 0
    cold = raw['colds'] == 1
    arrived = activity['until_tick'] >= 0
    body = body_for(agent, raw['life_stages'])
    facing = direction(move,inputs['previous_facing'])
    active = raw['states'] == 4 or raw['target_kinds'] == 8
    sleeping = raw['states'] == 3 and raw['target_kinds'] == 3
    if not active and not sleeping:
        raise EvidenceError('holder in unsupported non-activity state; extend oracle explicitly')
    if active and arrived and not pickup:
        facing = expected_facing(kind,(activity['place_x'],activity['place_y']),inputs['previous_facing'],buildings,formations.get(homes.get(agent)))
    pose = 'asleep'
    logical = 'asleep'
    if active and not pickup:
        pose,facing = daily(kind,body,facing,seconds,arrived,atlas)
        logical = ('tend_fire','craft','repair','hearth_circle','explore','play','sit_by_fire')[kind] if arrived else pose
    overrides: list[str] = []
    if pickup:
        # Pickup bypasses daily facing and chain, before meal and cold.
        facing = direction(move,inputs['previous_facing'])
        pose = logical = 'gather_wood'; overrides.append('pickup')
    if meal:
        pose = 'eat'; overrides.append('meal')
    base_pose = pose
    if (cold or chill) and not moving and pose in ('idle','sit','hop'):
        pose = 'crossed'
        if cold: overrides.append('cold')
        if chill: overrides.append('morning_chill')
    frame = final_frame(body,pose,facing,seconds,agent,atlas)
    expected = {'p10_activity_schema':1,'entity_bits':entity,'art_id':agent,'activity_kind':kind,
        'activity_place_known':arrived,'body':body,'activity':logical,'pose':pose,'facing':facing,
        'activity_frame_activity':frame.pose,'activity_frame_view':frame.view,'sprite_id':frame.sprite_id,
        'cold_base_pose':base_pose,'cold_pose':frame.pose,'moving':moving,'activity_overrides':overrides}
    for key,value in expected.items():
        if meta[key] != value or type(meta[key]) is not type(value):
            raise EvidenceError(f'{key}: expected {value!r}, got {meta[key]!r}')
    sheltered = logical == 'asleep' and any(b['kinds']==1 and (b['xs'],b['ys'])==(raw['xs'],raw['ys']) for b in buildings)
    visible = required_bool(inputs,'capture_visible') and not sheltered
    parent_visible = required_bool(inputs,'parent_visible_in_tree')
    if (row['visible'],row['visible_in_tree'],meta['sheltered_sleep']) != (visible,visible and parent_visible,sheltered):
        raise EvidenceError('visibility mismatch')
    if row['flip_h'] != (facing=='W') or row['texture_path'] != frame.page or row['region_enabled'] is not True or tuple(row['region_rect']) != frame.rect:
        raise EvidenceError('actual sprite texture/region/flip mismatch')
    x,y,w,h = numbers(row['viewport_rect'],4); width,height = numbers(data['viewport'],2)
    if min(w,h,width,height) <= 0:
        raise EvidenceError('empty sprite/viewport')
    intersects = x < width and y < height and x+w > 0 and y+h > 0
    inside = x >= 0 and y >= 0 and x+w <= width and y+h <= height
    if (row['intersects_viewport'],row['fully_inside_viewport']) != (intersects,inside):
        raise EvidenceError('viewport flags mismatch')
    return {'entity_bits':entity,'agent_id':agent,'kind':kind,'overrides':overrides,'hidden':not visible,'offscreen':not intersects,'expected_pose':pose,'expected_frame_pose':frame.pose,'expected_view':frame.view}


def check_capture(data: dict, atlas: tuple[Frame, ...]) -> dict:
    errors: list[dict] = []; checked: list[dict] = []
    try:
        if data['complete'] is not True or data['errors'] != [] or data['pose_oracle_checked'] is not False:
            raise EvidenceError('capture incomplete or capture falsely claims oracle result')
        agents = columns(data['raw_agent_snapshot'],('ids','agent_ids','states','target_kinds','life_stages','colds','xs','ys'))
        activities = columns(data['raw_activity_snapshot'],('ids','kinds','place_x','place_y','until_tick'))
        buildings = columns(data['raw_building_snapshot'],('xs','ys','kinds'))
        society = columns(data['raw_society_snapshot'],('home_agent_ids','home_settlement_ids'))
        settlements = columns(data['settlements'],('settlement_ids','formation_xs','formation_ys'))
        homes = {r['home_agent_ids']:r['home_settlement_ids'] for r in society}
        formations = {r['settlement_ids']:(r['formation_xs'],r['formation_ys']) for r in settlements}
        by_entity = {r['ids']:r for r in agents}; raw_activity = {r['ids']:r for r in activities}
        holders = data['activity_holders']; ids = [r['entity_bits'] for r in holders]
        if len(by_entity)!=len(agents) or len({r['agent_ids'] for r in agents})!=len(agents) or len(raw_activity)!=len(activities) or len(set(ids))!=len(ids) or set(ids)!=set(raw_activity) or not ids or data['holder_count']!=len(ids) or data['population']!=len(agents):
            raise EvidenceError('missing/duplicate holders or population mismatch')
        for row in holders:
            entity = row['entity_bits']
            try:
                checked.append(check_holder(row,by_entity[entity],raw_activity[entity],data,atlas,buildings,homes,formations))
            except (EvidenceError,KeyError,TypeError,IndexError) as error:
                errors.append({'entity_bits':entity,'error':str(error)})
    except (EvidenceError,KeyError,TypeError,IndexError) as error:
        errors.append({'entity_bits':None,'error':str(error)})
    return {'pose_check_complete':not errors,'mismatches':len(errors),'errors':errors,'checked':checked,'h7_pass':False,'blind_review_pending':True,'animation_time_basis':'120 repeated IEEE754 additions of fixed delta 1/60 after settled reset frame10; JSON value checked for consistency'}


def main(arguments: list[str]) -> int:
    checkout = Path(arguments[0]); atlas = load_atlas(checkout); results = []
    # Existing scaffold's structural validator is reused, never its renderer.
    sys.path.insert(0,str(checkout/'tools/art_pipeline'))
    from run_activity_captures_p10 import Capture, validate_capture
    for path_text in arguments[1:]:
        path = Path(path_text); data = json.loads(path.read_text())
        validate_capture(data,Capture(data['seed'],data['display_day'],data['hour'],data['zoom']))
        from PIL import Image
        image_path=path.with_name('capture.png')
        with Image.open(image_path) as image:
            if list(image.size)!=data['viewport']: raise EvidenceError('image viewport mismatch')
        result = check_capture(data,atlas); result['sidecar']=str(path)
        sources=[path,image_path,checkout/'assets/art/atlas/world_atlas.json',checkout/'scripts/ui/cold_audit.json']
        result['sha256']={str(source):hashlib.sha256(source.read_bytes()).hexdigest() for source in sources}
        results.append(result)
    if not results: raise EvidenceError('no sidecars supplied')
    print(json.dumps(results,indent=2))
    return int(any(not r['pose_check_complete'] for r in results))

if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
