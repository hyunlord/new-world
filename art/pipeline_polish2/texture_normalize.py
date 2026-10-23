# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.texture_normalize
"""Normalize generated material assets and preserve literal seam/projection evidence."""

from __future__ import annotations

import hashlib
import json
from dataclasses import asdict
from pathlib import Path
from typing import ClassVar, Final, Literal

from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, Field, JsonValue, TypeAdapter

from art.compositor.projection_check import ProjectionSpec, check_projection
from art.compositor.wall_metrics import measure_strip

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = ROOT.parent / "artpolish2-data/materials"


class TextureRequest(BaseModel):
    """Generation request completed by the built-in image tool."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    id: str
    style: str
    part: str
    size: tuple[int, int]
    kind: Literal["strip", "component", "door"]
    axis: Literal["x", "y", "xy", "none"]
    attempt: int
    prompt: str
    referenced_image_paths: tuple[str, ...]
    transparent_background: bool
    source: Path
    generated_source: Path
    visual_rejection: str = ""


class ReviewedTextureSelection(BaseModel):
    """Hash-bound review may select a failed diagnostic without passing its gates."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    attempt: int = Field(ge=1, le=3)
    source_sha256: str = Field(pattern=r"^[0-9a-f]{64}$")
    reason: str = Field(min_length=1)
    review: dict[str, JsonValue]


