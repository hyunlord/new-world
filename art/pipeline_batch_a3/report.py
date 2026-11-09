#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run: PYTHONPATH=. python -m art.pipeline_batch_a3.report CANDIDATES OUT_PREFIX
"""Recheck candidate bytes and build native-scale and raw-alpha review sheets."""

from __future__ import annotations

import json
import math
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import ClassVar, Final

from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, TypeAdapter

from art.pipeline_batch_a3.checks import AlphaCheck, inspect_tile
from art.pipeline_batch_a3.icons import icon_tiles, inspect_icon
from art.pipeline_batch_a3.process import REPO, Candidate
from art.pipeline_env1.items_prepare import normalize
from art.pipeline_struct.normalize import digest

REFERENCES: Final = (
    ("adopted: firewood", "art/style05/polish/items/firewood_try3.png"),
    ("adopted: stone axe", "art/style05/polish/items/stone_axe_try1.png"),
    ("adopted: hide", "art/style05/polish/items/cured_hide_try2.png"),
    ("adopted: empty pot", "art/style05/env1/items/clay_pot_empty.png"),
    ("adopted: meat", "art/style05/env1/items/meat.png"),
    ("adopted: berries", "art/style05/polish/items/berry_basket_try1.png"),
)
CLI_ARGC: Final = 3


class ImportedFile(BaseModel):
    """The live import configuration's declared source of one file."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    role: str
    source: str


class ImportConfig(BaseModel):
    """Provenance for confirming that comparison references are imported items."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    source_commit: str
    files: tuple[ImportedFile, ...]


@dataclass(frozen=True, slots=True)
class Cell:
    """One labelled image in a review sheet."""

    label: str
    path: Path


@dataclass(frozen=True, slots=True)
class SheetRequest:
    """A comparison board with a fixed display size and backdrop policy."""

    cells: tuple[Cell, ...]
    output: Path
    size: int
    raw: bool = False


@dataclass(frozen=True, slots=True)
class AttemptSummary:
    """Independent numeric outcomes rather than one misleading approval flag."""

    id: str
    attempt: int
    status: str
    source_alpha: AlphaCheck
    numeric_style_pass: bool
    projection_pass: bool


