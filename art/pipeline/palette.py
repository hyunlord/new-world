"""Nearest-master-colour mapping in CIE Lab for every visible alpha level."""
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .color import rgb_to_lab, vector_length
from .models import (
    RGB,
    PipelineError,
    decode_json,
    json_array,
    json_object,
    json_string,
)


@dataclass(frozen=True, slots=True)
class PaletteResult:
    image: Image.Image
    moved_over_8: int
    max_delta_e: float

def load_palette(path: Path) -> tuple[RGB, ...]:
    """Parse the palette JSON boundary and reject malformed or duplicate colours."""
    data = json_object(decode_json(path.read_text()))
    colors: list[RGB] = []
    for entry in json_array(data['colors']):
        value = json_string(json_object(entry)['hex']).lstrip('#')
        if len(value) != 6:
            raise PipelineError('Palette hex must have six digits')
        colors.append((int(value[:2], 16), int(value[2:4], 16), int(value[4:], 16)))
    if not colors or len(set(colors)) != len(colors):
        raise PipelineError('Palette must contain distinct colours')
    return tuple(colors)

def map_palette(image: Image.Image, colors: Sequence[RGB]) -> PaletteResult:
    """Preserve alpha and use palette order to break equal-distance ties."""
    if not colors:
        raise PipelineError('Palette is empty')
    pixels: NDArray[np.uint8] = np.array(image.convert('RGBA'))
    visible: NDArray[np.bool_] = pixels[:, :, 3] > 0
    pixels[~visible] = 0
    palette: NDArray[np.uint8] = np.asarray(colors, dtype=np.uint8)
    rgb, inverse = np.unique(pixels[visible, :3], axis=0, return_inverse=True)
    if len(rgb) == 0:
        return PaletteResult(Image.fromarray(pixels), 0, 0.0)
    distances: NDArray[np.float64] = vector_length(rgb_to_lab(rgb)[:, None] - rgb_to_lab(palette)[None], axis=2)
    nearest = np.empty(len(rgb), dtype=np.intp)
    _ = np.argmin(distances, axis=1, out=nearest)
    movement: NDArray[np.float64] = distances[np.arange(len(rgb)), nearest][inverse]
    pixels[visible, :3] = palette[nearest][inverse]
    return PaletteResult(Image.fromarray(pixels), int(np.count_nonzero(movement > 8)), float(np.max(movement)))
