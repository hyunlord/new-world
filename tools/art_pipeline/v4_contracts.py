# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_v4_contracts.py
"""Typed scale planning and rejection guards for Art Pass 2 section 6."""
from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from statistics import median
from typing import Final

import numpy as np
from PIL import Image

STANDING: Final = frozenset(('idle','walk','carry','gather','build','talk','crossed','hop','run','carry_wood','gather_wood'))
CYCLES: Final = frozenset(('walk','carry','run','carry_wood'))


@dataclass(frozen=True, slots=True)
class WorldScale:
    adult_height: int = 28
    child_height: int = 21

    def __post_init__(self) -> None:
        if (self.adult_height,self.child_height) not in ((28,21),(33,25)):
            raise ContractError('unsupported_world_scale')


class ContractError(ValueError):
    def __init__(self, code: str) -> None:
        self.code = code
        super().__init__(code)


@dataclass(frozen=True, slots=True)
class FrameSpec:
    sheet: str
    cell: int
    attempt: int
    body: str = ''
    view: str = ''
    activity: str = ''
    native_size: tuple[int,int] = (1,1)

    def __post_init__(self) -> None:
        if not 1 <= self.cell <= 16 or self.attempt < 1 or min(self.native_size) < 1:
            raise ContractError('invalid_frame_spec')


@dataclass(frozen=True, slots=True)
class FactorDecision:
    frame: FrameSpec
    factor: float
    source: str
    warnings: tuple[str,...] = ()


@dataclass(frozen=True, slots=True)
class FrameCandidate:
    frame: FrameSpec
    image: Image.Image
    mean_delta_e: float
    palette_match_percent: float
    binary_alpha: bool
    berry_texels: int
    fire_texels: int


@dataclass(frozen=True, slots=True)
class GuardReport:
    errors: tuple[str,...] = ()
    warnings: tuple[str,...] = ()

    @property
    def passed(self) -> bool:
        return not self.errors

    @property
    def ready_for_review(self) -> bool:
        return self.passed and "pending_cycle_completion" not in self.warnings


def target_size(frame: FrameSpec, scale: WorldScale = WorldScale()) -> tuple[int,int]:
    """Return pre-outline budgets, with one global noncharacter escape multiplier."""
    if bool(frame.body):
        child = frame.body == 'child'
        if frame.activity == 'asleep':
            return (24,12) if child else (30,16)
        return (17,scale.child_height) if child else (22,scale.adult_height)
    index = frame.cell
    if frame.sheet == 'W1':
        if index in (12,16):
            return (16,16)
        return (24,16) if index <= 8 else (24,20)
    targets = {'S1': (24,22), 'S2': (24,22), 'N1': (24,18), 'N2': (24,22),
               'N3': (12,12), 'F1': (16,16), 'F2': (16,12), 'U1': (16,16)}
    if frame.sheet not in targets:
        raise ContractError('unknown_object_sheet')
    target = targets[frame.sheet]
    if frame.sheet == 'S1' and (index <= 7 or index == 16):
        target = (24,26)
    if (frame.sheet == 'S1' and 11 <= index <= 13) or (frame.sheet == 'S2' and index <= 6):
        target = (34,30)
    if frame.sheet == 'N1':
        if index <= 3:
            target = (40,48)
        if index in (4,5,6,7,8,16):
            target = (30,40)
        if index == 14:
            target = (16,24)
    if frame.sheet == 'N2' and index <= 4:
        target = (26,22)
    if frame.sheet == 'N2' and index in (10,11):
        target = (32,30)
    ratio = scale.adult_height / 28
    return int(np.floor(target[0]*ratio+.5)), int(np.floor(target[1]*ratio+.5))


def plan_factors(frames: Sequence[FrameSpec], scale: WorldScale = WorldScale()) -> tuple[FactorDecision,...]:
    """Choose one factor per group; use median group factors for nonstanding poses."""
    groups: dict[tuple[str,str,int,str,str],list[FrameSpec]] = {}
    for frame in frames:
        if frame.body and frame.activity in STANDING:
            key = (frame.body,frame.sheet,frame.attempt,frame.view,frame.activity)
            groups.setdefault(key,[]).append(frame)
    factors = {key:min(1.0,target_size(group[0],scale)[1]/median(f.native_size[1] for f in group)) for key,group in groups.items()}
    result: list[FactorDecision] = []
    for frame in frames:
        if not frame.body:
            width,height = target_size(frame,scale)
            factor = min(1.0,width/frame.native_size[0],height/frame.native_size[1])
            result.append(FactorDecision(frame,factor,'object_bounds'))
            continue
        key = (frame.body,frame.sheet,frame.attempt,frame.view,frame.activity)
        if key in factors:
            result.append(FactorDecision(frame,factors[key],'standing_group'))
            continue
        own = [value for group,value in factors.items() if group[:3] == key[:3]]
        other_attempts: dict[tuple[str,str,int],list[float]] = {}
        for group,value in factors.items():
            if group[0] == frame.body and group[:3] != key[:3]:
                other_attempts.setdefault(group[:3],[]).append(value)
        references = own or [float(median(values)) for values in other_attempts.values()]
        source = 'attempt_standing_median' if own else 'body_other_attempts_median'
        result.append(FactorDecision(frame,float(median(references)) if references else 1.0,source,
                                     () if references else ('no_standing_reference',)))
    return tuple(result)


