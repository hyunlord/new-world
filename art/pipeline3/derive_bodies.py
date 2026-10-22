# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline3.derive_bodies
"""Derive template age variants by recorded band composition; preserve failures."""
import hashlib
import json
import shutil
from dataclasses import asdict
from pathlib import Path
from typing import Final, Literal

import numpy as np
from PIL import Image

from .check_layer import check_layer
from .layer_schema import load_layer
from .set_checks import combinations, fixed_upper
from .structure import GEOMETRY, Pixels

ROUTE: Final = Path(__file__).resolve().parents[1] / "character" / "route_a"
DATA: Final = Path("/Users/rexxa/work/arttrial3-data/derived")
Age = Literal["child", "elder"]
Band = tuple[int, int, int, int, int, int]
BANDS: Final[dict[Age, tuple[Band, ...]]] = {
    "child": ((0, 32, 22, 46, 48, 8), (32, 50, 46, 60, 48, 8),
              (50, 70, 60, 74, 50, 7), (70, 88, 74, 88, 48, 8),
              (88, 96, 88, 94, 48, 8)),
    "elder": ((0, 32, 4, 36, 64, 1), (32, 50, 36, 54, 64, 1),
              (50, 70, 54, 72, 64, 0), (70, 88, 72, 88, 64, 0),
              (88, 96, 88, 96, 64, 0)),
}


def resize_bands(source: Image.Image, bands: tuple[Band, ...], categorical: bool) -> Image.Image:
    """Map template landmarks without altering source artwork."""
    output = Image.new(source.mode, (64, 96))
    method = Image.Resampling.NEAREST if categorical else Image.Resampling.LANCZOS
    for y0, y1, dy0, dy1, width, dx in bands:
        band = source.crop((0, y0, 64, y1)).resize((width, dy1 - dy0), method)
        output.paste(band, (dx, dy0))
    return output


def complete_mask(pixels: Pixels, mask: Pixels) -> Pixels:
    """Extend categorical labels to Lanczos edge pixels without editing RGBA."""
    visible = pixels[:, :, 3] > 0
    missing = np.argwhere(visible & (mask == 0))
    known = np.argwhere(visible & (mask > 0))
    for y, x in missing:
        nearest = known[np.argmin(np.sum((known - [y, x]) ** 2, axis=1))]
        mask[y, x] = mask[nearest[0], nearest[1]]
    mask[~visible] = 0
    return mask


def derive(source_path: Path, age: Age) -> Path:
    """Write one derivative and its source hashes to the external evidence area."""
    layer = load_layer(source_path)
    original = json.loads(source_path.read_text())
    identifier = f"{age}_{layer.id}"
    target = DATA / age / identifier
    target.mkdir(parents=True, exist_ok=True)
    operations: list[str] = []
    bands = BANDS[age]
    if age == "elder" and layer.direction == "S":
        bands = tuple((*band[:5], -1 if band[0] < 50 else band[5]) for band in bands)
    with Image.open(layer.image) as source, Image.open(layer.mask) as labels:
        if layer.frame == "sleep":
            bbox = source.getbbox()
            if bbox is None:
                raise RuntimeError(identifier)
            length = 42 if age == "child" else 56
            factor = length / (bbox[2] - bbox[0])
            size = (round(96 * factor), round(64 * factor))
            offset = (round(48 - (bbox[0] + bbox[2]) * factor / 2),
                      round(36 - (bbox[1] + bbox[3]) * factor / 2))
            result = Image.new("RGBA", (96, 64))
            result.paste(source.resize(size, Image.Resampling.LANCZOS), offset)
            regions = Image.new("L", (96, 64))
            regions.paste(labels.resize(size, Image.Resampling.NEAREST), offset)
            operations.append(f"Sleep whole-canvas scale {factor}, output size {size}, offset {offset}; actual source alpha length to{length}; center48,36")
        else:
            result = resize_bands(source, bands, False)
            regions = resize_bands(labels, bands, True)
            operations.append(f"Piecewise source y0,y1 -> target y0,y1,width,left bands: {bands}; RGBA Lanczos, categorical masks nearest")
    if layer.kind == "top":
        target_box = GEOMETRY[age].torso
        target_size = (target_box[2] - target_box[0], target_box[3] - target_box[1])
        with Image.open(layer.image) as source, Image.open(layer.mask) as labels:
            result.paste(source.crop(GEOMETRY["adult"].torso).resize(target_size, Image.Resampling.LANCZOS), target_box[:2])
            regions.paste(labels.crop(GEOMETRY["adult"].torso).resize(target_size, Image.Resampling.NEAREST), target_box[:2])
        operations.append(f"Top central torso maps exact adult torso to{target_box}; source-pixel Lanczos composition, not filling or repaint")
    pixels, mask = np.array(result), np.array(regions)
    if layer.kind == "top":
        selected = (pixels[:, :, 3] >= 248) & (pixels[:, :, 3] < 255)
        pixels[selected, 3] = 255
        operations.append(f"Allowed top alpha cleanup248..254 to255 on{int(np.count_nonzero(selected))} pixels; RGB unchanged; zero-alpha pixels untouched")
    pixels[pixels[:, :, 3] < 16] = 0
    mask = complete_mask(pixels, mask)
    operations.append("Alpha cleanup<16; missing categorical edge labels take nearest visible nonzero label, transparent labels0; no RGBA repaint or hole fill")
    meta = {"schema_version": 1, "id": identifier, "kind": layer.kind,
            "body_type": age, "direction": layer.direction, "frame": layer.frame,
            "scale": 1, "mask": identifier + "_mask.png", "variant": layer.variant,
            "operations": operations, "source_sidecar": str(source_path),
            "source_image_sha256": hashlib.sha256(layer.image.read_bytes()).hexdigest(),
            "source_mask_sha256": hashlib.sha256(layer.mask.read_bytes()).hexdigest(),
            "source_sidecar_sha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
            "generation_calls": 0, "extras": list(layer.extras)}
    if layer.frame == "sleep":
        meta["head_box"] = [round(original["head_box"][i] * factor + offset[i % 2]) for i in range(4)]
    for field in ("grip", "shoulder"):
        if field in original:
            x, y = original[field]
            for sy0, sy1, dy0, dy1, width, dx in bands:
                if sy0 <= y < sy1:
                    meta[field] = [round(x * width / 64 + dx), round(dy0 + (y - sy0) * (dy1 - dy0) / (sy1 - sy0))]
    Image.fromarray(pixels).save(target / "pre_copyback.png")
    Image.fromarray(mask).save(target / "pre_copyback_mask.png")
    Image.fromarray(pixels).save(target / (identifier + ".png"))
    Image.fromarray(mask).save(target / meta["mask"])
    sidecar = target / (identifier + ".json")
    sidecar.write_text(json.dumps(meta, indent=2) + "\n")
    return sidecar


