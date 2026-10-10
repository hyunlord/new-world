"""Run A1–A12 against original windowed capture evidence; absent evidence never passes."""
from __future__ import annotations
import argparse
from dataclasses import asdict
import json
from pathlib import Path
import re
import unicodedata
import numpy as np
from ingame_qa_atlas import distinct_activities, forest
from ingame_qa_types import Capture, box, load_capture, luma
from ingame_qa_pixels import Check, calibration_order, checker, contrast, night, result


def squash(captures: list[Capture], repo: Path) -> Check:
    bad, count = [], 0
    for capture in captures:
        for entity in capture.entities:
            if entity.kind not in {'agent', 'villager'} or not entity.visible:
                continue
            count += 1
            expected = np.array(entity.size) * capture.zoom * capture.scale
            if min(entity.size) <= 0 or not np.allclose(entity.rect[2:], expected, atol=0.01):
                bad.append(f'{capture.stem}:{entity.ident}')
    atlas = json.loads((repo/'assets/art/atlas/world_atlas.json').read_text())['sprites']
    idle = [key for key,value in atlas.items() if str(value.get('body','')).startswith('adult') and value.get('activity') == 'idle' and value.get('view') == 'S']
    short = [key for key in idle if atlas[key]['rect'][3] < 1.5*atlas[key]['rect'][2]]
    return result('A3', count > 0 and not bad and bool(idle) and not short,
                  f'{count} drawn rects; wrong={bad}; adult S-idle frames={len(idle)}, aspect failures={short}; absent S-idle cannot pass')


def animation(directory: Path, captures: list[Capture]) -> Check:
    path = directory/'strip.json'
    if not path.exists():
        return Check('A7', 'N/A', 'No live strip')
    strip = json.loads(path.read_text())
    frames = strip['frames']
    followed = strip['followed_agent_id']
    if followed < 0:
        return Check('A7', 'N/A', 'No moving villager available to follow')
    by_name = {capture.path.name:capture for capture in captures}
    history: dict[int, set[str]] = {}
    seconds: dict[int, set[str]] = {}
    start = frames[0]['real_time_usec']
    moving_samples = 0
    for frame in frames:
        capture = by_name[frame['file']]
        second = int((frame['real_time_usec']-start)/1_000_000)
        for entity in capture.entities:
            if entity.kind != 'agent' or not entity.visible:
                continue
            history.setdefault(entity.ident, set()).add(entity.frame)
            if entity.ident == followed and entity.activity in {'walk','carry'}:
                seconds.setdefault(second, set()).add(entity.frame)
                moving_samples += 1
    duration = (frames[-1]['real_time_usec']-start)/1_000_000 + .1
    full_seconds = {s:len(v) for s,v in seconds.items() if s+1 <= duration+.02}
    fraction = sum(len(v)>1 for v in history.values())/max(1,len(history))
    intervals = np.diff([frame['real_time_usec'] for frame in frames])/1_000_000
    valid = bool(full_seconds) and min(full_seconds.values()) >= 3 and fraction >= .60
    return result('A7', valid, f'moving samples={moving_samples}; distinct frames/full moving second={full_seconds}; changing fraction={fraction:.3f}; duration={duration:.3f}s; interval range={intervals.min():.3f}..{intervals.max():.3f}s')


def clutter(capture: Capture) -> Check:
    missing = [name for name,value in [('head dots',capture.dots),('need icons',capture.icons),('goal lines',capture.seek),('trails',capture.trails)] if value is None]
    if missing:
        return Check('A8', 'N/A', 'Missing actual draw instrumentation: '+', '.join(missing))
    valid = capture.selected < 0 and capture.seek == 0 and capture.trails == 0 and capture.dots == 0 and capture.icons == len(capture.need_ids)
    ids_valid = set(capture.icon_ids) == set(capture.need_ids)
    return result('A8', valid and ids_valid, f'selection={capture.selected}; lines/trails/dots={capture.seek}/{capture.trails}/{capture.dots}; icons={capture.icons}; expected>=75={len(capture.need_ids)}; actual IDs={capture.icon_ids}; expected IDs={capture.need_ids}')


def interface(capture: Capture) -> Check:
    errors = []
    panels = [entity for entity in capture.entities if entity.kind == 'panel' and entity.visible]
    rects = [box(entity,capture.scale) for entity in panels]
    for panel, (x0,y0,x1,y1) in zip(panels,rects):
        if not (0 <= x0 < x1 <= 1920 and 0 <= y0 < y1 <= 1080):
            errors.append('panel outside viewport: '+panel.name)
    for i,a in enumerate(rects):
        for b in rects[i+1:]:
            if min(a[2],b[2])>max(a[0],b[0]) and min(a[3],b[3])>max(a[1],b[1]):
                errors.append('intersecting panels')
    for label in capture.labels:
        text = label.text.strip()
        if text and not re.search('[가-힣]',text) and not all(c.isspace() or unicodedata.category(c)[0] in {'N','P'} for c in text):
            errors.append('non-Korean label: '+text)
        if 'Galmuri' not in label.font:
            errors.append('font: '+label.font)
        x,y,w,h = [value/capture.scale for value in label.rect]
        candidates = [rect for panel,rect in zip(panels,rects) if '/'+panel.name+'/' in label.name]
        if not any(a<=x and b<=y and x+w<=c+.1 and y+h<=d+.1 for a,b,c,d in candidates):
            errors.append('label outside own panel: '+text)
    return result('A9', bool(capture.labels) and bool(panels) and not errors, f'{len(capture.labels)} actual labels, {len(panels)} panels; {errors}')


