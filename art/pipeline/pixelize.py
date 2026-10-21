"""Exact RGBA block-majority reduction without resampling or colour averaging."""
import numpy as np
from PIL import Image

from .models import PipelineError, Size


def pixelize(image: Image.Image, target_size: Size) -> Image.Image:
    """Partition source into integer-bounded blocks; ties choose RGBA lexicographically.

    Non-divisible crops use floor-partitioned blocks, each source pixel exactly
    once. Upscaling is rejected. Transparent RGB is canonicalized before voting.
    """
    width, height = target_size
    if min(target_size) < 1 or width > image.width or height > image.height:
        raise PipelineError('Block reduction requires positive targets no larger than source')
    source = np.array(image.convert('RGBA'))
    source[source[:, :, 3] == 0] = 0
    output = np.zeros((height, width, 4), dtype=np.uint8)
    for y in range(height):
        for x in range(width):
            block = source[y * image.height // height:(y + 1) * image.height // height,
                           x * image.width // width:(x + 1) * image.width // width]
            colors, counts = np.unique(block.reshape(-1, 4), axis=0, return_counts=True)
            output[y, x] = colors[int(counts.argmax())]
    return Image.fromarray(output)
