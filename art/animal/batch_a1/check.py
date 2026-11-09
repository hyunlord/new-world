# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "pydantic"]
# ///
# How to run: python3 -m art.animal.batch_a1.check MANIFEST.json [MANIFEST.json ...]
"""Recompute A1 structure and reproduce normalized RGBA from recorded source panels."""
from __future__ import annotations

import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import ClassVar, Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, JsonValue, model_validator

from art.pipeline3.structure import components
from art.pipeline_env1.animal_normalize import digest, normalize

ROOT: Final = Path(__file__).resolve().parents[3]


class Spec(BaseModel):
    """Parse one measured generation, including optional tool evidence."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    species: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    attempt: int = Field(gt=0)
    raw: Path
    prompt: Path
    shoulder_y: int = Field(ge=0)
    hoof_y: int = Field(gt=0)
    shoulder_target: int = Field(gt=0, le=64)
    dead_length: int = Field(gt=0, le=90)
    poses: tuple[str, ...] = ("stand", "walk", "run", "graze", "dead")
    references: tuple[Path, ...] = Field(min_length=1)
    seed: int | None = None
    generator_record: Path | None = None
    landmark_note: str = "Manually measured S/stand withers and hoof in source coordinates."

    @model_validator(mode="after")
    def geometry(self) -> Spec:
        """Reject ambiguous grids and scale before touching output files."""
        valid_grid = (len(self.poses) == 5 and self.poses[0] == "stand"
                      and self.poses[-1] == "dead") or self.poses == ("attack",)
        if (not valid_grid or len(set(self.poses)) != len(self.poses)
                or any(p not in {"stand", "walk", "run", "graze", "attack", "dead"} for p in self.poses)
                or self.hoof_y <= self.shoulder_y):
            message = "Need 5 distinct poses from stand to dead, or one attack pair, and positive shoulder span"
            raise ValueError(message)
        if self.raw.resolve().is_relative_to(ROOT) or self.prompt.resolve().is_relative_to(ROOT):
            message = "Raw generation and prompt evidence must live outside the repository"
            raise ValueError(message)
        return self



def split_attack(source: Image.Image) -> tuple[tuple[Image.Image, ...], tuple[tuple[int, int, int, int], ...]]:
    """Crop an S/E pair across an alpha<16 gutter, preserving even invisible RGBA."""
    middle = source.width // 2
    alpha = np.asarray(source)[:, :, 3]
    if source.width < 4 or np.any(alpha[:, middle - 1:middle + 1] >= 16):
        message = "Attack pair requires no visible alpha>=16 in the two-column center gutter"
        raise ValueError(message)
    boxes = ((0, 0, middle, source.height), (middle, 0, source.width, source.height))
    return tuple(source.crop(box) for box in boxes), boxes



@dataclass(frozen=True, slots=True)
class Checks:
    """All fields are mandatory structural conditions, not visual approvals."""

    canvas: bool
    alpha: bool
    contact: bool
    border: bool
    connectivity: bool
    dead_length: bool
    dead_axis: bool


def inspect_frame(image: Image.Image, dead: bool, dead_length: int) -> Checks:
    """Check alpha, registration and anatomy connectivity without altering pixels."""
    expected = (96, 64) if dead else (96, 80)
    alpha: NDArray[np.uint8] = np.asarray(image.convert("RGBA"))[:, :, 3]
    visible: NDArray[np.bool_] = alpha >= 16
    opaque: NDArray[np.bool_] = alpha >= 128
    transparent: NDArray[np.bool_] = np.equal(alpha, 0)
    box = Image.fromarray(visible).getbbox()
    width = box[2] - box[0] if box is not None else 0
    height = box[3] - box[1] if box is not None else 0
    return Checks(
        image.size == expected,
        image.mode == "RGBA" and box is not None and bool(np.any(transparent)) and bool(np.any(opaque)),
        box is not None and box[3] - 1 == (56 if dead else 72),
        not any(np.any(edge) for edge in (visible[0], visible[-1], visible[:, 0], visible[:, -1])),
        len(components(opaque)) == 1,
        not dead or dead_length - 4 <= width <= dead_length + 4,
        not dead or width > height,
    )


class Reference(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    path: Path
    sha256: str


class Frame(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    image: Path
    image_sha256: str
    facing: str
    pose: str
    source_panel: Path
    source_panel_sha256: str
    canvas_px: tuple[int, int]
    anchor_px: tuple[int, int]
    footprint_tiles: tuple[int, int]
    scale: float
    checks: dict[str, bool]
    structural_pass: bool


class Manifest(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    spec: dict[str, JsonValue]
    spec_path: Path
    spec_sha256: str
    raw_sha256: str
    prompt_sha256: str
    references: tuple[Reference, ...]
    generator_record: Reference | None = None
    ownership: dict[str, JsonValue]
    frames: tuple[Frame, ...]
    frame_count: int
    structural_pass: bool
    contact_sheet: Path
    contact_sheet_sha256: str


def verify(path: Path) -> bool:
    """Verify nonempty complete frame grids, exact bytes and source reconstruction."""
    from art.pipeline5.sheet_geometry import split_panels

    document = Manifest.model_validate_json(path.read_text())
    spec = Spec.model_validate(document.spec)
    failures: list[str] = []
    references = document.references + ((document.generator_record,) if document.generator_record is not None else ())
    for source, expected in ((document.spec_path, document.spec_sha256), (spec.raw, document.raw_sha256),
                             (spec.prompt, document.prompt_sha256),
                             (ROOT / document.contact_sheet, document.contact_sheet_sha256),
                             *((ref.path, ref.sha256) for ref in references)):
        if digest(source) != expected:
            failures.append(f"hash mismatch: {source}")
    expected_grid = {(f, p) for f in ("S", "E") for p in spec.poses}
    actual_grid = {(f.facing, f.pose) for f in document.frames}
    if not document.frames or document.frame_count != len(expected_grid) or len(document.frames) != len(expected_grid) or actual_grid != expected_grid:
        failures.append("frame grid is incomplete or duplicated")
    with Image.open(spec.raw) as source:
        raw_alpha = np.asarray(source)[:, :, 3]
        if source.mode != "RGBA" or any(np.any(e >= 128) for e in (raw_alpha[0], raw_alpha[-1], raw_alpha[:, 0], raw_alpha[:, -1])):
            failures.append("source RGBA/border check failed")
        panels = split_attack(source)[0] if spec.poses == ("attack",) else split_panels(source, rows=2).images
    for frame, original in zip(document.frames, panels, strict=True):
        image_path = ROOT / frame.image
        sidecar = Frame.model_validate_json(image_path.with_suffix(".json").read_text())
        with Image.open(image_path) as image, Image.open(frame.source_panel) as panel:
            checks: dict[str, bool] = asdict(inspect_frame(image, frame.pose == "dead", spec.dead_length))
            reproduced = normalize(panel, frame.scale, frame.pose == "dead")
            same = np.array_equal(np.asarray(image), np.asarray(reproduced.image))
            ownership = np.array_equal(np.asarray(panel), np.asarray(original))
            metadata = image.size == frame.canvas_px and frame.anchor_px == (48, 56 if frame.pose == "dead" else 72)
            opaque_box = Image.fromarray(np.asarray(panel)[:, :, 3] >= 128).getbbox()
            expected_scale = spec.shoulder_target / (spec.hoof_y - spec.shoulder_y)
            if frame.pose == "dead" and opaque_box is not None:
                expected_scale = spec.dead_length / max(opaque_box[2] - opaque_box[0], opaque_box[3] - opaque_box[1])
        if (not all(checks.values()) or any(frame.checks.get(k) != v for k, v in checks.items())
                or not all(frame.checks.values()) or not frame.structural_pass
                or not same or not ownership or not metadata or frame.footprint_tiles != (1, 1)
                or sidecar != frame or frame.scale != expected_scale or reproduced.transform.discarded_max_alpha >= 16
                or digest(image_path) != frame.image_sha256 or digest(frame.source_panel) != frame.source_panel_sha256):
            failures.append(f"frame failed structure, provenance or RGBA reproduction: {frame.image}")
    if not document.structural_pass or document.ownership.get("reconstructed_exactly") is not True or document.ownership.get("foreign_major_pixels") != 0:
        failures.append("manifest records failed structure or ownership")
    print(json.dumps({"manifest": str(path), "passed": not failures, "frames": len(document.frames), "failures": failures}))
    return not failures


def main() -> int:
    """Exit nonzero for any failed manifest; never pass an empty invocation."""
    if len(sys.argv) < 2:
        print("Usage: python3 -m art.animal.batch_a1.check MANIFEST.json [MANIFEST.json ...]", file=sys.stderr)
        return 2
    results = [verify(Path(arg)) for arg in sys.argv[1:]]
    return 0 if all(results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
