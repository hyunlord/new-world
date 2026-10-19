"""Compare six RGBA capture triplets without weakening the control bound.

Each input directory contains P1.png through P6.png and matching JSON sidecars
with capture_frame, state_change_frame, tick, seed and fixed_fps fields. P3's
FV manifest row must be normalized to that schema by the evidence collector.
"""
from __future__ import annotations

import hashlib
import json
from dataclasses import asdict, dataclass
from functools import reduce
from pathlib import Path
from typing import Final

import typer
from PIL import Image, ImageChops
from pydantic import BaseModel, ConfigDict, StrictInt

FRAMINGS: Final = ("P1", "P2", "P3", "P4", "P5", "P6")


@dataclass(frozen=True, slots=True)
class Difference:
    """RGBA difference measured in pixels and unsigned channel levels."""

    pixels: int
    max_channel: int


class CaptureStamp(BaseModel):
    """Strict boundary schema for renderer-produced metadata."""

    model_config = ConfigDict(frozen=True)
    capture_frame: StrictInt
    state_change_frame: StrictInt
    tick: StrictInt
    seed: StrictInt
    fixed_fps: StrictInt


@dataclass(frozen=True, slots=True)
class Capture:
    """Image and deterministic capture provenance."""

    path: Path
    capture_frame: int
    state_change_frame: int
    tick: int
    seed: int
    fixed_fps: int

    @classmethod
    def read(cls, path: Path) -> Capture:
        """Parse the five integer capture invariants at the file boundary."""
        stamp = CaptureStamp.model_validate_json(path.with_suffix(".json").read_text())
        capture = cls(path, stamp.capture_frame, stamp.state_change_frame, stamp.tick, stamp.seed, stamp.fixed_fps)
        if capture.fixed_fps != 60 or capture.capture_frame - capture.state_change_frame != 120:
            raise typer.BadParameter(f"Invalid fixed-fps/120-frame schedule: {path}")
        return capture

    def schedule(self) -> tuple[int, int, int, int, int]:
        return (self.capture_frame, self.state_change_frame, self.tick, self.seed, self.fixed_fps)


@dataclass(frozen=True, slots=True)
class Result:
    """One framing's pixel identity verdict."""

    framing: str
    control: Difference
    candidate: Difference
    bit_reproducible: bool
    passed: bool
    cause: str
    rgba_sha256: tuple[str, str, str]
    capture_frame: int
    state_change_frame: int


def difference(left: Image.Image, right: Image.Image) -> tuple[Difference, Image.Image]:
    """Include alpha-only changes, which ordinary RGBA getbbox can overlook."""
    if left.size != right.size:
        raise typer.BadParameter(f"Image dimensions differ: {left.size} versus {right.size}")
    delta = ImageChops.difference(left.convert("RGBA"), right.convert("RGBA"))
    maximum = reduce(ImageChops.lighter, delta.split())
    histogram = maximum.histogram()
    metric = Difference(left.width * left.height - histogram[0], max(maximum.getextrema()))
    return metric, maximum


def accepts(control: Difference, candidate: Difference) -> bool:
    """Both independent bounds must hold, including the exact-control case."""
    return candidate.pixels <= control.pixels and candidate.max_channel <= control.max_channel


def compare(captures: tuple[Capture, Capture, Capture], output: Path) -> Result:
    """Write full-resolution differences and downscaled review previews."""
    baseline, repeat, candidate = captures
    if baseline.schedule() != repeat.schedule() or baseline.schedule() != candidate.schedule():
        raise typer.BadParameter(f"Capture schedules differ for {baseline.path.stem}")
    with Image.open(baseline.path) as a, Image.open(repeat.path) as ap, Image.open(candidate.path) as b:
        control, control_diff = difference(a, ap)
        changed, candidate_diff = difference(a, b)
        framing = baseline.path.stem
        for label, image in (("control-diff", control_diff), ("candidate-diff", candidate_diff)):
            image.save(output / f"{framing}-{label}.png")
            preview = image.point(lambda value: min(255, value * 16)).convert("RGB")
            preview.thumbnail((640, 360))
            preview.save(output / "previews" / f"{framing}-{label}-x16.png")
        for label, image in (("A", a), ("Aprime", ap), ("B", b)):
            preview = image.convert("RGB")
            preview.thumbnail((640, 360))
            preview.save(output / "previews" / f"{framing}-{label}.png")
        hashes = tuple(hashlib.sha256(image.convert("RGBA").tobytes()).hexdigest() for image in (a, ap, b))
    cause = "bit-reproducible" if control.pixels == 0 else "not bit-reproducible; cause unresolved, inspect control diff"
    passed = accepts(control, changed)
    if framing == "P5" and (control.pixels != 0 or changed.pixels != 0):
        passed = False
        cause = "P5 requires exact A/A′ and A/B RGBA identity; inspect preserved differences"
    return Result(framing, control, changed, control.pixels == 0, passed, cause,
                  (hashes[0], hashes[1], hashes[2]), baseline.capture_frame, baseline.state_change_frame)


# Four explicit paths are the CLI contract: three independently captured runs and one output.
def main(a: Path, aprime: Path, b: Path, output: Path) -> None:
    """Require all six controls; write machine and human-readable verdicts."""
    # A fresh output directory preserves rejected runs and prevents stale PASS reports.
    output.mkdir(parents=True, exist_ok=False)
    (output / "previews").mkdir(exist_ok=True)
    results = []
    for framing in FRAMINGS:
        captures = tuple(Capture.read(folder / f"{framing}.png") for folder in (a, aprime, b))
        results.append(compare((captures[0], captures[1], captures[2]), output))
    (output / "comparison.json").write_text(json.dumps({"schema_version": 1, "passed": all(row.passed for row in results), "framings": [asdict(row) for row in results]}, indent=2) + "\n")
    lines = ["# Art Pass 11 pixel identity", "", "RGBA comparison; both differing-pixel count and maximum channel difference must be no greater than the A/A′ control. P5 additionally requires zero differing pixels in both A/A′ and A/B.", "", "| Framing | A/A′ pixels | A/A′ max | A/B pixels | A/B max | Result | Reproducibility |", "|---|---:|---:|---:|---:|---|---|"]
    for row in results:
        lines.append(f"| {row.framing} | {row.control.pixels} | {row.control.max_channel} | {row.candidate.pixels} | {row.candidate.max_channel} | {'PASS' if row.passed else 'FAIL'} | {row.cause} |")
    lines.extend(["", "Previews are downscaled for review only. Metrics use full-resolution RGBA; diff images show the maximum absolute channel difference, with x16 previews.", ""])
    (output / "comparison.md").write_text("\n".join(lines))
    raise typer.Exit(0 if all(row.passed for row in results) else 1)


if __name__ == "__main__":
    typer.run(main)
