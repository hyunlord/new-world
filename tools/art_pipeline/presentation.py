# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/presentation.py hud docs/art/D1
"""Pixel-exact Korean HUD proofs and direction comparison boards."""
from __future__ import annotations

import argparse
import colorsys
import json
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, ImageOps

from palette_tools import Color, load_palette

ROOT = Path(__file__).resolve().parents[2]
FONTS = ROOT / "assets/fonts/galmuri"


@dataclass(frozen=True, slots=True)
class Theme:
    """Three neutral palette roles plus a single shared accent."""

    dark: Color
    fill: Color
    text: Color
    accent: Color


def luminance(color: Color) -> float:
    """WCAG relative luminance after the sRGB transfer function."""
    values = [c / 255 for c in color]
    linear = [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in values]
    return sum(a * b for a, b in zip(linear, (0.2126, 0.7152, 0.0722)))


def contrast(first: Color, second: Color) -> float:
    """Return the WCAG contrast ratio of two actual palette entries."""
    lower, upper = sorted((luminance(first), luminance(second)))
    return (upper + 0.05) / (lower + 0.05)


def theme_for(palette: list[Color]) -> Theme:
    """Choose low-saturation roles within luminance quartiles, then enforce contrast."""
    ordered = sorted(palette, key=luminance)
    quartile = max(1, (len(palette) + 3) // 4)
    fill = min(ordered[:quartile], key=lambda c: colorsys.rgb_to_hsv(*c)[1])
    text = min(ordered[-quartile:], key=lambda c: colorsys.rgb_to_hsv(*c)[1])
    if contrast(fill, text) < 4.5:
        fill, text = ordered[0], ordered[-1]
    accent = min(palette, key=lambda c: abs(colorsys.rgb_to_hsv(*c)[0] - 0.09))
    return Theme(ordered[0], fill, text, accent)


def panel(size: tuple[int, int], theme: Theme) -> Image.Image:
    """Stretch a palette-only 9-slice source with fixed three-pixel corners."""
    seed = Image.new("RGBA", (9, 9), theme.fill)
    draw = ImageDraw.Draw(seed)
    draw.rectangle((0, 0, 8, 8), outline=theme.dark)
    draw.rectangle((1, 1, 7, 7), outline=theme.accent)
    output = Image.new("RGBA", size)
    xs, ys = (0, 3, size[0] - 3, size[0]), (0, 3, size[1] - 3, size[1])
    for row in range(3):
        for col in range(3):
            patch = seed.crop((col * 3, row * 3, col * 3 + 3, row * 3 + 3))
            patch = patch.resize((xs[col + 1] - xs[col], ys[row + 1] - ys[row]), Image.Resampling.NEAREST)
            output.paste(patch, (xs[col], ys[row]))
    return output


def hud_layer(theme: Theme, night: bool = False) -> Image.Image:
    """Render at native 960x540, then scale all UI texels exactly 2x."""
    layer = Image.new("RGBA", (960, 540))
    body = ImageFont.truetype(str(FONTS / "Galmuri11.ttf"), 12)
    heading = ImageFont.truetype(str(FONTS / "Galmuri11-Bold.ttf"), 12)
    small = ImageFont.truetype(str(FONTS / "Galmuri9.ttf"), 10)
    for box in ((12, 12, 936, 30), (12, 47, 330, 27), (12, 436, 328, 90), (742, 288, 206, 238)):
        x, y, width, height = box
        layer.alpha_composite(panel((width, height), theme), (x, y))
    draw = ImageDraw.Draw(layer)
    draw.fontmode = "1"
    time = "3일차 21:00 · 밤" if night else "3일차 12:00 · 낮"
    draw.text((24, 20), f"{time} · 인구 31 · 정착지 1 · 거처 5 · 건설 중 1", font=body, fill=theme.text)
    draw.text((24, 54), "식량 42 · 열매 18 · 출생 9 · 사망 0", font=body, fill=theme.text)
    draw.text((24, 447), "마을 소식", font=heading, fill=theme.text)
    for y, line in zip((468, 486, 504), ("주민 17이 물을 찾으러 갑니다.", "새 거처의 뼈대가 세워졌습니다.", "채집한 열매 6개를 저장했습니다.")):
        draw.text((24, y), line, font=body, fill=theme.text)
    for y, line in zip((302, 330, 352, 374), ("주민 17", "위치 (33, 30)", "상태: 이동 중", "목표: 물")):
        draw.text((756, y), line, font=heading if y == 302 else body, fill=theme.text)
    for y, label, level in ((408, "허기", 72), (444, "갈증", 38), (480, "수면", 86)):
        draw.text((756, y), label, font=body, fill=theme.text)
        draw.text((901, y + 1), str(level), font=small, fill=theme.text)
        draw.rectangle((756, y + 19, 932, y + 24), fill=theme.dark)
        draw.rectangle((757, y + 20, 757 + int(174 * level / 100), y + 23), fill=theme.accent)
    return layer.resize((1920, 1080), Image.Resampling.NEAREST)


def make_swatch(directory: Path) -> None:
    """Keep palette labels outside solid 72-pixel swatches."""
    palette = load_palette(directory / "palette.hex")
    rows = (len(palette) + 7) // 8
    image = Image.new("RGB", (640, rows * 96 + 32), "#eee9db")
    draw = ImageDraw.Draw(image)
    draw.fontmode = "1"
    font = ImageFont.truetype(str(FONTS / "Galmuri9.ttf"), 10)
    draw.text((8, 8), f"{directory.name} / {len(palette)} colours", fill="black", font=font)
    for index, color in enumerate(palette):
        x, y = (index % 8) * 80 + 4, (index // 8) * 96 + 32
        draw.rectangle((x, y, x + 71, y + 71), fill=color)
        draw.text((x, y + 76), "#%02X%02X%02X" % color, fill="black", font=font)
    image.save(directory / "palette.png")


def make_hud(directory: Path) -> None:
    """Composite identical untinted UI rules on noon and night sources."""
    theme = theme_for(load_palette(directory / "palette.hex"))
    metrics = {
        "selection": "Lowest-saturation entry in lower/upper luminance quartile; fallback to full-palette darkest/lightest pair if contrast <4.5",
        "roles": {"dark": "#%02X%02X%02X" % theme.dark, "fill": "#%02X%02X%02X" % theme.fill,
                  "text": "#%02X%02X%02X" % theme.text, "accent": "#%02X%02X%02X" % theme.accent},
        "text_fill_wcag_contrast": contrast(theme.text, theme.fill),
        "target": 4.5,
        "target_pass": contrast(theme.text, theme.fill) >= 4.5,
        "palette_colors_only": True,
        "font_sizes_native": {"Galmuri11": 12, "Galmuri11-Bold": 12, "Galmuri9": 10},
        "fontmode": "1",
        "integer_ui_scale": 2,
        "ui_tinted_at_night": False,
    }
    (directory / "hud_metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")
    make_swatch(directory)
    for period in ("noon", "night"):
        layer = hud_layer(theme, period == "night")
        with Image.open(directory / f"{directory.name}_F2_{period}.jpg") as source:
            if source.size != (1920, 1080):
                raise ValueError(f"Expected 1920x1080 frame, got {source.size}")
            composite = Image.alpha_composite(source.convert("RGBA"), layer)
            composite.convert("RGB").save(directory / f"{directory.name}_F2_{period}_hud.jpg", quality=88)
        if period == "noon":
            layer.save(directory / "hud_proof.png")


def make_comparison(root: Path) -> None:
    """Build the specified three-row, six-column contact board."""
    width, height = 480, 330
    board = Image.new("RGB", (width * 6, height * 3 + 36), "#181c22")
    draw = ImageDraw.Draw(board)
    font = ImageFont.truetype(str(FONTS / "Galmuri11-Bold.ttf"), 12)
    draw.fontmode = "1"
    labels = ("정오 HUD", "밤", "전체 지도", "팔레트", "원본 / 정규화", "가독성 1× / 4×")
    for col, label in enumerate(labels):
        draw.text((col * width + 12, 10), label, fill="white", font=font)
    for row, direction in enumerate(("D1", "D2", "D3")):
        directory = root / direction
        names = (f"{direction}_F2_noon_hud.jpg", f"{direction}_F2_night.jpg", f"{direction}_F1_overview.jpg", "palette.png", "sprites_contact.png", "readability.png")
        for col, name in enumerate(names):
            with Image.open(directory / name) as source:
                thumb = ImageOps.contain(source.convert("RGB"), (width - 12, height - 36), Image.Resampling.NEAREST)
            x, y = col * width + (width - thumb.width) // 2, 36 + row * height
            board.paste(thumb, (x, y + 24 + (height - 36 - thumb.height) // 2))
            draw.text((col * width + 8, y + 4), f"{direction} / {labels[col]}", font=font, fill="white")
    board.save(root / "direction_compare.jpg", quality=88)


def main() -> None:
    """Run a single direction HUD render or full comparison render."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("hud", "compare"))
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    actions = {"hud": make_hud, "compare": make_comparison}
    actions[args.command](args.directory)


if __name__ == "__main__":
    main()
