# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic python -m art.batch_c3a.roads.build OUTPUT
"""Reuse accepted pavement with the existing terrain edge compositor."""

from __future__ import annotations

import hashlib
import io
import json
import subprocess
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.compositor.edges import Boundary, EdgeField, boundary_steps, render_tile
from art.pipeline2.ground import measure

ROOT: Final = Path(__file__).resolve().parents[3]
A2_PIN: Final = "227529c26307bc3a854eb7bf819e88d02af39046"
ART_PIN: Final = "f182769fe06b1cb60105e4d0bf127b6524277838"
PAVEMENT: Final = "art/style05/env/ground/a2/iron/paved_road.png"
MEADOW: Final = "art/style05/env/ground/meadow.png"
DIRECTIONS: Final = ((1, (0, -1)), (2, (1, 0)), (4, (0, 1)), (8, (-1, 0)))


def transition(mask: int, source: Image.Image, neighbour: Image.Image) -> Image.Image:
    """A bit means the neighbour has the same material; absent bits get edges."""
    terrain = {(0, 0): "source"}
    terrain.update(
        {cell: "source" if mask & bit else "other" for bit, cell in DIRECTIONS}
    )
    return render_tile(
        EdgeField(terrain, {"source": source, "other": neighbour}), (0, 0)
    )


def read_pinned(pin: str, path: str) -> tuple[Image.Image, str]:
    """Read immutable Git objects, never another art worktree's live files."""
    data = subprocess.check_output(["git", "show", f"{pin}:{path}"], cwd=ROOT)
    with Image.open(io.BytesIO(data)) as image:
        result = image.convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    return result, hashlib.sha256(data).hexdigest()


def export(output: Path) -> None:
    """Create all transitions and one connected road network at native scale."""
    output.mkdir(parents=True, exist_ok=False)
    pavement, paving_hash = read_pinned(A2_PIN, PAVEMENT)
    meadow, meadow_hash = read_pinned(ART_PIN, MEADOW)
    tiles = {
        name: [transition(mask, source, other) for mask in range(16)]
        for name, source, other in (
            ("paved_road", pavement, meadow),
            ("meadow_edge", meadow, pavement),
        )
    }
    board = Image.new("RGB", (8 * 80, 4 * 86), "#d4ceba")
    draw = ImageDraw.Draw(board)
    records: list[dict[str, str | int | list[int]]] = []
    for family_index, (name, images) in enumerate(tiles.items()):
        for mask, tile in enumerate(images):
            path = output / f"{name}_{mask:02}.png"
            tile.save(path)
            records.append(
                {
                    "id": path.stem,
                    "mask": mask,
                    "file": path.name,
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    "size": [64, 64],
                    "anchor": [32, 43],
                    "footprint": [1, 1],
                }
            )
            x, y = mask % 8 * 80 + 8, (mask // 8 + 2 * family_index) * 86 + 17
            board.paste(tile, (x, y))
            draw.text(
                (x, y - 13),
                f"{'road' if family_index == 0 else 'grass'} {mask:02}",
                fill="#30271d",
            )
    board.save(output / "contact_sheet_1x.png")
    board.resize((board.width * 2, board.height * 2), Image.Resampling.NEAREST).save(
        output / "contact_sheet_2x.png"
    )
    road_cells = frozenset(
        (x, y)
        for y in range(7)
        for x in range(7)
        if x == 3 or y == 3 or (x == 1 and 1 <= y <= 3) or (y == 1 and 1 <= x <= 3)
    )
    scene = Image.new("RGBA", (7 * 64, 7 * 64))
    for y in range(7):
        for x in range(7):
            paved = (x, y) in road_cells
            mask = sum(
                bit
                for bit, (dx, dy) in DIRECTIONS
                if (((x + dx, y + dy) in road_cells) == paved)
            )
            scene.alpha_composite(
                tiles["paved_road" if paved else "meadow_edge"][mask], (x * 64, y * 64)
            )
    scene.save(output / "network_1x.png")
    boundaries = [
        boundary_steps(scene, Boundary((64 * line, 0), 448, "vertical"))
        for line in range(1, 7)
    ]
    boundaries.extend(
        boundary_steps(scene, Boundary((0, 64 * line), 448, "horizontal"))
        for line in range(1, 7)
    )
    manifest = {
        "schema_version": 1,
        "status": "COMPOSED_TERRAIN_CANDIDATE_NOT_IMPORTED",
        "era": "iron",
        "canvas": [64, 64],
        "anchor": [32, 43],
        "footprint": [1, 1],
        "bits": {"N": 1, "E": 2, "S": 4, "W": 8},
        "mask_meaning": "same-material cardinal neighbours; absent bits blend to other material",
        "seed": 5005,
        "phase_cell": [0, 0],
        "ground_blend_px": 12,
        "sources": [
            {"commit": A2_PIN, "path": PAVEMENT, "sha256": paving_hash},
            {"commit": ART_PIN, "path": MEADOW, "sha256": meadow_hash},
        ],
        "operations": [
            "existing full-frame Lanczos to64",
            "existing EdgeField/render_tile12px transition",
            "both sides of boundary exported; no hand-painted outline or source recolouring",
        ],
        "files": records,
        "helpers": [
            {
                "path": path,
                "sha256": hashlib.sha256((ROOT / path).read_bytes()).hexdigest(),
            }
            for path in (
                "art/compositor/edges.py",
                "art/pipeline2/ground.py",
                "art/batch_c3a/roads/build.py",
            )
        ],
        "seams": {
            "unblended_pavement_wrap_diagnostic": asdict(measure(pavement)),
            "network_tile_boundaries": [asdict(result) for result in boundaries],
            "no_constant_step_longer_than24px": all(
                result.longest_identical_step <= 24 for result in boundaries
            ),
            "scope": "network boundary variation only; not a seamless-all-topologies claim",
        },
        "limits": [
            "16 cardinal masks do not distinguish diagonal terrain neighbours",
            "fixed0,0 noise phase: actual world-coordinate compositor can vary per cell",
            "no runtime road-edge selector binding applied",
        ],
    }
    _ = (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    export(Path(sys.argv[1]))
