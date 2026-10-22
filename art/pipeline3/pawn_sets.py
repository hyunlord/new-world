# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline3.check_set
"""Pawn inventory and registered layer composition checks."""
from itertools import product
from dataclasses import replace
import numpy as np
from PIL import Image
from .check_layer import Finding, Result, read_pixels
from .layer_schema import Layer
from .set_checks import Combination
from .structure import Pixels, holes

FRAMES = ('idle', 'walk_a', 'walk_b', 'work_a', 'work_b', 'carry', 'sleep')


def registered(pixels: Pixels, body: Layer) -> Pixels:
    """Apply the specified pawn motion, rotating sleep with its head left."""
    if body.frame == 'sleep':
        return np.rot90(pixels)
    dx = ( -2 if body.direction == 'N' else 2) if body.frame in ('work_a', 'work_b') else 0
    dy = {'walk_a': -1, 'walk_b': 1}.get(body.frame, 0)
    dx, dy = dx * body.scale, dy * body.scale
    result = np.zeros_like(pixels)
    height, width = pixels.shape[:2]
    source_x, source_y = max(0, -dx), max(0, -dy)
    target_x, target_y = max(0, dx), max(0, dy)
    copy_width, copy_height = width - abs(dx), height - abs(dy)
    if copy_width > 0 and copy_height > 0:
        result[target_y:target_y + copy_height, target_x:target_x + copy_width] = pixels[source_y:source_y + copy_height, source_x:source_x + copy_width]
    return result


def place_overlay(pixels: Pixels, body: Layer, kind: str) -> Pixels:
    """Use the generator's declared sex/age placement before frame registration."""
    from art.pipeline4.pawn_gen import PawnLook, age_overlay
    look = PawnLook(sex=body.sex, age=body.body_type, facing=body.direction, state=body.frame)
    scaled = age_overlay(Image.fromarray(pixels), look, kind)
    return registered(np.array(scaled), body)


def top_occlusion(body_pixels: Pixels, top_pixels: Pixels, body: Layer) -> bool:
    """Cloth must hide every opaque actual capsule pixel in its upper70 percent."""
    band = np.zeros((96 * body.scale, 64 * body.scale, 4), dtype=np.uint8)
    band[54 * body.scale:78 * body.scale, :, 3] = 255
    placed_band = registered(band, body)[:, :, 3] > 0
    return bool(np.any(placed_band & (top_pixels[:, :, 3] < body_pixels[:, :, 3])))


def place_prop(pixels: Pixels, body: Layer, item: Layer) -> tuple[Pixels, bool]:
    """Use the same walk sway, carry anchor and pose placement as lab playback."""
    from art.pipeline4.pawn_gen import PawnLook, item_placement
    look = PawnLook(sex=body.sex, age=body.body_type, facing=body.direction,
                    state=body.frame, item=item.variant)
    placed, intact = item_placement(Image.fromarray(pixels), look)
    return np.array(placed), intact


def inventory(layers: list[Layer], directions: list[str]) -> list[Result]:
    """Require the full factorized adult inventory without counting combinations."""
    found: list[Finding] = []
    if not {'S', 'E'}.issubset(directions) or not set(directions).issubset({'S', 'E', 'N'}):
        found.append(Finding('inventory', 'S/E required; N optional'))
    if len({layer.id for layer in layers}) != len(layers):
        found.append(Finding('inventory', 'duplicate ids'))
    if any(layer.template_version != 2 for layer in layers):
        found.append(Finding('inventory', 'route P requires template_version2'))
    expected = set(product(('M', 'F'), range(4), directions, FRAMES))
    bodies = [layer for layer in layers if layer.kind == 'body' and layer.body_type == 'adult']
    actual = [(layer.sex, layer.skin_tone, layer.direction, layer.frame) for layer in bodies]
    if set(actual) != expected or len(actual) != len(expected):
        found.append(Finding('inventory', f'adult bodies expected{len(expected)}, unique{len(set(actual))}'))
    for kind, variants, colors in [('hair', ('none', 'short', 'long_tied', 'bun', 'braid', 'grey_thin'), range(4)),
                                    ('top', ('none', 'hide_wrap', 'fur_cloak', 'woven_tunic'), range(2))]:
        expected_overlays = set(product(variants, colors, directions))
        overlays = [layer for layer in layers if layer.kind == kind]
        actual_overlays = [(layer.variant, layer.hair_color if kind == 'hair' else layer.top_palette, layer.direction) for layer in overlays]
        if set(actual_overlays) != expected_overlays or len(actual_overlays) != len(expected_overlays):
            found.append(Finding('inventory', f'{kind} variant/color/direction inventory incomplete or duplicated'))
    expected_items = set(product(('none', 'stone_axe', 'log_bundle', 'basket'), directions, ('idle', 'work_a', 'work_b')))
    actual_items = [(layer.variant, layer.direction, layer.frame) for layer in layers if layer.kind == 'item']
    if set(actual_items) != expected_items or len(actual_items) != len(expected_items):
        found.append(Finding('inventory', 'item variant/direction/pose inventory incomplete or duplicated'))
    return [Result('minimum_set', not found, tuple(found))]


