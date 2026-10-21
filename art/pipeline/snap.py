"""Pad declared boxes without scaling or dropping image pixels."""
from dataclasses import dataclass

from PIL import Image

from .models import PipelineError, Size


@dataclass(frozen=True, slots=True)
class SnapResult:
    image: Image.Image
    offset: Size

def snap(image: Image.Image, box: Size) -> SnapResult:
    """Centre the entire source canvas; callers share this offset for all layers."""
    if min(box) < 1 or image.width > box[0] or image.height > box[1]:
        raise PipelineError('Source exceeds the declared box')
    offset = ((box[0] - image.width) // 2, (box[1] - image.height) // 2)
    result = Image.new('RGBA', box)
    result.paste(image.convert('RGBA'), offset)
    return SnapResult(result, offset)
