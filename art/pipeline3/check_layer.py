# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python -m art.pipeline3.check_layer path/to/layer.json
"""Structural checks only: no style approval or silent pixel repair."""
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
import numpy as np
from PIL import Image
from .layer_schema import Layer, load_layer
from .structure import GEOMETRY, Pixels, box_region, components, holes

@dataclass(frozen=True, slots=True)
class Finding:
    rule: str
    detail: str

@dataclass(frozen=True, slots=True)
class Result:
    id: str
    passed: bool
    findings: tuple[Finding,...]

def read_pixels(layer: Layer) -> tuple[Pixels,Pixels]:
    """Read pixels without resizing, changing alpha, or converting mask ids."""
    with Image.open(layer.image) as image, Image.open(layer.mask) as mask:
        if image.mode != 'RGBA' or mask.mode not in ('L','P'):
            raise ValueError(f'{layer.id}: image must be RGBA; mask must be L or P')
        return np.array(image),np.array(mask)

def check_layer(layer: Layer) -> Result:
    """Check a layer against geometry; declarations never replace pixel checks."""
    findings: list[Finding] = []
    def fail(rule: str, detail: str) -> None:
        findings.append(Finding(rule,detail))
    try:
        pixels,mask = read_pixels(layer)
    except (OSError,ValueError) as error:
        return Result(layer.id,False,(Finding('input',str(error)),))
    s = layer.scale
    width,height = (96,64) if layer.frame == 'sleep' else (64,96)
    if pixels.shape != (height*s,width*s,4) or mask.shape != pixels.shape[:2]:
        return Result(layer.id,False,(Finding('canvas',f'expected {width*s}x{height*s} RGBA and same-size mask'),))
    alpha = pixels[:,:,3]
    silhouette = alpha > 0
    opaque = alpha == 255
    if not np.any(silhouette):
        fail('empty','empty silhouette')
    if np.any(silhouette & ((mask < 1) | (mask > 5))) or np.any((~silhouette) & (mask != 0)):
        fail('mask_completeness','visible pixels require region1..5; transparent pixels require0')
    if layer.kind != 'top' and np.any((mask == 3) | (mask == 4)):
        fail('mask_semantics','cloth regions only on tops')
    if layer.kind in ('hair','beard') and np.any(silhouette & (mask != 2)):
        fail('mask_semantics','hair/beard visible pixels must use region2')
    if layer.kind in ('item','carry') and np.any(silhouette & (mask != 5)):
        fail('mask_semantics','item/carry visible pixels must use region5')
    if layer.kind != 'body' and np.any(mask == 1):
        fail('mask_semantics','skin region only on body')
    if layer.kind in ('body','item','carry') and len(components(silhouette)) != 1:
        fail('connectivity','silhouette must be one 8-connected component')
    if holes(silhouette,s):
        fail('holes','enclosed transparent region exceeds12 game px squared')
    if np.any(silhouette[0,:]) or np.any(silhouette[-1,:]) or np.any(silhouette[:,0]) or np.any(silhouette[:,-1]):
        fail('clipping','visible pixel touches canvas edge; source extent cannot be verified')
    geometry = GEOMETRY[layer.body_type]
    if layer.kind == 'body' and layer.frame == 'sleep' and np.any(silhouette):
        ys,xs = np.nonzero(silhouette)
        x0,y0,x1,y1 = int(xs.min()),int(ys.min()),int(xs.max())+1,int(ys.max())+1
        if x1-x0 <= y1-y0:
            fail('sleep_axis','sleep alpha bbox must be wider than tall')
        center = ((x0+x1)/(2*s),(y0+y1)/(2*s))
        if max(abs(center[0]-48),abs(center[1]-36)) > 2:
            fail('sleep_center',f'alpha bbox center{center} must be(48,36)±2')
        if layer.head_box is None:
            fail('sleep_head','declare head_box on left; mask does not encode head semantics')
        else:
            hx0,hy0,hx1,hy1 = layer.head_box
            if not (0 <= hx0 < hx1 <= 48 and 0 <= hy0 < hy1 <= 64):
                fail('sleep_head','declared head_box must be inside left half of sleep canvas')
            elif not np.any(box_region(mask.shape,layer.head_box,s) & silhouette & (mask == 1)):
                fail('sleep_head','declared left head_box contains no visible skin')
    if layer.kind == 'body' and layer.frame != 'sleep' and np.any(silhouette):
        ys,xs = np.nonzero(silhouette)
        bottom = (int(ys.max())+1)/s
        if abs(bottom-88) > 2:
            fail('feet',f'foot bottom {bottom:.2f}, expected88±2')
        x0,y0,x1,y1 = geometry.head
        head = silhouette & box_region(silhouette.shape,(0,0,64,y1),s)
        hy,hx = np.nonzero(head)
        if not len(hx) or hx.min() < (x0-3)*s or hx.max()+1 > (x1+3)*s or hy.min() < (y0-3)*s or hy.min() > (y0+3)*s:
            fail('head_alignment','head alpha bbox outside template head±3 or missing')
        expected = geometry.east if layer.direction == 'E' else geometry.south
        if layer.direction == 'N':
            expected = tuple(reversed(expected))
        if layer.hands and len(layer.hands) != 2:
            fail('hand_alignment','exactly two declared hands required')
        for index,(ax,ay) in enumerate(expected):
            region = box_region(mask.shape,(ax-4,ay-4,ax+5,ay+5),s)
            if not np.any(region & silhouette & (mask == 1)):
                fail('hand_alignment',f'no skin within4px of hand{index} anchor{ax,ay}')
            if len(layer.hands) == 2:
                hx,hy = layer.hands[index]
                if max(abs(hx-ax),abs(hy-ay)) > 4 or not (0 <= hx*s < width*s and 0 <= hy*s < height*s) or mask[hy*s,hx*s] != 1 or not silhouette[hy*s,hx*s]:
                    fail('hand_alignment',f'declared hand{index} lacks skin or is misaligned')
    if layer.kind == 'top' and layer.frame != 'sleep':
        torso = box_region(mask.shape,geometry.torso,s)
        coverage = float(np.count_nonzero(opaque & torso))/float(np.count_nonzero(torso))
        if coverage < .95:
            fail('coverage',f'opaque torso coverage {coverage:.4f}<0.95')
        x0,y0,x1,y1 = geometry.torso
        allowed = box_region(mask.shape,(x0-4,y0-4,x1+4,y1+4),s)
        if 'cloak_hem' in layer.extras:
            allowed |= box_region(mask.shape,(x0-4,y0,x1+4,geometry.hip+12),s)
        if 'collar' in layer.extras:
            allowed |= box_region(mask.shape,(x0-4,y0-4,x1+4,y0),s)
        if np.any(silhouette & ~allowed):
            fail('overlay_extent','top redraws outside torso±4 and declared extras')
        if np.any(silhouette & ~((mask == 3)|(mask == 4))):
            fail('mask_semantics','top visible pixels must be cloth3/4')
    if layer.kind in ('item','carry'):
        anchor = layer.grip if layer.kind == 'item' else layer.shoulder
        if anchor is None:
            fail('anchor','item requires grip; carry requires shoulder')
        else:
            x,y = anchor
            if not (0 <= x*s < width*s and 0 <= y*s < height*s) or not silhouette[y*s,x*s]:
                fail('anchor','declared anchor must land on a visible pixel')
    return Result(layer.id,not findings,tuple(findings))

def main() -> int:
    """Exit nonzero on rejection; output machine-readable findings."""
    if len(sys.argv) != 2:
        print('usage: python -m art.pipeline3.check_layer SIDECAR',file=sys.stderr)
        return 2
    try:
        result = check_layer(load_layer(Path(sys.argv[1])))
    except (ValueError,TypeError,KeyError,OSError) as error:
        print(json.dumps({'passed':False,'error':str(error)}))
        return 2
    print(json.dumps(asdict(result),indent=2))
    return 0 if result.passed else 1

if __name__ == '__main__':
    raise SystemExit(main())