def write_sheet(request: SheetRequest) -> None:
    """Composite without alpha clipping; raw sheets show both grey and green."""
    columns = 3 if request.raw else 6
    width = request.size * 2 + 24 if request.raw else 152
    height = request.size + 32 if request.raw else 96
    sheet = Image.new(
        "RGB",
        (columns * width, math.ceil(len(request.cells) / columns) * height),
        (72, 73, 57),
    )
    draw = ImageDraw.Draw(sheet)
    for index, cell in enumerate(request.cells):
        left, top = index % columns * width, index // columns * height
        with Image.open(cell.path) as opened:
            sprite = opened.convert("RGBA")
        sprite.thumbnail((request.size, request.size), Image.Resampling.LANCZOS)
        if request.raw:
            draw.rectangle(
                (left, top, left + request.size + 8, top + request.size + 8),
                fill=(128, 128, 128),
            )
            draw.rectangle(
                (left + request.size + 12, top, left + width, top + request.size + 8),
                fill=(60, 100, 65),
            )
            sheet.paste(sprite, (left + 4, top + 4), sprite)
            sheet.paste(sprite, (left + request.size + 16, top + 4), sprite)
        else:
            sheet.paste(sprite, (left + (width - sprite.width) // 2, top + 4), sprite)
        draw.text((left + 5, top + height - 24), cell.label, fill=(235, 226, 203))
    request.output.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(request.output, optimize=True)


def recheck(candidates: tuple[Candidate, ...]) -> tuple[list[Cell], list[str]]:
    """Re-read source and export evidence so stale checks cannot conceal edits."""
    cells: list[Cell] = []
    failures: list[str] = []
    for candidate in candidates:
        if not candidate.source_alpha.passed:
            failures.append(f"source alpha rejection: {candidate.raw}")
        for path, expected in (
            (candidate.raw, candidate.raw_sha256),
            (candidate.generation_record, candidate.generation_record_sha256),
        ):
            if digest(Path(path)) != expected:
                failures.append(f"changed evidence: {path}")
        expected_icons = (
            dict(icon_tiles(normalize(Path(candidate.raw)), counts=True))
            if candidate.exports
            else {}
        )
        for exported in candidate.exports:
            if digest(Path(exported.path)) != exported.sha256:
                failures.append(f"changed export: {exported.path}")
            with Image.open(exported.path) as image:
                check = (
                    inspect_icon(
                        image, exported.bucket, expected_icons[exported.bucket]
                    )
                    if exported.check.geometry_contract == "ENV1_CENTERED_ICON"
                    else inspect_tile(image)
                )
            if not check.structural_pass:
                failures.append(f"structural rejection: {exported.path}")
            cells.append(
                Cell(
                    f"{candidate.id} t{candidate.attempt} / {exported.bucket}",
                    Path(exported.path),
                )
            )
    return cells, failures


def report(directory: Path, prefix: Path) -> int:
    """Fail on missing candidates or changed source/export bytes; report all trials."""
    paths = sorted(directory.glob("*_try*.json"))
    if not paths:
        message = f"No candidate checks found in {directory}"
        raise ValueError(message)
    candidates = tuple(
        TypeAdapter(Candidate).validate_json(path.read_text()) for path in paths
    )
    config_path = directory.parent / "sources/art_import.json"
    config = ImportConfig.model_validate_json(config_path.read_text())
    imported_items = {row.source for row in config.files if row.role == "item"}
    missing_refs = {path for _, path in REFERENCES} - imported_items
    if missing_refs:
        message = (
            f"Comparison references are not declared imported items: {missing_refs}"
        )
        raise ValueError(message)
    cells, failures = recheck(candidates)
    rows = tuple(
        AttemptSummary(
            candidate.id,
            candidate.attempt,
            candidate.status,
            candidate.source_alpha,
            bool(
                candidate.style_diagnostic and candidate.style_diagnostic.numeric_pass
            ),
            bool(candidate.exports)
            and all(tile.check.projection.passed for tile in candidate.exports),
        )
        for candidate in candidates
    )
    references = tuple(Cell(name, REPO / path) for name, path in REFERENCES)
    sheets: list[str] = []
    for page, offset in enumerate(range(0, len(cells), 42), start=1):
        page_cells = references + tuple(cells[offset : offset + 42])
        for size in (64, 32):
            path = prefix.with_name(f"{prefix.name}_{size}px_{page:02}.png")
            write_sheet(SheetRequest(page_cells, path, size))
            sheets.append(str(path))
    raw_cells = tuple(
        Cell(f"{row.id} try{row.attempt}", Path(row.raw)) for row in candidates
    )
    for page, offset in enumerate(range(0, len(raw_cells), 12), start=1):
        path = prefix.with_name(f"{prefix.name}_raw_alpha_{page:02}.png")
        write_sheet(SheetRequest(raw_cells[offset : offset + 12], path, 256, raw=True))
        sheets.append(str(path))
    summary = {
        "attempts": len(candidates),
        "export_count": len(cells),
        "failures": failures,
        "checks": tuple(asdict(row) for row in rows),
        "sheets": sheets,
        "adoption": "NOT_DECIDED_BY_THIS_TOOL",
        "reference_config": str(config_path),
        "reference_source_commit": config.source_commit,
        "references": [
            {"label": name, "source": path, "sha256": digest(REPO / path)}
            for name, path in REFERENCES
        ],
    }
    _ = prefix.with_suffix(".json").write_text(json.dumps(summary, indent=2) + "\n")
    print(
        f"{len(candidates)} attempts; {len(cells)} exports;",
        f"{len(failures)} integrity/geometry failures; {len(sheets)} sheets",
    )
    return int(bool(failures))


def main() -> int:
    """Build review evidence only; never copy a candidate into adopted assets."""
    if len(sys.argv) != CLI_ARGC:
        print("Usage: report CANDIDATES OUT_PREFIX", file=sys.stderr)
        return 2
    directory, prefix = TypeAdapter(tuple[Path, Path]).validate_python(sys.argv[1:])
    return report(directory, prefix)


if __name__ == "__main__":
    raise SystemExit(main())
