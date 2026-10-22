# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline3.prepare_s_frames SIDECAR.json
"""Apply logged S-body mask annotation and exact contract-authorized copyback."""
import hashlib
import json
import shutil
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw

from .check_layer import check_layer
from .layer_schema import load_layer
from .prepare_a import prepare
from .set_checks import fixed_upper

ROOT: Final = Path(__file__).resolve().parents[1] / "character" / "route_a"
POLYGON: Final = [(25, 58), (29, 60), (34, 61), (39, 58), (41, 59),
                  (41, 61), (40, 62), (42, 65), (41, 68), (38, 69),
                  (36, 71), (34, 70), (33, 73), (31, 72), (29, 71),
                  (27, 70), (25, 68), (24, 66), (22, 65), (23, 62)]


def process(sidecar: Path) -> None:
    """Retain pre-copyback evidence, verify final pixels, and promote only passes."""
    meta = json.loads(sidecar.read_text())
    raw = Path(meta["raw"]) if "raw" in meta else sidecar.parent / "raw.png"
    prepare(raw, sidecar)
    layer = load_layer(sidecar)
    with Image.open(layer.image) as source:
        pixels = np.array(source)
    polygon = Image.new("L", (64, 96))
    ImageDraw.Draw(polygon).polygon(POLYGON, fill=5)
    mask = np.where(pixels[:, :, 3] > 0, 1, 0).astype(np.uint8)
    mask[(np.array(polygon) == 5) & (pixels[:, :, 3] > 0)] = 5
    Image.fromarray(mask).save(layer.mask)
    idle = load_layer(ROOT / "body_S_idle.json")
    before = [asdict(check_layer(layer))]
    is_frame = layer.frame != "idle"
    if is_frame:
        before.extend(asdict(result) for result in fixed_upper([idle, layer]))
        shutil.copy2(layer.image, sidecar.parent / "before_copyback.png")
        with Image.open(idle.image) as source, Image.open(idle.mask) as source_mask:
            ref, ref_mask = np.array(source), np.array(source_mask)
        for x0, y0, x1, y1 in [(20, 32, 44, 50), (18, 50, 46, 70)]:
            pixels[y0:y1, x0:x1] = ref[y0:y1, x0:x1]
            mask[y0:y1, x0:x1] = ref_mask[y0:y1, x0:x1]
        Image.fromarray(pixels).save(layer.image)
        Image.fromarray(mask).save(layer.mask)
    (sidecar.parent / "checks_before_copyback.json").write_text(json.dumps(before, indent=2))
    meta = json.loads(sidecar.read_text())
    meta["operations"].append(f"Hand mask polygon game pixels {POLYGON}: visible pixels to worn-item region5; all other body pixels skin1; source image unchanged")
    if is_frame:
        meta["operations"].append("Exact idle RGBA and mask copyback in half-open head(20,32,44,50), torso(18,50,46,70); goal3.3 authorization")
    meta["mask_polygon_game_pixels"] = POLYGON
    meta["reference_hashes"] = {path: hashlib.sha256(Path(path).read_bytes()).hexdigest() for path in meta["references"]}
    meta["image_sha256"] = hashlib.sha256(layer.image.read_bytes()).hexdigest()
    meta["mask_sha256"] = hashlib.sha256(layer.mask.read_bytes()).hexdigest()
    after = [check_layer(layer)]
    if is_frame:
        after.extend(fixed_upper([idle, layer]))
    meta["structural_pass"] = all(result.passed for result in after)
    sidecar.write_text(json.dumps(meta, indent=2) + "\n")
    (sidecar.parent / "checks_final.json").write_text(json.dumps([asdict(result) for result in after], indent=2))
    if meta["structural_pass"]:
        for path in (sidecar, layer.image, layer.mask):
            if path.parent != ROOT:
                shutil.copy2(path, ROOT / path.name)
    print(json.dumps([asdict(result) for result in after]))


if __name__ == "__main__":
    process(Path(sys.argv[1]))
