"""Art Pass 4 edge contrast and crowd measurements on observed opaque masks."""
from __future__ import annotations
from dataclasses import asdict
import json
from pathlib import Path
import numpy as np
from PIL import Image
from ingame_qa_pixels import Check, contrast, result
from ingame_qa_types import BoolArray, Capture, Entity, box, luma, placed, sprite_pixels


def order(entity: Entity) -> tuple[int, float, int]:
    return entity.draw_order[0], entity.world_anchor[1], entity.draw_order[1]


def edge_values(image: np.ndarray, visible: BoolArray, free: BoolArray) -> np.ndarray:
    """Collect each directed four-neighbour pair once, without boundary wrapping."""
    pairs = []
    for dy, dx in ((0, 1), (0, -1), (1, 0), (-1, 0)):
        py = slice(max(0, -dy), min(image.shape[0], image.shape[0]-dy))
        px = slice(max(0, -dx), min(image.shape[1], image.shape[1]-dx))
        qy = slice(max(0, dy), min(image.shape[0], image.shape[0]+dy))
        qx = slice(max(0, dx), min(image.shape[1], image.shape[1]+dx))
        valid = visible[py, px] & free[qy, qx]
        pairs.append(np.abs(image[py, px]-image[qy, qx])[valid])
    return np.concatenate(pairs)


def edge_contrast(capture: Capture, repo: Path) -> Check:
    """A5 preserves old foreground exclusions while requiring clear edge neighbours."""
    pixels = luma(capture.image())
    occupied = np.zeros(pixels.shape, dtype=bool)
    panels = np.zeros_like(occupied)
    blockers = []
    for entity in capture.entities:
        if not entity.visible or entity.alpha <= 0:
            continue
        if entity.kind == 'panel':
            x0,y0,x1,y1 = box(entity, capture.scale)
            panels[max(0,y0):min(1080,y1),max(0,x0):min(1920,x1)] = True
            continue
        _, mask = placed(entity, capture, repo)
        occupied |= mask
        if 'shadow' not in entity.kind and not (entity.kind == 'tree' and entity.alpha <= .5):
            blockers.append((entity, mask))
    rows = []
    actors = [e for e in capture.entities if e.kind == 'agent' and e.visible]
    for actor in actors:
        _, mask = placed(actor, capture, repo)
        visible = mask & ~panels
        for other, front in blockers:
            if other is not actor and order(other) > order(actor):
                visible &= ~front
        pairs = edge_values(pixels, visible, ~occupied & ~panels & ~mask)
        rows.append({'id': actor.ident, 'pairs': len(pairs), 'median': float(np.median(pairs)) if len(pairs) >= 40 else None})
    measured = [row['median'] for row in rows if row['median'] is not None]
    median = float(np.median(measured)) if measured else 0.0
    above = sum(value >= .10 for value in measured)/len(measured) if measured else 0.0
    coverage = len(measured)/len(actors) if actors else 0.0
    return result('A5', bool(measured) and median >= .20 and above >= .90 and coverage >= .60,
                  json.dumps({'median': median, 'fraction_ge_010': above, 'measurable_fraction': coverage, 'actors': rows, 'A5-old (diagnostic)': asdict(contrast(capture, repo))}))


def crowd(capture: Capture, repo: Path) -> Check:
    """A18 counts opaque coverage by the union of strictly front-drawn villagers."""
    actors = [e for e in capture.entities if e.kind == 'agent' and e.visible]
    drawn = [e for e in capture.entities if e.kind == 'agent' and e.shown and e.alpha > 0]
    full_masks = []
    for actor in drawn:
        source = sprite_pixels(actor,repo)
        x0,y0,x1,y1 = box(actor,capture.scale)
        mask = np.zeros((max(0,y1-y0),max(0,x1-x0)),dtype=bool)
        if source is not None and x1>x0 and y1>y0:
            enlarged = np.asarray(Image.fromarray(source).resize((x1-x0,y1-y0),Image.Resampling.NEAREST))
            mask = enlarged[...,3]>0
        full_masks.append((actor,(x0,y0,x1,y1),mask))
    rows = []
    for actor in actors:
        record = next((record for record in full_masks if record[0] is actor),None)
        if record is None:
            rows.append({'id':actor.ident,'opaque':0,'visible_opaque':0,'covered':0,'fraction':None})
            continue
        _,(x0,y0,x1,y1),mask = record
        front = np.zeros_like(mask)
        for other,(a,b,c,d),other_mask in full_masks:
            if order(other) <= order(actor):
                continue
            left,top,right,bottom = max(x0,a),max(y0,b),min(x1,c),min(y1,d)
            if right>left and bottom>top:
                front[top-y0:bottom-y0,left-x0:right-x0] |= other_mask[top-b:bottom-b,left-a:right-a]
        pixels = int(mask.sum())
        covered = int((mask & front).sum())
        visible_pixels = int(placed(actor,capture,repo)[1].sum())
        rows.append({'id': actor.ident, 'opaque': pixels, 'visible_opaque': visible_pixels, 'covered': covered, 'fraction': covered/pixels if pixels else None})
    count = sum(row['fraction'] is not None and row['fraction'] >= .5 for row in rows)
    limit = max(1, len(actors)//20)
    return result('A18', bool(rows) and all(row['opaque'] > 0 for row in rows) and count <= limit,
                  json.dumps({'visible': len(actors), 'half_occluded': count, 'limit': limit, 'mask_space':'unclipped screen coordinates; viewport clipping is not an occluder', 'actors': rows}))
