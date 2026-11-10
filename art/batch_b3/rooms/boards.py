# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: uv run --with pillow --with pydantic python -m art.batch_b3.rooms.boards RENDERED_DIR BOARD_DIR
"""Native-scale lifecycle comparison sheets beside the existing56px adult."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.batch_b3.rooms.ghost import ghost
from art.batch_b3.rooms.schema import Recipes

ROOT: Final = Path(__file__).resolve().parents[3]
STATES: Final = ("blueprint", "foundation", "half_built", "built", "damaged", "ruined")


def main() -> int:
    """Compose readable sheets without changing any candidate pixel scale."""
    rendered, output = Path(sys.argv[1]), Path(sys.argv[2])
    output.mkdir(parents=True, exist_ok=True)
    recipes = Recipes.model_validate_json(
        (ROOT / "art/batch_b3/rooms/recipes.json").read_bytes()
    )
    families = tuple(dict.fromkeys(room.family for room in recipes.rooms))
    with Image.open(
        ROOT / "art/character/route_i/adult_F_hide_wrap_bun/S/idle.png"
    ) as person:
        adult = person.convert("RGBA")
        adult = adult.crop(adult.getchannel("A").getbbox())
    for family in families:
        board = Image.new("RGBA", (2280, 370), "#dedbcf")
        draw = ImageDraw.Draw(board)
        for column, state in enumerate(STATES):
            source_state = "built" if state == "blueprint" else state
            with Image.open(
                rendered / f"{family}_{source_state}" / "preview.png"
            ) as source:
                frame = (
                    ghost(source) if state == "blueprint" else source.convert("RGBA")
                )
            draw.text((column * 380 + 12, 8), f"{family} / {state}", fill="#30291f")
            draw.text(
                (column * 380 + 12, 25),
                "64px grid | layered recipe | native 1x",
                fill="#51483a",
            )
            board.alpha_composite(frame, (column * 380 + 12, 60))
            board.alpha_composite(
                adult, (column * 380 + 346, 60 + frame.height - adult.height)
            )
        board.convert("RGB").save(output / f"{family}_1x.png")
        board.convert("RGB").resize((4560, 740), Image.Resampling.NEAREST).save(
            output / f"{family}_2x.png"
        )
    _ = sys.stdout.write(f"Wrote {len(families)} native/2x board pairs\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
