# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline4.pawn_gen OUTPUT (tests: python3 -m pytest art/pipeline4)
"""Four-times raster ink: flat light bands, inner contour and seeded paper grain."""
import hashlib
import json
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw, ImageFilter
from . import pawn_spec as spec

Primitive = tuple[str, tuple, str]
PARAMS = json.loads(Path(__file__).with_name('ink_params.json').read_text())
PALETTE = json.loads((Path(__file__).parents[1]/'style05/palette_05.json').read_text())['rgb']
SPEC_HASH = hashlib.sha256(Path(spec.__file__).read_bytes()+Path(__file__).with_name('ink_params.json').read_bytes()).hexdigest()

def raster(shapes: list[Primitive], colors: dict[str, tuple[int,int,int]]) -> Image.Image:
    """Paint declarative primitives without changing their alpha silhouette."""
    size = tuple(v*spec.SCALE for v in spec.CANVAS)
    result = Image.new('RGBA', size)
    yy,xx = np.indices((size[1],size[0]))
    for primitive,coords,material in shapes:
        mask = Image.new('L',size)
        draw = ImageDraw.Draw(mask)
        match primitive:
            case 'ellipse' | 'capsule':
                box = tuple(round(v*spec.SCALE) for v in coords)
                if primitive == 'ellipse':
                    draw.ellipse(box,fill=255)
                else:
                    draw.rounded_rectangle(box,radius=min(box[2]-box[0],box[3]-box[1])//2,fill=255)
            case 'polygon' | 'polyline':
                points = [(round(x*spec.SCALE),round(y*spec.SCALE)) for x,y in coords]
                if primitive == 'polygon':
                    draw.polygon(points,fill=255)
                else:
                    draw.line(points,fill=255,width=spec.SCALE*PARAMS['stroke_width'],joint='curve')
            case _:
                raise ValueError(f'Unsupported primitive {primitive}')
        support = np.asarray(mask)>0
        bounds = mask.getbbox()
        if bounds is None:
            continue
        x0,y0,x1,y1 = bounds
        diagonal = ((xx-x0)/max(1,x1-x0)+(yy-y0)/max(1,y1-y0))/2
        shade = np.ones(support.shape)
        shade[diagonal>np.quantile(diagonal[support],1-PARAMS['shadow_fraction'])] = PARAMS['shadow_multiplier']
        shade[diagonal<np.quantile(diagonal[support],PARAMS['highlight_fraction'])] = PARAMS['highlight_multiplier']
        rgb = np.clip(np.array(colors[material])[None,None,:]*shade[:,:,None],0,255).astype('uint8')
        rgba = np.dstack((rgb,np.asarray(mask)))
        result.alpha_composite(Image.fromarray(rgba))
    alpha = result.getchannel('A')
    inner = alpha.filter(ImageFilter.MinFilter(PARAMS['outline_width']*spec.SCALE*2+1))
    edge = (np.asarray(alpha)>0)&(np.asarray(inner)==0)
    pixels = np.array(result)
    pixels[edge,:3] = PARAMS['outline_rgb']
    rng = np.random.default_rng(int(SPEC_HASH[:16],16))
    grain = 1+rng.uniform(-PARAMS['grain_amplitude'],PARAMS['grain_amplitude'],pixels.shape[:2])
    pixels[:,:,:3] = np.clip(pixels[:,:,:3]*grain[:,:,None],0,255).astype('uint8')
    return Image.fromarray(pixels)


def finish(image: Image.Image, region: int) -> tuple[Image.Image,Image.Image]:
    """Lanczos downsample; remove sub-visible ringing consistently for all layers."""
    pixels = np.array(image.resize(spec.CANVAS,Image.Resampling.LANCZOS))
    pixels[pixels[:,:,3]<PARAMS['alpha_support_cutoff']] = 0
    return Image.fromarray(pixels),Image.fromarray(np.where(pixels[:,:,3]>0,region,0).astype('uint8'))
