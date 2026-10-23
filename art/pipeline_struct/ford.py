# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with pydantic --with scikit-image python -m art.pipeline_struct.ford
"""Extract the declared four equal ford-stone cells without retouching generated art."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Final

from PIL import Image
from pydantic import TypeAdapter

from art.compositor.models import AssetEntry, AssetManifest
from art.pipeline_struct.normalize import NormalizedSprite, digest, normalize_sprite

REPO: Final = Path(__file__).resolve().parents[2]
SOURCE: Final = Path("/Users/rexxa/work/artstruct-data/generations/ford_stones_try1.png")


@dataclass(frozen=True, slots=True)
class StoneEvidence:
    id: str
    source: str
    source_sha256: str
    sheet_cell: tuple[int, int, int, int]
    alpha_crop_within_cell: tuple[int, int, int, int]
    uniform_scale: float
    opaque_target_height: int
    anchor: tuple[int, int]
    export_sha256: str
    operations: tuple[str, ...]
    status: str


def split_stones(source: Image.Image) -> tuple[NormalizedSprite, ...]:
    """Split exactly four columns and apply the existing uniform normalizer."""
    if source.width % 4:
        message = "Ford sheet width must divide exactly into four equal cells"
        raise ValueError(message)
    width = source.width // 4
    return tuple(normalize_sprite(source.crop((index * width, 0, (index + 1) * width, source.height)), 12) for index in range(4))


def main() -> None:
    with Image.open(SOURCE) as raw:
        source = raw.convert("RGBA")
    stones = split_stones(source)
    entries: list[AssetEntry] = []
    for index, stone in enumerate(stones):
        name = f"ford_stone_{index + 1}"
        path = REPO / f"art/style05/struct/{name}.png"
        stone.image.save(path, optimize=True)
        evidence = StoneEvidence(name, str(SOURCE), digest(SOURCE),
            (index * source.width // 4, 0, (index + 1) * source.width // 4, source.height),
            stone.crop, stone.scale, 12, stone.anchor, digest(path),
            ("equal four-cell split", "alpha>=128 bbox plus2px fringe crop", "uniform Lanczos to opaque height12", "transparent pad4; no local pixel edits"),
            "CANDIDATE_NORMALIZED_NOT_ACCEPTED")
        _ = path.with_suffix(".json").write_bytes(TypeAdapter(StoneEvidence).dump_json(evidence, indent=2) + b"\n")
        entries.append(AssetEntry(id=name, path=str(path.relative_to(REPO)), width=stone.image.width,
                                  height=stone.image.height, anchor=stone.anchor, footprint=(.5, .5),
                                  family="ground", status=evidence.status))
    _ = (REPO / "docs/v8/artstruct/ford_assets.json").write_text(AssetManifest(assets=tuple(entries)).model_dump_json(indent=2) + "\n")


if __name__ == "__main__":
    main()
