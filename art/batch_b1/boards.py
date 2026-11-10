# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy"]
# ///
# How to run: imported by python3 -m art.batch_b1.process
"""Exact-size and nearest 2x comparison against unchanged adopted people."""

from pathlib import Path

from PIL import Image, ImageDraw

from art.batch_b1.models import POSES, REPO, Summary


def review_board(trial: Path, summary: Summary) -> Path:
    """Draw 1x/2x candidate and old reference rows without modifying assets."""
    board = Image.new("RGB", (1140, 1140), (230, 226, 211))
    draw = ImageDraw.Draw(board)
    draw.text(
        (12, 12), f"B1 {summary.look} / structural: {summary.passed}", fill=(45, 35, 25)
    )
    sex = summary.look.split("_", 2)[1]
    hair = next(
        hair
        for hair in ("short", "long_tied", "bun")
        if summary.look.endswith("_" + hair)
    )
    reference = f"{summary.age}_{sex}_hide_wrap_{hair}"
    if not (REPO / "art/character/route_i" / reference / "S/idle.png").exists():
        reference = f"{summary.age}_{sex}_hide_wrap_bun"
    rows = [
        (1, "reference", "S"),
        (1, "candidate", "S"),
        (1, "candidate", "E"),
        (2, "reference", "S"),
        (2, "candidate", "S"),
        (2, "candidate", "E"),
    ]
    for row, (zoom, kind, facing) in enumerate(rows):
        top = 35 + row * 180
        draw.text(
            (12, top),
            f"{kind} {reference if kind == 'reference' else summary.look} {facing} {zoom}x",
            fill=(45, 35, 25),
        )
        poses = POSES if kind == "reference" else summary.frame_ids
        for column, pose in enumerate(poses):
            root = (
                REPO / "art/character/route_i" / reference
                if kind == "reference"
                else trial
            )
            path = root / facing / f"{pose}.png"
            left = 12 + column * 224
            draw.rectangle(
                (left, top + 18, left + 218, top + 172), fill=(117, 125, 100)
            )
            draw.text((left + 4, top + 20), pose, fill=(245, 243, 228))
            if path.exists():
                with Image.open(path) as source:
                    image = source.convert("RGBA")
                if zoom == 2:
                    image = image.resize(
                        (image.width * 2, image.height * 2), Image.Resampling.NEAREST
                    )
                # Whole canvas can exceed the row; the visible figure fits at its fixed anchor.
                anchor_y = (72 if pose == "sleep" else 104) * zoom
                board.paste(
                    image,
                    (left + (218 - image.width) // 2, top + 165 - anchor_y),
                    image,
                )
            else:
                draw.text(
                    (left + 4, top + 70), "MISSING / FAILED", fill=(255, 220, 180)
                )
    path = trial / "review.png"
    board.save(path)
    return path
