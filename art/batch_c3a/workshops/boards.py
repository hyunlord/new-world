"""Compare independently layered workshop previews at native and doubled scale."""

from pathlib import Path

from PIL import Image, ImageDraw

from art.pipeline_struct.normalize import normalize_sprite


def contacts(output: Path, root: Path) -> None:
    """The adult reference is56px high; room previews retain native256px width."""
    person = root / "art/character/route_i/adult_M_hide_wrap_short/S/idle.png"
    with Image.open(person) as opened:
        reference = normalize_sprite(opened, 56).image
    for scale in (1, 2):
        board = Image.new("RGBA", (768 * scale, 1240 * scale), "#576047")
        draw = ImageDraw.Draw(board)
        for index, folder in enumerate(sorted((output / "rooms").iterdir())):
            with Image.open(folder / "preview.png") as opened:
                image = opened.resize(
                    (256 * scale, 280 * scale), Image.Resampling.NEAREST
                )
            x, y = (index % 3) * 256 * scale, (index // 3) * 310 * scale
            board.alpha_composite(image, (x, y))
            adult = reference.resize(
                (reference.width * scale, reference.height * scale),
                Image.Resampling.NEAREST,
            )
            board.alpha_composite(adult, (x + 210 * scale, y + 210 * scale))
            draw.text((x, y + 282 * scale), folder.name.replace("_", " "), fill="white")
        board.save(output / f"contact_{scale}x.png")


def station_contact(output: Path, root: Path) -> None:
    """Compare generated stations with the adult and the shipped A2 forge."""
    sources = [root / "art/character/route_i/adult_M_hide_wrap_short/S/idle.png"]
    sources += sorted((output / "stations").glob("*.png"))
    sources += [Path("/Users/rexxa/work/art-batch/C3A/reference_forge.png")]
    for scale in (1, 2):
        board = Image.new("RGBA", (520 * scale, 110 * scale), "#576047")
        draw = ImageDraw.Draw(board)
        for index, path in enumerate(sources):
            with Image.open(path) as opened:
                sprite = (
                    normalize_sprite(opened, 56).image
                    if index == 0
                    else opened.convert("RGBA")
                )
            sprite = sprite.resize(
                (sprite.width * scale, sprite.height * scale), Image.Resampling.NEAREST
            )
            board.alpha_composite(
                sprite, ((index * 120 + 15) * scale, 72 * scale - sprite.height)
            )
            draw.text(((index * 120 + 5) * scale, 82 * scale), path.stem, fill="white")
        board.save(output / f"stations_{scale}x.png")
