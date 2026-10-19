"""Create numbered real-game crops without printing answers; grade only after labels exist."""
from __future__ import annotations
import argparse
from collections import defaultdict
from dataclasses import replace
import hashlib
import json
from pathlib import Path
import random
import time
from PIL import Image, ImageDraw, ImageFont
from ingame_qa_types import Capture, Entity, box, load_capture

ACTIVITIES = ('walk','carry','gather','build','eat','drink','sit','talk','child')


def cue_box(entity: Entity, scale: float) -> tuple[int,int,int,int]:
    return box(replace(entity,rect=entity.cue_rect) if entity.cue_rect is not None else entity,scale)


def category(entity: Entity) -> str:
    if entity.body == 'child':
        return 'child'
    return entity.activity


def compose_grid(crops: list[Image.Image], font: ImageFont.FreeTypeFont | ImageFont.ImageFont) -> Image.Image:
    """Reserve the label band and fit complete cues without rescaling them."""
    cell_width = max(200,max((crop.width for crop in crops),default=0)+20)
    cue_height = max(154,max((crop.height for crop in crops),default=0))
    cell_height = 38+cue_height+8
    rows = max(1,(len(crops)+5)//6)
    canvas = Image.new('RGB',(6*cell_width,rows*cell_height),(34,37,37))
    draw = ImageDraw.Draw(canvas)
    for index,crop in enumerate(crops):
        left,top = (index%6)*cell_width,(index//6)*cell_height
        canvas.paste(crop,(left+(cell_width-crop.width)//2,top+38+(cue_height-crop.height)//2))
        draw.text((left+10,top+10),str(index+1),font=font,fill='white')
    return canvas


def create(raw: Path, output: Path, key_path: Path, repo: Path) -> None:
    """Balance observable activities and keep the answer mapping out of stdout."""
    manifest = json.loads((raw/'manifest.json').read_text())
    lineup = (raw/'a_lineup_z4.entities.json').exists()
    two_x = load_capture(raw/'a_lineup_z2.png') if lineup else None
    groups: dict[str,list[tuple[Capture,Entity]]] = defaultdict(list)
    seen: set[tuple[int,str,str]] = set()
    for row in manifest['captures']:
        if 'calib' in row['file'] or (lineup and row['file'] != 'a_lineup_z4.png'):
            continue
        capture = load_capture(raw/row['file'])
        if capture.zoom < 4:
            continue
        for entity in capture.entities:
            if lineup and entity.role != 'blind_lineup':
                continue
            label = category(entity)
            x0,y0,x1,y1 = cue_box(entity,capture.scale)
            identity = (entity.ident,entity.frame,label)
            if entity.kind != 'agent' or not entity.visible or label not in ACTIVITIES or identity in seen:
                continue
            if not (0<=x0<x1<=1920 and 0<=y0<y1<=1080):
                continue
            panel_boxes = [box(panel,capture.scale) for panel in capture.entities if panel.kind == 'panel' and panel.visible]
            if any(min(x1,c)>max(x0,a) and min(y1,d)>max(y0,b) for a,b,c,d in panel_boxes):
                continue
            seen.add(identity)
            groups[label].append((capture,entity))
    rng = random.Random(20261010)
    for candidates in groups.values():
        rng.shuffle(candidates)
    selected: list[tuple[Capture,Entity]] = []
    while len(selected)<24 and any(groups.values()):
        for activity in ACTIVITIES:
            if groups[activity] and len(selected)<24:
                selected.append(groups[activity].pop())
    rng.shuffle(selected)
    output.mkdir(parents=True,exist_ok=True)
    key_path.parent.mkdir(parents=True,exist_ok=True)
    font_path = repo/'assets/fonts/galmuri/Galmuri11.ttf'
    font = ImageFont.truetype(str(font_path),18) if font_path.exists() else ImageFont.load_default()
    answers = []
    for zoom in [4,2]:
        crops: list[Image.Image] = []
        for index,(capture,entity) in enumerate(selected):
            source_capture, source_entity = capture, entity
            if zoom == 2 and two_x is not None:
                source_capture = two_x
                source_entity = next(e for e in two_x.entities if e.ident == entity.ident and e.role == 'blind_lineup')
            x0,y0,x1,y1 = cue_box(source_entity,source_capture.scale)
            source = Image.fromarray(source_capture.image()).crop((x0,y0,x1,y1))
            width,height = round((x1-x0)*zoom/source_capture.zoom),round((y1-y0)*zoom/source_capture.zoom)
            crop = source.resize((width,height),Image.Resampling.NEAREST)
            crops.append(crop)
            if zoom == 4:
                answers.append({'number':index+1,'activity':category(entity),'source':capture.stem,'agent_id':entity.ident,'frame_id':entity.frame})
        compose_grid(crops,font).save(output/f'blind_{zoom}x.png')
    key = {'created_ns':time.time_ns(),'count':len(selected),'answers':answers,'calibration_lineup':lineup,
           'absent_activities':[a for a in ACTIVITIES if not any(category(e)==a for _,e in selected)]}
    key_path.write_text(json.dumps(key,indent=2)+'\n')
    (output/'blind_instructions.txt').write_text('Label each number at both zooms using: '+', '.join(ACTIVITIES)+'.\nSave JSON {"4x":{"1":"label",...},"2x":{"1":"label",...}} before grading.\nAnswers are stored separately and must remain unopened until labels are saved.\n')
    print(f'Created {len(selected)} numbered crops at each zoom: {output}/blind_4x.png and blind_2x.png. Answer key was written but not displayed.')


def grade(labels_path: Path, key_path: Path, output: Path) -> None:
    """Require saved observer labels before opening the answer key."""
    if not labels_path.is_file():
        raise FileNotFoundError(labels_path)
    label_bytes = labels_path.read_bytes()
    labels = json.loads(label_bytes)
    label_hash = hashlib.sha256(label_bytes).hexdigest()
    labels_path.with_suffix('.sha256').write_text(label_hash+'  '+labels_path.name+'\n')
    key = json.loads(key_path.read_text())
    if labels_path.stat().st_mtime_ns < key['created_ns']:
        raise RuntimeError('Labels predate the blind grid; fresh blind observation is required')
    results = {'count':key['count'],'absent_activities':key['absent_activities'],
               'observer_labels_sha256':label_hash,'calibration_lineup':key.get('calibration_lineup',False),'confusion':{}}
    for zoom in ['4x','2x']:
        matrix: dict[str,dict[str,int]] = defaultdict(lambda:defaultdict(int))
        correct = 0
        for answer in key['answers']:
            observed = str(labels.get(zoom,{}).get(str(answer['number']),'unanswered'))
            expected = answer['activity']
            matrix[expected][observed] += 1
            correct += observed == expected
        results['accuracy_'+zoom] = correct/max(1,key['count'])
        results['confusion'][zoom] = matrix
    output.write_text(json.dumps(results,indent=2)+'\n')
    print(f'Blind results written: {output}')

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode',choices=['create','grade'])
    parser.add_argument('--raw',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--key',type=Path,required=True)
    parser.add_argument('--labels',type=Path)
    parser.add_argument('--repo',type=Path,default=Path('.'))
    args = parser.parse_args()
    if args.mode == 'create':
        create(args.raw,args.output,args.key,args.repo)
    else:
        grade(args.labels,args.key,args.output)
