"""Native-sprite palette preservation without synthesizing missing accents."""
from __future__ import annotations

from collections.abc import Mapping
from typing import TypedDict

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from palette_tools import Color, rgb_to_lab


class AddedColor(TypedDict):
    role: str
    color: str
    source: str


class PaletteMetadata(TypedDict):
    scene_count: int
    sprite_median_cut_count: int
    final_count: int
    added_colors: list[AddedColor]
    accent_ramps: dict[str, list[str]]
    missing_accents: list[str]
    classification: str


def hex_color(color: Color) -> str:
    """Serialize a color for reports."""
    return '#%02X%02X%02X' % color


def opaque(image: Image.Image) -> NDArray[np.uint8]:
    """Collect opaque texels, excluding transparent RGB values."""
    rgba = np.asarray(image.convert('RGBA'))
    return rgba[..., :3][rgba[..., 3] > 0]


def hsv(rgb: NDArray[np.generic]) -> NDArray[np.float64]:
    """Vectorized HSV, with hue measured in degrees."""
    values = np.asarray(rgb, dtype=np.float64) / 255
    maximum, minimum = values.max(axis=-1), values.min(axis=-1)
    delta = maximum - minimum
    safe = np.where(delta > 0, delta, 1)
    r, g, b = np.moveaxis(values, -1, 0)
    hue = np.select([maximum == r, maximum == g],
                    [((g-b) / safe) % 6, (b-r) / safe + 2], default=(r-g) / safe + 4) * 60
    hue = np.where(delta == 0, 0, hue)
    saturation = delta / np.where(maximum > 0, maximum, 1)
    return np.stack([hue, saturation, maximum], axis=-1)


def skin_mask(rgb: NDArray[np.generic]) -> NDArray[np.bool_]:
    """Conservative warm skin-color proxy; does not identify anatomical regions."""
    h, s, v = np.moveaxis(hsv(rgb), -1, 0)
    return (h >= 12) & (h <= 48) & (s >= .15) & (s <= .65) & (v >= .35)


def merge_palette(colors: list[Color]) -> list[Color]:
    """Stable ΔE76 < 3 merge retaining a real input color, never a centroid."""
    result: list[Color] = []
    for color in colors:
        if not result or np.min(np.linalg.norm(rgb_to_lab(np.array(result)) -
                                               rgb_to_lab(np.array(color)), axis=-1)) >= 3:
            result.append(color)
    return result


def _ramp(candidates: NDArray[np.uint8], count: int) -> list[Color]:
    """Pick actual input colors nearest evenly spaced lightness quantiles."""
    if not len(candidates):
        return []
    unique = np.unique(candidates, axis=0)
    ordered = unique[np.argsort(rgb_to_lab(unique)[:, 0])]
    picked = [tuple(int(v) for v in ordered[i]) for i in
              np.rint(np.linspace(0, len(ordered)-1, count)).astype(int)]
    return merge_palette(picked)


def build_master(scene_palette: list[Color], natives: Mapping[str, Image.Image]
                 ) -> tuple[list[Color], PaletteMetadata]:
    """Merge scene and native median-cut colors, then preserve native accent ramps."""
    pixels = np.concatenate([opaque(image) for image in natives.values()], axis=0)
    if not len(pixels):
        raise RuntimeError('No opaque native texels for master palette')
    indexed = Image.fromarray(pixels.reshape(1, -1, 3)).quantize(
        colors=32, method=Image.Quantize.MEDIANCUT)
    raw_palette, counts = indexed.getpalette(), indexed.getcolors()
    if raw_palette is None or counts is None:
        raise RuntimeError('Median-cut returned no palette')
    sprite_palette: list[Color] = [tuple(raw_palette[index*3:index*3+3]) for _, index in counts]
    palette = merge_palette(scene_palette + sprite_palette)
    bushes = np.concatenate([opaque(image) for name, image in natives.items()
                              if 'berry_bush' in name], axis=0)
    actors = np.concatenate([opaque(image) for name, image in natives.items()
                              if 'villager' in name and 'child' not in name], axis=0)
    fire = opaque(next(image for name, image in natives.items() if 'campfire' in name))
    h, s, v = np.moveaxis(hsv(fire), -1, 0)
    reds = bushes[(bushes[:, 0] > 130) & (bushes[:, 1] < 90) & (bushes[:, 2] < 90)]
    ramps = {'berry_red': _ramp(reds, 2), 'skin': _ramp(actors[skin_mask(actors)], 3),
             'fire_orange': _ramp(fire[(h >= 15) & (h < 40) & (s >= .35) & (v >= .45)], 1),
             'fire_yellow': _ramp(fire[(h >= 40) & (h <= 65) & (s >= .25) & (v >= .6)], 1),
             'fire_hot_white': _ramp(fire[(v >= .85) & (s <= .25)], 1)}
    # The sheet has no water sprite: use the nearest real native texel to a cool highlight.
    native_unique = np.unique(pixels, axis=0)
    target = rgb_to_lab(np.array([190, 220, 230]))
    nearest = native_unique[np.argmin(np.linalg.norm(rgb_to_lab(native_unique)-target, axis=-1))]
    ramps['water_highlight'] = [tuple(int(value) for value in nearest)]
    additions: list[AddedColor] = []
    missing: list[str] = []
    expected = {'berry_red': 2, 'skin': 3, 'fire_orange': 1, 'fire_yellow': 1,
                'fire_hot_white': 1, 'water_highlight': 1}
    mapped: dict[str, list[str]] = {}
    for role, colors in ramps.items():
        if len(colors) < expected[role]:
            missing.append(role)
        mapped[role] = []
        for color in colors:
            distances = np.linalg.norm(rgb_to_lab(np.array(palette))-rgb_to_lab(np.array(color)), axis=-1)
            if float(distances.min()) >= 3:
                palette.append(color)
                additions.append({'role': role, 'color': hex_color(color), 'source': 'native opaque texel'})
                chosen = color
            else:
                chosen = palette[int(distances.argmin())]
            mapped[role].append(hex_color(chosen))
    # Skin ramp includes preserved intermediate palette shades matching actual adult skin candidates.
    actor_skin = actors[skin_mask(actors)]
    if len(actor_skin):
        labs = rgb_to_lab(actor_skin)
        mapped['skin'] = [hex_color(color) for color in palette if bool(skin_mask(np.array(color)))
                          and float(np.linalg.norm(labs-rgb_to_lab(np.array(color)), axis=-1).min()) <= 12]
    return palette, {'scene_count': len(scene_palette), 'sprite_median_cut_count': len(sprite_palette),
                     'final_count': len(palette), 'added_colors': additions, 'accent_ramps': mapped,
                     'missing_accents': missing,
                     'classification': 'Skin: HSV H12..48 S.15...65 V>=.35; palette shades within ΔE12 of native adult candidates. Color proxy can include clothing. Water: nearest native to #BEDCE6; no water sprite.'}


def quantize(image: Image.Image, palette: list[Color]) -> Image.Image:
    """Map opaque native texels to their nearest CIELAB master color."""
    rgba = np.array(image.convert('RGBA'))
    mask = rgba[..., 3] > 0
    if mask.any():
        labs = rgb_to_lab(rgba[..., :3][mask])
        distances = np.linalg.norm(labs[:, None]-rgb_to_lab(np.array(palette))[None, :], axis=-1)
        rgba[..., :3][mask] = np.array(palette, dtype=np.uint8)[distances.argmin(axis=1)]
    rgba[..., :3][~mask] = 0
    return Image.fromarray(rgba)
