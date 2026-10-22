# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline4.pawn_gen OUTPUT (tests: python3 -m pytest art/pipeline4)
"""Deterministic layered pawn generator. Run: python -m art.pipeline4.pawn_gen OUTPUT."""
import json
import sys
import time
from dataclasses import asdict, dataclass
from itertools import product
from pathlib import Path
from typing import Iterator
from PIL import Image, ImageDraw
import numpy as np
from . import pawn_spec as spec
from .ink import PALETTE, PARAMS, SPEC_HASH, finish, raster

@dataclass(frozen=True, slots=True)
class PawnLook:
    sex: str = 'M'
    age: str = 'adult'
    skin_tone: int = 0
    hair_style: str = 'short'
    hair_color: int = 0
    top: str = 'woven_tunic'
    top_palette: int = 0
    item: str = 'none'
    facing: str = 'S'
    state: str = 'idle'


def colors(look: PawnLook) -> dict[str,tuple[int,int,int]]:
    """Select exact palette05 entries; ink color is explicit in ink parameters."""
    indices = {'skin':look.skin_tone,'hair':look.hair_color,'cloth':look.top_palette,'secondary':look.top_palette,'wood':0,'stone':0}
    return {**{key:tuple(PALETTE[values[indices[key]]]) for key,values in spec.PALETTE_INDICES.items()},'ink':tuple(PARAMS['outline_rgb'])}


def registered(image: Image.Image, look: PawnLook) -> Image.Image:
    """Apply whole-layer integer motion after the single idle rasterization."""
    if look.state == 'sleep':
        return image.transpose(Image.Transpose.ROTATE_90)
    dx,dy,_ = spec.MOTION[look.state]
    output = Image.new(image.mode,image.size)
    output.paste(image,(-dx if look.facing=='N' else dx,dy))
    return output


def age_overlay(image: Image.Image, look: PawnLook, kind: str) -> Image.Image:
    """Child garments narrow with capsule; child hair scales about head centre."""
    sx,sy = spec.AGE_OVERLAY[look.age]
    if kind == 'top':
        sx,sy = spec.BODY_WIDTH[look.age][look.sex]/26,1
    if sx == 1 and sy == 1:
        return image
    cx,cy = (32,42) if kind=='hair' else (32,54)
    return image.transform(image.size,Image.Transform.AFFINE,(1/sx,0,cx*(1-1/sx),0,1/sy,cy*(1-1/sy)),Image.Resampling.NEAREST)


