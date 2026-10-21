"""Registered single-frame layer extraction without trim, recenter or pixel repair."""
from collections.abc import Sequence
from dataclasses import dataclass

import numpy as np
from PIL import Image

from .ingest import ingest
from .layer_extract import LayerResult, extract_layer
from .models import RGB, Box, PipelineError, Size
from .outline import enforce_outline
from .palette import PaletteResult, map_palette
from .pixelize import pixelize


@dataclass(frozen=True, slots=True)
class SingleFrameResult:
    base: Image.Image
    variant: Image.Image
    layer: LayerResult
    outline_context: Image.Image
    source_size: Size
    changed_pixels: int
    reduced_changed_pixels: int
    base_palette: PaletteResult
    variant_palette: PaletteResult
    method: str = 'whole-square floor-partition RGBA block majority to 32x32; no alignment correction'


def prepare_reference(base: Image.Image) -> Image.Image:
    """Place the exact frame at (0,0) on magenta 32-square, then nearest 32x."""
    if base.size not in ((32, 32), (32, 16)):
        raise PipelineError('Single-frame base must be 32x32 or 32x16')
    canvas = Image.new('RGBA', (32, 32), (255, 0, 255, 255))
    canvas.alpha_composite(base.convert('RGBA'), (0, 0))
    return canvas.resize((1024, 1024), Image.Resampling.NEAREST)


def process_single_variant(
    base: Image.Image,
    raw: Image.Image,
    palette: Sequence[RGB],
    allowed_region: Box | None = None,
) -> SingleFrameResult:
    """Compare registered inputs under one palette and an optional declared ROI.

    The four inputs are independently required: registered body, generated image,
    shared colour contract, and optional spatial acceptance boundary. The ROI is
    checked before quantization and again after outlining; it is not a repair mask.
    """
    if base.size not in ((32, 32), (32, 16)):
        raise PipelineError('Single-frame base must be 32x32 or 32x16')
    if raw.width != raw.height or raw.width < 32:
        raise PipelineError('Generated reference edit must remain square and at least 32px')
    source = ingest(raw, (0, 0, raw.width, raw.height)).image
    if base.height == 16 and source.crop((0, raw.height // 2, raw.width, raw.height)).getbbox():
        raise PipelineError('Sleep crop would discard generated foreground, including subpixel noise')
    reduced = pixelize(source, (32, 32)).crop((0, 0, 32, base.height))
    original = np.array(base.convert('RGBA'), dtype=np.uint8)
    original[original[:, :, 3] == 0] = 0
    reduced_pixels = np.array(reduced, dtype=np.uint8)
    raw_changes = np.empty((base.height, 32), dtype=np.bool_)
    _ = np.any(np.not_equal(original, reduced_pixels), axis=2, out=raw_changes)
    allowed = np.ones((base.height, 32), dtype=np.bool_)
    if allowed_region is not None:
        left, top, right, bottom = allowed_region
        if not (0 <= left < right <= 32 and 0 <= top < bottom <= base.height):
            raise PipelineError('Allowed change region must lie within the base frame')
        allowed[:] = False
        allowed[top:bottom, left:right] = True
        if np.any(raw_changes & ~allowed):
            raise PipelineError('Generated pixels changed outside the predeclared region')
    mapped_base = map_palette(Image.fromarray(original), palette)
    mapped_variant = map_palette(reduced, palette)
    prepared_base = enforce_outline(mapped_base.image)
    prepared_variant = enforce_outline(mapped_variant.image)
    changed = np.empty((base.height, 32), dtype=np.bool_)
    _ = np.any(np.not_equal(np.array(prepared_base), np.array(prepared_variant)), axis=2, out=changed)
    if np.any(changed & ~allowed):
        raise PipelineError('Outline changes extend outside the predeclared region')
    layer = extract_layer(prepared_base, prepared_variant)
    extracted = np.array(layer.image)
    if np.any((extracted[:, :, 3] > 0) & ~allowed):
        raise PipelineError('Difference-mask closing extends outside the predeclared region')
    return SingleFrameResult(
        prepared_base, prepared_variant, layer, prepared_base.copy(), raw.size,
        int(np.count_nonzero(changed)), int(np.count_nonzero(raw_changes)), mapped_base, mapped_variant,
    )
