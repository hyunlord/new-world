"""Deterministic shelf packing using the legacy tile rectangle schema."""
from collections.abc import Mapping, Sequence
from dataclasses import dataclass

from PIL import Image

from .models import Asset, AtlasMetadata, PipelineError


@dataclass(frozen=True, slots=True)
class SheetResult:
    image: Image.Image
    metadata: AtlasMetadata

# Four parameters retain the established packing API while adding independent
# legacy palette metadata; a config wrapper would unnecessarily break callers.
def pack_sheet(assets: Sequence[Asset], atlas_name: str = 'sheet.png', width: int = 512,
               *, palettes: Mapping[str, str | list[str]] | None = None) -> SheetResult:
    """Pack in name order; no alpha compositing that could change stored RGBA."""
    if not assets or width < 1:
        raise PipelineError('Packing needs assets and positive width')
    tiles: dict[str, list[int]] = {}
    x = y = row_height = 0
    for asset in sorted(assets, key=lambda item: item.name):
        if asset.name in tiles or asset.image.size != asset.box or asset.box[0] > width:
            raise PipelineError('Duplicate name, mismatched box, or asset wider than atlas')
        if x + asset.box[0] > width:
            x, y, row_height = 0, y + row_height, 0
        tiles[asset.name] = [x, y, *asset.box]
        x += asset.box[0]
        row_height = max(row_height, asset.box[1])
    image = Image.new('RGBA', (width, y + row_height))
    for asset in assets:
        image.paste(asset.image.convert('RGBA'), (tiles[asset.name][0], tiles[asset.name][1]))
    metadata: AtlasMetadata = {
        'atlas': atlas_name, 'size': list(image.size), 'tile_size': 32,
        'palettes': {} if palettes is None else dict(palettes), 'tiles': tiles,
        'boxes': {asset.name: list(asset.box) for asset in assets},
        'anchors': {asset.name: list(asset.anchor) for asset in assets},
        'kinds': {asset.name: asset.kind for asset in assets},
    }
    return SheetResult(image, metadata)
