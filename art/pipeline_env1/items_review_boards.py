"""Render geometry-only raw-candidate contact sheets for the image-model judge."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline_env1.items_records import JsonRecord

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/artenv1-data/items")
GROUPS: Final = (
    ("cured_hide", "clay", "raw_pot", "clay_pot_filled"),
    ("firewood", "ground_axe", "hide_wrap", "fur_cloak"),
    ("wolf_hide", "clay_pot_empty", "meat", "flint_nodule", "spear"),
)
RESAMPLING: Final = (Image.Resampling.BICUBIC, Image.Resampling.LANCZOS, Image.Resampling.LANCZOS)


def main() -> None:
    manifest = JsonRecord.read(ROOT / "docs/v8/artenv1/items_manifest.json")
    assets = {row.text("id"): row for row in manifest.records("assets")}
    for group_number, names in enumerate(GROUPS, start=1):
        canvas = Image.new("RGB", (1024, 532*((len(names)+1)//2)), "#e1d7c3")
        draw = ImageDraw.Draw(canvas)
        entries: list[dict[str, str]] = []
        for index, name in enumerate(names):
            asset = assets[name]
            with Image.open(asset.text("source")) as source:
                sprite = source.convert("RGBA")
            sprite = sprite.crop(sprite.getbbox())
            sprite.thumbnail((470, 470), RESAMPLING[group_number-1])
            x, y = (index%2)*512, (index//2)*532
            canvas.paste(sprite, (x+(512-sprite.width)//2, y+35+(470-sprite.height)//2), sprite)
            draw.text((x+20, y+12), name, fill="#181812")
            entries.append({"id": name, "source": asset.text("source"), "sha256": asset.text("source_sha256")})
        path = DATA / f"style_candidates{group_number}.png"
        canvas.save(path)
        _ = path.with_suffix(".json").write_text(json.dumps(entries, indent=2)+"\n")


if __name__ == "__main__":
    main()