def guard_frame(candidate: FrameCandidate, scale: WorldScale = WorldScale()) -> GuardReport:
    """Reject broken format/geometry or gross drift; moderate drift is a warning."""
    frame,image = candidate.frame,candidate.image
    errors: list[str] = []
    warnings: list[str] = []
    if candidate.palette_match_percent != 100.0:
        errors.append('palette_membership')
    alpha = np.asarray(image.convert('RGBA'))[...,3]
    if not candidate.binary_alpha or not np.all((alpha==0)|(alpha==255)):
        errors.append('binary_alpha')
    if not np.isfinite(candidate.mean_delta_e) or candidate.mean_delta_e > 12:
        errors.append('gross_colour_error')
    elif candidate.mean_delta_e > 6:
        warnings.append('colour_drift')
    width,height = target_size(frame,scale)
    if image.width > width or (not frame.body or frame.activity == 'asleep') and image.height > height:
        errors.append('geometry')
    if frame.body and frame.activity in STANDING and abs(image.height-height)>2:
        errors.append('standing_height')
    if (frame.sheet == 'U1' or frame.sheet == 'W1' and frame.cell in (12,16)) and image.size != target_size(frame,scale):
        errors.append('icon_canvas')
    bbox = image.getchannel('A').getbbox()
    if bbox is None:
        errors.append('empty_sprite')
    elif frame.body and bbox[3] != image.height:
        errors.append('feet_baseline')
    if frame.sheet == 'S1' and 2 <= frame.cell <= 5 and candidate.fire_texels < 12:
        errors.append('fire_ramp')
    return GuardReport(tuple(errors),tuple(warnings))


def _leg_difference(first: Image.Image, second: Image.Image) -> int:
    width,height = max(first.width,second.width),max(first.height,second.height)
    arrays = []
    for image in (first,second):
        canvas = Image.new('RGBA',(width,height))
        canvas.paste(image,((width-image.width)//2,height-image.height))
        pixels = np.array(canvas)
        pixels[pixels[...,3]==0] = 0
        arrays.append(pixels[int(np.floor(height*.6)):])
    return int(np.any(arrays[0]!=arrays[1],axis=-1).sum())


def guard_group(candidates: Sequence[FrameCandidate], scale: WorldScale = WorldScale()) -> GuardReport:
    """Group-only guards; caller filters frame guards and rechecks derived cycles.

    pending_cycle_completion means provisional, never ready for acceptance.
    Frame-local failures intentionally do not invalidate their passing peers.
    """
    if not candidates:
        return GuardReport(('empty_group',))
    frames = [c.frame for c in candidates]
    keys = {(f.body,f.sheet,f.attempt,f.view,f.activity) for f in frames}
    if len(keys) != 1:
        return GuardReport(('mixed_animation_group',))
    first = frames[0]
    errors: list[str] = []
    warnings: list[str] = []
    if first.body and first.activity in STANDING:
        if abs(median(c.image.height for c in candidates)-target_size(first,scale)[1]) > 1:
            errors.append('group_median_height')
    if first.activity in CYCLES:
        ordered = sorted(candidates,key=lambda c:c.frame.cell)
        spans = [box[3]-box[1] for c in ordered if (box:=c.image.getchannel('A').getbbox()) is not None]
        if spans and max(spans)-min(spans)>1:
            errors.append('head_top')
        if len(ordered) == 2 and first.view == 'N' and first.activity == 'carry':
            pairs = [(ordered[0],ordered[1])]
            warnings.append('two_frame_cycle')
        else:
            pairs = [(a,b) for a in ordered[::2] for b in ordered[1::2]]
            if len(ordered) != 4:
                warnings.append('pending_cycle_completion')
                pairs = []
        if pairs and any(_leg_difference(a.image,b.image)<4 for a,b in pairs):
            errors.append('leg_difference')
    return GuardReport(tuple(dict.fromkeys(errors)),tuple(dict.fromkeys(warnings)))


def guard_chosen(bushes: Sequence[FrameCandidate] = (), adults: Sequence[FrameCandidate] = (),
                 children: Sequence[FrameCandidate] = (), baskets: Sequence[FrameCandidate] = ()) -> GuardReport:
    """Cross-slot semantics use chosen output sprites, never unrelated rejected cells."""
    errors: list[str] = []
    if bushes:
        counts = [c.berry_texels for c in bushes]
        if len(counts)!=4 or not (counts[0]>counts[1]>counts[2]>=1 and counts[3]==0):
            errors.append('berry_stage_order')
    adult_heights = [c.image.height for c in adults if c.frame.activity in STANDING]
    if children and not adult_heights:
        errors.append('missing_chosen_adult_reference')
    if adult_heights and any(c.image.height>=min(adult_heights) for c in children if c.frame.activity in STANDING):
        errors.append('child_relative_height')
    if baskets and len(bushes)!=4:
        errors.append('missing_chosen_bush_reference')
    for basket in baskets:
        if any(basket.image.width>b.image.width or basket.image.height>b.image.height or basket.image.width*basket.image.height>=b.image.width*b.image.height for b in bushes):
            errors.append('basket_relative_size')
    return GuardReport(tuple(dict.fromkeys(errors)))
