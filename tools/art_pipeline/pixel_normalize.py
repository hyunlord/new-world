# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/pixel_normalize.py --help
"""Normalize equal-grid chroma-key sprite sheets at native texel density."""
from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

from palette_tools import Color, load_palette, rgb_to_lab


@dataclass(frozen=True, slots=True)
class Cell:
    index: int
    name: str
    width: int
    height: int


@dataclass(frozen=True, slots=True)
class Metrics:
    index: int
    name: str
    width: int
    height: int
    source_opaque: int
    source_edge_pixels: int
    opaque: int
    colors: int
    palette_match_percent: float
    partial_alpha: int
    forbidden_edge_pixels: int
    baseline_pixels: int
    mean_luma: float
    passed: bool


def key_out(image: Image.Image) -> Image.Image:
    """Remove RGB-distance key pixels followed by magenta HSV spill."""
    rgba = np.array(image.convert("RGBA"))
    rgb = rgba[..., :3].astype(np.float64)
    close = np.linalg.norm(rgb - [255, 0, 255], axis=-1) <= 60
    hsv = np.asarray(image.convert("RGB").convert("HSV"), dtype=np.float64)
    hue = hsv[..., 0] * 360 / 255
    hue_distance = np.abs((hue - 300 + 180) % 360 - 180)
    spill = (hue_distance <= 20) & (hsv[..., 1] / 255 > 0.45)
    rgba[..., 3] = np.where(close | spill | (rgba[..., 3] < 128), 0, 255)
    rgba[rgba[..., 3] == 0, :3] = 0
    return Image.fromarray(rgba)