def render_layer(kind: str, look: PawnLook) -> tuple[Image.Image,Image.Image]:
    """Render one layer from declarative idle geometry; masks cover every alpha pixel."""
    palette = colors(look)
    strokes = []
    match kind:
        case 'body':
            width = spec.BODY_WIDTH['adult']['M']
            shapes = []
            for primitive,coords,material in spec.BODY:
                if primitive == 'polygon':
                    adjusted = tuple((32+(x-32)*width/26,y) for x,y in coords)
                else:
                    adjusted = coords if coords==spec.BODY[-1][1] else (32+(coords[0]-32)*width/26,coords[1],32+(coords[2]-32)*width/26,coords[3])
                shapes.append((primitive,adjusted,material))
            if look.age == 'child':
                shapes[-1] = ('ellipse',spec.HEAD_CHILD,'skin')
            region = 1
        case 'hair':
            shapes = list(spec.HAIR[look.hair_style])
            if look.facing=='N' and look.hair_style not in ('none','grey_thin'):
                shapes = list(spec.HAIR_BACK)+shapes
            strokes = spec.HAIR_STROKES[look.hair_style]
            region = 2
        case 'top':
            shapes = (list(spec.TOP_ENVELOPE)+list(spec.TOP[look.top])) if look.top!='none' else []
            strokes = spec.TOP_STROKES[look.top]
            region = 3
        case 'item':
            shapes = []
            gx,gy = spec.GRIPS[look.facing]
            if look.state=='carry':
                gx,gy = 42,56
            for primitive,coords,material in spec.ITEM[look.item]:
                if primitive in ('polygon','polyline'):
                    adjusted = tuple((gx+x,gy+y) for x,y in coords)
                else:
                    adjusted = (gx+coords[0],gy+coords[1],gx+coords[2],gy+coords[3])
                if look.facing=='N' and look.state!='carry':
                    if primitive in ('polygon','polyline'):
                        adjusted = tuple((2*gx-x,y) for x,y in adjusted)
                    else:
                        adjusted = (2*gx-adjusted[2],adjusted[1],2*gx-adjusted[0],adjusted[3])
                shapes.append((primitive,adjusted,material))
            region = 5
        case _:
            raise ValueError(f'Unsupported layer {kind}')
    hi = raster(shapes,palette)
    if kind=='top' and look.top!='none':
        cropped = Image.new('RGBA',hi.size)
        cropped.paste(hi.crop((0,spec.TOP_START*4,spec.CANVAS[0]*4,spec.TOP_CROP[3]*4)),(0,spec.TOP_START*4))
        hi = cropped
    draw = ImageDraw.Draw(hi)
    for points in strokes:
        draw.line([(x*4,y*4) for x,y in points],fill=palette['ink'],width=spec.SCALE*PARAMS['stroke_width'])
    if kind=='body':
        for primitive,coords,material in spec.FACE[look.facing]+(spec.ELDER_FACE if look.age=='elder' and look.facing=='S' else []):
            if primitive=='ellipse':
                draw.ellipse(tuple(v*4 for v in coords),fill=palette[material])
            else:
                draw.line([(x*4,y*4) for x,y in coords],fill=palette[material],width=spec.SCALE*PARAMS['stroke_width'])
    if kind=='item' and look.state!='sleep':
        angle = spec.MOTION[look.state][2]*(-1 if look.facing=='N' else 1)
        hi = hi.rotate(angle,Image.Resampling.BICUBIC,center=tuple(v*4 for v in spec.GRIPS[look.facing]))
    image,mask = finish(hi,region)
    if kind=='top' and look.top!='none':
        secondary = [shape for shape in shapes if shape[2]=='secondary']
        secondary_image,_ = finish(raster(secondary,palette),4)
        values = np.array(mask)
        values[(np.asarray(secondary_image)[:,:,3]>0)&(values>0)] = 4
        mask = Image.fromarray(values)
    if kind=='body':
        scaled_image,scaled_mask = age_overlay(image,look,'top'),age_overlay(mask,look,'top')
        scaled_image.paste(image.crop((0,0,64,54)),(0,0))
        scaled_mask.paste(mask.crop((0,0,64,54)),(0,0))
        image,mask = scaled_image,scaled_mask
    if kind in ('hair','top'):
        image,mask = age_overlay(image,look,kind),age_overlay(mask,look,kind)
        envelope,_ = render_layer('body',PawnLook(sex=look.sex,age=look.age,facing=look.facing))
        support = np.asarray(envelope)[:,:,3]>0
        pixels,regions = np.array(image),np.array(mask)
        pixels[~support],regions[~support] = 0,0
        image,mask = Image.fromarray(pixels),Image.fromarray(regions)
    if kind == 'item':
        return image,mask
    return registered(image,look),registered(mask,look)


def compose(look: PawnLook) -> Image.Image:
    """Compose the exact same layers used by the inventory generator."""
    image,_ = render_layer('body',look)
    for kind in ('top','hair','item'):
        overlay,_ = render_layer(kind,look)
        if kind == 'item':
            overlay = registered(overlay,look)
        image.alpha_composite(overlay)
    return image




def item_placement(overlay: Image.Image, look: PawnLook) -> tuple[Image.Image,bool]:
    """Place on a padded canvas and expose any alpha lost outside output bounds."""
    pad = max(spec.CANVAS)
    padded = Image.new('RGBA',(overlay.width+2*pad,overlay.height+2*pad))
    padded.paste(overlay,(pad,pad))
    if look.state in ('walk_a','walk_b'):
        angle = spec.MOTION[look.state][2]*(-1 if look.facing=='N' else 1)
        gx,gy = spec.GRIPS[look.facing]
        padded = padded.rotate(angle,Image.Resampling.BICUBIC,center=(gx+pad,gy+pad))
    if look.state=='carry' and look.item=='log_bundle':
        gx,gy = spec.GRIPS[look.facing]
        cx,cy = spec.CARRY_GRIPS[look.facing]
        shifted = Image.new('RGBA',padded.size)
        shifted.paste(padded,(cx-gx,cy-gy))
        padded = shifted
    placed = registered(padded,look)
    width,height = reversed(spec.CANVAS) if look.state=='sleep' else spec.CANVAS
    image = placed.crop((pad,pad,pad+width,pad+height))
    intact = np.count_nonzero(np.asarray(placed)[:,:,3]) == np.count_nonzero(np.asarray(image)[:,:,3])
    return image,bool(intact)


def place_item(overlay: Image.Image, look: PawnLook) -> Image.Image:
    """Return the composed view while item_placement exposes clipping evidence."""
    return item_placement(overlay,look)[0]


