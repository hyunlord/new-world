"""Freeze society screenshot questions before independent observers label them."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import shutil
import sys
import time
from ingame_qa_pixels import Check, result


def create(raw: Path, output: Path) -> None:
    """Copy original pixels under opaque numbered names and freeze a separate key."""
    manifest = json.loads((raw/'manifest.json').read_text())
    frames = []
    for entry in manifest['captures']:
        path = raw/entry['file']
        data = json.loads(path.with_suffix('.entities.json').read_text())
        frames.append((path,data))
    selected = []
    for day,clock in [(29,'12:00'),(51,'12:00'),(84,'12:00'),(85,'08:42')]:
        matches = [(p,d) for p,d in frames if d.get('capture_plan_row',[])[:3] == [day,clock,'FV'] and d.get('selected_settlement_id') == -1]
        if len(matches) != 1:
            raise ValueError(f'exactly one unselected FV required for {day} {clock}, got {len(matches)}')
        selected.append(matches[0])
    walking = []
    for clock in ['06:30','08:00']:
        matches = [(p,d) for p,d in frames if d.get('capture_plan_row',[])[:3] == [29,clock,'FD']]
        if len(matches) != 1:
            raise ValueError(f'exactly one walking frame required for d29 {clock}')
        walking.append(matches[0])
    output.mkdir(parents=True,exist_ok=True)
    answers = {'created_ns':time.time_ns(),'A27':{},'A28':{},'sources':{}}
    for code,items in [('A27',selected),('A28',walking)]:
        for number,(path,data) in enumerate(items,1):
            target = output/f'{code}_{number}{path.suffix}'
            shutil.copyfile(path,target)
            key = str(number)
            answers['sources'][f'{code}_{key}'] = {'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'source':str(path),'image':str(target)}
            if code == 'A27':
                raw_soc = data['society_snapshot']
                numbers = {r['id']:r['number'] for r in data['society_presentation']['settlements']}
                parents = {str(numbers[sid]):numbers[parent] for sid,parent in zip(raw_soc['settlement_ids'],raw_soc['parent_ids']) if parent >= 0}
                answers[code][key] = {'count':len(raw_soc['settlement_ids']),'parents':parents}
            else:
                if not data['society_snapshot']['founder_agent_ids']:
                    raise ValueError(f'{path}: no founders for walking question')
                answers[code][key] = 'moving away to settle'
    key_path = output/'frozen_answers.json'
    key_path.write_text(json.dumps(answers,indent=2)+'\n')
    (output/'frozen_answers.sha256').write_text(hashlib.sha256(key_path.read_bytes()).hexdigest()+'\n')
    (output/'instructions.txt').write_text('Do not open frozen_answers.json or sidecars. A27: each of four images, how many villages and which village was founded from which? A28: each of two images, what are these people doing? Allowed: working, wandering, moving away to settle, fleeing, unsure. Save JSON with observer_id, A27 and A28 objects keyed by the numbered image. For each A27 image give count (integer) and parents (daughter-number to parent-number mapping). For each A28 image give one allowed choice. Do not infer answers from filenames. Unknown answers remain unknown.\n')
    print(f'Frozen 4 A27 images + 2 A28 images in {output}; key not printed.')


def grade(key_path: Path, labels: list[Path], output: Path) -> None:
    """Freeze observer bytes first, then grade against the pre-existing answer key."""
    saved = [(path,path.read_bytes()) for path in labels]
    for path,blob in saved:
        path.with_suffix('.sha256').write_text(hashlib.sha256(blob).hexdigest()+'\n')
    key = json.loads(key_path.read_text())
    key_hash = hashlib.sha256(key_path.read_bytes()).hexdigest()
    if key_path.with_suffix('.sha256').read_text().strip() != key_hash:
        raise ValueError('frozen answer key changed')
    observers = []
    for path,blob in saved:
        row = json.loads(blob)
        if path.stat().st_mtime_ns < key['created_ns']:
            raise ValueError('observer labels predate frozen questions')
        scores = {code:sum(row.get(code,{}).get(number) == expected for number,expected in key[code].items()) for code in ['A27','A28']}
        observers.append({'observer_id':row.get('observer_id',''),'labels':str(path),'labels_sha256':hashlib.sha256(blob).hexdigest(),'correct':scores,'total':{'A27':4,'A28':2}})
    output.write_text(json.dumps({'key':str(key_path),'key_sha256':key_hash,'sources':key['sources'],'observers':observers},indent=2)+'\n')


def check_blind(path: Path, code: str) -> Check:
    """Both observers must independently meet the unchanged per-observer threshold."""
    if not path.exists():
        return Check(code,'N/A','Missing two independently frozen society observer results')
    data = json.loads(path.read_text())
    observers = data.get('observers',[])
    ids = [r.get('observer_id') for r in observers]
    required,total = (3,4) if code == 'A27' else (2,2)
    passed = len(observers)==2 and len(set(ids))==2 and all(ids) and bool(data.get('key_sha256'))
    passed = passed and all(row.get('labels_sha256') and row.get('correct',{}).get(code,0)>=required and row.get('total',{}).get(code)==total for row in observers)
    key_path = Path(data.get('key',''))
    if not key_path.is_file() or hashlib.sha256(key_path.read_bytes()).hexdigest() != data.get('key_sha256'):
        passed = False
    else:
        key = json.loads(key_path.read_text())
        for row in observers:
            labels_path = Path(row.get('labels',''))
            if not labels_path.is_file() or hashlib.sha256(labels_path.read_bytes()).hexdigest() != row.get('labels_sha256'):
                passed = False
                continue
            labels = json.loads(labels_path.read_text())
            actual = sum(labels.get(code,{}).get(number)==expected for number,expected in key[code].items())
            if actual != row.get('correct',{}).get(code) or labels.get('observer_id') != row.get('observer_id'):
                passed = False
        for source in key.get('sources',{}).values():
            original = Path(source.get('source',''))
            shown = Path(source.get('image',''))
            if not shown.is_file() or hashlib.sha256(shown.read_bytes()).hexdigest() != source.get('sha256'):
                passed = False
            if not original.is_file() or hashlib.sha256(original.read_bytes()).hexdigest() != source.get('sha256'):
                passed = False
    return result(code,passed,json.dumps({'required_each':f'{required}/{total}','observers':observers,'key_sha256':data.get('key_sha256')}))


if __name__ == '__main__':
    mode,*args = sys.argv[1:]
    if mode == 'create':
        create(*(Path(arg) for arg in args))
    elif mode == 'grade':
        key,first,second,output = map(Path,args)
        grade(key,[first,second],output)
    else:
        raise ValueError('usage: blind_p7.py create RAW OUTPUT | grade KEY OBSERVER1 OBSERVER2 OUTPUT')
