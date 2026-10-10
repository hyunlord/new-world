"""Strict production2 atlas activity and captured forest evidence checks."""
from __future__ import annotations
from dataclasses import dataclass
import json
from pathlib import Path
from PIL import Image
import numpy as np
from ingame_qa_pixels import Check, result
from v3_semantics import semantic_counts

@dataclass(frozen=True, slots=True)
class Frame:
    ident: str
    body: str
    activity: str
    view: str
    pixels: frozenset[tuple[int,int]]
    berries: int


def opaque_difference(a: frozenset[tuple[int,int]], b: frozenset[tuple[int,int]]) -> float:
    return len(a ^ b)/max(1,len(a | b))


def activity_frames(repo: Path) -> list[Frame]:
    entries = json.loads((repo/'assets/art/atlas/world_atlas.json').read_text())['sprites']
    frames = []
    for ident,entry in entries.items():
        if entry.get('body') not in {'adult_a','adult_b','child'}:
            continue
        x,y,w,h = entry['rect']
        with Image.open(repo/entry['page'].removeprefix('res://')) as atlas:
            image = atlas.convert('RGBA').crop((x,y,x+w,y+h))
        alpha = np.asarray(image)[...,3]
        ay,ax = np.nonzero(alpha)
        anchor_x,anchor_y = entry['anchor']
        coordinates = frozenset((int(px)-anchor_x,int(py)-anchor_y) for px,py in zip(ax,ay))
        frames.append(Frame(ident,entry['body'],entry.get('activity',''),entry.get('view',''),coordinates,semantic_counts(image)[0]))
    return frames


def distinct_activities(repo: Path) -> Check:
    frames = activity_frames(repo)
    evidence, missing = [], []
    for body in ['adult_a','adult_b','child']:
        idle = [frame for frame in frames if frame.body==body and frame.activity=='idle' and frame.view=='S']
        if not idle:
            missing.append(body+'/idle/S')
            continue
        for activity in ['walk','carry','gather','build','eat','drink','sit','talk']:
            options = [frame for frame in frames if frame.body==body and frame.activity==activity]
            if not options:
                missing.append(body+'/'+activity)
                continue
            # A changed idle animation phase must not make a copied activity pass.
            scored = [(frame.ident,min(opaque_difference(frame.pixels,base.pixels) for base in idle)) for frame in options]
            best = max(scored,key=lambda row:row[1])
            evidence.append((body,activity,best[0],best[1]))
    forbidden = {'idle','walk','build','talk','sit','drink','asleep'}
    baskets = [(f.ident,f.berries) for f in frames if f.activity in forbidden and f.berries>=6]
    return result('A13',len(evidence)==24 and not missing and not baskets and all(row[3]>=.15 for row in evidence),
        f'foot-anchor opaque symmetric difference/union; best frames={evidence}; missing={missing}; forbidden berry-ramp counts={baskets}')


def forest(directory: Path) -> Check:
    path = directory/'forest_evidence.json'
    if not path.exists():
        return Check('A14','N/A','Missing actual capture terrain/nature/clearing snapshots')
    data = json.loads(path.read_text())
    terrain = data['terrain']; width = int(terrain['width'])
    kinds = terrain['terrain']
    if isinstance(kinds,str):
        kinds = json.loads(kinds)
    forest_tiles = {(i%width,i//width) for i,kind in enumerate(kinds) if kind==1}
    centers = set()
    for domain in ['building','construction','resource','settlement']:
        snap = data[domain]
        xkey,ykey = ('formation_xs','formation_ys') if domain=='settlement' else ('xs','ys')
        for i,(x,y) in enumerate(zip(snap[xkey],snap[ykey])):
            if domain!='resource' or snap['kinds'][i]==0:
                centers.add((int(x),int(y)))
    objects = [e for e in data['entities'] if e['kind'] in {'tree','shrub','log'} and e.get('shown',True)]
    trees = [(int(e['tile_x']),int(e['tile_y'])) for e in objects if e['kind']=='tree']
    planted = set(trees)&forest_tiles
    violations = [(e['id'],e['kind'],e['tile_x'],e['tile_y']) for e in objects if any(max(abs(e['tile_x']-x),abs(e['tile_y']-y))<=4 for x,y in centers)]
    duplicate = len(trees)-len(set(trees))
    fraction = len(planted)/max(1,len(forest_tiles))
    return result('A14',bool(forest_tiles) and .25<=fraction<=.35 and not violations and not duplicate,
        f'captured tick={data["tick"]}; forest-tree tiles={len(planted)}/{len(forest_tiles)} ({fraction:.5f}); duplicate tree tiles={duplicate}; clearing centers={len(centers)}; checked objects={len(objects)}; violations={violations}')
