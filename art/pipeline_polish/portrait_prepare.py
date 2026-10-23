# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: uv run art/pipeline_polish/portrait_prepare.py
"""Prepare the nine authorized portrait requests and literal identity references."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Final

from PIL import Image, ImageOps
from pydantic import JsonValue

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = ROOT.parent / "artpolish-data/portraits"
TARGETS: Final = (
    "adult_F_fur_cloak_long_tied", "adult_F_hide_wrap_bun",
    "child_M_fur_cloak_bun", "child_M_loincloth_bun", "adult_F_fur_cloak_bun",
    "child_F_hide_wrap_bun", "child_F_loincloth_long_tied",
    "child_M_hide_wrap_bun", "child_M_hide_wrap_short",
)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prompt_for(look: str) -> str:
    age, sex, costume = look.split("_", 2)
    person = f"{'adult woman' if sex == 'F' else 'adult man'}" if age == "adult" else f"{'girl' if sex == 'F' else 'boy'}, a young child"
    hair = "a compact tied bun, with no hairpin" if costume.endswith("bun") else "short dark brown hair" if costume.endswith("short") else "long dark brown hair tied into a ponytail behind the head, visibly hanging beside the neck; not a round top bun"
    top = "a dark brown draped cloak with a broad pale fur collar" if costume.startswith("fur_cloak") else "the exact pale-edged one-shoulder hide wrap shown by the sprite, preserving which shoulder is covered" if costume.startswith("hide_wrap") else "the exact neckline and upper-body garment shown by the sprite; do not substitute a fur collar or unrelated outfit"
    return (
        "Create ONE square inspector portrait, shoulders-up bust, for the exact person in reference 1. "
        "Reference 1 is the actual route I S/idle identity sprite. Reference 2 is an 8x nearest-neighbor crop "
        "of that SAME sprite's head for identity. Reference 3 is portrait.png, STYLE ONLY. "
        f"The person is {person}; preserve sprite skin tone, {hair}, and {top}. "
        "The sprite's hairstyle and clothing take priority over the style anchor. DO NOT copy the anchor man's "
        "face, beard, necklace, bone pin or ornaments. Do not invent accessories. Neutral natural expression. "
        "Front three-quarter view in controlled dark-brown ink and restrained earth-colour watercolor washes, "
        "natural-history illustration, warm parchment palette, upper-left lighting. A clear silhouette contour, "
        "selective hair lines, broad lit and shaded planes; no photography, no 3D, no dense all-over hatching. "
        "Composition must survive full-frame downsampling to 128x128 and 64x64. Make the head comfortably smaller "
        "than the canvas: the entire head, ears, hair, bun or ponytail including flyaways must fit within "
        "x=14..114, y=10..104 in final 128px coordinates. Keep generous quiet margin above and beside the hair. "
        "Keep all face skin inside fixed ellipse center (64,61), radii (44,52). The shoulders may meet the bottom "
        "edge but the hair must not touch any edge. Quiet plain neutral warm parchment background. No text, "
        "border, landscape, labels, watermark, jewellery or head ornament. Render only one finished portrait."
    )


def main() -> None:
    DATA.mkdir(parents=True, exist_ok=True)
    refs_dir = DATA / "references"
    refs_dir.mkdir(exist_ok=True)
    requests: list[dict[str, JsonValue]] = []
    for look in TARGETS:
        source = ROOT / "art/character/route_i" / look / "S/idle.png"
        with Image.open(source) as sprite:
            bounds = sprite.getbbox()
            if bounds is None:
                message = f"Empty identity sprite: {source}"
                raise ValueError(message)
            head_fraction = 0.55 if look.startswith("child") else 0.40
            crop_box = (bounds[0], bounds[1], bounds[2], bounds[1] + round((bounds[3] - bounds[1]) * head_fraction))
            head = sprite.crop(crop_box)
            head = ImageOps.scale(head, 8, Image.Resampling.NEAREST)
        head_path = refs_dir / f"{look}_head_8x.png"
        head.save(head_path)
        refs = [source, head_path, ROOT / "art/style05/anchors/portrait.png"]
        requests.append({"id": look, "kind": "asset_generation", "attempt": 1,
                         "prompt": prompt_for(look), "referenced_image_paths": [str(p) for p in refs],
                         "references": [{"path": str(p), "sha256": digest(p)} for p in refs],
                         "head_crop": {"source_box": list(crop_box), "scale": 8, "filter": "NEAREST"},
                         "transparent_background": False})
    path = DATA / "requests.json"
    _ = path.write_text(json.dumps(requests, indent=2) + "\n")
    print(path)


if __name__ == "__main__":
    main()
