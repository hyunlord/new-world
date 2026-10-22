# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by check_set failure board renderer
"""Localized failure overlays in source pixel coordinates."""
import numpy as np
from PIL import Image, ImageDraw
from .check_layer import Finding
from .layer_schema import Layer
from .structure import GEOMETRY, Box, components, holes


def marked_preview(layer: Layer, findings: list[Finding]) -> Image.Image:
    """Draw geometry boxes and offending raster regions before preview resizing."""
    with Image.open(layer.image) as source:
        preview = source.convert('RGBA')
    draw = ImageDraw.Draw(preview)
    s = layer.scale
    g = GEOMETRY[layer.body_type]
    alpha = np.array(preview)[:,:,3] > 0
    def rectangle(box: Box) -> None:
        x0,y0,x1,y1 = box
        draw.rectangle((x0*s,y0*s,x1*s-1,y1*s-1),outline='#ff2020',width=max(1,s))
    for finding in findings:
        match finding.rule:
            case 'feet':
                rectangle((0,86,64,91))
            case 'head_alignment':
                rectangle(g.head)
            case 'coverage' | 'overlay_extent' | 'body_through_top':
                rectangle(g.torso)
            case 'fixed_upper':
                rectangle(g.head)
                rectangle(g.torso)
            case 'hand_alignment':
                hands = g.east if layer.direction == 'E' else g.south
                for x,y in hands:
                    rectangle((x-4,y-4,x+5,y+5))
            case 'sleep_axis' | 'sleep_center':
                draw.line((46*s,36*s,50*s,36*s),fill='#ff2020',width=s)
                draw.line((48*s,34*s,48*s,38*s),fill='#ff2020',width=s)
            case 'sleep_head':
                rectangle(layer.head_box or (0,0,48,64))
            case 'holes' | 'connectivity':
                groups = holes(alpha,s) if finding.rule == 'holes' else components(alpha)
                for group in groups:
                    xs,ys = zip(*group,strict=True)
                    draw.rectangle((min(xs),min(ys),max(xs),max(ys)),outline='#ff2020',width=max(1,s))
            case 'mask_completeness' | 'mask_semantics':
                with Image.open(layer.mask) as source:
                    mask = np.array(source)
                if mask.shape == alpha.shape:
                    bad = alpha & ((mask < 1)|(mask > 5)) | (~alpha & (mask != 0))
                    if finding.rule == 'mask_semantics':
                        bad |= alpha
                    for y,x in zip(*np.nonzero(bad),strict=True):
                        draw.point((int(x),int(y)),fill='#ff2020')
            case _:
                draw.rectangle((0,0,preview.width-1,preview.height-1),outline='#ff2020',width=max(1,s))
    return preview
