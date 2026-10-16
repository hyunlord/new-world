# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/readability.py docs/art/D1
"""Reproducible palette-only grass proofs with measured villager contrast."""
from __future__ import annotations

import argparse
import colorsys
import json
from pathlib import Path
from typing import TypedDict

import numpy as np
from PIL import Image, ImageDraw, ImageFont

from palette_tools import Color, load_palette
from presentation import FONTS


class VillagerMeasure(TypedDict):
    """Auditable encoded-sRGB measurements, including actual ground beneath."""

    sprite: str
    opaque_texels: int
    sprite_mean_luma: float
    ground_mean_luma: float
    ground_beneath_opaque_mean_luma: float
    absolute_difference: float
    beneath_difference: float
    target_pass: bool


def luma(color: Color) -> float:
    """Encoded-sRGB Rec.709 luma, explicitly not linear-light luminance."""
    return sum(a * b for a, b in zip(color, (0.2126, 0.7152, 0.0722))) / 255


def ground_colors(palette: list[Color]) -> list[Color]:
    """Rank only hue and saturation before looking at any sprite values."""
    green = [c for c in palette if 0.14 <= colorsys.rgb_to_hsv(*c)[0] <= 0.47]
    candidates = green or palette
    selected = sorted(candidates, key=lambda c: (colorsys.rgb_to_hsv(*c)[1], c))[:3]
    return sorted(selected, key=luma)


def ground_strip(palette: list[Color]) -> Image.Image:
    """Repeat one 16x16 tile; all strokes stay inside its border."""
    colors = ground_colors(palette)
    tile = Image.new("RGB", (16, 16), colors[len(colors) // 2])
    draw = ImageDraw.Draw(tile)
    draw.line((3, 5, 5, 4), fill=colors[0])
    draw.line((10, 11, 12, 10), fill=colors[-1])
    draw.point((8, 3), fill=colors[-1])
    strip = Image.new("RGB", (384, 64))
    for y in range(0, 64, 16):
        for x in range(0, 384, 16):
            strip.paste(tile, (x, y))
    return strip


def build_readability(directory: Path) -> None:
    """Render unchanged sprites; report failures without corrective recoloring."""
    palette = load_palette(directory / "palette.hex")
    ground = ground_strip(palette)
    ground_rgb = np.asarray(ground, dtype=np.float64) / 255
    weights = np.array((0.2126, 0.7152, 0.0722))
    ground_mean = float(np.mean(ground_rgb @ weights))
    scene = ground.convert("RGBA")
    measurements: list[VillagerMeasure] = []
    indices = (9, 10, 11, 12, 5, 6, 7, 8, 2, 1)
    positions: list[tuple[int, str]] = []
    for slot, index in enumerate(indices):
        paths = sorted(p for p in (directory / "sprites").glob("*.png") if int(p.stem.split("_")[0]) == index)
        if len(paths) != 1:
            raise ValueError(f"Expected one sprite for cell {index}, found {paths}")
        with Image.open(paths[0]) as source:
            sprite = source.convert("RGBA")
        x, y = 8 + slot * 37, 48 - sprite.height
        positions.append((x, str(index)))
        pixels = np.asarray(sprite)
        mask = pixels[..., 3] == 255
        if not np.any(mask):
            raise ValueError(f"Empty sprite: {paths[0]}")
        if index in (9, 10, 11, 12):
            sprite_mean = float(np.mean((pixels[..., :3][mask] / 255) @ weights))
            beneath = ground_rgb[y:y + sprite.height, x:x + sprite.width]
            beneath_mean = float(np.mean(beneath[mask] @ weights))
            delta = abs(sprite_mean - ground_mean)
            measurements.append(VillagerMeasure(sprite=paths[0].name, opaque_texels=int(mask.sum()),
                sprite_mean_luma=sprite_mean, ground_mean_luma=ground_mean,
                ground_beneath_opaque_mean_luma=beneath_mean, absolute_difference=delta,
                beneath_difference=abs(sprite_mean - beneath_mean), target_pass=delta >= 0.15))
        scene.alpha_composite(sprite, (x, y))
    minimum = min(item["absolute_difference"] for item in measurements)
    report = {
        "direction": directory.name,
        "formula": "mean((0.2126 R + 0.7152 G + 0.0722 B)/255); encoded sRGB, not linear-light luminance",
        "ground_selection": "Hue in [0.14,0.47], lowest saturation 3; RGB tie break; median luma base, extrema strokes. If no green, lowest saturation palette colors.",
        "ground_colors": ["#%02X%02X%02X" % c for c in ground_colors(palette)],
        "ground_mean_luma": ground_mean,
        "villagers": measurements,
        "minimum_difference": minimum,
        "threshold": 0.15,
        "target_pass": minimum >= 0.15,
        "adjustments": "None: original palette and normalized sprites remain unchanged.",
    }
    (directory / "readability.json").write_text(json.dumps(report, indent=2) + "\n")
    board = Image.new("RGB", (1568, 570), "#eee9db")
    draw = ImageDraw.Draw(board)
    draw.fontmode = "1"
    body = ImageFont.truetype(str(FONTS / "Galmuri11.ttf"), 12)
    draw.text((16, 14), f"{directory.name} / 가독성 검증 / 원본 팔레트와 스프라이트 유지", font=body, fill="black")
    draw.text((16, 42), "1× : 1 texel = 1 pixel", font=body, fill="black")
    board.paste(scene.convert("RGB"), (16, 64))
    for x, label in positions:
        draw.text((16 + x, 130), label, fill="black", font=body)
    draw.text((16, 158), "4× : nearest / integer zoom", font=body, fill="black")
    board.paste(scene.convert("RGB").resize((1536, 256), Image.Resampling.NEAREST), (16, 182))
    for x, label in positions:
        draw.text((16 + x * 4, 442), label, fill="black", font=body)
    for row, item in enumerate(measurements):
        text = f"{item['sprite']} : ΔY′ = {item['absolute_difference']:.4f}  / target 0.1500"
        draw.text((16 + (row % 2) * 768, 478 + (row // 2) * 22), text, font=body, fill="black")
    verdict = "PASS" if minimum >= 0.15 else "FAIL - review required; no palette or sprite changes applied"
    draw.text((16, 534), f"MIN {minimum:.4f} / {verdict}", font=body, fill="black")
    board.save(directory / "readability.png")


def main() -> None:
    """Measure a normalized direction folder."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    build_readability(args.directory)


if __name__ == "__main__":
    main()
