# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow"]
# ///
# How to run: imported by art.batch_c3a.export_items
"""Contact sheets preserve native item size and show actual alpha composites."""

from pathlib import Path
from textwrap import fill

from PIL import Image, ImageDraw


def boards(names: list[str], out: Path, data: Path) -> None:
    """Show old references and candidates at native and half scale on two grounds."""
    entries = [
        ("REF linen", data / "reference_linen.png"),
        ("REF iron axe", data / "reference_iron.png"),
    ]
    entries.extend((name, out / (name + ".png")) for name in names)
    for size in (64, 32):
        for background in ("#586349", "#777777"):
            sheet = Image.new(
                "RGB", (720, 36 + 104 * ((len(entries) + 5) // 6)), background
            )
            draw = ImageDraw.Draw(sheet)
            draw.text(
                (10, 10),
                f"C3A item comparison {size}px",
                fill="white",
            )
            for index, (name, source) in enumerate(entries):
                with Image.open(source) as opened:
                    tile = opened.convert("RGBA").resize(
                        (size, size), Image.Resampling.LANCZOS
                    )
                x, y = 10 + index % 6 * 120, 35 + index // 6 * 104
                sheet.paste(tile, (x + (64 - size) // 2, y), tile)
                draw.multiline_text((x, y + 70), fill(name, width=18), fill="white")
            target = out / "boards" / f"items_{size}_{background[1:]}.png"
            target.parent.mkdir(parents=True, exist_ok=True)
            sheet.save(target)
            external = data / "sheets" / target.name
            external.parent.mkdir(parents=True, exist_ok=True)
            _ = external.write_bytes(target.read_bytes())
