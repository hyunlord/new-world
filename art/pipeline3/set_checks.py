# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline3.check_set
"""Cross-frame and exhaustive composition checks."""
from dataclasses import dataclass
from itertools import product
from typing import Final
import numpy as np
from PIL import Image, ImageCms
from .check_layer import Finding, Result, check_layer, read_pixels
from .layer_schema import Layer
from .structure import FRAMES, GEOMETRY, Pixels, box_region, holes

LAB: Final = ImageCms.buildTransformFromOpenProfiles(ImageCms.createProfile('sRGB'),ImageCms.createProfile('LAB'),'RGB','LAB')

@dataclass(frozen=True, slots=True)
class Combination:
    ids: tuple[str,...]
    passed: bool
    findings: tuple[Finding,...]
    coverage_scope: str = 'upright_cartesian'

def fixed_upper(layers: list[Layer]) -> list[Result]:
    """Compare alpha exactly and CIE76 Delta E <=3 in locked boxes."""
    results: list[Result] = []
    for body_type,direction in sorted({(l.body_type,l.direction) for l in layers if l.kind == 'body'}):
        bodies = [l for l in layers if l.kind == 'body' and l.body_type == body_type and l.direction == direction and l.frame in FRAMES]
        idle = next((l for l in bodies if l.frame == 'idle'),None)
        if idle is None:
            results.append(Result(f'{body_type}_{direction}',False,(Finding('fixed_upper','missing idle reference'),)))
            continue
        reference,_ = read_pixels(idle)
        g = GEOMETRY[body_type]
        locked = box_region(reference.shape,g.head,idle.scale) | box_region(reference.shape,g.torso,idle.scale)
        # Shoulder-line margin is outside the two immutable boxes, never a loophole inside them.
        ref_lab = np.array(ImageCms.applyTransform(Image.fromarray(reference[:,:,:3]),LAB)).astype(float)
        ref_lab[:,:,0] *= 100/255
        for layer in bodies:
            pixels,_ = read_pixels(layer)
            if pixels.shape != reference.shape or layer.scale != idle.scale:
                results.append(Result(layer.id,False,(Finding('fixed_upper','scale/canvas differs from idle'),)))
                continue
            lab = np.array(ImageCms.applyTransform(Image.fromarray(pixels[:,:,:3]),LAB)).astype(float)
            lab[:,:,0] *= 100/255
            delta = np.sqrt(np.sum((lab-ref_lab)**2,axis=2))
            bad_alpha = np.any((pixels[:,:,3] != reference[:,:,3]) & locked)
            # Transparent RGB is not visible, but alpha equality still covers all locked pixels.
            bad_color = np.any((delta > 3) & locked & (reference[:,:,3] > 0))
            found = (Finding('fixed_upper',f'alpha_equal={not bad_alpha}; max_deltaE={float(delta[locked & (reference[:,:,3]>0)].max(initial=0)):.3f}'),) if bad_alpha or bad_color else ()
            results.append(Result(layer.id,not found,found))
    return results

def placed(layer: Layer, body: Layer) -> tuple[Pixels,bool]:
    """Place item at canonical hand/shoulder anchor without losing off-canvas pixels."""
    pixels,_ = read_pixels(layer)
    source = layer.grip if layer.kind == 'item' else layer.shoulder
    if source is None:
        return pixels,False
    g = GEOMETRY[body.body_type]
    target = g.shoulder if layer.kind == 'carry' else (g.east[0] if body.direction == 'E' else g.south[1] if body.direction == 'S' else g.south[0])
    dx,dy = ((target[i]-source[i])*body.scale for i in (0,1))
    out = np.zeros_like(pixels)
    ys,xs = np.nonzero(pixels[:,:,3])
    nx,ny = xs+dx,ys+dy
    height,width = pixels.shape[:2]
    inside = (nx >= 0)&(nx < width)&(ny >= 0)&(ny < height)
    out[ny[inside],nx[inside]] = pixels[ys[inside],xs[inside]]
    return out,bool(np.all(inside))

def combinations(layers: list[Layer]) -> list[Combination]:
    """Enumerate full Cartesian products; rejected inputs remain diagnostic only."""
    results: list[Combination] = []
    for body in [l for l in layers if l.kind == 'body']:
        if body.frame == 'sleep':
            result = check_layer(body)
            results.append(Combination((body.id,),result.passed,result.findings,'bare_sleep_only'))
            continue
        compatible = [l for l in layers if l.body_type == body.body_type and l.direction == body.direction]
        tops = [l for l in compatible if l.kind == 'top']
        hairs = [l for l in compatible if l.kind == 'hair']
        items: list[Layer | None] = [None]+[l for l in layers if l.body_type == body.body_type and l.kind == 'item']
        carries: list[Layer | None] = [None]+[l for l in layers if l.body_type == body.body_type and l.kind == 'carry']
        for top,hair,item,carry in product(tops,hairs,items,carries):
            chosen = [body,top,hair]+([item] if item is not None else [])+([carry] if carry is not None else [])
            findings: list[Finding] = []
            arrays = [read_pixels(l)[0] for l in chosen]
            if any(a.shape != arrays[0].shape for a in arrays) or any(l.scale != body.scale for l in chosen):
                results.append(Combination(tuple(l.id for l in chosen),False,(Finding('canvas','combination scales or sizes differ'),)))
                continue
            for index,prop in enumerate(chosen[3:],start=3):
                arrays[index],unclipped = placed(prop,body)
                if not unclipped:
                    findings.append(Finding('clipping',f'anchor placement clips {prop.id} pixels'))
            torso = box_region(arrays[0].shape,GEOMETRY[body.body_type].torso,body.scale)
            if np.any((arrays[0][:,:,3] > 0)&(arrays[1][:,:,3] < 255)&torso):
                findings.append(Finding('body_through_top','body visible through top inside torso'))
            union = np.logical_or.reduce([a[:,:,3] > 0 for a in arrays])
            if holes(union,body.scale):
                findings.append(Finding('union_holes','composed silhouette has enclosed transparent region >12px²'))
            results.append(Combination(tuple(l.id for l in chosen),not findings,tuple(findings)))
    return results
