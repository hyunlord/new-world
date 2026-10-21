"""Remove near-magenta backgrounds while retaining reusable crop coordinates."""
from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .models import Box, PipelineError, Size


@dataclass(frozen=True, slots=True)
class IngestResult:
    image: Image.Image
    source_size: Size
    bbox: Box
    keyed_pixels: int

def ingest(image: Image.Image, crop_box: Box | None = None) -> IngestResult:
    """Key RGB within 48 per channel of magenta; reuse crop_box for aligned pairs.

    Existing alpha is preserved. Transparent RGB is canonical zero. A supplied
    crop must contain all visible pixels, so drift outside it is rejected.
    """
    pixels: NDArray[np.uint8] = np.array(image.convert('RGBA'))
    distance: NDArray[np.int16] = np.abs(pixels[:, :, :3].astype(np.int16) - (255, 0, 255))
    key = np.empty(distance.shape[:2], dtype=np.bool_)
    _ = np.all(np.less_equal(distance, 48), axis=2, out=key)
    pixels[key] = 0
    pixels[pixels[:, :, 3] == 0] = 0
    keyed = Image.fromarray(pixels)
    bbox = keyed.getbbox()
    if bbox is None:
        raise PipelineError('Image contains no foreground after magenta keying')
    selected = bbox if crop_box is None else crop_box
    left, top, right, bottom = selected
    if not (0 <= left < right <= image.width and 0 <= top < bottom <= image.height):
        raise PipelineError('Crop is outside the source image')
    if not (left <= bbox[0] and top <= bbox[1] and right >= bbox[2] and bottom >= bbox[3]):
        raise PipelineError('Shared crop would discard foreground; reject alignment drift')
    return IngestResult(keyed.crop(selected), image.size, selected, int(np.count_nonzero(key)))
