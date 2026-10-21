"""Frequency-selected material subpalettes; explicit membership, no hue inference."""
from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .color import rgb_to_lab, vector_length
from .models import RGB, PipelineError


@dataclass(frozen=True, slots=True)
class MaterialSubset:
    name: str
    members: tuple[RGB, ...]
    maximum: int = 4
    retained: tuple[RGB, ...] | None = None

@dataclass(frozen=True, slots=True)
class MaterialRemap:
    source: RGB
    target: RGB
    pixels: int
    delta_e: float

@dataclass(frozen=True, slots=True)
class MaterialResult:
    image: Image.Image
    retained: tuple[RGB, ...]
    frequencies: tuple[tuple[RGB, int], ...]
    remaps: tuple[MaterialRemap, ...]
    changed_pixels: int

def reduce_material(image: Image.Image, subset: MaterialSubset, region: NDArray[np.bool_] | None = None) -> MaterialResult:
    """Keep up to maximum most-used member RGBs; nearest Lab maps the rest.

    An explicit retained set can preserve reviewed material extremes.
    Frequency ties sort RGB lexicographically; nearest-distance ties use retained
    frequency order. Non-member pixels and all alpha bytes remain untouched.
    Membership must be declared from reviewed material pixels, never guessed
    from hue by this function. A low source colour count is not padded or invented.
    """
    if not subset.members or subset.maximum < 1 or len(set(subset.members)) != len(subset.members):
        raise PipelineError('Material needs distinct explicit members and positive maximum')
    pixels: NDArray[np.uint8] = np.array(image.convert('RGBA'))
    visible: NDArray[np.bool_] = pixels[:, :, 3] > 0
    if region is not None:
        if region.shape != visible.shape:
            raise PipelineError('Material region must match image dimensions')
        visible &= region
    frequencies: list[tuple[RGB, int]] = []
    for color in subset.members:
        matches = np.empty(visible.shape, dtype=np.bool_)
        _ = np.all(np.equal(pixels[:, :, :3], color), axis=2, out=matches)
        count = int(np.count_nonzero(visible & matches))
        if count:
            frequencies.append((color, count))
    frequencies.sort(key=lambda entry: (-entry[1], entry[0]))
    if not frequencies:
        raise PipelineError('Declared material has no visible pixels')
    retained = tuple(color for color, _ in frequencies[:subset.maximum]) if subset.retained is None else subset.retained
    observed = {color for color, _ in frequencies}
    if not retained or len(retained) > subset.maximum or len(set(retained)) != len(retained) or not set(retained) <= observed:
        raise PipelineError('Explicit retained colours must be distinct observed material RGBs within maximum')
    target_lab = rgb_to_lab(np.asarray(retained, dtype=np.uint8))
    remaps: list[MaterialRemap] = []
    output = pixels.copy()
    for source, count in frequencies:
        if source in retained:
            continue
        source_lab = rgb_to_lab(np.asarray(source, dtype=np.uint8))
        distances = vector_length(target_lab - source_lab, axis=1)
        selected = int(distances.argmin())
        target = retained[selected]
        matches = np.empty(visible.shape, dtype=np.bool_)
        _ = np.all(np.equal(pixels[:, :, :3], source), axis=2, out=matches)
        mask = visible & matches
        output[mask, :3] = target
        remaps.append(MaterialRemap(source, target, count, float(np.min(distances))))
    return MaterialResult(Image.fromarray(output), retained, tuple(frequencies),
                          tuple(remaps), sum(remap.pixels for remap in remaps))
