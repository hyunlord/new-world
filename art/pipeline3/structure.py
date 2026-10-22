# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline3.check_layer
"""Template geometry and scale-aware connected-region measurements."""
from collections import deque
from dataclasses import dataclass
from typing import Final, Literal
import numpy as np
from numpy.typing import NDArray

BoolImage = NDArray[np.bool_]
Pixels = NDArray[np.uint8]
Body = Literal['adult', 'child', 'elder']
Point = tuple[int, int]
Box = tuple[int, int, int, int]
FRAMES: Final = ('idle', 'walk_a', 'walk_b', 'work_a', 'work_b')

@dataclass(frozen=True, slots=True)
class Geometry:
    head: Box
    torso: Box
    hip: int
    south: tuple[Point, Point]
    east: tuple[Point, Point]
    shoulder: Point

GEOMETRY: Final = {
    'adult': Geometry((20,32,44,50),(18,50,46,70),70,((14,66),(50,66)),((44,64),(26,62)),(32,52)),
    'child': Geometry((23,46,41,60),(21,60,43,74),74,((17,70),(47,70)),((41,68),(27,66)),(32,60)),
    'elder': Geometry((20,36,44,54),(18,54,46,72),72,((14,68),(50,68)),((44,66),(26,64)),(32,55)),
}

def box_region(shape: tuple[int, ...], box: Box, scale: int) -> BoolImage:
    """Return half-open rectangular region, safely intersected with the canvas."""
    region = np.zeros(shape[:2], dtype=bool)
    x0,y0,x1,y1 = (v * scale for v in box)
    region[max(0,y0):max(0,y1), max(0,x0):max(0,x1)] = True
    return region

def components(binary: BoolImage, diagonal: bool = True) -> list[list[Point]]:
    """Enumerate foreground components without scipy or a pixel-count shortcut."""
    remaining = binary.copy()
    height,width = remaining.shape
    groups: list[list[Point]] = []
    offsets = [(x,y) for y in (-1,0,1) for x in (-1,0,1) if (x or y) and (diagonal or x*y == 0)]
    for y,x in zip(*np.nonzero(binary), strict=True):
        if not remaining[y,x]:
            continue
        queue = deque([(int(x),int(y))])
        remaining[y,x] = False
        group: list[Point] = []
        while queue:
            px,py = queue.popleft()
            group.append((px,py))
            for dx,dy in offsets:
                nx,ny = px+dx,py+dy
                if 0 <= nx < width and 0 <= ny < height and remaining[ny,nx]:
                    remaining[ny,nx] = False
                    queue.append((nx,ny))
        groups.append(group)
    return groups

def holes(alpha: BoolImage, scale: int) -> list[list[Point]]:
    """Use dual 4-connectivity for background; open leg gaps are not holes."""
    height,width = alpha.shape
    return [group for group in components(~alpha, False)
            if len(group) > 12 * scale * scale
            and not any(x in (0,width-1) or y in (0,height-1) for x,y in group)]
