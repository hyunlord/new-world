# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "typer"]
# ///
# How to run: python3 -m art.batch_b1.adopt TRIAL_DIR VERDICT.json
"""Copy a whole reviewed look only after current-byte verification."""

import shutil
from pathlib import Path

import typer
from PIL import Image

from art.batch_b1.models import (
    FACINGS,
    REPO,
    Frame,
    Generation,
    Summary,
    VisualVerdict,
    digest,
)
from art.pipeline5.whole_checks import check_whole


def adopt(
    trial: Path, verdict_path: Path, root: Path = REPO / "art/character/batch_b1"
) -> Path:
    """Require complete structural and visual acceptance of the exact trial."""
    summary = Summary.model_validate_json((trial / "summary.json").read_bytes())
    review = VisualVerdict.model_validate_json(verdict_path.read_bytes())
    if review.summary_sha256 != digest(trial / "summary.json"):
        message = "visual review does not identify this summary"
        raise ValueError(message)
    if review.score < 90 or review.verdict != "pass" or not review.category_match:
        message = "visual review must pass at 90 or above"
        raise ValueError(message)
    expected = {(facing, pose) for facing in FACINGS for pose in summary.frame_ids}
    if (
        not summary.passed
        or len(summary.frames) != 10
        or {(frame.facing, frame.pose) for frame in summary.frames} != expected
    ):
        message = "structural acceptance requires the complete ten-frame sheet"
        raise ValueError(message)
    generation = Generation.model_validate_json(
        (trial / "generation.json").read_bytes()
    )
    if (
        digest(trial / "generation.json") != summary.record_sha256
        or generation.look != summary.look
    ):
        message = "generation record changed"
        raise ValueError(message)
    if (
        digest(trial / "raw.png") != summary.raw_sha256
        or digest(trial / "prompt.txt") != summary.prompt_sha256
    ):
        message = "source or prompt changed"
        raise ValueError(message)
    for frame in summary.frames:
        path = trial / f"{frame.facing}/{frame.pose}.png"
        if (
            frame.path != f"{frame.facing}/{frame.pose}.png"
            or digest(path) != frame.sha256
        ):
            message = f"frame bytes changed: {frame.path}"
            raise ValueError(message)
        sidecar = Frame.model_validate_json(path.with_suffix(".json").read_bytes())
        with Image.open(path) as image:
            checked = check_whole(image, summary.age, frame.check_pose)
        if sidecar != frame or not checked.passed or checked != frame.checks:
            message = f"structural or sidecar disagreement: {frame.path}"
            raise ValueError(message)
    destination = root / summary.look
    if destination.exists():
        message = f"delivery already exists; preserve prior adoption before replacement: {destination}"
        raise FileExistsError(message)
    destination.mkdir(parents=True)
    for frame in summary.frames:
        target = destination / frame.path
        target.parent.mkdir(exist_ok=True)
        _ = shutil.copyfile(trial / frame.path, target)
        _ = shutil.copyfile(
            (trial / frame.path).with_suffix(".json"), target.with_suffix(".json")
        )
    _ = shutil.copyfile(trial / "summary.json", destination / "summary.json")
    _ = shutil.copyfile(verdict_path, destination / "visual_verdict.json")
    return destination


def main(
    trial: Path, verdict: Path, root: Path = REPO / "art/character/batch_b1"
) -> None:
    """Adopt all ten frames of one fully reviewed trial."""
    typer.echo(str(adopt(trial, verdict, root)))


if __name__ == "__main__":
    typer.run(main)
