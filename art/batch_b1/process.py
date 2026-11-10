# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "scikit-image", "typer"]
# ///
# How to run: python3 -m art.batch_b1.process RECORD.json TRIAL_DIR
"""Normalize complete B1 wardrobe sheets without changing Route I gates."""

import hashlib
import json
import shutil
from dataclasses import asdict
from pathlib import Path

import typer
from PIL import Image

from art.batch_b1.boards import review_board
from art.batch_b1.models import FACINGS, POSES, Frame, Generation, Summary, digest, save
from art.pipeline5.judge_panels import judge_panels
from art.pipeline5.sheet_geometry import normalize_panel, sleep_body_scale, split_panels
from art.pipeline5.whole_checks import check_whole


def process(record_path: Path, trial: Path) -> Summary:
    """Produce external review evidence; adoption is a separate operation."""
    generation = Generation.model_validate_json(record_path.read_bytes())
    for reference in generation.references:
        if digest(reference.path) != reference.sha256:
            message = f"reference changed: {reference.path}"
            raise ValueError(message)
    raw_hash = digest(generation.raw)
    record_hash = digest(record_path)
    prompt_hash = hashlib.sha256(generation.prompt.encode()).hexdigest()
    trial.mkdir(parents=True, exist_ok=False)
    _ = shutil.copyfile(record_path, trial / "generation.json")
    _ = shutil.copyfile(generation.raw, trial / "raw.png")
    _ = (trial / "prompt.txt").write_text(generation.prompt)
    frames: list[Frame] = []
    error: str | None = None
    numeric_pass = False
    numeric_passing = 0
    numeric_hash: str | None = None
    try:
        with Image.open(trial / "raw.png") as source:
            if source.mode != "RGBA" or source.size != (1536, 1024):
                message = (
                    f"raw sheet requires 1536x1024 RGBA: {source.size} {source.mode}"
                )
                raise ValueError(message)
            panels = split_panels(source, rows=2)
        judges = judge_panels(
            trial / "raw.png", panels.images, trial / "judge", expected_count=10
        )
        numeric_pass = judges.numeric_pass
        numeric_passing = sum(panel.verdict.numeric_pass for panel in judges.panels)
        numeric_hash = digest(trial / "judge/panel_judge.json")
        target = {"child": 40, "adult": 56}[generation.age]
        scale = target / panels.idle_height
        ownership = {
            "standing_scale": scale,
            "idle_source_height": panels.idle_height,
            "source_boxes": panels.boxes,
            "ownership": asdict(panels.ownership),
        }
        _ = (trial / "ownership.json").write_text(
            json.dumps(ownership, indent=2) + "\n"
        )
        for index, panel in enumerate(panels.images):
            facing, pose = FACINGS[index // 5], generation.frame_ids[index % 5]
            check_pose = (
                POSES[index % 5]
                if generation.layout == "wardrobe"
                else "idle"
                if index % 5 == 0
                else "work_a"
            )
            sleep_scale = (
                sleep_body_scale(panels.images[index // 5 * 5 + 4], target)
                if generation.layout == "wardrobe"
                else scale
            )
            normalized = normalize_panel(
                panel, scale, pose == "sleep", sleep_length=target
            )
            destination = trial / facing / f"{pose}.png"
            destination.parent.mkdir(exist_ok=True)
            normalized.image.save(destination)
            frame = Frame(
                look=generation.look,
                age=generation.age,
                attempt=generation.attempt,
                raw=(trial / "raw.png").resolve(),
                raw_sha256=raw_hash,
                prompt_sha256=prompt_hash,
                facing=facing,
                pose=pose,
                check_pose=check_pose,
                path=f"{facing}/{pose}.png",
                sha256=digest(destination),
                source_box=panels.boxes[index],
                normalization=normalized.transform,
                checks=check_whole(normalized.image, generation.age, check_pose),
                standing_scale=scale,
                sleep_scale=sleep_scale,
                anchor=(56, 72) if pose == "sleep" else (40, 104),
            )
            save(destination.with_suffix(".json"), frame)
            frames.append(frame)
    except (ValueError, OSError) as failure:
        error = str(failure)
    result = Summary(
        look=generation.look,
        age=generation.age,
        attempt=generation.attempt,
        record_sha256=record_hash,
        raw_sha256=raw_hash,
        prompt_sha256=prompt_hash,
        passed=error is None
        and len(frames) == 10
        and all(frame.checks.passed for frame in frames),
        frames=tuple(frames),
        error=error,
        layout=generation.layout,
        frame_ids=generation.frame_ids,
        numeric_pass=numeric_pass,
        numeric_passing_panels=numeric_passing,
        numeric_report_sha256=numeric_hash,
    )
    save(trial / "summary.json", result)
    _ = review_board(trial, result)
    return result


def main(record: Path, trial: Path) -> None:
    """Normalize a generation record into a new external trial directory."""
    result = process(record, trial)
    typer.echo(result.model_dump_json())
    if not result.passed:
        raise typer.Exit(code=1)


if __name__ == "__main__":
    typer.run(main)