def compose_layers(root: Path, look: PawnLook) -> Image.Image:
    """Load checked inventory layers and register them for browser/lab playback."""
    root = Path(root)
    body_id = f'body_{look.age}_{look.sex}_{look.skin_tone}_{look.facing}_{look.state}'
    with Image.open(root/'BODY'/f'{body_id}.png') as source:
        image = source.convert('RGBA')
    names = [('top',f'top_{look.top}_{look.top_palette}_{look.facing}'),('hair',f'hair_{look.hair_style}_{look.hair_color}_{look.facing}')]
    for kind,name in names:
        with Image.open(root/kind.upper()/f'{name}.png') as source:
            overlay = source.convert('RGBA')
        image.alpha_composite(registered(age_overlay(overlay,look,kind),look))
    pose = look.state if look.state in ('work_a','work_b') else 'idle'
    with Image.open(root/'ITEM'/f'item_{look.item}_{look.facing}_{pose}.png') as source:
        overlay = source.convert('RGBA')
    image.alpha_composite(place_item(overlay,look))
    return image


def iter_inventory(include_ages: bool = True) -> Iterator[tuple[str,str,PawnLook]]:
    """Enumerate independent reusable layers, never Cartesian outfit combinations."""
    ages = ('adult','child','elder') if include_ages else ('adult',)
    for age,sex,skin,direction,state in product(ages,('M','F'),range(4),spec.DIRECTIONS,spec.STATES):
        yield 'body',f'body_{age}_{sex}_{skin}_{direction}_{state}',PawnLook(sex=sex,age=age,skin_tone=skin,facing=direction,state=state)
    for style,color,direction in product(spec.HAIRS,range(4),spec.DIRECTIONS):
        yield 'hair',f'hair_{style}_{color}_{direction}',PawnLook(hair_style=style,hair_color=color,facing=direction)
    for top,palette,direction in product(spec.TOPS,range(2),spec.DIRECTIONS):
        yield 'top',f'top_{top}_{palette}_{direction}',PawnLook(top=top,top_palette=palette,facing=direction)
    for item,direction,state in product(spec.ITEMS,spec.DIRECTIONS,('idle','work_a','work_b')):
        yield 'item',f'item_{item}_{direction}_{state}',PawnLook(item=item,facing=direction,state=state)


def generate(destination: Path) -> None:
    """Write PNG/mask/sidecars and regeneration measurements to an external directory."""
    started = time.perf_counter()
    inventory = []
    for kind,identifier,look in iter_inventory():
        folder = destination/kind.upper()
        folder.mkdir(parents=True,exist_ok=True)
        image,mask = render_layer(kind,look)
        image.save(folder/f'{identifier}.png')
        mask.save(folder/f'{identifier}.mask.png')
        variant = {'body':look.age,'hair':look.hair_style,'top':look.top,'item':look.item}[kind]
        metadata = {**asdict(look),'schema_version':1,'template_version':2,'id':identifier,'kind':kind,'body_type':look.age,'direction':look.facing,'frame':look.state,'scale':1,'variant':variant,'image':f'{identifier}.png','mask':f'{identifier}.mask.png','operations':['declarative_spec','raster_4x','ink05','lanczos','template_body_support_envelope'],'spec_hash':SPEC_HASH,'anchors':{'grip':spec.GRIPS[look.facing]},'grip':spec.GRIPS[look.facing],'head_box':([32,21,52,43] if look.state=='sleep' else [21,32,43,52])}
        (folder/f'{identifier}.json').write_text(json.dumps(metadata,indent=2)+'\n')
        inventory.append(f'{kind.upper()}/{identifier}.json')
    (destination/'INVENTORY.json').write_text(json.dumps(inventory,indent=2)+'\n')
    (destination/'manifest.json').write_text(json.dumps({'schema_version':1,'route':'P','require_minimum':True,'directions':list(spec.DIRECTIONS),'layers':inventory},indent=2)+'\n')
    sizes = [(destination/path).with_suffix('.png').stat().st_size for path in inventory]
    (destination/'GENERATION.json').write_text(json.dumps({'layers':len(inventory),'wall_seconds':time.perf_counter()-started,'png_bytes_total':sum(sizes),'png_bytes_min':min(sizes),'png_bytes_max':max(sizes),'spec_hash':SPEC_HASH,'renderer':'Pillow + numpy'},indent=2)+'\n')

if __name__=='__main__':
    generate(Path(sys.argv[1]))
