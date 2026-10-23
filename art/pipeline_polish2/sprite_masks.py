# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_frame_diagnostic --revision <new-revision>
"""Seeded Lab segmentation within recorded anatomy regions, without pixel repair."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

import numpy as np
from PIL import Image

from art.pipeline_polish2.sprite_colour import (
    BoolPixels,
    BytePixels,
    FloatPixels,
    IntPixels,
    lab_triple,
    mean_colour,
    nearest_label,
    rgb_to_lab,
    squared_distance,
)
from art.pipeline_polish2.sprite_landmarks import FrameGuide, locate_head
from art.pipeline_polish2.sprite_regions import (
    facial_hair_support,
    local_skin_shadow,
    polygon_union,
    semantic_hair_support,
    source_hair_reference,
)

SEEDS: Final = np.array(((242, 165, 96), (185, 108, 54), (91, 58, 33),
                        (28, 18, 10), (130, 89, 44), (222, 181, 135), (116, 112, 99), (124, 102, 71)), dtype=np.float64)


@dataclass(frozen=True, slots=True)
class Box:
    left: int
    top: int
    right: int
    bottom: int

    def mask(self, shape: tuple[int, int]) -> BoolPixels:
        region = np.zeros(shape, dtype=np.bool_)
        region[max(0, self.top):self.bottom, max(0, self.left):self.right] = True
        return region


@dataclass(frozen=True, slots=True)
class Anatomy:
    body: Box
    head: Box
    face: Box
    forearms: tuple[Box, Box]
    skin_support: BoolPixels


@dataclass(frozen=True, slots=True)
class Masks:
    skin: BoolPixels
    hair: BoolPixels
    outline: BoolPixels
    alpha_excluded: BoolPixels
    excluded: BoolPixels
    anatomy: Anatomy
    skin_lab: tuple[float, float, float]
    hair_lab: tuple[float, float, float]
    hair_silhouette: BoolPixels
    classifier_hair_silhouette: BoolPixels
    recovered_skin: BoolPixels
    recovered_hair: BoolPixels


def cluster(lab: FloatPixels, support: BoolPixels) -> tuple[FloatPixels, FloatPixels]:
    """Eight fixed semantic seeds; bounded Lloyd updates retain seed identities."""
    initial = rgb_to_lab(SEEDS)
    centres = initial.copy()
    pixels = lab[support]
    for _ in range(8):
        delta: FloatPixels = squared_distance(pixels[:, None, :] - centres[None, :, :])
        labels: IntPixels = nearest_label(delta)
        for index in range(len(centres)):
            members = pixels[labels == index]
            if len(members):
                # Anchor regularization prevents small dark garments stealing skin.
                centres[index] = .75 * mean_colour(members) + .25 * initial[index]
    distances: FloatPixels = squared_distance(lab[:, :, None, :] - centres[None, None, :, :])
    return centres, distances



def landmark_box(rgba: BytePixels, search: Box, size: tuple[int, int]) -> Box:
    """Locate exposed anatomy from source red/yellow contrast, independently of Lab masks."""
    rgb = rgba[:, :, :3].astype(np.int16)
    warm = ((rgb[:, :, 0] > 130) & (rgb[:, :, 0] - rgb[:, :, 1] > 55)
            & (rgb[:, :, 1] - rgb[:, :, 2] > 30) & (rgba[:, :, 3] >= 128))
    yy, xx = np.nonzero(warm & search.mask(warm.shape))
    cx = round(float(np.median(xx))) if len(xx) else (search.left + search.right) // 2
    cy = round(float(np.median(yy))) if len(yy) else (search.top + search.bottom) // 2
    width, height = size
    return Box(cx - width // 2, cy - height // 2, cx - width // 2 + width, cy - height // 2 + height)


def anatomy(source: Image.Image, guide: FrameGuide) -> Anatomy:
    """Derive bounded body/head geometry from TEMPLATE_V3 anchors and source alpha."""
    child, bare_chest = guide.child, guide.bare_chest
    rgba = np.asarray(source.convert("RGBA"))
    opaque = rgba[:, :, 3] >= 128
    height = 40 if child else 56
    sleeping = source.width > source.height
    if sleeping:
        yy, xx = np.nonzero(opaque)
        body = Box(int(np.min(xx)), int(np.min(yy)), int(np.max(xx)) + 1, int(np.max(yy)) + 1)
        span = body.right - body.left
        head = Box(body.left, body.top, body.left + round(span * .30), body.bottom)
        face = Box(head.left + 2, head.top + 2, head.right - 2, head.bottom - 2)
        arms = (Box(head.right, body.top, head.right + 4, body.bottom), Box(head.right + 4, body.top, head.right + 8, body.bottom))
        return Anatomy(body, head, face, arms, body.mask(opaque.shape))
    top = 104 - height
    # Head search is centre-constrained so tools above a pose cannot shift anatomy.
    central = opaque[max(0, top - 6):top + round(height * .35), 30:50]
    yy, xx = np.nonzero(central)
    head_top = max(top - 4, int(np.min(yy)) + top - 6)
    centre = round(float(np.median(xx))) + 30
    head_width = 12 if child else 15
    head_height = 17 if child else 19
    head = Box(centre - head_width // 2, head_top, centre + (head_width + 1) // 2, head_top + head_height)
    if guide.idle_reference is not None:
        ref = anatomy(guide.idle_reference, FrameGuide(child, bare_chest)).head
        hx, hy = locate_head(source, guide.idle_reference, (ref.left, ref.top, ref.right, ref.bottom))
        head = Box(hx, hy, hx + ref.right - ref.left, hy + ref.bottom - ref.top)
        head_top, head_height = head.top, head.bottom - head.top
        centre = (head.left + head.right) // 2
    body = Box(16 if not child else 23, top - 6, 64 if not child else 57, 106)
    face = landmark_box(rgba, Box(head.left + 2, head_top + round(head_height * .35), head.right - 1, head_top + round(head_height * .70)), (3, 4) if child else (4, 4))
    arm_y = top + round(height * (.58 if child else .45))
    arm_bottom = top + round(height * (.80 if child else .68))
    arm_size = (2, 5) if child else (3, 8)
    arms = (landmark_box(rgba, Box(body.left, arm_y, centre - 1, arm_bottom), arm_size),
            landmark_box(rgba, Box(centre + 1, arm_y, body.right, arm_bottom), arm_size))
    if guide.annotation is not None:
        face = Box(*guide.annotation.face)
        arms = (Box(*guide.annotation.forearms[0]), Box(*guide.annotation.forearms[1]))
    if guide.envelope is not None:
        points = [point for polygon in guide.envelope.all_hair_envelopes for point in polygon]
        head = Box(min(face.left, min(point[0] for point in points)),
                   min(face.top, min(point[1] for point in points)),
                   max(face.right, max(point[0] for point in points) + 1),
                   max(face.bottom, max(point[1] for point in points) + 1))
    support = body.mask(opaque.shape)
    # The garment core is excluded before clustering; face, arms, legs stay available.
    garment_half = 6 if child else 9
    garment_top = top + round(height * .53) if bare_chest else max(head.bottom, top + round(height * .35))
    waist = top + round(height * (.65 if child else .60))
    garment_bottom = top + round(height * (.84 if child else .79))
    yy, xx = np.indices(opaque.shape, dtype=np.float64)
    lower = np.clip((garment_bottom - yy) / max(1, garment_bottom - waist), .15, 1)
    upper = np.clip((yy - garment_top) / max(1, waist - garment_top), 0, 1)
    taper = np.where(yy < waist, .55 + .45 * upper, lower)
    distances = np.empty(opaque.shape, dtype=np.float64)
    limits = np.empty(opaque.shape, dtype=np.float64)
    inside = np.empty(opaque.shape, dtype=np.bool_)
    _ = np.abs(xx - centre, out=distances)
    _ = np.ceil(garment_half * taper, out=limits)
    _ = np.less_equal(distances, limits, out=inside)
    garment = (yy >= garment_top) & (yy < garment_bottom) & inside
    support &= ~garment
    # The independent warm landmarks can locate a forearm overlapping the coarse
    # torso box in a narrow child or turned pose. Skin classification still applies.
    if guide.pose in ("idle", "walk_a"):
        support |= arms[0].mask(opaque.shape) | arms[1].mask(opaque.shape)
    return Anatomy(body, head, face, arms, support)


def detect(source: Image.Image, guide: FrameGuide) -> Masks:
    """Classify source colours once; semantic colour exclusions take precedence."""
    rgba = np.asarray(source.convert("RGBA"))
    lab = rgb_to_lab(rgba[:, :, :3].astype(np.float64))
    bounds = anatomy(source, guide)
    opaque = rgba[:, :, 3] >= 128
    centres, distances = cluster(lab, opaque & bounds.body.mask(opaque.shape))
    labels: IntPixels = nearest_label(distances)
    # Low-light strokes and antialiased edge pixels are immutable.
    outline = (lab[:, :, 0] <= 18) | (rgba[:, :, 3] < 128)
    excluded = np.greater_equal(labels, 4) | np.equal(labels, 3)
    allowed_skin = bounds.skin_support.copy()
    allowed_hair = np.ones(opaque.shape, dtype=np.bool_)
    yy, xx = np.indices(opaque.shape, dtype=np.float64)
    if guide.pose == "carry":
        # The carried horizontal bundle is never a skin/hair source. Hands partly
        # hidden by it remain untinted instead of recolouring wood.
        prop = (yy < bounds.head.bottom) & ((xx < bounds.head.left + 2) | (xx >= bounds.head.right - 2))
        rgb = rgba[:, :, :3].astype(np.int16)
        bright_skin = (rgb[:, :, 0] >= 210) & (rgb[:, :, 0] - rgb[:, :, 1] >= 60) & (rgb[:, :, 1] - rgb[:, :, 2] >= 40)
        allowed_skin &= ~prop | bright_skin
        allowed_hair &= ~prop
    if guide.pose == "work_a":
        prop = yy < bounds.head.top
        allowed_skin &= ~prop
        allowed_hair &= ~prop
    selected_hair_support = allowed_hair.copy()
    if guide.envelope is not None:
        allowed_skin = bounds.body.mask(opaque.shape)
        if guide.envelope.skin_supports:
            allowed_skin &= polygon_union(opaque.shape, guide.envelope.skin_supports)
        garment = polygon_union(opaque.shape, guide.envelope.garment_exclusions + guide.envelope.prop_exclusions)
        allowed_skin &= ~garment
        allowed_skin &= ~facial_hair_support(guide.envelope, opaque.shape)
        selected_hair_support &= semantic_hair_support(guide.envelope, opaque.shape)

    if guide.annotation is not None:
        for region in guide.annotation.clothing_regions:
            clothing = Box(*region).mask(opaque.shape)
            allowed_skin &= ~clothing
            selected_hair_support &= ~clothing
        if guide.annotation.hair_region is not None:
            selected_hair_support &= Box(*guide.annotation.hair_region).mask(opaque.shape)
        if guide.annotation.hair_regions:
            envelope = np.zeros(opaque.shape, dtype=np.bool_)
            for region in guide.annotation.hair_regions:
                envelope |= Box(*region).mask(opaque.shape)
            selected_hair_support &= envelope
    recovered = np.zeros(opaque.shape, dtype=np.bool_)
    if guide.refine_skin:
        local = bounds.face.mask(opaque.shape)
        for arm in bounds.forearms:
            local |= arm.mask(opaque.shape)
        if guide.envelope is not None and guide.envelope.skin_supports:
            local |= polygon_union(opaque.shape, guide.envelope.skin_supports)
        recovered = local_skin_shadow(rgba, lab) & np.greater_equal(labels, 4) & local & allowed_skin & opaque & ~outline
        if guide.envelope is not None and guide.envelope.skin_supports:
            declared_skin = polygon_union(opaque.shape, guide.envelope.skin_supports)
            recovered |= np.equal(labels, 5) & declared_skin & bounds.face.mask(opaque.shape) & allowed_skin & opaque & ~outline
    excluded &= ~recovered
    skin = ((np.equal(labels, 0) | np.equal(labels, 1)) & allowed_skin & opaque & ~outline) | recovered
    head_bottom = bounds.head.bottom if source.width > source.height or guide.envelope is not None else bounds.head.top + round((bounds.head.bottom - bounds.head.top) * .70)
    head_support = Box(bounds.head.left - 4, bounds.head.top - 4, bounds.head.right + 4, head_bottom).mask(opaque.shape)
    classifier_hair_silhouette = np.equal(labels, 2) & head_support & opaque & selected_hair_support
    hair = classifier_hair_silhouette & ~outline & ~skin
    recovered_hair = np.zeros(opaque.shape, dtype=np.bool_)
    if guide.refine_hair and guide.envelope is not None:
        rgb = rgba[:, :, :3].astype(np.int16)
        brown = (lab[:, :, 0] <= 52) & (rgb[:, :, 0] - rgb[:, :, 1] < 55)
        recovered_hair = (brown & selected_hair_support & head_support & opaque & ~outline & ~skin
                          & ~bounds.face.mask(opaque.shape) & ~hair)
        hair |= recovered_hair
        excluded &= ~recovered_hair
    if guide.fixed_masks is not None:
        skin, hair = guide.fixed_masks
    # This source-space reference is independent of the Lloyd labels and includes
    # dark ink. It remains a conservative proxy until the visual audit validates it.
    rgb = rgba[:, :, :3].astype(np.int16)
    hair_silhouette = ((lab[:, :, 0] <= 48) & (rgb[:, :, 0] - rgb[:, :, 1] < 55)
                       & head_support & opaque & allowed_hair)
    if guide.envelope is not None:
        hair_silhouette = source_hair_reference(rgba, lab, guide.envelope)
    skin_pixels = lab[skin]
    hair_pixels = lab[hair]
    skin_centre: FloatPixels = mean_colour(skin_pixels) if len(skin_pixels) else np.take(centres, 0, axis=0)
    hair_centre: FloatPixels = mean_colour(hair_pixels) if len(hair_pixels) else np.take(centres, 2, axis=0)
    return Masks(skin, hair, outline, rgba[:, :, 3] < 128, excluded, bounds,
                 lab_triple(skin_centre), lab_triple(hair_centre), hair_silhouette, classifier_hair_silhouette, recovered, recovered_hair)