def lock_upper(paths: list[Path]) -> None:
    """Reuse derived idle pixels in the age-specific immutable rectangles."""
    layers = [load_layer(path) for path in paths]
    for layer, path in zip(layers, paths, strict=True):
        if layer.kind != "body" or layer.frame in ("idle", "sleep"):
            continue
        idle = next(item for item in layers if item.kind == "body" and item.direction == layer.direction and item.frame == "idle")
        geometry = GEOMETRY[layer.body_type]
        for current, reference in [(layer.image, idle.image), (layer.mask, idle.mask)]:
            with Image.open(current) as image, Image.open(reference) as source:
                for box in (geometry.head, geometry.torso):
                    image.paste(source.crop(box), box[:2])
                image.save(current)
        meta = json.loads(path.read_text())
        meta["operations"].append(f"Exact derived-idle RGBA and mask copyback head{geometry.head}, torso{geometry.torso}; no other repaint")
        path.write_text(json.dumps(meta, indent=2) + "\n")


def main() -> None:
    """Check every derivative; promote only passing layers and retain all records."""
    sources = [path for path in sorted(ROUTE.glob("*.json")) if json.loads(path.read_text()).get("body_type") == "adult"]
    for age in ("child", "elder"):
        paths = [derive(path, age) for path in sources if check_layer(load_layer(path)).passed]
        lock_upper(paths)
        layers = [load_layer(path) for path in paths]
        upper = fixed_upper(layers)
        report = [check_layer(layer) for layer in layers] + upper
        failed = {result.id for result in report if not result.passed}
        for path, layer in zip(paths, layers, strict=True):
            meta = json.loads(path.read_text())
            meta["structural_pass"] = layer.id not in failed
            meta["image_sha256"] = hashlib.sha256(layer.image.read_bytes()).hexdigest()
            meta["mask_sha256"] = hashlib.sha256(layer.mask.read_bytes()).hexdigest()
            path.write_text(json.dumps(meta, indent=2) + "\n")
            if layer.id not in failed:
                for file in (path, layer.image, layer.mask):
                    shutil.copy2(file, ROUTE / file.name)
        (DATA / age / "checks.json").write_text(json.dumps([asdict(result) for result in report], indent=2))
        accepted = [layer for layer in layers if layer.id not in failed]
        combos = combinations(accepted)
        (DATA / age / "combinations.json").write_text(json.dumps([asdict(combo) for combo in combos], indent=2))
        print(age, len(layers), "layers; failures", sorted(failed), "combinations", len(combos), "passing", sum(combo.passed for combo in combos))


if __name__ == "__main__":
    main()
