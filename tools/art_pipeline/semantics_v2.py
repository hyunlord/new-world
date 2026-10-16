"""Content checks for recovered native sprites, evaluated before outlines."""
from __future__ import annotations

from collections.abc import Mapping
from typing import TypedDict

import numpy as np
from PIL import Image

from palette_tools import Color, rgb_to_lab
from palette_v2 import PaletteMetadata, hex_color, hsv, opaque


class SpriteMetrics(TypedDict):
    opaque_texels: int
    size: list[int]
    mean_delta_e: float
    palette_membership: float
    binary_alpha: bool
    silhouette_unchanged: bool


class SemanticMetrics(TypedDict):
    berry_red_counts: dict[str, int]
    fire_orange_yellow_texels: int
    adult_skin_ratios: dict[str, float]
    child_height: int
    adult_heights: dict[str, int]
    basket_box_area: int
    bush_box_areas: dict[str, int]
    mean_delta_e: float


class SemanticReport(TypedDict):
    passed: bool
    checks: dict[str, bool]
    metrics: SemanticMetrics
    sprites: dict[str, SpriteMetrics]
    definitions: dict[str, str]


def _height(image: Image.Image) -> int:
    bbox = image.getchannel('A').getbbox()
    return bbox[3]-bbox[1] if bbox else 0


def _area(image: Image.Image) -> int:
    bbox = image.getchannel('A').getbbox()
    return (bbox[2]-bbox[0])*(bbox[3]-bbox[1]) if bbox else 0


def evaluate(natives: Mapping[str, Image.Image], quantized: Mapping[str, Image.Image],
             palette: list[Color], palette_metadata: PaletteMetadata) -> SemanticReport:
    """Return independently auditable checks; never alter pixels to make a check pass."""
    palette_set = {hex_color(color) for color in palette}
    skin_set = set(palette_metadata['accent_ramps'].get('skin', []))
    sprites: dict[str, SpriteMetrics] = {}
    weighted_delta_e, total_opaque = 0.0, 0
    for name, image in quantized.items():
        native = np.asarray(natives[name].convert('RGBA'))
        final = np.asarray(image.convert('RGBA'))
        mask = native[..., 3] > 0
        same_shape = native.shape == final.shape
        same_silhouette = same_shape and bool(np.array_equal(mask, final[..., 3] > 0))
        delta_e = float(np.linalg.norm(rgb_to_lab(native[..., :3][mask]) -
                                       rgb_to_lab(final[..., :3][mask]), axis=-1).mean()) if same_shape and mask.any() else 1e9
        pixels = opaque(image)
        membership = sum(hex_color(tuple(int(c) for c in pixel)) in palette_set for pixel in pixels) / max(1, len(pixels))
        sprites[name] = {'opaque_texels': len(pixels), 'size': list(image.size),
                         'mean_delta_e': delta_e, 'palette_membership': membership,
                         'binary_alpha': bool(np.isin(final[..., 3], [0, 255]).all()),
                         'silhouette_unchanged': same_silhouette}
        weighted_delta_e += delta_e * int(mask.sum())
        total_opaque += int(mask.sum())
    berries: dict[str, int] = {}
    for stage in ('full', 'two', 'one', 'bare'):
        image = next(image for name, image in quantized.items() if name.endswith('berry_bush_'+stage))
        colors = opaque(image)
        berries[stage] = int(((colors[:, 0] > 130) & (colors[:, 1] < 90) & (colors[:, 2] < 90)).sum())
    fire = opaque(next(image for name, image in quantized.items() if 'campfire' in name))
    hue, saturation, value = np.moveaxis(hsv(fire), -1, 0)
    fire_count = int(((hue >= 15) & (hue <= 65) & (saturation >= .35) & (value >= .45)).sum())
    adults = {name: image for name, image in quantized.items() if 'villager' in name and 'child' not in name}
    ratios = {name: sum(hex_color(tuple(int(c) for c in pixel)) in skin_set for pixel in opaque(image)) /
              max(1, len(opaque(image))) for name, image in adults.items()}
    heights = {name: _height(image) for name, image in adults.items()}
    child_height = _height(next(image for name, image in quantized.items() if 'child' in name))
    basket_area = _area(next(image for name, image in quantized.items() if 'basket' in name))
    bush_areas = {name: _area(image) for name, image in quantized.items() if 'berry_bush' in name}
    mean_delta_e = weighted_delta_e / max(1, total_opaque)
    checks = {'berry_stages': berries['full'] > berries['two'] > berries['one'] > 0 and berries['bare'] == 0,
              'fire': fire_count >= 20, 'skin': bool(ratios) and all(ratio >= .12 for ratio in ratios.values()),
              'child_smaller': bool(heights) and all(child_height < height for height in heights.values()),
              'basket_smaller': all(basket_area < area for area in bush_areas.values()),
              'fidelity': mean_delta_e <= 6 and all(row['mean_delta_e'] <= 6 for row in sprites.values()),
              'palette_membership': all(row['palette_membership'] == 1 for row in sprites.values()),
              'binary_alpha': all(row['binary_alpha'] for row in sprites.values()),
              'accent_ramps': not palette_metadata['missing_accents']}
    return {'passed': all(checks.values()), 'checks': checks,
            'metrics': {'berry_red_counts': berries, 'fire_orange_yellow_texels': fire_count,
                        'adult_skin_ratios': ratios, 'child_height': child_height, 'adult_heights': heights,
                        'basket_box_area': basket_area, 'bush_box_areas': bush_areas,
                        'mean_delta_e': mean_delta_e}, 'sprites': sprites,
            'definitions': {'red': 'R>130, G<90, B<90', 'fire': 'HSV H15..65 degrees, S>=0.35, V>=0.45',
                            'skin': palette_metadata['classification'],
                            'spill': 'Silhouette changes are diagnostic because native spill removal is intentional. Lost opaque texels still contribute to fidelity against final RGB.',
                            'size': 'Opaque bounding-box height for child; opaque bounding-box area for basket.',
                            'fidelity': 'Opaque-texel weighted direction mean and every sprite mean must be <=6 ΔE76. Input native is pre-spill step 3; final is before outline.'}}
