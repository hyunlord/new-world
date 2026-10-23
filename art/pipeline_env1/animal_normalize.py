# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python3 -m art.pipeline_env1.animal_normalize
"""Normalize whole animals with route I ownership and explicit shoulder landmarks."""
from __future__ import annotations

import hashlib
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final, TypeAlias

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from art.pipeline3.structure import components
from art.pipeline5.sheet_geometry import Normalization, NormalizedPanel, split_panels

JsonValue: TypeAlias = str | int | float | bool | None | list["JsonValue"] | tuple["JsonValue", ...] | dict[str, "JsonValue"]

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/artenv1-data/animals")
OUT: Final = ROOT / "art/animal"


@dataclass(frozen=True, slots=True)
class Animal:
    species: str
    attempt: int
    shoulder_y: int
    hoof_y: int
    shoulder_target: int
    dead_length: int
    special_pose: str


ANIMALS: Final = (
    Animal("wolf", 3, 252, 474, 28, 56, "attack"),
    Animal("deer", 1, 194, 495, 40, 64, "graze"),
)


def digest(path: Path) -> str:
    """Bind evidence to exact source bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def normalize(panel: Image.Image, scale: float, dead: bool) -> NormalizedPanel:
    """Uniformly scale and register visible alpha; never edit in-canvas RGBA."""
    size = (max(1, round(panel.width * scale)), max(1, round(panel.height * scale)))
    resized = panel.resize(size, Image.Resampling.LANCZOS)
    alpha: NDArray[np.uint8] = np.asarray(resized)[:, :, 3]
    box = Image.fromarray(alpha >= 16).getbbox()
    if box is None:
        message = "Animal panel has no visible alpha"
        raise ValueError(message)
    canvas_size = (96, 64) if dead else (96, 80)
    baseline = 56 if dead else 72
    width, height = box[2] - box[0], box[3] - box[1]
    x, y = (96 - width) // 2, baseline - height + 1
    if x < 1 or y < 1 or x + width >= 96 or y + height >= canvas_size[1]:
        message = f"Visible animal does not fit without clipping: {width}x{height}"
        raise ValueError(message)
    dx, dy = x - box[0], y - box[1]
    discarded = alpha.copy()
    discarded[max(0, -dy):min(size[1], canvas_size[1] - dy),
              max(0, -dx):min(size[0], 96 - dx)] = 0
    canvas = Image.new("RGBA", canvas_size)
    canvas.paste(resized, (dx, dy))
    transform = Normalization(size, box, (x, y), (dx, dy),
                              int(np.count_nonzero(alpha >= 16)), 0, scale,
                              int(np.count_nonzero(discarded)), int(discarded.max()))
    return NormalizedPanel(canvas, transform)


def process(animal: Animal) -> None:
    """Record all checks before any model or independent readability acceptance."""
    raw = DATA / f"{animal.species}_try{animal.attempt}.png"
    prompt = raw.with_suffix(".prompt.txt")
    with Image.open(raw) as source:
        panels = split_panels(source, rows=2)
        source_alpha = np.asarray(source)[:, :, 3]
        source_border = int(sum(np.count_nonzero(e >= 128) for e in (
            source_alpha[0], source_alpha[-1], source_alpha[:, 0], source_alpha[:, -1])))
    scale = animal.shoulder_target / (animal.hoof_y - animal.shoulder_y)
    frames = ("stand", "walk", "run", animal.special_pose, "dead")
    records: list[dict[str, JsonValue]] = []
    for index, panel in enumerate(panels.images):
        facing, pose = ("S", "E")[index // 5], frames[index % 5]
        dead = pose == "dead"
        opaque = Image.fromarray(np.asarray(panel)[:, :, 3] >= 128).getbbox()
        if opaque is None:
            message = "Missing opaque animal"
            raise ValueError(message)
        actual_scale = animal.dead_length / max(opaque[2] - opaque[0], opaque[3] - opaque[1]) if dead else scale
        normalized = normalize(panel, actual_scale, dead)
        image = normalized.image
        alpha = np.asarray(image)[:, :, 3]
        visible = alpha >= 16
        ys, xs = np.nonzero(visible)
        component_count = len(components(alpha >= 128))
        canvas_ok = image.size == ((96, 64) if dead else (96, 80))
        contact_ok = int(ys.max()) == (56 if dead else 72)
        border_ok = not any(np.any(e) for e in (visible[0], visible[-1], visible[:, 0], visible[:, -1]))
        width, height = int(xs.max() - xs.min() + 1), int(ys.max() - ys.min() + 1)
        body_length_ok = not dead or animal.dead_length - 4 <= width <= animal.dead_length + 4
        axis_ok = not dead or width > height
        checks: dict[str, JsonValue] = {"canvas": canvas_ok, "contact": contact_ok, "border": border_ok,
                  "connectivity": component_count == 1, "dead_length": body_length_ok,
                  "dead_axis": axis_ok, "source_border": source_border == 0}
        directory = OUT / animal.species / facing
        directory.mkdir(parents=True, exist_ok=True)
        path = directory / f"{pose}.png"
        image.save(path)
        source_path = DATA / f"{animal.species}_try{animal.attempt}_{facing}_{pose}.png"
        panel.save(source_path)
        record: dict[str, JsonValue] = {"species": animal.species, "facing": facing, "pose": pose,
                  "attempt": animal.attempt, "image": str(path.relative_to(ROOT)),
                  "image_sha256": digest(path), "source_panel": str(source_path),
                  "source_panel_sha256": digest(source_path), "source_box": panels.boxes[index],
                  "raw_sha256": digest(raw), "prompt_sha256": digest(prompt),
                  "standing_scale": scale, "scale": actual_scale,
                  "normalization": asdict(normalized.transform), "checks": checks,
                  "component_count": component_count, "visible_width": width, "visible_height": height,
                  "structural_pass": all(checks.values()), "status": "CANDIDATE_PENDING_REVIEW"}
        _ = path.with_suffix(".json").write_text(json.dumps(record, indent=2) + "\n")
        records.append(record)
    summary = {"species": animal.species, "attempt": animal.attempt, "raw": str(raw),
               "raw_sha256": digest(raw), "prompt": str(prompt), "prompt_sha256": digest(prompt),
               "shoulder_landmarks": asdict(animal), "standing_scale": scale,
               "normalization": "route I ownership; one S-stand shoulder scale; dead body-length scale",
               "ownership": asdict(panels.ownership), "major_components": panels.original_major_components,
               "source_opaque_border_pixels": source_border,
               "structural_pass": all(r["structural_pass"] for r in records), "frames": records}
    _ = (OUT / animal.species / "MANIFEST.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps({"species": animal.species, "structural_pass": summary["structural_pass"],
                      "dimensions": [(r["facing"], r["pose"], r["visible_width"], r["visible_height"],
                                      r["component_count"]) for r in records]}))


if __name__ == "__main__":
    for animal_specification in ANIMALS:
        process(animal_specification)