def block_mode(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    """Use area-weighted 5-bit mode, retaining at least 50% opaque area."""
    source = np.asarray(image)
    height, width = source.shape[:2]
    tw, th = size
    result = np.zeros((th, tw, 4), dtype=np.uint8)
    for y in range(th):
        y0, y1 = y * height / th, (y + 1) * height / th
        ys = np.arange(math.floor(y0), math.ceil(y1))
        wy = np.minimum(ys + 1, y1) - np.maximum(ys, y0)
        for x in range(tw):
            x0, x1 = x * width / tw, (x + 1) * width / tw
            xs = np.arange(math.floor(x0), math.ceil(x1))
            wx = np.minimum(xs + 1, x1) - np.maximum(xs, x0)
            pixels = source[ys[:, None], xs]
            weights = wy[:, None] * wx
            opaque = pixels[..., 3] == 255
            if weights[opaque].sum() + 1e-10 < weights.sum() / 2:
                continue
            buckets = pixels[..., :3][opaque].astype(np.uint16) >> 3
            codes = (buckets[:, 0] << 10) | (buckets[:, 1] << 5) | buckets[:, 2]
            totals = np.bincount(codes, weights=weights[opaque], minlength=32768)
            winner = int(np.argmax(totals))
            result[y, x] = [((winner >> 10) & 31) * 8 + 4,
                            ((winner >> 5) & 31) * 8 + 4, (winner & 31) * 8 + 4, 255]
    return Image.fromarray(result)


def normalize(image: Image.Image, size: tuple[int, int], palette: list[Color]) -> Image.Image:
    """Bottom-fit a keyed silhouette, palette-match it, add outside outline."""
    keyed = key_out(image)
    bbox = keyed.getbbox()
    if bbox is None:
        raise ValueError("Cell contains no opaque sprite after chroma key")
    crop = keyed.crop(bbox)
    width, height = size
    # Reserve one transparent border at top/sides plus the outside outline.
    scale = min((width - 4) / crop.width, (height - 3) / crop.height)
    fit = (max(1, round(crop.width * scale)), max(1, round(crop.height * scale)))
    sampled_image = block_mode(crop, fit)
    retained_bbox = sampled_image.getbbox()
    if retained_bbox is None:
        raise ValueError("Sprite vanished during majority-coverage downsampling")
    # Thin features can disappear under the coverage rule; anchor retained texels.
    retained = sampled_image.crop(retained_bbox)
    sampled = np.array(retained)
    opaque = sampled[..., 3] == 255
    palette_array = np.array(palette, dtype=np.uint8)
    labs = rgb_to_lab(sampled[..., :3][opaque])
    distances = np.sum((labs[:, None] - rgb_to_lab(palette_array)[None, :]) ** 2, axis=-1)
    sampled[opaque, :3] = palette_array[np.argmin(distances, axis=1)]
    canvas = np.zeros((height, width, 4), dtype=np.uint8)
    x, y = (width - retained.width) // 2, height - 1 - retained.height
    canvas[y:y + retained.height, x:x + retained.width] = sampled
    mask = canvas[..., 3] == 255
    dilated = np.zeros_like(mask)
    padded = np.pad(mask, 1)
    for dy in range(3):
        for dx in range(3):
            dilated |= padded[dy:dy + height, dx:dx + width]
    # Four-connected exterior excludes enclosed holes, even diagonally enclosed ones.
    exterior = np.zeros_like(mask)
    frontier = [(0, x) for x in range(width)] + [(height - 1, x) for x in range(width)]
    frontier += [(y, 0) for y in range(height)] + [(y, width - 1) for y in range(height)]
    while frontier:
        ey, ex = frontier.pop()
        if not (0 <= ey < height and 0 <= ex < width) or mask[ey, ex] or exterior[ey, ex]:
            continue
        exterior[ey, ex] = True
        frontier.extend(((ey - 1, ex), (ey + 1, ex), (ey, ex - 1), (ey, ex + 1)))
    outline = dilated & exterior
    darkest = palette_array[np.argmin(rgb_to_lab(palette_array)[:, 0])]
    canvas[outline, :3] = darkest
    canvas[outline, 3] = 255
    return Image.fromarray(canvas)


def measure(image: Image.Image, source: Image.Image, cell: Cell) -> Metrics:
    """Measure alpha, borders and native color usage; palette checked by run."""
    rgba = np.asarray(image)
    mask = rgba[..., 3] == 255
    original = np.asarray(key_out(source))[..., 3] == 255
    edge = int(original[0].sum() + original[-1].sum() + original[1:-1, 0].sum() + original[1:-1, -1].sum())
    colors = np.unique(rgba[mask, :3], axis=0)
    partial = int(np.count_nonzero((rgba[..., 3] > 0) & (rgba[..., 3] < 255)))
    forbidden = int(mask[0].sum() + mask[1:, 0].sum() + mask[1:, -1].sum())
    luma = float(np.mean(rgba[mask, :3] @ np.array([0.2126, 0.7152, 0.0722]) / 255))
    return Metrics(cell.index, cell.name, cell.width, cell.height, int(original.sum()), edge,
                   int(mask.sum()), len(colors), 100.0, partial, forbidden, int(mask[-1].sum()),
                   luma, partial == 0 and forbidden == 0 and edge == 0 and bool(mask[-1].any()))


def run(sheet: Path, manifest: Path, palette_path: Path, output: Path) -> bool:
    """Process one sheet and persist auditable metrics and native images."""
    spec = json.loads(manifest.read_text())
    columns, rows = int(spec["columns"]), int(spec["rows"])
    cells = [Cell(int(c["index"]), str(c["name"]), int(c["width"]), int(c["height"])) for c in spec["cells"]]
    if len(cells) != columns * rows or [c.index for c in cells] != list(range(1, len(cells) + 1)):
        raise ValueError("Manifest cells must cover the grid in contiguous row-major order")
    if any(c.width < 5 or c.height < 4 or not c.name.replace("_", "").isalnum() for c in cells):
        raise ValueError("Invalid cell name or dimensions")
    palette = load_palette(palette_path)
    with Image.open(sheet) as opened:
        image = opened.convert("RGBA")
    if columns < 1 or rows < 1 or image.width < columns or image.height < rows:
        raise ValueError("Grid must have positive dimensions and nonempty cells")
    (output / "sprites").mkdir(parents=True, exist_ok=True)
    contact = Image.new("RGB", (columns * 544, rows * 288), "#25262b")
    draw = ImageDraw.Draw(contact)
    reports: list[Metrics] = []
    allowed = set(palette)
    for cell in cells:
        col, row = (cell.index - 1) % columns, (cell.index - 1) // columns
        raw = image.crop((col * image.width // columns, row * image.height // rows,
                          (col + 1) * image.width // columns, (row + 1) * image.height // rows))
        sprite = normalize(raw, (cell.width, cell.height), palette)
        rgba = np.asarray(sprite)
        pixels = rgba[rgba[..., 3] == 255, :3]
        if any(tuple(map(int, color)) not in allowed for color in pixels):
            raise ValueError(f"Out-of-palette output in cell {cell.index}")
        reports.append(measure(sprite, raw, cell))
        sprite.save(output / "sprites" / f"{cell.index:02d}_{cell.name}.png")
        bx, by = col * 544, row * 288
        preview = raw.copy()
        preview.thumbnail((256, 256), Image.Resampling.NEAREST)
        contact.paste(preview, (bx, by + 24), preview)
        native = sprite.resize((cell.width * 8, cell.height * 8), Image.Resampling.NEAREST)
        contact.paste(native, (bx + 272, by + 24 + 256 - native.height), native)
        draw.text((bx + 4, by + 4), f"{cell.index:02d} {cell.name} | raw / native 8x", fill="white")
    contact.save(output / "sprites_contact.png")
    (output / "normalize_report.json").write_text(json.dumps([asdict(r) for r in reports], indent=2) + "\n")
    lines = ["# Normalization report", "", "RGB key distance <=60; HSV spill +/-20 degrees, S>0.45. Area-weighted 5-bit block mode; CIELAB DeltaE76 nearest palette; outside 8-neighbor outline restricted to perimeter-connected transparent background. Grid bounds use floor(i*size/count), covering arbitrary dimensions without lost pixels.", "", "Top/side output edges must stay transparent. Bottom outline may touch the baseline. Source cell edge contact is a failure. Aspect fit may differ by at most one target texel due to integer rounding. Luma uses encoded sRGB Rec.709 coefficients, including outline.", "", "|Cell|Size|Source opaque|Source edge|Opaque|Colors|Palette %|Partial alpha|Forbidden edge|Baseline|Luma|Pass|", "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|"]
    for r in reports:
        lines.append(f"|{r.index:02d} {r.name}|{r.width}x{r.height}|{r.source_opaque}|{r.source_edge_pixels}|{r.opaque}|{r.colors}|{r.palette_match_percent:.1f}|{r.partial_alpha}|{r.forbidden_edge_pixels}|{r.baseline_pixels}|{r.mean_luma:.4f}|{r.passed}|")
    (output / "normalize_report.md").write_text("\n".join(lines) + "\n")
    return all(r.passed for r in reports)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sheet", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--palette", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    return 0 if run(args.sheet, args.manifest, args.palette, args.output) else 1


if __name__ == "__main__":
    raise SystemExit(main())
