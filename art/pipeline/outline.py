"""A one-pixel inner silhouette boundary, without dilation or footprint growth."""
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image

OUTLINE: Final = (20, 18, 16)

def silhouette_boundary(mask: NDArray[np.bool_]) -> NDArray[np.bool_]:
    """Return four-neighbour inner boundary, including image-edge foreground."""
    padded = np.pad(mask, 1)
    interior = (padded[1:-1, :-2] & padded[1:-1, 2:] &
                padded[:-2, 1:-1] & padded[2:, 1:-1])
    return mask & ~interior

def enforce_outline(image: Image.Image) -> Image.Image:
    """Overwrite only the silhouette boundary; repeated application is identical.

    Interior dark pixels are retained: their material/line semantics cannot be
    inferred from colour alone. This does not certify pre-existing thick lines.
    """
    pixels = np.array(image.convert('RGBA'))
    boundary = silhouette_boundary(pixels[:, :, 3] > 0)
    pixels[boundary, :3] = OUTLINE
    return Image.fromarray(pixels)
