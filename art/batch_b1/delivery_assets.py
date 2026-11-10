# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "typer"]
# ///
# How to run: imported by python3 -m art.batch_b1.delivery assemble
"""Preflight the entire reviewed family before copying baseline bytes."""

from collections.abc import Sequence
from pathlib import Path

from PIL import Image

from art.batch_b1.delivery_models import Catalog, CopyPair, CopyRow, FileRow
from art.batch_b1.models import FACINGS, POSES, Frame, Summary, VisualVerdict, digest
from art.batch_b1.portraits import VARIANTS, Portrait, PortraitSummary
from art.pipeline5.whole_checks import check_whole


def plan_copies(pairs: Sequence[CopyPair]) -> tuple[CopyRow, ...]:
    """Reject missing sources or conflicting destinations before any write."""
    rows: list[CopyRow] = []
    destinations: set[Path] = set()
    for pair in pairs:
        sha = digest(pair.source)
        if pair.destination in destinations:
            raise ValueError(f"duplicate destination: {pair.destination}")
        destinations.add(pair.destination)
        existed = pair.destination.exists()
        if existed and digest(pair.destination) != sha:
            raise FileExistsError(pair.destination)
        rows.append(
            CopyRow(
                source=pair.source,
                destination=pair.destination,
                sha256=sha,
                existed=existed,
            )
        )
    return tuple(rows)


def reviewed(directory: Path) -> None:
    """Bind visual acceptance to the summary currently on disk."""
    review = VisualVerdict.model_validate_json(
        (directory / "visual_verdict.json").read_bytes()
    )
    if (
        review.summary_sha256 != digest(directory / "summary.json")
        or review.score < 90
        or review.verdict != "pass"
        or not review.category_match
    ):
        raise ValueError(f"missing current passing visual review: {directory}")


def collect_new(root: Path, catalog: Catalog) -> tuple[FileRow, ...]:
    """Recheck each adopted PNG and sidecar; missing coverage is an error."""
    rows: list[FileRow] = []
    if len({look.id for look in catalog.looks}) != catalog.expected_new_looks:
        raise ValueError("catalog look count or uniqueness mismatch")
    for look in catalog.looks:
        directory = root / "art/character/batch_b1" / look.id
        reviewed(directory)
        summary = Summary.model_validate_json((directory / "summary.json").read_bytes())
        expected = {(f, p) for f in FACINGS for p in POSES}
        if (
            summary.look != look.id
            or summary.age != look.age
            or not summary.passed
            or summary.layout != "wardrobe"
            or summary.frame_ids != POSES
            or len(summary.frames) != 10
            or {(f.facing, f.pose) for f in summary.frames} != expected
        ):
            raise ValueError(f"incomplete or mismatched frameset: {look.id}")
        for frame in summary.frames:
            path = directory / f"{frame.facing}/{frame.pose}.png"
            if (
                frame.path != f"{frame.facing}/{frame.pose}.png"
                or frame.look != look.id
                or frame.age != look.age
                or frame.check_pose != frame.pose
                or frame.anchor != ((56, 72) if frame.pose == "sleep" else (40, 104))
                or frame.footprint != (1, 1)
                or frame.template_version != 3
                or digest(path) != frame.sha256
                or Frame.model_validate_json(path.with_suffix(".json").read_bytes())
                != frame
            ):
                raise ValueError(f"frame hash/identity/sidecar mismatch: {path}")
            with Image.open(path) as image:
                checked = check_whole(image, summary.age, frame.check_pose)
            if not checked.passed or checked != frame.checks:
                raise ValueError(f"frame structural disagreement: {path}")
            rows.append(
                FileRow(
                    source=path.relative_to(root).as_posix(),
                    sha256=frame.sha256,
                    role="people_frame",
                    look=look.id,
                )
            )
        directory = root / "art/portrait/batch_b1" / look.id
        reviewed(directory)
        portraits = PortraitSummary.model_validate_json(
            (directory / "summary.json").read_bytes()
        )
        if (
            portraits.look != look.id
            or not portraits.passed
            or tuple(p.variant for p in portraits.portraits) != VARIANTS
        ):
            raise ValueError(f"incomplete or mismatched portrait family: {look.id}")
        for portrait in portraits.portraits:
            path = directory / f"{portrait.variant}.png"
            if (
                portrait.path != path.name
                or portrait.canvas != (128, 128)
                or portrait.anchor != (64, 64)
                or digest(path) != portrait.sha256
                or Portrait.model_validate_json(path.with_suffix(".json").read_bytes())
                != portrait
            ):
                raise ValueError(f"portrait hash/sidecar mismatch: {path}")
            with Image.open(path) as image:
                if (
                    image.mode != "RGBA"
                    or image.size != (128, 128)
                    or image.getchannel("A").getextrema() != (255, 255)
                ):
                    raise ValueError(f"portrait canvas/alpha mismatch: {path}")
            rows.append(
                FileRow(
                    source=path.relative_to(root).as_posix(),
                    sha256=portrait.sha256,
                    role="portrait",
                    look=look.id,
                )
            )
    if (
        sum(r.role == "people_frame" for r in rows) != catalog.expected_new_frames
        or sum(r.role == "portrait" for r in rows) != catalog.expected_new_portraits
    ):
        raise ValueError("catalog frame/portrait count mismatch")
    return tuple(rows)
