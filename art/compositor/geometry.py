# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by python3 -m art.compositor.render.
"""World-to-pixel geometry and exact wall-neighbour connectivity."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final

WINDOW: Final = (1600, 900)
HUD_HEIGHT: Final = 112


@dataclass(frozen=True, slots=True)
class Viewport:
    camp: tuple[int, int]
    zoom: float

    @property
    def tile(self) -> int:
        return round(64 * self.zoom)

    def project(self, point: tuple[float, float]) -> tuple[int, int]:
        """Project tile centres into the HUD-excluded, camp-centred viewport."""
        x, y = point
        return (round(WINDOW[0] / 2 + (x - self.camp[0]) * self.tile),
                round((WINDOW[1] + HUD_HEIGHT) / 2 + (y - self.camp[1]) * self.tile))

    def visible(self, point: tuple[float, float], margin: int = 0) -> bool:
        x, y = self.project(point)
        return -margin <= x < WINDOW[0] + margin and HUD_HEIGHT - margin <= y < WINDOW[1] + margin

    def tile_bounds(self, dimensions: tuple[int, int]) -> tuple[range, range]:
        """Include one extra tile for foreground overhang at the visible edges."""
        half_width = WINDOW[0] // (2 * self.tile) + 2
        half_height = (WINDOW[1] - HUD_HEIGHT) // (2 * self.tile) + 2
        return (range(max(0, self.camp[0] - half_width), min(dimensions[0], self.camp[0] + half_width + 1)),
                range(max(0, self.camp[1] - half_height), min(dimensions[1], self.camp[1] + half_height + 1)))


def wall_mask(point: tuple[int, int], walls: frozenset[tuple[int, int]]) -> int:
    """N=1, E=2, S=4, W=8; no diagonal or renderer-specific cases."""
    x, y = point
    return sum(bit for bit, neighbour in ((1, (x, y - 1)), (2, (x + 1, y)),
                                         (4, (x, y + 1)), (8, (x - 1, y)))
               if neighbour in walls)


def perimeter(size: tuple[int, int]) -> frozenset[tuple[int, int]]:
    width, height = size
    return frozenset((x, y) for y in range(height) for x in range(width)
                     if x in (0, width - 1) or y in (0, height - 1))
