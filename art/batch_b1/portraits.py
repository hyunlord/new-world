# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "typer"]
# ///
# How to run: python3 -m art.batch_b1.portraits process RECORD.json TRIAL_DIR
"""Slice six opaque inspector portraits without background extraction."""

from __future__ import annotations

import hashlib
import shutil
from pathlib import Path
from typing import Final

import typer
from PIL import Image

from art.batch_b1.models import REPO, Evidence, Generation, VisualVerdict, digest, save

VARIANTS: Final = ("v1", "v2", "v3", "v4", "v5", "v6")


class Portrait(Evidence):
    variant: str
    path: str
    sha256: str
    source_box: tuple[int, int, int, int]
    canvas: tuple[int, int] = (128, 128)
    anchor: tuple[int, int] = (64, 64)
    footprint: None = None


class PortraitSummary(Evidence):
    look: str
    record_sha256: str
    raw_sha256: str
    prompt_sha256: str
    portraits: tuple[Portrait, ...]
    passed: bool = True
    method: str = "exact 3x2 source cells; full-frame uniform Lanczos; no extraction"


def process(record: Path, trial: Path) -> PortraitSummary:
    """Keep all six full cells at 128px; no crop inferred from subject pixels."""
    generation = Generation.model_validate_json(record.read_bytes())
    for reference in generation.references:
        if digest(reference.path) != reference.sha256:
            message = f"reference changed: {reference.path}"
            raise ValueError(message)
    with Image.open(generation.raw) as source:
        if source.size != (1536, 1024) or source.mode not in ("RGB", "RGBA"):
            message = "portrait sheet requires 1536x1024 RGB/RGBA"
            raise ValueError(message)
        image = source.convert("RGBA")
    if image.getchannel("A").getextrema() != (255, 255):
        message = "portrait cells must be opaque; do not extract their backgrounds"
        raise ValueError(message)
    trial.mkdir(parents=True, exist_ok=False)
    _ = shutil.copyfile(record, trial / "generation.json")
    _ = shutil.copyfile(generation.raw, trial / "raw.png")
    _ = (trial / "prompt.txt").write_text(generation.prompt)
    portraits: list[Portrait] = []
    board = Image.new("RGBA", (768, 512))
    for index, variant in enumerate(VARIANTS):
        x, y = index % 3 * 512, index // 3 * 512
        box = (x, y, x + 512, y + 512)
        normalized = image.crop(box).resize((128, 128), Image.Resampling.LANCZOS)
        path = trial / f"{variant}.png"
        normalized.save(path)
        portrait = Portrait(
            variant=variant, path=path.name, sha256=digest(path), source_box=box
        )
        save(path.with_suffix(".json"), portrait)
        portraits.append(portrait)
        board.paste(
            normalized.resize((256, 256), Image.Resampling.NEAREST),
            (index % 3 * 256, index // 3 * 256),
        )
    board.save(trial / "review_2x.png")
    summary = PortraitSummary(
        look=generation.look,
        record_sha256=digest(record),
        raw_sha256=digest(generation.raw),
        prompt_sha256=hashlib.sha256(generation.prompt.encode()).hexdigest(),
        portraits=tuple(portraits),
    )
    save(trial / "summary.json", summary)
    return summary


def adopt(
    trial: Path, verdict: Path, root: Path = REPO / "art/portrait/batch_b1"
) -> Path:
    """Require a hash-bound review and six unchanged opaque128px portraits."""
    summary = PortraitSummary.model_validate_json((trial / "summary.json").read_bytes())
    if not summary.passed:
        message = "portrait structural summary must pass before adoption"
        raise ValueError(message)
    review = VisualVerdict.model_validate_json(verdict.read_bytes())
    if (
        review.summary_sha256 != digest(trial / "summary.json")
        or review.score < 90
        or review.verdict != "pass"
        or not review.category_match
    ):
        message = "portrait visual review must pass at 90 and bind the current summary"
        raise ValueError(message)
    generation = Generation.model_validate_json(
        (trial / "generation.json").read_bytes()
    )
    if (
        generation.look != summary.look
        or digest(trial / "generation.json") != summary.record_sha256
        or digest(trial / "raw.png") != summary.raw_sha256
        or digest(trial / "prompt.txt") != summary.prompt_sha256
    ):
        message = "portrait provenance changed"
        raise ValueError(message)
    if (
        len(summary.portraits) != 6
        or tuple(p.variant for p in summary.portraits) != VARIANTS
    ):
        message = "portrait adoption requires exactly v1 through v6"
        raise ValueError(message)
    for portrait in summary.portraits:
        path = trial / f"{portrait.variant}.png"
        if (
            portrait.path != path.name
            or digest(path) != portrait.sha256
            or Portrait.model_validate_json(path.with_suffix(".json").read_bytes())
            != portrait
        ):
            message = f"portrait changed: {portrait.variant}"
            raise ValueError(message)
        with Image.open(path) as image:
            if (
                image.mode != "RGBA"
                or image.size != (128, 128)
                or image.getchannel("A").getextrema() != (255, 255)
            ):
                message = f"portrait canvas/alpha invalid: {portrait.variant}"
                raise ValueError(message)
    destination = root / summary.look
    destination.mkdir(parents=True, exist_ok=False)
    for portrait in summary.portraits:
        _ = shutil.copyfile(trial / portrait.path, destination / portrait.path)
        _ = shutil.copyfile(
            (trial / portrait.path).with_suffix(".json"),
            (destination / portrait.path).with_suffix(".json"),
        )
    _ = shutil.copyfile(trial / "summary.json", destination / "summary.json")
    _ = shutil.copyfile(verdict, destination / "visual_verdict.json")
    return destination


app: Final = typer.Typer()


@app.command("process")
def process_cli(record: Path, trial: Path) -> None:
    """Slice a reviewed-layout generation into six full-frame portraits."""
    typer.echo(process(record, trial).model_dump_json())


@app.command("adopt")
def adopt_cli(
    trial: Path, verdict: Path, root: Path = REPO / "art/portrait/batch_b1"
) -> None:
    """Adopt an unchanged portrait family after independent visual review."""
    typer.echo(str(adopt(trial, verdict, root)))


if __name__ == "__main__":
    app()
