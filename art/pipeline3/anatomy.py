# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline3.check_layer
"""Contact and projected anatomy measurements; never repair source pixels."""
import numpy as np
from PIL import Image
from .layer_schema import Layer
from .structure import GEOMETRY, BoolImage, Pixels, box_region, components


def extent(region: BoolImage) -> tuple[int, int]:
    """Measure occupied bounding box, returning zero for missing evidence."""
    y, x = np.nonzero(region)
    return (int(x.max()-x.min()+1), int(y.max()-y.min()+1)) if len(x) else (0, 0)


def longest_run(row: BoolImage) -> int:
    """Measure contiguous occupied pixels, never bridge transparent gaps."""
    edges = np.diff(np.r_[False, row, False].astype(np.int8))
    starts, ends = np.flatnonzero(edges == 1), np.flatnonzero(edges == -1)
    return int(np.max(ends-starts)) if len(starts) else 0


def minimum_run(region: BoolImage) -> int:
    """Minimum occupied scanline width across the dominant axis, excluding caps."""
    width, height = extent(region)
    if not width or not height:
        return 0
    rows = region if height >= width else region.T
    occupied = rows[np.any(rows, axis=1)]
    interior = occupied[1:-1] if len(occupied) > 2 else occupied
    lengths = []
    for row in interior:
        lengths.append(longest_run(row))
    return min(lengths)


def projected_parts(layer: Layer, alpha: Pixels) -> tuple[BoolImage, ...]:
    """Use fixed template windows for historical assets without anatomical masks."""
    g, s = GEOMETRY[layer.body_type], layer.scale
    visible = alpha >= 128
    anchors = g.east if layer.direction == 'E' else g.south
    head = visible & box_region(alpha.shape, g.head, s)
    hands = tuple(visible & box_region(alpha.shape, (x-6,y-4,x+7,y+5),s) for x,y in anchors)
    feet = tuple(visible & box_region(alpha.shape, (x0,82,x1,89),s) for x0,x1 in ((8,32),(32,56)))
    arms = tuple(visible & box_region(alpha.shape, (x-6,g.torso[1]+2,x+7,y-4),s) for x,y in anchors)
    legs = tuple(visible & box_region(alpha.shape, (x0,g.hip,x1,83),s) for x0,x1 in ((8,32),(32,56)))
    return (head, *hands, *feet, *arms, *legs)


def anatomy_findings(layer: Layer, alpha: Pixels) -> tuple[tuple[str, str], ...]:
    """Return failed rules; N/A overlays and sleep contact do not become passes."""
    if layer.kind != 'body':
        return ()
    findings: list[tuple[str,str]] = []
    s = layer.scale
    opaque = alpha == 255
    if layer.frame != 'sleep':
        ys, _ = np.nonzero(opaque)
        bottom = (int(ys.max())+1)/s if len(ys) else 0
        if not 86 <= bottom <= 88:
            findings.append(('contact_ok', f'opaque bottom={bottom:g}; required86..88'))
        yy, xx = np.indices(alpha.shape)
        rx, ry = (11,3) if layer.body_type == 'child' else (14,4)
        ellipse = ((xx+.5)/s-32)**2/rx**2 + ((yy+.5)/s-88)**2/ry**2 <= 1
        overlap = opaque & ellipse
        # Three game pixels of contiguous horizontal overlap at some scanline.
        if max((longest_run(row) for row in overlap), default=0) < 3*s:
            findings.append(('contact_ok','foot/shadow projected overlap <3px; center=(32,88)'))
    if layer.anatomical_mask is None:
        if layer.frame == 'sleep':
            if layer.head_box is None:
                return tuple(findings + [('limb_ok','sleep head measurement missing')])
            region = (alpha >= 128) & box_region(alpha.shape,layer.head_box,s)
            width,height = extent(region)
            x0,y0,x1,y1 = layer.head_box
            if width*height < .7*(x1-x0)*(y1-y0)*s*s:
                findings.append(('limb_ok','sleep head alpha bbox <70% declared head box'))
            # Upright anchor ROIs cannot establish lying or hidden limb sizes.
            findings.append(('limb_ok', 'sleep limbs NOT_VERIFIED: anatomical mask required'))
            return tuple(findings)
        parts = projected_parts(layer, alpha)
    else:
        try:
            with Image.open(layer.anatomical_mask) as image:
                labels = np.array(image)
                valid = image.mode == 'L' and labels.shape == alpha.shape
        except OSError as error:
            return tuple(findings + [('limb_ok',f'anatomical evidence unreadable: {error}')])
        if not valid:
            return tuple(findings + [('limb_ok','anatomical mask requires same-size L image')])
        if np.any((labels > 0) & (alpha < 128)) or np.any(labels > 10):
            findings.append(('limb_ok','anatomical labels outside visible body or invalid ids'))
        parts = tuple((labels == index) & (alpha >= 128) for index in range(1,10))
    head,*rest = parts
    if layer.anatomical_mask is not None and len(components(head)) != 1:
        findings.append(('limb_ok', 'anatomical head must be one connected visible shape'))
    g = GEOMETRY[layer.body_type]
    x0,y0,x1,y1 = layer.head_box if layer.frame == 'sleep' and layer.head_box else g.head
    hw,hh = extent(head)
    if hw*hh < .7*(x1-x0)*(y1-y0)*s*s:
        findings.append(('limb_ok',f'head alpha bbox area={hw*hh/s**2:g}; required70% of {(x1-x0)*(y1-y0)}'))
    for index, region in enumerate(rest):
        width,height = extent(region)
        if index < 2:
            size, minimum, name = width/s, 6, 'hand'
        elif index < 4:
            size, minimum, name = max(width,height)/s, 8, 'foot'
        else:
            size, minimum, name = minimum_run(region)/s, 4, 'limb'
        if size < minimum or len(components(region)) != 1:
            findings.append(('limb_ok',f'{name}{index} measured={size:g}px; required>={minimum}, one connected visible shape'))
    return tuple(findings)
