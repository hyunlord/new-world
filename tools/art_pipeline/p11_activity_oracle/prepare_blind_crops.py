"""Plan 14 real crops; optional --extract writes 4x nearest PNGs.
Run: python3 prepare_blind_crops.py CHECKOUT NEWDIR CAPTURE_JSON ... [--extract]
Reviewer receives only blind/ and names.json; key.json stays sealed until answers.
"""
from __future__ import annotations
import hashlib
import json
import math
import sys
from pathlib import Path
from activity_rules import EvidenceError, load_atlas
from check_activity_sidecars import check_capture

NAMES = ['TendFire','Craft','Repair','HearthCircle','Explore','Play','SitByFire']


def inventory(captures: list[tuple[str, dict]]) -> dict:
    pools: dict[int,list[dict]] = {kind:[] for kind in range(7)}
    for path,data in captures:
        for row in data['activity_holders']:
            if not (row['visible'] and row['visible_in_tree'] and row['fully_inside_viewport']):
                continue
            x,y,w,h = row['viewport_rect']; width,height = data['viewport']
            # Center on the actual drawn sprite, retain surrounding context, never fabricate a crop.
            side = math.ceil(max(w,h)+16); cx,cy = x+w/2,y+h/2
            left,top = math.floor(cx-side/2), math.floor(cy-side/2)
            if left < 0 or top < 0 or left+side > width or top+side > height:
                continue
            pools[row['kind']].append({'source':path,'entity_bits':row['entity_bits'],'agent_id':row['agent_id'],
                'kind':row['kind'],'box':[left,top,left+side,top+side], 'upscale':4,'resampling':'nearest',
                'overrides':row['metadata']['activity_overrides']})
    selected=[]; missing=[]
    for kind,pool in pools.items():
        pool.sort(key=lambda r: (len(r['overrides']),r['source'],r['agent_id']))
        unique=[]; seen=set()
        for item in pool:
            identity=(item['source'],item['entity_bits'])
            if identity not in seen:
                unique.append(item); seen.add(identity)
        if len(unique)<2:
            missing.append({'kind':kind,'name':NAMES[kind],'available':len(unique),'required':2})
        selected.extend(unique[:2])
    selected.sort(key=lambda r: hashlib.sha256(f"{r['source']}:{r['entity_bits']}".encode()).hexdigest())
    for index,item in enumerate(selected): item['blind_id']=f'crop_{index+1:02d}'
    return {'complete':not missing,'missing_kinds':missing,'crops':selected,'blind_score':None,'review_answers_recorded':False}


def main(args: list[str]) -> int:
    extract='--extract' in args; args=[v for v in args if v!='--extract']
    checkout,out=Path(args[0]),Path(args[1]); out.mkdir(parents=True,exist_ok=False)
    atlas=load_atlas(checkout); captures=[]; failures=[]
    sys.path.insert(0,str(checkout/"tools/art_pipeline"))
    from run_activity_captures_p10 import Capture, validate_capture
    from PIL import Image
    for name in args[2:]:
        path=Path(name); data=json.loads(path.read_text())
        validate_capture(data,Capture(data['seed'],data['display_day'],data['hour'],data['zoom']))
        check=check_capture(data,atlas)
        image_path=path.with_name('capture.png')
        if not check['pose_check_complete']:
            failures.append({'sidecar':name,'errors':check['errors']}); continue
        with Image.open(image_path) as image:
            if list(image.size)!=data['viewport']: raise EvidenceError('image viewport mismatch')
        captures.append((str(path),data))
    result=inventory(captures); result['source_errors']=failures
    result['complete']=result['complete'] and not failures
    result['source_hashes']={str(Path(name)):hashlib.sha256(Path(name).read_bytes()).hexdigest() for name in args[2:]}
    result['image_hashes']={name:hashlib.sha256(Path(name).with_name('capture.png').read_bytes()).hexdigest() for name,_ in captures}
    (out/'key.json').write_text(json.dumps(result,indent=2)+'\n')
    if not result['complete']: return 1
    blind=out/'blind'; blind.mkdir()
    (blind/'names.json').write_text(json.dumps({'choices':NAMES,'instructions':'Identify each numbered crop. Record answers before opening the key.'},indent=2)+'\n')
    if extract:
        for row in result['crops']:
            with Image.open(Path(row['source']).with_name('capture.png')) as image:
                crop=image.crop(row['box']); crop.resize((crop.width*4,crop.height*4),Image.Resampling.NEAREST).save(blind/(row['blind_id']+'.png'))
    return 0

if __name__=='__main__': raise SystemExit(main(sys.argv[1:]))
