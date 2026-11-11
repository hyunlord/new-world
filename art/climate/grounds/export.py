# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: python -m art.climate.grounds.export
"""Export only reviewed passing ground masters and disclosed existing season/edge recipes."""

from __future__ import annotations

import json
from dataclasses import asdict
from itertools import pairwise
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.climate.grounds.process import DATA, ROOT
from art.compositor.edges import Boundary, EdgeField, boundary_steps, render_tile
from art.compositor.tone import Condition, season_ground
from art.flora.normalize import digest
from art.pipeline2.ground import measure

OUT: Final = ROOT / "art/style05/climate/grounds"
META: Final = ROOT / "art/climate/grounds"
CLIMATES: Final = {
    "tundra": ["harsh"],
    "taiga_floor": ["harsh", "neutral"],
    "steppe": ["harsh", "neutral", "mild"],
    "temperate_meadow": ["neutral"],
    "mediterranean_scrub": ["mild"],
    "marsh": ["harsh", "neutral", "mild"],
    "river_gravel": ["harsh", "neutral", "mild"],
    "highland_rock": ["harsh", "neutral", "mild"],
}


def main() -> None:
    """Fail closed on missing selections and retain all source diagnostics."""
    selection = json.loads((DATA / "selection.json").read_text())
    assert selection, "No passing ground master selected; export is blocked"
    diagnostics = json.loads((DATA / "diagnostics.json").read_text())
    rows = []
    textures = {}
    OUT.mkdir(parents=True, exist_ok=True)
    for asset, attempt in selection.items():
        found = [
            r
            for r in diagnostics
            if r["id"] == asset + "_summer" and r["attempt"] == attempt
        ]
        assert len(found) == 1
        source = found[0]
        assert (
            source["seam"]["passed"]
            and source["tone_pass"]
            and source["style"]["numeric_pass"]
        )
        assert source["alpha_extrema"] == [255, 255]
        with Image.open(source["tile"]) as opened:
            summer = opened.convert("RGBA")
        textures[asset] = summer
        for season, condition in [
            ("spring", Condition.DAY),
            ("summer", Condition.DAY),
            ("autumn", Condition.DROUGHT),
            ("winter", Condition.WINTER),
        ]:
            image = season_ground(summer, condition).convert("RGBA")
            path = OUT / f"{asset}_{season}.png"
            image.save(path, optimize=True)
            seam = measure(image)
            assert seam.passed and image.getchannel("A").getextrema() == (255, 255)
            row = {
                "id": f"{asset}_{season}",
                "path": str(path.relative_to(ROOT)),
                "sha256": digest(path),
                "canvas": [64, 64],
                "anchor": [32, 43],
                "footprint": [1, 1],
                "climate": CLIMATES[asset],
                "source_attempt": source,
                "derived": season != "summer",
                "operation": f"season_ground:{condition.value}",
                "season_note": "Spring shares summer; autumn uses existing drought tint; winter existing chroma35%, no new snow artwork.",
                "seam": asdict(seam),
                "status": "ART_CANDIDATE_NOT_IMPORTED",
            }
            path.with_suffix(".json").write_text(json.dumps(row, indent=2) + "\n")
            rows.append(row)
    boards = META / "boards"
    boards.mkdir(exist_ok=True)
    board = Image.new("RGB", (640, max(1, len(selection)) * 152 + 40), "#d6d2c3")
    draw = ImageDraw.Draw(board)
    draw.text(
        (5, 5),
        "B2 ground: 2x seasons; existing meadow reference at right",
        fill="black",
    )
    for y, asset in enumerate(selection):
        draw.text((4, y * 152 + 25), asset, fill="black")
        for x, season in enumerate(("spring", "summer", "autumn", "winter")):
            with Image.open(OUT / f"{asset}_{season}.png") as image:
                board.paste(
                    image.resize((128, 128), Image.Resampling.NEAREST),
                    (x * 128, y * 152 + 42),
                )
        with Image.open(ROOT / "art/style05/env/ground/meadow.png") as ref:
            board.paste(
                ref.resize((128, 128), Image.Resampling.LANCZOS), (512, y * 152 + 42)
            )
    board.save(boards / "seasons_2x.png")
    native = Image.new("RGB", (320, max(1, len(selection)) * 86 + 24), "#d6d2c3")
    for y, asset in enumerate(selection):
        ImageDraw.Draw(native).text((2, y * 86), asset, fill="black")
        for x, season in enumerate(("spring", "summer", "autumn", "winter")):
            with Image.open(OUT / f"{asset}_{season}.png") as image:
                native.paste(image, (x * 64, y * 86 + 18))
        with Image.open(ROOT / "art/style05/env/ground/meadow.png") as ref:
            native.paste(
                ref.resize((64, 64), Image.Resampling.LANCZOS), (256, y * 86 + 18)
            )
    native.save(boards / "seasons_1x.png")
    edges = []
    for climate in ("harsh", "neutral", "mild"):
        members = [key for key in textures if climate in CLIMATES[key]]
        for a, b in pairwise(members):
            terrain = {(x, y): a if x < 2 else b for y in range(3) for x in range(4)}
            field = EdgeField(terrain, textures)
            joined = Image.new("RGBA", (256, 192))
            for cell in terrain:
                joined.paste(render_tile(field, cell), (cell[0] * 64, cell[1] * 64))
            path = boards / f"edge_{climate}_{a}_{b}.png"
            joined.save(path)
            metrics = boundary_steps(joined, Boundary((128, 0), 192, "vertical"))
            assert metrics.longest_identical_step <= 24
            edges.append(
                {
                    "climate": climate,
                    "a": a,
                    "b": b,
                    "board": str(path.relative_to(ROOT)),
                    "sha256": digest(path),
                    "seed": 5005,
                    "blend_px": 12,
                    "operation": "art.compositor.edges.EdgeField/render_tile; no rotation",
                    "boundary": asdict(metrics),
                    "derived": True,
                }
            )
    (META / "manifest.json").write_text(
        json.dumps(
            {
                "assets": rows,
                "edges": edges,
                "missing": sorted(set(CLIMATES) - set(selection)),
                "edge_band_status": {
                    climate: (
                        "rendered_candidate"
                        if any(e["climate"] == climate for e in edges)
                        else "GAP: fewer than two accepted inputs"
                    )
                    for climate in ("harsh", "neutral", "mild")
                },
            },
            indent=2,
        )
        + "\n"
    )
    (META / "art_import.draft.json").write_text(
        json.dumps(
            {
                "applied": False,
                "source_commit": "LEADER_TO_PIN",
                "files": [
                    {
                        "role": "ground_texture",
                        "id": r["id"],
                        "source": r["path"],
                        "dest": "ground/b2/" + Path(r["path"]).name,
                    }
                    for r in rows
                ],
            },
            indent=2,
        )
        + "\n"
    )
    (META / "bindings.draft.json").write_text(
        json.dumps(
            {
                "applied": False,
                "runtime_rows": [],
                "proposed_climate_grounds": {
                    key: {
                        "climate": CLIMATES[key],
                        "seasons": {
                            s: f"{key}_{s}"
                            for s in ("spring", "summer", "autumn", "winter")
                        },
                    }
                    for key in selection
                },
                "note": "Climate/season selection needs C2/C3 adapter.",
            },
            indent=2,
        )
        + "\n"
    )
    print(f"Exported {len(rows)} season frames and {len(edges)} edge recipes")


if __name__ == "__main__":
    main()
