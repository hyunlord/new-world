# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: python3 art/pipeline3/build_guides.py
"""Generate geometric reference guides, never character artwork."""
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

ROOT: Final = Path(__file__).resolve().parents[1] / "character/template_guides"
CYAN: Final = (0, 220, 230, 255)


def main() -> None:
    """Write the nine exact authoring guides from the registered template."""
    ROOT.mkdir(parents=True, exist_ok=True)
    specs = (
        ("adult", (20, 32, 44, 50), (18, 50, 46, 70), 66, ((44, 64), (26, 62)), 52),
        ("child", (23, 46, 41, 60), (21, 60, 43, 74), 70, ((41, 68), (27, 66)), 60),
        ("elder", (20, 36, 44, 54), (18, 54, 46, 72), 68, ((44, 66), (26, 64)), 55),
    )
    for body, head, torso, hand_y, east, shoulder_y in specs:
        south = ((17, hand_y), (47, hand_y)) if body == "child" else ((14, hand_y), (50, hand_y))
        for direction, hands in (("S", south), ("E", east), ("N", south[::-1])):
            with Image.new("RGBA", (256, 384)) as guide:
                draw = ImageDraw.Draw(guide)
                for box in (head, torso):
                    draw.rectangle(tuple(v * 4 for v in box), outline=CYAN, width=1)
                for y in (88, torso[3]):
                    draw.line((0, y * 4, 255, y * 4), fill=CYAN, width=1)
                for x, y in (*hands, (32, shoulder_y)):
                    draw.line((x * 4 - 5, y * 4, x * 4 + 5, y * 4), fill=CYAN)
                    draw.line((x * 4, y * 4 - 5, x * 4, y * 4 + 5), fill=CYAN)
                guide.save(ROOT / f"{body}_{direction}.png")


if __name__ == "__main__":
    main()
