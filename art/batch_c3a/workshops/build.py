"""Compose proposed4x4 iron workshops from existing, independently sorted layers."""

from __future__ import annotations

import hashlib
import io
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from PIL import Image
from pydantic import BaseModel

from art.batch_c3a.workshops.boards import contacts, station_contact
from art.batch_c3a.workshops.drafts import write_drafts
from art.batch_c3a.workshops.stations import station
from art.compositor.walls import wall_tiles
from art.materials.render import load_surface, load_textures
from art.materials.resolve import read_catalog, resolve
from art.pipeline_env1.items_records import JsonValue

ROOT: Final = Path(__file__).resolve().parents[3]
OUT: Final = ROOT / "art/batch_c3a/workshops"
DATA: Final = Path("/Users/rexxa/work/art-batch/C3A")
A2: Final = "227529c26307bc3a854eb7bf819e88d02af39046"
Point = tuple[int, int]
Course = tuple[int, tuple[Point, ...]]


class StationAnchor(BaseModel):
    """Read a generated station geometry adapter."""

    anchor: Point


@dataclass(frozen=True, slots=True)
class Layer:
    """Keep every world layer's own origin and source anchor."""

    kind: str
    image: Image.Image
    cell: Point
    anchor: Point
    origin: Point
    source: str
    source_sha256: str


def sha(data: bytes) -> str:
    """Hash literal source bytes."""
    return hashlib.sha256(data).hexdigest()


def stages() -> tuple[tuple[str, tuple[Course, ...]], ...]:
    """B3's exact top/face cropping keeps the plan fixed through construction."""
    ring = tuple(
        (x, y)
        for y in range(4)
        for x in range(4)
        if (x in (0, 3) or y in (0, 3)) and (x, y) != (2, 3)
    )
    north = tuple(cell for cell in ring if cell[1] < 2)
    south = tuple(cell for cell in ring if cell[1] >= 2)
    intact: tuple[Course, ...] = ((60, ring),)
    return (
        ("foundation", ((24, ring),)),
        ("half_built", ((60, north), (24, south))),
        ("built", intact),
        ("damaged", ((60, north), (24, south))),
        ("ruined", ((24, north),)),
    )


def pinned(asset: str) -> tuple[Image.Image, str, str]:
    """Read an existing A2 sprite without modifying any other worktree."""
    path = f"art/style05/struct/a2/iron/{asset}.png"
    data = subprocess.check_output(["git", "show", f"{A2}:{path}"], cwd=ROOT)
    with Image.open(io.BytesIO(data)) as opened:
        return opened.convert("RGBA"), path, sha(data)


def layers(
    family: str, working: bool, courses: tuple[Course, ...], state: str
) -> list[Layer]:
    """Reuse material topology and B3 course crops; sort by physical world base."""
    catalog = read_catalog(ROOT)
    textures = load_textures(ROOT, resolve(catalog, "drystone", "granite"))
    floor = resolve(catalog, "packed_earth", "soil", kind="floor")
    surface = load_surface(ROOT, floor)
    floor_path = floor.textures.floor
    if floor_path is None:
        raise RuntimeError("Packed earth floor source is required")
    result: list[Layer] = []
    for y in range(4):
        for x in range(4):
            h = 44 if y == 3 else 64
            result.append(
                Layer(
                    "floor",
                    surface.crop((x * 64, y * 64, x * 64 + 64, y * 64 + h)),
                    (x, y),
                    (32, 43),
                    (x * 64, y * 64),
                    floor_path,
                    sha((ROOT / floor_path).read_bytes()),
                )
            )
    occupancy = frozenset(cell for _, cells in courses for cell in cells) | {(2, 3)}
    for height, cells in courses:
        for tile in wall_tiles(frozenset(cells), textures, occupancy=occupancy):
            image = (
                tile.image.copy()
                if tile.image.height > 60
                else tile.image.crop((0, 0, 64, height))
            )
            result.append(
                Layer(
                    "wall",
                    image,
                    tile.cell,
                    (32, height - 1),
                    (tile.cell[0] * 64, tile.cell[1] * 64 + 44 - height),
                    "art/materials/construction_styles.json",
                    sha((ROOT / "art/materials/construction_styles.json").read_bytes()),
                )
            )
    if state == "built":
        door = textures.door_open
        result.append(
            Layer(
                "door",
                door,
                (2, 3),
                (32, 59),
                (128, 176),
                "art/materials/textures/drystone/door_open.png",
                sha(
                    (
                        ROOT / "art/materials/textures/drystone/door_open.png"
                    ).read_bytes()
                ),
            )
        )
        asset = (
            ("forge_working" if working else "forge_idle")
            if family == "smithy"
            else ("bloomery_lit" if working else "bloomery_unlit")
        )
        image, source, digest = pinned(asset)
        sidecar = subprocess.check_output(
            ["git", "show", f"{A2}:{source[:-4]}.json"], cwd=ROOT
        )
        anchor = StationAnchor.model_validate_json(sidecar).anchor
        result.append(
            Layer(
                "prop",
                image,
                (1, 1),
                anchor,
                (96 - anchor[0], 107 - anchor[1]),
                source,
                digest,
            )
        )
        anvil = OUT / "stations/anvil.png"
        if family == "smithy" and anvil.exists():
            point = StationAnchor.model_validate_json(
                anvil.with_suffix(".json").read_bytes()
            ).anchor
            with Image.open(anvil) as opened:
                result.append(
                    Layer(
                        "prop",
                        opened.convert("RGBA"),
                        (2, 2),
                        point,
                        (160 - point[0], 171 - point[1]),
                        str(anvil.relative_to(ROOT)),
                        sha(anvil.read_bytes()),
                    )
                )
    return sorted(
        result,
        key=lambda layer: (
            layer.kind != "floor",
            layer.origin[1] + layer.anchor[1],
            layer.origin[0] + layer.anchor[0],
        ),
    )


