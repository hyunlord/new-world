"""Required room plans and every corner/junction shown at literal nearest4x."""

from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from PIL import Image, ImageDraw

Point = tuple[int, int]


@dataclass(frozen=True, slots=True)
class Plan:
    """A fixed occupancy and optional straight-run doorway."""

    name: str
    cells: frozenset[Point]
    ew: Point | None
    state: Literal["closed", "open"]


@dataclass(frozen=True, slots=True)
class JoinView:
    """An actual room-space join, not an isolated tile approximation."""

    name: str
    cell: Point
    mask: int
    images: Mapping[str, Image.Image]


def plans() -> tuple[Plan, ...]:
    """Keep prior rooms and add one cross-partition room with all four Ts and X."""
    rectangle = frozenset(
        (x, y) for y in range(4) for x in range(5) if x in (0, 4) or y in (0, 3)
    )
    shaped = frozenset(
        {
            *((x, 0) for x in range(5)),
            *((0, y) for y in range(4)),
            *((x, 3) for x in range(3)),
            (2, 2),
            (2, 1),
            (3, 1),
            (4, 1),
        }
    )
    crossing = rectangle | frozenset((2, y) for y in range(4))
    crossing |= frozenset((x, 2) for x in range(5))
    return (
        Plan("5x4", rectangle, (2, 3), "closed"),
        Plan("L", shaped, (1, 3), "open"),
        Plan("T+X", crossing, None, "closed"),
    )


def write_zooms(views: tuple[JoinView, ...], output: Path) -> None:
    """Show each full join plus16px neighbour context with no smoothing."""
    folder = output / "join_zooms_4x"
    folder.mkdir()
    for index, view in enumerate(views):
        board = Image.new("RGB", (768, 420), "#d0ccbb")
        draw = ImageDraw.Draw(board)
        x, y = view.cell
        for col, material in enumerate(("shipped log", "native stone")):
            crop = view.images[material].crop(
                (x * 64, y * 64, x * 64 + 96, y * 64 + 96)
            )
            board.paste(
                crop.resize((384, 384), Image.Resampling.NEAREST), (col * 384, 30)
            )
            draw.text(
                (col * 384 + 5, 8),
                f"{view.name} {view.cell} mask{view.mask:02} / {material} / 4x",
                fill="#302923",
            )
        board.save(
            folder
            / f"{index:02}_{view.name.replace('+', '_')}_{x}_{y}_mask{view.mask:02}.png"
        )
