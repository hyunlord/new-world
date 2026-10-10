# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: import fit_cell_v4; tests: python3 -m unittest discover -s tools/art_pipeline -p test_normalize_v4.py
"""Palette-first source-grid fitting with auditable literal v4 measurements."""
from __future__ import annotations
from dataclasses import dataclass
from typing import TypedDict
import numpy as np
from numpy.typing import NDArray
from PIL import Image
from native_grid import _blocks, key_mask, remove_spill
from palette_tools import Color, rgb_to_lab
from pixel_normalize import outline
from pixel_normalize_v3 import RecoveryMetadata, recover_cell_v3, quantize_native

class FitMetrics(TypedDict):
    native_before: list[int]
    native_after: list[int]
    period_xy: list[float]
    new_period_xy: list[float]
    phase_xy: list[float]
    phase_origin_xy: list[float]
    sampling_phase_xy: list[float]
    outlined_canvas: list[int]
    factor: float
    mean_delta_e: float
    common_opaque: int
    keyed_out: int
    palette_match_percent: float
    binary_alpha: bool
    berry_texels: int
    fire_texels: int
    spill_removed: int
    fringe_repaired: int
    post_spill_projected_texels: int
    warnings: list[str]

@dataclass(frozen=True, slots=True)
class FitResult:
    native: Image.Image
    sprite: Image.Image
    median_sample: Image.Image
    metrics: FitMetrics


def measure_cell_v4(image: Image.Image) -> RecoveryMetadata:
    """Expose unchanged v3 source-grid evidence for cross-cell factor planning."""
    _,metadata=recover_cell_v3(image)
    return metadata


def palette_mode(pixels: NDArray[np.uint8]) -> Color:
    """Pick the most frequent RGB palette entry, lower L* resolving ties."""
    if not len(pixels):raise ValueError('Cannot pool an empty foreground block')
    colors,counts=np.unique(pixels.reshape(-1,3),axis=0,return_counts=True)
    tied=np.flatnonzero(counts==counts.max())
    winner=int(tied[np.argmin(rgb_to_lab(colors[tied])[:,0])])
    return tuple(map(int,colors[winner]))


def _quantize_crop(crop: NDArray[np.uint8], foreground: NDArray[np.bool_], palette: list[Color]) -> NDArray[np.uint8]:
    """Bound palette distance allocations while using every foreground pixel."""
    result=crop.copy();result[...,3]=foreground.astype(np.uint8)*255
    pixels=result[foreground,:3];colors=np.asarray(palette,dtype=np.uint8);labs=rgb_to_lab(colors)
    for start in range(0,len(pixels),4096):
        block=pixels[start:start+4096]
        distances=np.sum((rgb_to_lab(block)[:,None]-labs[None])**2,axis=2)
        pixels[start:start+4096]=colors[np.argmin(distances,axis=1)]
    result[foreground,:3]=pixels
    return result


def pool_crop(crop: NDArray[np.uint8], foreground: NDArray[np.bool_], palette: list[Color],
              periods: tuple[float,float], phases: tuple[float,float]) -> tuple[Image.Image,Image.Image]:
    """Mode-pool exact inner-half source blocks and retain parallel raw medians."""
    quantized=_quantize_crop(crop,foreground,palette)
    columns=_blocks(crop.shape[1],periods[0],phases[0]);rows=_blocks(crop.shape[0],periods[1],phases[1])
    result=np.zeros((len(rows),len(columns),4),dtype=np.uint8);median=result.copy()
    for y,(y0,y1) in enumerate(rows):
        for x,(x0,x1) in enumerate(columns):
            coverage=foreground[y0:y1,x0:x1]
            if coverage.sum()*2>=coverage.size:
                result[y,x]=[*palette_mode(quantized[y0:y1,x0:x1,:3][coverage]),255]
                median[y,x]=[*np.median(crop[y0:y1,x0:x1,:3][coverage],axis=0).astype(np.uint8),255]
    image=Image.fromarray(result);bbox=image.getbbox()
    if bbox is None:raise ValueError('Fitting retained no opaque texels')
    return image.crop(bbox),Image.fromarray(median).crop(bbox)


def fit_cell_v4(image: Image.Image,palette: list[Color],factor: float,*,icon_canvas: bool=False,
                measurement: RecoveryMetadata | None=None) -> FitResult:
    """Use p/f and unchanged source phase; never upscale or interpolate texels."""
    if not palette:raise ValueError('Production palette must not be empty')
    if not np.isfinite(factor) or factor<=0:raise ValueError('Factor must be positive and finite')
    factor=min(1.,float(factor));metadata=measurement or measure_cell_v4(image)
    bbox=metadata['source_bbox'];crop=np.asarray(image.convert('RGBA').crop(tuple(bbox)))
    foreground=key_mask(crop)
    px,py=map(float,metadata['period_xy'])
    phix,phiy=map(float,metadata['phase_xy'])
    periods=(px/factor,py/factor)
    origin=(phix-px,phiy-py)
    # _blocks starts at phase - period: preserve that physical boundary.
    phases=(origin[0]+periods[0],origin[1]+periods[1])
    pooled,median=pool_crop(crop,foreground,palette,periods,phases)
    clean,spill=remove_spill(pooled)
    projected=quantize_native(clean,palette)
    changed=int(np.any(np.asarray(projected)[...,:3]!=np.asarray(clean)[...,:3],axis=2).sum())
    if icon_canvas:
        if projected.width>16 or projected.height>16:raise ValueError('Icon factor did not fit native16canvas')
        canvas=Image.new('RGBA',(16,16));reference=canvas.copy()
        offset=((16-projected.width)//2,(16-projected.height)//2)
        canvas.paste(projected,offset);reference.paste(median,offset)
        projected,median=canvas,reference
    before=np.asarray(median);after=np.asarray(projected)
    common=(before[...,3]==255)&(after[...,3]==255)
    delta=np.linalg.norm(rgb_to_lab(before[common,:3])-rgb_to_lab(after[common,:3]),axis=1)
    mean=float(delta.mean()) if len(delta) else float('inf')
    from v3_semantics import semantic_counts
    berry,fire=semantic_counts(projected)
    allowed=set(palette);opaque=after[...,3]==255
    membership=100.*sum(tuple(map(int,c)) in allowed for c in after[opaque,:3])/max(1,int(opaque.sum()))
    warnings=[]
    if any(min(abs(p-4),abs(p-8))<=.05+1e-9 for p in metadata['period_xy']):warnings.append('grid_suspect')
    if mean>6:warnings.append('colour_drift')
    if changed:warnings.append('post_spill_palette_projection')
    dark=palette[int(np.argmin(rgb_to_lab(np.asarray(palette,dtype=np.uint8))[:,0]))]
    sprite=outline(projected,dark)
    metrics=FitMetrics(native_before=list(metadata['native_size']),native_after=list(projected.size),
        period_xy=list(metadata['period_xy']),new_period_xy=list(periods),phase_xy=[phix,phiy],
        phase_origin_xy=list(origin),sampling_phase_xy=list(phases),outlined_canvas=list(sprite.size),factor=factor,
        mean_delta_e=mean,common_opaque=int(common.sum()),keyed_out=int(((before[...,3]==255)&~opaque).sum()),
        palette_match_percent=membership,binary_alpha=bool(np.isin(after[...,3],[0,255]).all()),
        berry_texels=berry,fire_texels=fire,spill_removed=spill['spill_removed'],fringe_repaired=spill['fringe_repaired'],
        post_spill_projected_texels=changed,warnings=warnings)
    return FitResult(projected,sprite,median,metrics)
