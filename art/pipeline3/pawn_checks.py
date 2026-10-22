# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline3.check_layer
"""Template v2 rules; no limbs, hands or anatomical masks."""
import numpy as np
from .layer_schema import Layer
from .structure import Pixels, box_region


def pawn_findings(layer: Layer, pixels: Pixels, mask: Pixels) -> list[tuple[str, str]]:
    """Check contact, head location and explicit region semantics."""
    found: list[tuple[str, str]] = []
    alpha = pixels[:, :, 3]
    visible = alpha > 0
    if layer.variant == 'none' and np.any(visible):
        found.append(('empty_variant', 'none layer must have zero alpha everywhere'))
    if layer.kind == 'body' and np.any(visible):
        upright = np.rot90(visible, -1) if layer.frame == 'sleep' else visible
        dx = (-2 if layer.direction == 'N' else 2) if layer.frame in ('work_a', 'work_b') else 0
        dy = {'walk_a': -1, 'walk_b': 1}.get(layer.frame, 0)
        half_width, half_height = (9, 8) if layer.body_type == 'child' else (11, 10)
        center_x, center_y = 32 + dx, 42 + dy
        yy, xx = np.indices(upright.shape, dtype=float)
        xx, yy = (xx + .5) / layer.scale, (yy + .5) / layer.scale
        core = ((xx - center_x) / (half_width - 1)) ** 2 + ((yy - center_y) / (half_height - 1)) ** 2 <= 1
        head_y, head_x = np.nonzero(upright & (yy < center_y + half_height - 1))
        if not len(head_x) or not np.all(upright[core]):
            found.append(('head_alignment', 'template ellipse head core missing'))
        elif (head_x.min() / layer.scale < center_x - half_width - 3 or
              head_x.max() / layer.scale > center_x + half_width + 3 or
              abs(head_y.min() / layer.scale - (center_y - half_height)) > 3):
            found.append(('head_alignment', 'head alpha outside template ellipse bounds'))
        if layer.frame == 'sleep':
            ys, xs = np.nonzero(visible)
            if np.ptp(xs) <= np.ptp(ys):
                found.append(('sleep_axis', 'sleep must be horizontal'))
            if layer.head_box is None or not (0 <= layer.head_box[0] < layer.head_box[2] <= 96 and 0 <= layer.head_box[1] < layer.head_box[3] <= 64) or (layer.head_box[0] + layer.head_box[2]) / 2 >= 48:
                found.append(('sleep_head', 'declare head box in left half'))
            elif not np.any(box_region(mask.shape, layer.head_box, layer.scale) & visible & (mask == 1)):
                found.append(('sleep_head', 'left head box contains no skin'))
        else:
            ys, _ = np.nonzero(alpha == 255)
            bottom = int(ys.max()) // layer.scale if len(ys) else -1
            if not 86 <= bottom <= 88:
                found.append(('contact_ok', f'lowest opaque pixel y={bottom}; expected86..88'))
        if np.any(visible & (mask != 1)):
            found.append(('mask_semantics', 'pawn body pixels require skin region1'))
    if layer.kind == 'top' and layer.variant != 'none':
        yy, xx = np.indices(alpha.shape, dtype=float)
        xx, yy = (xx + .5) / layer.scale, (yy + .5) / layer.scale
        cap = ((xx - 32) ** 2 + (yy - 67) ** 2 < 12 ** 2) & (yy < 67)
        trunk = (np.abs(xx - 32) < 12) & (yy >= 67)
        upper = (cap | trunk) & (yy >= 55) & (yy < 78)
        if np.any(upper & (alpha == 0)):
            found.append(('coverage', 'top must cover canonical capsule upper70 percent interior; per-set checks alpha occlusion'))
    if layer.kind == 'top' and np.any(visible & (mask != 3) & (mask != 4)):
        found.append(('mask_semantics', 'top pixels require cloth region3/4'))
    if layer.kind in ('hair', 'top') and layer.frame != 'idle':
        found.append(('fixed_overlays', 'pawn overlays must be authored from idle only'))
    if layer.kind in ('item', 'carry') and np.any(visible):
        anchor = layer.grip if layer.kind == 'item' else layer.shoulder
        if anchor is None:
            found.append(('anchor', 'item needs grip; carry needs shoulder'))
        else:
            x, y = (v * layer.scale for v in anchor)
            if not (0 <= y < alpha.shape[0] and 0 <= x < alpha.shape[1]) or not visible[y, x]:
                found.append(('anchor', 'anchor must land on visible alpha'))
    return found