def export(
    family: str, state: str, courses: tuple[Course, ...], working: bool = False
) -> None:
    """Save importable layers separately; flattened image is review evidence only."""
    name = f"{family}_{state}" + ("_working" if working else "")
    folder = OUT / "rooms" / name
    folder.mkdir(parents=True, exist_ok=True)
    rows = layers(family, working, courses, state)
    preview = Image.new("RGBA", (256, 280))
    records: list[dict[str, JsonValue]] = []
    shared = OUT / "layers"
    shared.mkdir(exist_ok=True)
    for layer in rows:
        path = (
            shared
            / f"{sha(str(layer.image.size).encode() + layer.image.tobytes())}.png"
        )
        layer.image.save(path)
        preview.alpha_composite(layer.image, (layer.origin[0], layer.origin[1] + 16))
        records.append(
            {
                "path": str(path.relative_to(ROOT)),
                "sha256": sha(path.read_bytes()),
                "kind": layer.kind,
                "cell": layer.cell,
                "anchor": layer.anchor,
                "origin": layer.origin,
                "source": layer.source,
                "source_sha256": layer.source_sha256,
            }
        )
    preview.save(folder / "preview.png")
    metadata = {
        "id": name,
        "era": "iron",
        "footprint": [4, 4],
        "state": state,
        "status": "LAYERED_RECIPE_VISUAL_REVIEW_PENDING",
        "source_commit": A2,
        "layers": records,
        "preview_origin": [0, -16],
        "preview_only": True,
        "notes": [
            "Proposed visual states; runtime construction selector not applied.",
            "Exact B3 top/face crop; no facade rescale or alpha construction trick.",
            "Damaged removes south upper courses; ruined retains north low courses.",
        ],
    }
    _ = (folder / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n")


def main() -> None:
    """Build station candidates and the complete proposed workshop lifecycle."""
    OUT.mkdir(parents=True, exist_ok=True)
    _ = station("anvil", 28, (1, 1))
    _ = station("mason_bench", 30, (2, 1))
    for family in ("smithy", "bloomery_house"):
        for state, courses in stages():
            export(family, state, courses)
            if state == "built":
                export(family, state, courses, working=True)
    sources = {
        str(path.relative_to(ROOT)): sha(path.read_bytes())
        for path in sorted((ROOT / "art/materials/textures/drystone").glob("*.png"))
    }
    for path in (
        "art/materials/construction_styles.json",
        "art/materials/material_looks.json",
        "art/materials/category_defaults.json",
    ):
        sources[path] = sha((ROOT / path).read_bytes())
    _ = (OUT / "material_sources.json").write_text(json.dumps(sources, indent=2) + "\n")
    blueprints = {
        family: {
            "source": f"{family}_built",
            "operation": "existing B3 blueprint:25%alpha wash,alpha96 silhouette,4-neighbour1px pale outline",
            "status": "DISPLAY_RECIPE_NOT_RUNTIME_APPLIED",
        }
        for family in ("smithy", "bloomery_house")
    }
    _ = (OUT / "blueprints.json").write_text(json.dumps(blueprints, indent=2) + "\n")
    contacts(OUT, ROOT)
    station_contact(OUT, ROOT)
    write_drafts(OUT, ROOT)


if __name__ == "__main__":
    main()
