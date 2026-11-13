"""Normalize generated iron workstation candidates through unchanged gates."""

from __future__ import annotations

import hashlib
import json
from dataclasses import asdict
from pathlib import Path
from typing import ClassVar, Final, Literal, Self

from PIL import Image
from pydantic import BaseModel, ConfigDict, Field, model_validator

from art.compositor.projection_check import ProjectionSpec, check_projection
from art.pipeline2.judge import judge, load_calibration
from art.pipeline_env1.items_records import read_json
from art.pipeline_struct.normalize import normalize_sprite

ROOT: Final = Path(__file__).resolve().parents[3]
OUT: Final = ROOT / "art/batch_c3a/workshops"
DATA: Final = Path("/Users/rexxa/work/art-batch/C3A")
Point = tuple[int, int]


def sha(data: bytes) -> str:
    """Hash original bytes."""
    return hashlib.sha256(data).hexdigest()


class Raw(BaseModel):
    """Read only generation fields relevant to deterministic normalization."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: Literal["anvil", "mason_bench"]
    raw: Path
    attempt: int = Field(ge=1, le=6)
    prompt: str

    @model_validator(mode="after")
    def attempt_budget(self) -> Self:
        """BRIEF4 grants only the anvil three additional attempts."""
        if self.attempt > {"anvil": 6, "mason_bench": 3}[self.id]:
            message = f"Attempt budget exceeded for {self.id}"
            raise ValueError(message)
        return self


def station(asset: str, height: int, footprint: Point) -> bool:
    """Leave numeric failures outside adoption, preserving the original alpha."""
    maximum = {"anvil": 6, "mason_bench": 3}[asset]
    records = sorted(DATA.glob(f"{asset}_try[1-{maximum}].json"))
    record = records[-1] if records else DATA / f"{asset}_try1.json"
    if not record.exists():
        return False
    row = Raw.model_validate_json(record.read_bytes())
    if row.id != asset:
        message = f"Record id {row.id} does not match station {asset}"
        raise ValueError(message)
    with Image.open(row.raw) as source:
        normalized = normalize_sprite(source, height)
    target = OUT / "stations" / f"{asset}.png"
    target.parent.mkdir(parents=True, exist_ok=True)
    numeric = judge(
        row.raw,
        load_calibration(ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json"),
        family="buildings",
    )
    projection = check_projection(
        normalized.image, ProjectionSpec(asset, normalized.anchor, (height, height))
    )
    ready = numeric.numeric_pass and projection.passed
    candidates = DATA / "candidates/workshops"
    candidates.mkdir(parents=True, exist_ok=True)
    normalized.image.save(candidates / f"{asset}_try{row.attempt}.png")
    evidence = {
        "id": asset,
        "era": "iron",
        "footprint": footprint,
        "anchor": normalized.anchor,
        "canvas": normalized.image.size,
        "target_height": height,
        "raw_sha256": sha(row.raw.read_bytes()),
        "record_sha256": sha(record.read_bytes()),
        "calibration_sha256": sha(
            (ROOT / "docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json").read_bytes()
        ),
        "operations": [
            "existing normalize_sprite:opaque alpha bounds+2px fringe crop",
            "uniform Lanczos height fit",
            "4px transparent padding; anchor opaque bottom",
        ],
        "generation": read_json(record),
        "numeric": asdict(numeric),
        "projection": asdict(projection),
        "projection_band_reason": "New station family target declared in C3A before generation; bench30px/anvil28px, adult56px",
        "status": "CANDIDATE_VISUAL_REVIEW_PENDING"
        if ready
        else "HELD_NUMERIC_OR_PROJECTION_FAILURE",
    }
    if ready:
        normalized.image.save(target)
        evidence["sha256"] = sha(target.read_bytes())
    else:
        target.unlink(missing_ok=True)
    checks = DATA / "checks/workshops"
    checks.mkdir(parents=True, exist_ok=True)
    _ = (checks / f"{asset}_try{row.attempt}.json").write_text(
        json.dumps(evidence, indent=2) + "\n"
    )
    _ = target.with_suffix(".json").write_text(json.dumps(evidence, indent=2) + "\n")
    return ready
