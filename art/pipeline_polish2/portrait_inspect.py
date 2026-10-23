# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m art.pipeline_polish2.portrait_inspect
"""Make coordinate-labelled inspection sheets; never infer observed bounding boxes."""
from __future__ import annotations

from typing import Final

from PIL import Image, ImageDraw

from art.pipeline_polish2.portrait_models import DATA, CandidateSet

PAPER: Final = "#e5dcc8"


def main() -> None:
    candidates = CandidateSet.model_validate_json((DATA / "candidates.json").read_bytes())
    destination = DATA / "inspection/packets"
    destination.mkdir(parents=True, exist_ok=True)
    for look in sorted({row.look for row in candidates.candidates}):
        rows = sorted((row for row in candidates.candidates if row.look == look), key=lambda row: row.variant)
        board = Image.new("RGB", (6 * 280, 470), PAPER)
        draw = ImageDraw.Draw(board)
        draw.text((8, 4), look, fill="#33271d")
        for index, row in enumerate(rows):
            row.portrait.verify()
            x = index * 280
            with Image.open(row.portrait.path) as source:
                board.paste(source, (x + 76, 24))
                board.paste(source.resize((256, 256), Image.Resampling.NEAREST), (x + 20, 190))
            draw.text((x + 76, 156), f"{row.variant} try{row.attempt} / 128px", fill="#33271d")
            for coordinate in range(0, 129, 16):
                draw.text((x + 12 + coordinate * 2, 174), str(coordinate), fill="#33271d")
                draw.text((x + 1, 184 + coordinate * 2), str(coordinate), fill="#33271d")
            draw.ellipse((x + 60, 208, x + 236, 416), outline="#159bb8", width=1)
        board.save(destination / f"{look}.png", optimize=True)
    instructions = (
        "Observe all candidates individually. Upper image is native128px; lower is2x nearest with coordinate labels "
        "and the fixed face ellipse. Report observed head_bbox and face_bbox in ORIGINAL128px coordinates, "
        "head_uncropped, hair_match, top_match, extra_accessory, identity_note, inspection_method. "
        "Use conservative observed bounds (approximately2px uncertainty); never copy intended prompt bounds. "
        "The head includes all hair. Face excludes hair/neck. Parent evaluator checks bbox corners against ellipse. "
        "One Inspection JSON per candidate, look='<look>/<vN>', attempt=1or2. "
        "Style may be assessed medium/line/light/palette report-only; it cannot override geometry/blind outcomes.\n")
    _ = (destination / "INSTRUCTIONS.txt").write_text(instructions)
    print(f"Inspection sheets: {destination}")


if __name__ == "__main__":
    main()
