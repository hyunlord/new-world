"""Extract registered paper-doll differences without synthesizing layer colours."""
from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .color import rgb_to_lab, vector_length
from .models import PipelineError


@dataclass(frozen=True, slots=True)
class LayerResult:
    image: Image.Image
    difference_pixels: int
    closed_pixels: int

def close_mask(mask: NDArray[np.bool_]) -> NDArray[np.bool_]:
    """One 3x3 dilation then erosion with a padded exterior, including edge pixels."""
    extended = np.pad(mask, 2)
    patches = np.lib.stride_tricks.sliding_window_view(extended, (3, 3))
    dilated = patches.any(axis=(-1, -2))
    return np.asarray(np.lib.stride_tricks.sliding_window_view(dilated, (3, 3)).all(axis=(-1, -2)), dtype=np.bool_)

def extract_layer(base: Image.Image, variant: Image.Image) -> LayerResult:
    """Require registered frames; reject erased body pixels as likely pose drift.

    Colour changes can also be geometry drift: visual registration review is
    mandatory. A close may fill only pixels present in the real variant.
    """
    if base.size != variant.size:
        raise PipelineError('Layer and mannequin must share exact dimensions and transform')
    body: NDArray[np.uint8] = np.array(base.convert('RGBA'))
    worn: NDArray[np.uint8] = np.array(variant.convert('RGBA'))
    body_mask, worn_mask = body[:, :, 3] > 0, worn[:, :, 3] > 0
    if np.any(body_mask & ~worn_mask):
        raise PipelineError('Variant erases mannequin pixels; reject geometry drift')
    delta: NDArray[np.float64] = vector_length(rgb_to_lab(body[:, :, :3]) - rgb_to_lab(worn[:, :, :3]), axis=2)
    difference: NDArray[np.bool_] = worn_mask & ((delta > 6) | np.not_equal(body[:, :, 3], worn[:, :, 3]))
    closed: NDArray[np.bool_] = close_mask(difference) & worn_mask
    output: NDArray[np.uint8] = np.zeros_like(worn)
    output[closed] = worn[closed]
    return LayerResult(Image.fromarray(output), int(np.count_nonzero(difference)), int(np.count_nonzero(closed & ~difference)))