def variety(capture: Capture) -> Check:
    actors = [entity for entity in capture.entities if entity.kind == 'agent' and entity.visible]
    looks = [(entity.body,*entity.variant) for entity in actors]
    pairs = [(i,j) for i,a in enumerate(actors) for j,b in enumerate(actors) if i<j and max(abs(a.tile[0]-b.tile[0]),abs(a.tile[1]-b.tile[1]))<=1]
    same = sum(looks[i]==looks[j] for i,j in pairs)
    ratio = same/len(pairs) if pairs else 0
    return result('A12', bool(actors) and len(set(looks))>=min(len(actors),12) and ratio<=.1,
                  f'{len(set(looks))} tuples/{len(actors)} villagers; identical neighbors={same}/{len(pairs)} ({ratio:.3f})')


def run(directory: Path, output: Path, repo: Path, blind: Path | None = None) -> list[Check]:
    manifest = json.loads((directory/'manifest.json').read_text())
    captures = [load_capture(directory/row['file']) for row in manifest['captures']]
    expected = {'a_d1_0830_F2','a_d2_1200_F2','a_d2_1200_F2A','a_d2_1200_F2Z','a_d2_1200_F3','a_d2_1200_F4','a_d2_1200_F1','a_d2_1930_F2','a_d2_2300_F2N','a_d6_1200_F2','a_calib_z4','a_calib_z6'}
    expected |= {'a_lineup_z4','a_lineup_z2'}
    expected |= {f'{prefix}_{index:02d}' for prefix in ['a_strip','a_evening'] for index in range(40)}
    actual = {capture.stem for capture in captures}
    delivery = result('CAPTURE_SET',expected == actual and len(captures)==94,f'{len(captures)}/94 entries; missing={sorted(expected-actual)}; extra={sorted(actual-expected)}')
    by_stem = {capture.stem:capture for capture in captures}
    failures = []
    for capture in captures:
        image = capture.image()
        packed = image[...,0].astype(np.uint32)*65536 + image[...,1].astype(np.uint32)*256 + image[...,2]
        spread, colors = float(luma(image).std()), len(np.unique(packed))
        if spread<.04 or colors<64:
            failures.append((capture.stem,round(spread,4),colors))
    checks = [delivery,result('A1',bool(captures) and not failures,f'{len(captures)} frames tested; failing (std,colours)={failures}')]
    calibration = [capture for capture in captures if capture.stem.startswith('a_calib')]
    checks += [checker(calibration),squash(captures,repo),*calibration_order(calibration,repo)]
    noon = by_stem.get('a_d2_1200_F2')
    dark = by_stem.get('a_d2_2300_F2N')
    checks.append(contrast(noon,repo) if noon else Check('A5','N/A','Missing noon capture'))
    checks.append(night(noon,dark) if noon and dark else Check('A6','N/A','Missing noon/night pair'))
    checks.append(animation(directory,captures))
    scene_captures = [capture for capture in captures if not capture.stem.startswith(('a_calib','a_lineup'))]
    for code, evidence in [('A8',[(capture.stem,clutter(capture)) for capture in scene_captures if capture.selected<0]),('A9',[(capture.stem,interface(capture)) for capture in scene_captures])]:
        failed = [(stem,check.detail) for stem,check in evidence if check.status != 'PASS']
        status = 'FAIL' if any(check.status == 'FAIL' for _,check in evidence) else ('N/A' if failed or not evidence else 'PASS')
        checks.append(Check(code,status,f'{len(evidence)} scenes checked; failures={failed}'))
    timing = directory/'frame_times.json'
    if timing.exists():
        data = json.loads(timing.read_text())
        samples = data['samples_ms']
        median,p95 = float(np.median(samples)),float(np.percentile(samples,95,method='higher'))
        checks.append(result('A10',len(samples)==300 and data['zoom']==1 and not data['simulation_frozen'] and median<=16.7 and p95<=25,f'n={len(samples)}, median={median:.3f}ms, p95={p95:.3f}ms; live simulation={not data["simulation_frozen"]}'))
    else:
        checks.append(Check('A10','N/A','Missing live300frame timing'))
    if blind and blind.exists():
        data = json.loads(blind.read_text())
        checks.append(result('A11',data['count']==24 and data.get('calibration_lineup',False) and not data.get('absent_activities',[]) and data['accuracy_4x']>=.8 and data['accuracy_2x']>=.6,json.dumps(data,ensure_ascii=False)))
    else:
        checks.append(Check('A11','N/A','Blind labels not yet graded; no inference from pixel-change visibility'))
    checks.extend([distinct_activities(repo),forest(directory)])
    checks.append(variety(noon) if noon else Check('A12','N/A','Missing noon capture'))
    output.parent.mkdir(parents=True,exist_ok=True)
    output.with_suffix('.json').write_text(json.dumps([asdict(check) for check in checks],indent=2,ensure_ascii=False)+'\n')
    text = '# 실제 게임 캡처 QA\n\n원본 windowed framebuffer와 실제 렌더 노드 증거. N/A는 통과가 아니다.\n\n'
    text += '\n'.join(f'## {check.code}: {check.status}\n\n{check.detail}\n' for check in checks)
    output.write_text(text)
    return checks

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('raw',type=Path)
    parser.add_argument('--output',type=Path,default=Path('docs/art/production/ingame_qa.md'))
    parser.add_argument('--repo',type=Path,default=Path('.'))
    parser.add_argument('--blind-result',type=Path)
    args = parser.parse_args()
    for check in run(args.raw,args.output,args.repo,args.blind_result):
        print(check.code,check.status)
