"""Generates placeholder wildlife sprites (32x32 RGBA PNG).

These are stand-ins until the ComfyUI sprite session generates real pixel art.
Output: assets/sprites/wildlife/{wolf,bear,boar}.png
"""
from pathlib import Path
from PIL import Image, ImageDraw

OUT_DIR = Path(__file__).parent.parent.parent / "assets" / "sprites" / "wildlife"
OUT_DIR.mkdir(parents=True, exist_ok=True)


def make_wolf() -> Image.Image:
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.polygon([(8, 28), (24, 28), (22, 14), (10, 14)], fill=(110, 110, 120, 255))
    d.polygon([(11, 14), (21, 14), (19, 6), (13, 6)], fill=(110, 110, 120, 255))
    d.point((14, 9), fill=(0, 0, 0, 255))
    d.point((18, 9), fill=(0, 0, 0, 255))
    d.polygon([(11, 8), (13, 4), (13, 8)], fill=(80, 80, 90, 255))
    d.polygon([(19, 8), (21, 4), (19, 8)], fill=(80, 80, 90, 255))
    return img


def make_bear() -> Image.Image:
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([6, 12, 26, 30], fill=(90, 60, 40, 255))
    d.ellipse([10, 4, 22, 16], fill=(90, 60, 40, 255))
    d.point((16, 11), fill=(40, 30, 20, 255))
    d.point((13, 9), fill=(0, 0, 0, 255))
    d.point((19, 9), fill=(0, 0, 0, 255))
    d.ellipse([9, 4, 12, 7], fill=(70, 45, 30, 255))
    d.ellipse([20, 4, 23, 7], fill=(70, 45, 30, 255))
    return img


def make_boar() -> Image.Image:
    img = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.polygon([(7, 28), (25, 28), (23, 13), (9, 13)], fill=(170, 130, 130, 255))
    d.polygon([(11, 13), (21, 13), (22, 6), (10, 6)], fill=(170, 130, 130, 255))
    d.rectangle([14, 5, 18, 8], fill=(150, 110, 110, 255))
    d.point((13, 9), fill=(240, 240, 230, 255))
    d.point((19, 9), fill=(240, 240, 230, 255))
    d.point((14, 9), fill=(0, 0, 0, 255))
    d.point((18, 9), fill=(0, 0, 0, 255))
    return img


make_wolf().save(OUT_DIR / "wolf.png")
make_bear().save(OUT_DIR / "bear.png")
make_boar().save(OUT_DIR / "boar.png")
print(f"Generated 3 placeholder PNGs → {OUT_DIR}")
