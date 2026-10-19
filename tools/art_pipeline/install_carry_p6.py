# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/install_carry_p6.py
"""Install only the six final-review-approved food groups, retaining source atlas geometry."""
import hashlib
import json
from typing import Final

from PIL import Image

from carry_boards_p6 import ROOT, installed_frames
from carry_candidate_round2_p6 import candidate_parts

ART: Final = ROOT / "assets/art/atlas"
DOC: Final = ROOT / "docs/art/production6/carry"


def main() -> None:
    """Bind reviewed composition pixels to original source crops with hashes."""
    reviews = json.loads((DOC / "review_round2.json").read_text())["verdicts"]
    approved = {row["group"] for row in reviews if row["verdict"] == "ok"}
    frames = [frame for frame in installed_frames() if f"{frame.body}_{frame.activity}_{frame.view}" in approved]
    with Image.open(ART / "world_0.png") as original:
        body_atlas = original.convert("RGBA")
    props = Image.new("RGBA", (9, 7 * len(frames)))
    entries = {}
    provenance = []
    for index, frame in enumerate(frames):
        body, prop, anchor = candidate_parts(frame)
        body_atlas.paste(body, frame.box[:2])
        props.paste(prop, (0, index * 7))
        entries[frame.sprite_id] = {"kind": "food", "anchor": anchor, "rect": [0, index * 7, 9, 7],
                                    "group": f"{frame.body}_{frame.activity}_{frame.view}"}
        provenance.append({"sprite_id": frame.sprite_id,
                           "source_rgba_sha256": hashlib.sha256(frame.image().tobytes()).hexdigest(),
                           "body_rgba_sha256": hashlib.sha256(body.tobytes()).hexdigest(),
                           "prop_rgba_sha256": hashlib.sha256(prop.tobytes()).hexdigest(),
                           "method": "exact candidate_round2 pixel procedure", "review": "review_round2.json"})
    body_atlas.save(ART / "carry_p6_body.png")
    props.save(ART / "carry_p6_props.png")
    (ART / "carry_p6.json").write_text(json.dumps({"body_page": "res://assets/art/atlas/carry_p6_body.png",
        "prop_page": "res://assets/art/atlas/carry_p6_props.png", "sprites": entries}, indent=2) + "\n")
    (DOC / "layer_provenance.json").write_text(json.dumps({"scope": "new carry layers only; inherited harvest A20 not certified here",
        "frames": provenance}, indent=2) + "\n")
    print(f"Installed {len(frames)} frames from {len(approved)} approved food groups; wood and child unchanged")


if __name__ == "__main__":
    main()
