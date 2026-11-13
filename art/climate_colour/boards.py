# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: python3 -m art.climate_colour.boards
"""Native before/control/after comparison pages grouped by proposed climate."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import ClassVar, Final

from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, TypeAdapter

from art.climate_colour.models import Draft, ShrubFamily
from art.climate_colour.references import GROUND_REFERENCES, ROOT, read, reference

DATA: Final = Path("/Users/rexxa/work/art-batch/R2")
BASE: Final = DATA / "baseline"
OUT: Final = ROOT / "art/climate_colour/boards"
BANDS: Final = ("harsh", "neutral", "mild", "unassigned")
GROUND_BANDS: Final = {
    "tundra": ("harsh",),
    "taiga_floor": ("harsh",),
    "steppe": ("harsh", "neutral"),
    "temperate_meadow": ("neutral",),
    "mediterranean_scrub": ("mild",),
    "marsh": BANDS[:3],
    "river_gravel": BANDS[:3],
    "highland_rock": BANDS[:3],
}
ROW_HEIGHT: Final = 228
PAGE_SIZE: Final = 8


class TreeMembership(BaseModel):
    """Read existing tree climate membership without inferred origin rules."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="ignore", frozen=True)
    climates: dict[str, tuple[str, ...]]


class GroundPanel(BaseModel):
    """Read only image provenance required by the contact sheet."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="ignore", frozen=True)
    id: str
    source: Path
    path: Path
    status: str


class Measurements(BaseModel):
    """Validate external ground comparison rows at the file boundary."""

    model_config: ClassVar[ConfigDict] = ConfigDict(extra="ignore", frozen=True)
    grounds: tuple[GroundPanel, ...]


@dataclass(frozen=True, slots=True)
class Panel:
    """One native-size comparison with explicit proposal membership."""

    id: str
    climates: tuple[str, ...]
    before: Image.Image
    control: Image.Image
    after: Image.Image
    status: str


def flora_panels() -> list[Panel]:
    """Keep B2 active and olive proposal rows visibly separate."""
    draft = Draft.model_validate_json(
        (BASE / "art/climate/art_import.draft.json").read_bytes(),
    )
    shrubs = TypeAdapter(list[ShrubFamily]).validate_json(
        (BASE / "art/climate/shrubs/catalog.json").read_bytes(),
    )
    trees = TreeMembership.model_validate_json(
        (BASE / "art/climate/trees/manifest.json").read_bytes(),
    )
    result: list[Panel] = []
    for row in (*draft.files, *draft.proposed_files):
        if row.lane == "shrubs":
            family, state = row.id.rsplit("_", 1)
            climates = next(item.climate for item in shrubs if item.id == family)
            row = row.model_copy(
                update={"species": family, "state": state, "climate": climates},
            )
        climates = row.climate or trees.climates.get(row.species, ("unassigned",))
        control, _, _ = reference(row)
        proposed = row.id in {item.id for item in draft.proposed_files}
        status = "OLIVE PROPOSAL / NOT ACTIVE" if proposed else "B2 ACTIVE CANDIDATE"
        result.append(
            Panel(
                row.id,
                climates,
                read(BASE / row.path),
                control,
                read(ROOT / row.path),
                status,
            ),
        )
    return result


def ground_panels() -> list[Panel]:
    """Show blocked grounds without promoting failed noncolour gates."""
    measurements = Measurements.model_validate_json(
        (DATA / "measurements.json").read_bytes(),
    )
    result: list[Panel] = []
    for row in measurements.grounds:
        name = GROUND_REFERENCES[row.id]
        control = read(ROOT / f"art/style05/env/ground/{name}.png").resize(
            (64, 64),
            Image.Resampling.LANCZOS,
        )
        result.append(
            Panel(
                row.id,
                GROUND_BANDS[row.id],
                read(row.source),
                control,
                read(row.path),
                "GROUND BLOCKED / control reduced to 64px",
            ),
        )
    return result


def render(panels: list[Panel], title: str) -> Image.Image:
    """Display pixels at 1x; standing figures share a bottom registration line."""
    sheet = Image.new("RGBA", (640, 44 + len(panels) * ROW_HEIGHT), "#ddd8ca")
    draw = ImageDraw.Draw(sheet)
    draw.text((8, 4), title, fill="black")
    for column, label in enumerate(("BEFORE B2", "SHIPPED CONTROL", "AFTER R2")):
        draw.text((column * 208 + 8, 24), label, fill="black")
    for index, panel in enumerate(panels):
        top = 44 + index * ROW_HEIGHT
        draw.rectangle(
            (0, top, 639, top + ROW_HEIGHT - 1),
            fill="#ddd8ca" if index % 2 == 0 else "#565c50",
        )
        ink = "black" if index % 2 == 0 else "white"
        draw.text((8, top + 3), panel.id, fill=ink)
        draw.text((8, top + 16), panel.status, fill=ink)
        for column, image in enumerate((panel.before, panel.control, panel.after)):
            assert image.width <= 192 and image.height <= 192, panel.id
            x = column * 208 + (208 - image.width) // 2
            sheet.alpha_composite(image, (x, top + 226 - image.height))
    return sheet.convert("RGB")


def main() -> None:
    """Write nonempty native pages and byte-identical external review copies."""
    panels = flora_panels() + ground_panels()
    OUT.mkdir(parents=True, exist_ok=True)
    evidence = DATA / "sheets"
    evidence.mkdir(exist_ok=True)
    index: list[dict[str, str | int]] = []
    for band in BANDS:
        selected = [panel for panel in panels if band in panel.climates]
        for start in range(0, len(selected), PAGE_SIZE):
            page = start // PAGE_SIZE + 1
            rows = selected[start : start + PAGE_SIZE]
            assert rows
            sheet = render(rows, f"R2 {band} / page {page} / native pixels")
            for scale in (1, 2):
                name = f"{band}_{page:02d}_{scale}x.png"
                image = (
                    sheet
                    if scale == 1
                    else sheet.resize(
                        (sheet.width * 2, sheet.height * 2),
                        Image.Resampling.NEAREST,
                    )
                )
                image.save(OUT / name, optimize=True)
                _ = (evidence / name).write_bytes((OUT / name).read_bytes())
                index.append(
                    {
                        "band": band,
                        "page": page,
                        "scale": scale,
                        "rows": len(rows),
                        "path": name,
                    }
                )
            with Image.open(OUT / f"{band}_{page:02d}_2x.png") as doubled:
                expected = sheet.resize(doubled.size, Image.Resampling.NEAREST)
                assert doubled.tobytes() == expected.tobytes()
    assert index and all(
        any(item["band"] == band for item in index) for band in BANDS[:3]
    )
    _ = (OUT / "index.json").write_text(json.dumps(index, indent=2) + "\n")
    print(f"R2 boards: {len(panels)} unique rows, {len(index)} PNGs")


if __name__ == "__main__":
    main()