def fixed_overlays(layers: list[Layer]) -> list[Result]:
    """Every idle overlay must fit every compatible registered body frame."""
    results: list[Result] = []
    bodies_all = [layer for layer in layers if layer.kind == 'body']
    for body in bodies_all:
        idle = next((layer for layer in bodies_all if layer.frame == 'idle' and
                     (layer.body_type, layer.sex, layer.skin_tone, layer.direction) ==
                     (body.body_type, body.sex, body.skin_tone, body.direction)), None)
        found: list[Finding] = []
        if idle is None:
            found.append(Finding('fixed_upper', 'missing matching idle body'))
        else:
            reference, _ = read_pixels(idle)
            expected = registered(reference, body)
            actual, _ = read_pixels(body)
            if expected.shape != actual.shape or not np.array_equal(expected, actual):
                found.append(Finding('fixed_upper', 'body differs from registered idle pixels'))
            if np.count_nonzero(expected[:, :, 3]) != np.count_nonzero(reference[:, :, 3]):
                found.append(Finding('clipping', 'body registration loses visible pixels'))
        results.append(Result(body.id, not found, tuple(found)))
    for overlay in [layer for layer in layers if layer.kind in ('top', 'hair')]:
        found: list[Finding] = []
        if overlay.frame != 'idle':
            found.append(Finding('fixed_overlays', 'overlay must come from idle'))
        pixels, _ = read_pixels(overlay)
        bodies = [layer for layer in layers if layer.kind == 'body' and layer.direction == overlay.direction ]
        if not bodies:
            found.append(Finding('fixed_overlays', 'no compatible bodies'))
        for body in bodies:
            body_pixels, _ = read_pixels(body)
            placed = place_overlay(pixels, body, overlay.kind)
            unregistered = place_overlay(pixels, replace(body, frame='idle'), overlay.kind)
            if np.count_nonzero(placed[:, :, 3]) != np.count_nonzero(unregistered[:, :, 3]):
                found.append(Finding('clipping', f'overlay registration clips: {body.id}'))

            if placed.shape != body_pixels.shape or np.any((placed[:, :, 3] > 0) & (body_pixels[:, :, 3] == 0)):
                found.append(Finding('fixed_overlays', f'overlay outside body alpha: {body.id}'))
            elif overlay.kind == 'top' and overlay.variant != 'none' and top_occlusion(body_pixels, placed, body):
                found.append(Finding('body_through_top', f'upper capsule visible through top: {body.id}'))
        results.append(Result(overlay.id, not found, tuple(found)))
    return results


def combinations(layers: list[Layer]) -> list[Combination]:
    """Sample 48 deterministic spread combinations including sleep and all facings."""
    candidates: list[tuple[Layer, list[Layer], list[Layer], list[Layer]]] = []
    for body in sorted((layer for layer in layers if layer.kind == 'body'), key=lambda layer: layer.id):
        compatible = [layer for layer in layers if layer.direction == body.direction]
        tops = [layer for layer in compatible if layer.kind == 'top']
        hairs = [layer for layer in compatible if layer.kind == 'hair']
        pose = body.frame if body.frame in ('work_a', 'work_b') else 'idle'
        items = [layer for layer in compatible if layer.kind == 'item' and layer.frame == pose]
        if tops and hairs and items:
            candidates.append((body, tops, hairs, items))
    if not candidates:
        return []
    results: list[Combination] = []
    seen: set[tuple[str, ...]] = set()
    for index in range(48):
        body, tops, hairs, items = candidates[(index * 17) % len(candidates)]
        chosen = [body, tops[index % len(tops)], hairs[(index * 5) % len(hairs)], items[(index * 7) % len(items)]]
        ids = tuple(layer.id for layer in chosen)
        if ids in seen:
            continue
        seen.add(ids)
        arrays = [read_pixels(layer)[0] for layer in chosen]
        arrays[1:3] = [place_overlay(pixels, body, layer.kind) for pixels, layer in zip(arrays[1:3], chosen[1:3], strict=True)]
        arrays[3], item_intact = place_prop(arrays[3], body, chosen[3])
        found: list[Finding] = []
        if not item_intact:
            found.append(Finding('clipping', 'registered composition loses visible pixels'))
        if any(pixels.shape != arrays[0].shape for pixels in arrays):
            found.append(Finding('canvas', 'composition canvas mismatch'))
        else:
            if chosen[1].variant != 'none' and top_occlusion(arrays[0], arrays[1], body):
                found.append(Finding('body_through_top', 'opaque body visible through top in upper70 percent'))
            overlap = np.count_nonzero((arrays[0][:, :, 3] > 0) & (arrays[3][:, :, 3] > 0)) / body.scale ** 2
            if chosen[3].variant != 'none' and overlap < 2:
                found.append(Finding('item_contact', f'body/item overlap{overlap}<2px'))
            union = np.logical_or.reduce([pixels[:, :, 3] > 0 for pixels in arrays])
            if holes(union, body.scale):
                found.append(Finding('union_holes', 'composed hole exceeds12px squared'))
        results.append(Combination(tuple(layer.id for layer in chosen), not found, tuple(found), 'pawn_sample48_with_sleep'))
    return results
