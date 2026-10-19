"""Framebuffer-based pixel, occlusion, contrast and night checks."""
from __future__ import annotations
from dataclasses import dataclass
from pathlib import Path
import numpy as np
from ingame_qa_types import Capture, Entity, box, luma, placed

@dataclass(frozen=True, slots=True)
class Check:
    code: str
    status: str
    detail: str


def result(code: str, passed: bool, detail: str) -> Check:
    return Check(code, 'PASS' if passed else 'FAIL', detail)


def checker(captures: list[Capture]) -> Check:
    samples = []
    for capture in captures:
        raw = capture.image(False)
        for entity in capture.entities:
            if entity.role != 'checker':
                continue
            x0, y0, x1, y1 = box(entity)
            block = round(capture.zoom * capture.scale)
            patch = raw[y0:y1, x0:x1]
            valid = patch.shape[:2] == (8*block, 8*block)
            if valid:
                expected = np.indices((8, 8)).sum(axis=0) % 2 == 0
                expected = np.repeat(np.repeat(expected, block, 0), block, 1)
                valid = bool(np.all(patch == expected[..., None] * 255))
            samples.append((capture.zoom, valid))
    return result('A2', {s[0] for s in samples} == {4, 6} and all(s[1] for s in samples), f'raw checker zero mixed pixels: {samples}') if samples else Check('A2', 'N/A', 'No raw calibration checker evidence')


def calibration_order(captures: list[Capture], repo: Path) -> list[Check]:
    outcomes, ramps = [], []
    for capture in captures:
        raw = capture.image(False)
        by_role = {e.role: e for e in capture.entities}
        for prefix, front_role in [('north', 'north_lean_to'), ('south', 'south_agent')]:
            roles = [prefix + '_agent', prefix + '_lean_to']
            if not all(role in by_role for role in roles):
                continue
            color_a, mask_a = placed(by_role[roles[0]], capture, repo, True, True)
            color_b, mask_b = placed(by_role[roles[1]], capture, repo, True, True)
            informative = mask_a & mask_b & np.any(color_a != color_b, axis=2)
            expected = color_a if front_role == roles[0] else color_b
            count = int(informative.sum())
            agreement = float((np.max(np.abs(raw.astype(int)-expected.astype(int)), axis=2)[informative] <= 1).mean()) if count else 0.0
            outcomes.append((capture.zoom, prefix, count, round(agreement, 4)))
        for entity in capture.entities:
            if entity.role != 'ramp_variant':
                continue
            region_path = repo / entity.texture.removeprefix('res://')
            region_path = region_path.with_name(region_path.stem + '_regions.png')
            if not region_path.exists() or min(entity.variant) < 0:
                ramps.append((capture.zoom, entity.ident, 0.0))
                continue
            expected, mask = placed(entity, capture, repo, True, True)
            count = int(mask.sum())
            agreement = float((np.max(np.abs(raw.astype(int)-expected.astype(int)), axis=2)[mask] <= 1).mean()) if count else 0.0
            ramps.append((capture.zoom, entity.ident, round(agreement, 4)))
    order = result('A4', len(outcomes) == 4 and all(o[2] > 0 and o[3] >= .99 for o in outcomes), str(outcomes)) if outcomes else Check('A4', 'N/A', 'No calibration overlap pairs')
    ramp = result('RAMP', bool(ramps) and all(r[2] >= .99 for r in ramps), f'actual framebuffer agrees with region-indexed ramp texels: {ramps}') if ramps else Check('RAMP', 'N/A', 'No calibration ramp variants')
    return [order, ramp]


def contrast(capture: Capture, repo: Path) -> Check:
    image_luma = luma(capture.image())
    occupied = np.zeros(image_luma.shape, dtype=np.uint16)
    actors = [e for e in capture.entities if e.kind == 'agent' and e.visible]
    blockers = []
    for entity in capture.entities:
        if not entity.visible or 'shadow' in entity.kind or entity.kind == 'panel' or entity.alpha <= 0:
            continue
        if entity.kind == 'tree' and entity.alpha <= .5:
            continue
        _, mask = placed(entity, capture, repo)
        occupied += mask.astype(np.uint16)
        blockers.append((entity,mask))
    values, skipped = [], []
    for entity in actors:
        _, mask = placed(entity, capture, repo)
        visible = mask.copy()
        order = (entity.draw_order[0],entity.world_anchor[1],entity.draw_order[1])
        for other,other_mask in blockers:
            other_order = (other.draw_order[0],other.world_anchor[1],other.draw_order[1])
            if other.ident != entity.ident or other.kind != entity.kind:
                if other_order > order:
                    visible &= ~other_mask
        x0, y0, x1, y1 = box(entity, capture.scale)
        ring = np.zeros_like(mask)
        ring[max(0,y0-3):min(1080,y1+3), max(0,x0-3):min(1920,x1+3)] = True
        ring[max(0,y0):min(1080,y1), max(0,x0):min(1920,x1)] = False
        ring &= occupied == 0
        for panel in capture.entities:
            if panel.kind == 'panel' and panel.visible:
                a,b,c,d = box(panel, capture.scale)
                visible[max(0,b):min(1080,d),max(0,a):min(1920,c)] = False
                ring[max(0,b):min(1080,d),max(0,a):min(1920,c)] = False
        if not visible.any() or not ring.any():
            skipped.append(entity.ident)
            continue
        values.append((entity.ident, abs(float(image_luma[visible].mean()-image_luma[ring].mean()))))
    if not values:
        return Check('A5', 'FAIL' if actors else 'N/A', f'No measurable actor/ring pixels; excluded {skipped}')
    numbers = [v for _,v in values]
    median, minimum = float(np.median(numbers)), min(numbers)
    return result('A5', median >= .15 and minimum >= .10 and not skipped,
                  f'median={median:.4f}; min={minimum:.4f}; per actor={values}; unmeasurable={skipped}; opaque front occluders excluded; faded canopy pixels measured directly from blended framebuffer')


def night(noon: Capture, dark: Capture) -> Check:
    means = []
    dark_luma = luma(dark.image())
    for capture in [noon, dark]:
        pixels = luma(capture.image())
        world = np.ones(pixels.shape, dtype=bool)
        for panel in capture.entities:
            if panel.kind == 'panel' and panel.visible:
                x0,y0,x1,y1 = box(panel, capture.scale)
                world[max(0,y0):min(1080,y1),max(0,x0):min(1920,x1)] = False
        means.append(float(pixels[world].mean()))
    yy, xx = np.indices(dark_luma.shape)
    radii = []
    for entity in dark.entities:
        if entity.kind == 'building' and entity.building_kind == 0 and entity.visible:
            x0,y0,x1,y1 = box(entity, dark.scale)
            ring = (xx-(x0+x1)/2)**2 + (yy-(y0+y1)/2)**2 <= 48**2
            radii.append(float(dark_luma[ring].mean()))
    if not radii:
        return Check('A6', 'N/A', f'No visible hearth; noon/night world means={means}')
    return result('A6', means[1] <= .65*means[0] and min(radii) >= 1.25*means[1], f'noon/night world={means}; 48px hearth means={radii}; each hearth tested')
