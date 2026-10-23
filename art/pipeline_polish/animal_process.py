# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image", "pydantic"]
# ///
# How to run: PYTHONPATH=. python -m art.pipeline_polish.animal_process
"""Preserve A1 sheet ownership and report POLISH animal structure and style."""
from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image
from pydantic import JsonValue, TypeAdapter

from art.pipeline2.judge import judge, load_calibration
from art.pipeline3.structure import components
from art.pipeline5.sheet_geometry import split_panels
from art.pipeline_env1.animal_normalize import digest, normalize

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/artpolish-data/animals")
DOCS: Final = ROOT / "docs/v8/artpolish"
OUT: Final = ROOT / "art/animal/polish"


@dataclass(frozen=True, slots=True)
class AnimalSpec:
    species: str
    attempt: int
    shoulder_y: int
    hoof_y: int
    shoulder_target: int
    dead_length: int
    special_pose: str


ROUND0_SPECS: Final = (
    AnimalSpec("wolf", 1, 247, 454, 28, 56, "attack"),
    AnimalSpec("deer", 1, 250, 486, 40, 64, "graze"),
)
SPECS: Final = (
    AnimalSpec("wolf", 2, 236, 440, 28, 56, "attack"),
    AnimalSpec("deer", 2, 258, 486, 40, 64, "graze"),
)


def write_json(path: Path, value: JsonValue) -> None:
    """Write reviewable evidence without changing previous pass records."""
    path.parent.mkdir(parents=True, exist_ok=True)
    _ = path.write_text(json.dumps(value, indent=2) + "\n")


def process(spec: AnimalSpec) -> None:
    """Use one measured shoulder scale for all upright frames, never fit poses."""
    raw = DATA / f"{spec.species}_try{spec.attempt}.png"
    with Image.open(raw) as source:
        panels = split_panels(source, rows=2)
        source_alpha = np.asarray(source)[:, :, 3]
        border = sum(int(np.count_nonzero(edge >= 128)) for edge in (
            source_alpha[0], source_alpha[-1], source_alpha[:, 0], source_alpha[:, -1],
        ))
    calibration_path = ROOT / "docs/v8/artenv1/animal_judge_calibration.json"
    calibration = load_calibration(calibration_path)
    upright_scale = spec.shoulder_target / (spec.hoof_y - spec.shoulder_y)
    poses = ("stand", "walk", "run", spec.special_pose, "dead")
    records: list[JsonValue] = []
    passes: list[bool] = []
    numeric: list[bool] = []
    for index, panel in enumerate(panels.images):
        facing, pose = ("S", "E")[index // 5], poses[index % 5]
        dead = pose == "dead"
        opaque = Image.fromarray(np.asarray(panel)[:, :, 3] >= 128).getbbox()
        if opaque is None:
            message = f"Missing animal in {raw}: {facing}/{pose}"
            raise ValueError(message)
        scale = spec.dead_length / max(opaque[2] - opaque[0], opaque[3] - opaque[1]) if dead else upright_scale
        product = normalize(panel, scale, dead)
        alpha = np.asarray(product.image)[:, :, 3]
        visible = alpha >= 16
        ys, xs = np.nonzero(visible)
        width, height = int(xs.max() - xs.min() + 1), int(ys.max() - ys.min() + 1)
        component_count = len(components(alpha >= 128))
        checks = {
            "canvas": product.image.size == ((96, 64) if dead else (96, 80)),
            "contact": int(ys.max()) == (56 if dead else 72),
            "border": not any(np.any(e) for e in (visible[0], visible[-1], visible[:, 0], visible[:, -1])),
            "connectivity": component_count == 1,
            "dead_length": not dead or spec.dead_length - 4 <= width <= spec.dead_length + 4,
            "dead_axis": not dead or width > height,
            "source_border": border == 0,
            "ownership": panels.ownership.reconstructed_exactly and panels.ownership.foreign_major_pixels == 0,
            "discarded_alpha": product.transform.discarded_max_alpha < 16,
        }
        target = OUT / spec.species / f"try{spec.attempt}" / facing / f"{pose}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        product.image.save(target, optimize=True)
        native = DATA / f"{spec.species}_try{spec.attempt}_{facing}_{pose}.png"
        panel.save(native)
        verdict = judge(native, calibration, family="animal")
        record = TypeAdapter[JsonValue](JsonValue).validate_json(json.dumps({
            "species": spec.species, "facing": facing, "pose": pose, "attempt": spec.attempt,
            "image": str(target.relative_to(ROOT)), "image_sha256": digest(target),
            "source_panel": str(native), "source_panel_sha256": digest(native),
            "source_box": list(panels.boxes[index]), "raw": str(raw), "raw_sha256": digest(raw),
            "scale": scale, "upright_scale": upright_scale,
            "normalization": asdict(product.transform), "checks": checks,
            "component_count": component_count, "visible_width": width, "visible_height": height,
            "structural_pass": all(checks.values()), "numeric_v2_report_only": asdict(verdict),
            "status": "CANDIDATE_NOT_OWNER_ADOPTED",
        }))
        write_json(target.with_suffix(".json"), record)
        records.append(record)
        passes.append(all(checks.values()))
        numeric.append(verdict.numeric_pass)
    summary: JsonValue = {
        "species": spec.species, "attempt": spec.attempt, "raw": str(raw), "raw_sha256": digest(raw),
        "prompt": str(raw.with_suffix(".prompt.txt")), "prompt_sha256": digest(raw.with_suffix(".prompt.txt")),
        "landmarks": asdict(spec), "landmark_method": "Manual source S-stand withers/back plane and lowest grounded hoof; source y coordinates",
        "upright_scale": upright_scale, "ownership": asdict(panels.ownership),
        "major_components": panels.original_major_components, "source_opaque_border_pixels": border,
        "calibration": str(calibration_path.relative_to(ROOT)), "calibration_sha256": digest(calibration_path),
        "structural_pass": all(passes), "numeric_pass_frames": sum(numeric), "numeric_gates": False,
        "style_policy": "D1: numeric v2 and image-model style scorecard are reported, never gates",
        "facing_rule": "S: front three-quarter 35 degrees toward viewer left; E: right-facing side three-quarter",
        "frames": records,
    }
    write_json(OUT / spec.species / f"try{spec.attempt}" / "MANIFEST.json", summary)
    write_json(DOCS / f"animal_{spec.species}_try{spec.attempt}.json", summary)
    print(f"{spec.species} try{spec.attempt}: structure {sum(passes)}/10; numeric report {sum(numeric)}/10")


if __name__ == "__main__":
    for specification in SPECS:
        process(specification)
