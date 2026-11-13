# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic python -m tools.v8.compose_wall_pieces art/batch_c3a/walls.
"""Export drystone topology pieces using the shipped material wall compositor.

This is an art exporter, not a change to Godot's construction-style schema.
"""

from __future__ import annotations

import hashlib
import json
import shutil
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.compositor.walls import render_walls, wall_tiles
from art.materials.render import load_textures
from art.materials.resolve import read_catalog, resolve

ROOT: Final = Path(__file__).resolve().parents[2]
DIRECTIONS: Final = ((1, (0, -1)), (2, (1, 0)), (4, (0, 1)), (8, (-1, 0)))
CAPS: Final = (
    ("end_cap_n", 1),
    ("end_cap_e", 2),
    ("end_cap_s", 4),
    ("end_cap_w", 8),
    ("corner_cap_ne", 3),
    ("corner_cap_se", 6),
    ("corner_cap_nw", 9),
    ("corner_cap_sw", 12),
)


@dataclass(frozen=True, slots=True)
class Piece:
    """One physical image and its topology contract."""

    id: str
    mask: int
    path: str
    sha256: str
    source_size: tuple[int, int]


@dataclass(frozen=True, slots=True)
class Source:
    """Immutable input provenance."""

    path: str
    sha256: str


def digest(path: Path) -> str:
    """Hash actual bytes, including generated outputs."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def export(output: Path) -> None:
    """Write a new directory; never overwrite an existing export or owner data."""
    output.mkdir(parents=True, exist_ok=False)
    textures = load_textures(ROOT, resolve(read_catalog(ROOT), "drystone", "granite"))
    pieces: list[Piece] = []
    board = Image.new("RGB", (4 * 144, 4 * 158), "#d0ccbb")
    draw = ImageDraw.Draw(board)
    for mask in range(16):
        neighbours = {cell for bit, cell in DIRECTIONS if mask & bit}
        occupancy = frozenset({(0, 0)} | neighbours)
        tile = wall_tiles(frozenset({(0, 0)}), textures, occupancy=occupancy)[0]
        image = Image.new("RGBA", (64, 64))
        image.alpha_composite(tile.image)
        name = f"stone_wall_{mask:02}"
        path = output / f"{name}.png"
        image.save(path)
        pieces.append(Piece(name, mask, path.name, digest(path), tile.image.size))
        x, y = mask % 4 * 144 + 8, mask // 4 * 158 + 22
        board.paste(
            image.resize((128, 128), Image.Resampling.NEAREST),
            (x, y),
            image.resize((128, 128), Image.Resampling.NEAREST),
        )
        draw.text((x, y - 17), f"{mask:02}: N1 E2 S4 W8", fill="#302923")
    board.save(output / "contact_sheet_2x.png")
    for name, mask in CAPS:
        _ = shutil.copyfile(
            output / f"stone_wall_{mask:02}.png", output / f"{name}.png"
        )
    for state, source in (
        ("closed", textures.door_closed),
        ("open", textures.door_open),
    ):
        image = Image.new("RGBA", (64, 64))
        image.alpha_composite(source)
        image.save(output / f"stone_door_ew_{state}.png")
    ring = frozenset(
        (x, y) for y in range(4) for x in range(5) if x in (0, 4) or y in (0, 3)
    )
    assembly = render_walls(ring, textures)
    assembly.image.save(output / "assembly_5x4_1x.png")
    sources = [
        Source(str(path.relative_to(ROOT)), digest(path))
        for path in sorted((ROOT / "art/materials/textures/drystone").glob("*"))
    ]
    sources.extend(
        Source(path, digest(ROOT / path))
        for path in (
            "art/compositor/walls.py",
            "art/materials/render.py",
            "art/materials/resolve.py",
            "art/materials/construction_styles.json",
            "art/materials/material_looks.json",
            "art/materials/category_defaults.json",
            "art/materials/tone.py",
            "tools/v8/compose_wall_pieces.py",
        )
    )
    capability = {
        "status": "PROPOSED_DISABLED_NOT_IMPORTED",
        "runtime_rows_to_append": [],
        "canvas": [64, 64],
        "anchor": [32, 59],
        "pieces": {str(piece.mask): piece.path for piece in pieces},
        "doors": {
            "ew": {state: f"stone_door_ew_{state}.png" for state in ("open", "closed")},
            "ns": None,
        },
        "gap": "NS doors require an independently projected source; no rotation substitute.",
    }
    manifest = {
        "schema_version": 1,
        "status": "COMPOSED_CANDIDATE_NOT_IMPORTED",
        "style": "drystone",
        "material": "granite",
        "era": "iron",
        "canvas": [64, 64],
        "anchor": [32, 59],
        "footprint": [1, 1],
        "world_base": [32, 43],
        "image_origin": [0, -16],
        "bits": {"N": 1, "E": 2, "S": 4, "W": 8},
        "operations": [
            "existing wall_tiles topology composition",
            "transparent bottom padding to 64px",
        ],
        "pieces": [asdict(piece) for piece in pieces],
        "caps": {name: mask for name, mask in CAPS},
        "auxiliary_outputs": [
            asdict(Source(path.name, digest(path)))
            for path in sorted(output.glob("*.png"))
            if not path.name.startswith("stone_wall_")
        ],
        "sources": [asdict(source) for source in sources],
        "capability": capability,
    }
    _ = (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


def main() -> int:
    """Read one destination argument; all source material is fixed by this pass."""
    export(Path(sys.argv[1]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
