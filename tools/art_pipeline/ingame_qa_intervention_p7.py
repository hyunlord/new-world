"""A29 requires raw intervention targeting and independent image observations."""
import hashlib
import json
from pathlib import Path
from ingame_qa_pixels import Check, result


def intervention(frames, output: Path) -> Check:
    """Check both fires in the same frame and the initial applied toast."""
    candidates = [(path,data) for path,data in frames if data.get('scenario')=='INTERVENTION SCENARIO' and data.get('seed')==20261013]
    if not candidates:
        return Check('A29','N/A','No society-seed intervention screenshots')
    review_path = output/'visual_observations.json'
    reviews = json.loads(review_path.read_text()).get('frames',{}) if review_path.exists() else {}
    errors: list[str] = []
    toast_count = 0
    same_frame_count = 0
    rain_frames = 0
    for path,data in candidates:
        society = data.get('society_snapshot',{})
        ids,parents = society.get('settlement_ids',[]),society.get('parent_ids',[])
        daughters = [ident for ident,parent in zip(ids,parents) if parent>=0]
        originals = [ident for ident,parent in zip(ids,parents) if parent<0]
        casts = [row for row in data.get('intervention_casts',[]) if row.get('kind')=='rain']
        if not daughters or not originals or not casts:
            errors.append(path.stem+': missing daughter/parent/rain cast');continue
        target = casts[-1].get('settlement_id')
        if target not in daughters or not casts[-1].get('reply',{}).get('ok'):
            errors.append(path.stem+': cast not accepted on daughter')
        rain = data.get('intervention_state',{}).get('rain_settlement_ids',[])
        if rain != [target]:
            errors.append(path.stem+': actual rain IDs are not only chosen daughter')
        rain_frames += 1
        labels = [row.get('text','') for row in data.get('labels',[]) if '/GodHandPanel/' in row.get('name','')]
        names = {row['id']:row['name'] for row in data.get('society_presentation',{}).get('settlements',[])}
        name = names.get(target,'').split(' · ', 1)[0]
        if '_soc_rain_daughter' in path.stem:
            if not name or not any(name in text and ('적용' in text or 'applied' in text) for text in labels):
                errors.append(path.stem+': actual toast does not name daughter')
            if data.get('selected_settlement_id') != target:
                errors.append(path.stem+': selected ID at cast differs from daughter')
            toast_count += 1
        review = reviews.get(path.stem,{})
        bound = review.get('sha256') == hashlib.sha256(path.read_bytes()).hexdigest()
        if not bound:
            errors.append(path.stem+': no hash-bound independent rain/fire observation')
        if '_soc_rain_daughter' in path.stem and (not name or name not in review.get('toast_text','')):
            errors.append(path.stem+': independent toast observation does not name daughter')
        if '_soc_both_night_' in path.stem:
            same_frame_count += 1
            settlements = data.get('settlement_snapshot',{})
            fires = dict(zip(settlements.get('settlement_ids',[]),settlements.get('fire_lit',[])))
            if fires.get(target)!=0 or any(fires.get(original)!=1 for original in originals):
                errors.append(path.stem+': daughter extinguished/original fire unchanged not established')
            hearths = [row for row in data.get('entities',[]) if row.get('kind')=='building' and row.get('building_kind')==0 and row.get('visible')]
            daughter_hearths = {row['id'] for row in hearths if row.get('settlement_id')==target}
            original_hearths = {row['id'] for row in hearths if row.get('settlement_id') in originals}
            if not daughter_hearths or not original_hearths:
                errors.append(path.stem+': both own hearths not visible in same frame')
            if not daughter_hearths.issubset(set(review.get('ash_hearth_ids',[]))) or not original_hearths.issubset(set(review.get('lit_hearth_ids',[]))):
                errors.append(path.stem+': independent ash/fire observation mismatch')
            if review.get('rain_settlement_ids') != [target]:
                errors.append(path.stem+': independently observed rain not confined to daughter')
    if not toast_count or not same_frame_count:
        errors.append('missing initial toast or same-frame two-village night')
    return result('A29',not errors,json.dumps({'rain_frames':rain_frames,'toast_frames':toast_count,'same_frame_nights':same_frame_count,'errors':errors},ensure_ascii=False))