class TextureSelections(BaseModel):
    """Versioned reviewed choices, independent of texture or material names."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    schema_version: Literal[1]
    selections: dict[str, ReviewedTextureSelection]


def digest(path: Path) -> str:
    """Hash immutable source bytes rather than interpreted pixels."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalize(request: TextureRequest) -> tuple[Image.Image, tuple[str, ...]]:
    """Only full-frame strip resize or whole-object uniform fit is permitted."""
    with Image.open(request.source) as source:
        image = source.convert("RGBA")
    if request.transparent_background:
        bbox = image.getchannel("A").point([0] * 128 + [255] * 128).getbbox()
        if bbox is None:
            message = f"Empty generated component: {request.source}"
            raise ValueError(message)
        crop = (
            max(0, bbox[0] - 2),
            max(0, bbox[1] - 2),
            min(image.width, bbox[2] + 2),
            min(image.height, bbox[3] + 2),
        )
        cropped = image.crop(crop)
        scale = min(request.size[0] / cropped.width, request.size[1] / cropped.height)
        scaled = cropped.resize(
            (
                max(1, round(cropped.width * scale)),
                max(1, round(cropped.height * scale)),
            ),
            Image.Resampling.LANCZOS,
        )
        image = Image.new("RGBA", request.size)
        image.alpha_composite(
            scaled,
            ((request.size[0] - scaled.width) // 2, request.size[1] - scaled.height),
        )
        operations = (
            f"STRUCT alpha>=128 bbox+2 fringe crop {crop}",
            f"uniform LANCZOS scale {scale}",
            "bottom-aligned transparent canvas padding; no pixel edits",
        )
    else:
        operations = (f"full-frame LANCZOS resize {image.size} -> {request.size}",)
        image = image.resize(request.size, Image.Resampling.LANCZOS)
    return image, operations


def main() -> None:
    """Export every attempt and honor hash-bound reviewed choices without waivers."""
    requests = TypeAdapter(list[TextureRequest]).validate_json(
        (DATA / "completed.json").read_bytes()
    )
    choices = TextureSelections.model_validate_json(
        (ROOT / "art/materials/texture_selections.json").read_bytes()
    )
    for asset_id, choice in choices.selections.items():
        matching = [
            row
            for row in requests
            if row.id == asset_id and row.attempt == choice.attempt
        ]
        if len(matching) != 1:
            message = f"Reviewed selection needs one existing attempt: {asset_id}"
            raise ValueError(message)
        if digest(matching[0].source) != choice.source_sha256:
            message = f"Reviewed selection source hash mismatch: {asset_id}"
            raise ValueError(message)
        if matching[0].visual_rejection:
            message = f"Reviewed selection has a visual rejection: {asset_id}"
            raise ValueError(message)
    attempts = DATA / "normalized"
    attempts.mkdir(exist_ok=True)
    evidence: list[dict[str, JsonValue]] = []
    selected: dict[str, tuple[TextureRequest, Image.Image, dict[str, JsonValue]]] = {}
    for request in sorted(requests, key=lambda row: (row.id, row.attempt)):
        image, operations = normalize(request)
        attempt_path = (
            attempts / f"{request.id.replace('/', '_')}_try{request.attempt}.png"
        )
        image.save(attempt_path, optimize=True)
        seam_x = measure_strip(image) if request.axis in {"x", "xy"} else None
        seam_y = (
            measure_strip(image.transpose(Image.Transpose.TRANSPOSE))
            if request.axis in {"y", "xy"}
            else None
        )
        projection = (
            check_projection(image, ProjectionSpec("door", (32, 59)))
            if request.kind == "door"
            else None
        )
        opaque = image.getchannel("A").getextrema() == (255, 255)
        passed = (opaque or request.transparent_background) and all(
            row.passed for row in (seam_x, seam_y, projection) if row is not None
        )
        row: dict[str, JsonValue] = {
            "id": request.id,
            "attempt": request.attempt,
            "source": str(request.source),
            "source_sha256": digest(request.source),
            "generated_source": str(request.generated_source),
            "prompt": request.prompt,
            "prompt_sha256": hashlib.sha256(request.prompt.encode()).hexdigest(),
            "references": [
                {"path": path, "sha256": digest(Path(path))}
                for path in request.referenced_image_paths
            ],
            "normalized_path": str(attempt_path),
            "normalized_sha256": digest(attempt_path),
            "size": list(image.size),
            "operations": list(operations),
            "opaque": opaque,
            "seam_x": asdict(seam_x) if seam_x else None,
            "seam_y": asdict(seam_y) if seam_y else None,
            "seam_applicability": "Repeat axes only; isolated end/junction/door do not tile.",
            "projection": asdict(projection) if projection else None,
            "projection_applicability": "Standing doors only; wall components/top/floor do not have a standing silhouette.",
            "numeric_pass": passed,
            "owner_approval": "NOT_CLAIMED",
            "visual_rejection": request.visual_rejection,
            "candidate_ready": passed and not request.visual_rejection,
        }
        evidence.append(row)
        choice = choices.selections.get(request.id)
        if choice is not None:
            if request.attempt == choice.attempt:
                reviewed_row: dict[str, JsonValue] = {
                    **row,
                    "final_diagnostic_selection": choice.review,
                }
                selected[request.id] = (request, image, reviewed_row)
        else:
            previous = selected.get(request.id)
            if previous is None or previous[2]["candidate_ready"] is False:
                selected[request.id] = (request, image, row)
    for request, image, row in selected.values():
        directory = ROOT / "art/materials/textures" / request.style
        directory.mkdir(parents=True, exist_ok=True)
        image.save(directory / f"{request.part}.png", optimize=True)
        _ = (directory / f"{request.part}.json").write_text(
            json.dumps(row, indent=2) + "\n"
        )
    _ = (DATA / "checks.json").write_text(json.dumps(evidence, indent=2) + "\n")
    width, height = 900, ((len(selected) + 2) // 3) * 290
    board = Image.new("RGB", (width, height), "#ddd0b7")
    draw = ImageDraw.Draw(board)
    for index, (request, image, row) in enumerate(selected.values()):
        x, y = (index % 3) * 300, (index // 3) * 290
        draw.text(
            (x + 5, y + 4),
            f"{request.id} try{request.attempt} PASS={row['numeric_pass']}",
            fill="#241a13",
        )
        image.thumbnail((280, 256), Image.Resampling.NEAREST)
        board.paste(image, (x + 8, y + 25), image)
    board.save(DATA / "contact_sheet.png")
    print(
        json.dumps(
            {
                "assets": len(selected),
                "attempts": len(evidence),
                "passed": sum(
                    row[2]["numeric_pass"] is True for row in selected.values()
                ),
                "failed": [
                    key
                    for key, row in selected.items()
                    if row[2]["numeric_pass"] is False
                ],
            }
        )
    )


if __name__ == "__main__":
    main()
