# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: python3 -m art.flora.canopy_texture RAW_SOURCE.png
"""Full-frame canopy texture reduction and literal two-axis repeat evidence."""
from __future__ import annotations

import json
import shutil
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image

from art.compositor.wall_metrics import measure_strip
from art.flora.check_assets import summer_metrics
from art.flora.normalize import digest
from art.flora.seasons import canopy_mask, read_palette
from art.flora.summer import center_chroma

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path("/Users/rexxa/work/arttrees-data")


def build_texture(source: Path, root: Path = ROOT, data: Path = DATA) -> None:
    """Preserve original and export one unretouched 256-square texture plus evidence."""
    source_hash = digest(source)
    original = data / "originals" / f"canopy_mass_{source_hash[:12]}.png"
    original.parent.mkdir(parents=True, exist_ok=True)
    if original.exists() and digest(original) != source_hash:
        message = "Canopy original hash conflict"
        raise ValueError(message)
    if not original.exists():
        shutil.copyfile(source, original)
    with Image.open(source) as opened:
        raw = opened.convert("RGBA")
    texture = raw.resize((256, 256), Image.Resampling.LANCZOS)
    output = root / "art/flora/textures/canopy_mass.png"
    output.parent.mkdir(parents=True, exist_ok=True)
    frozen_resize = data / "originals" / f"canopy_mass_resize_{source_hash[:12]}.png"
    texture.save(frozen_resize, optimize=True)
    initial_mask = canopy_mask(texture, root)
    before_green = summer_metrics(texture, initial_mask, root)
    target = read_palette(root).foliage_summer_mid
    corrected, delta = (texture.copy(), (0.0, 0.0, 0.0)) if before_green.passed else center_chroma(texture, initial_mask, target)
    initial_pixels, final_pixels = np.asarray(texture), np.asarray(corrected)
    protected = np.asarray(initial_mask) == 0
    alpha_unchanged = bool(np.array_equal(initial_pixels[..., 3], final_pixels[..., 3]))
    unmasked_unchanged = bool(np.array_equal(initial_pixels[protected], final_pixels[protected]))
    if not alpha_unchanged or not unmasked_unchanged:
        message = "Canopy tone normalization changed protected pixels"
        raise ValueError(message)
    texture = corrected
    texture.save(output, optimize=True)
    mask = canopy_mask(texture, root)
    mask_path = root / "art/flora/masks/canopy_mass.png"
    mask_path.parent.mkdir(parents=True, exist_ok=True)
    mask.save(mask_path, optimize=True)
    repeated = Image.new("RGBA", (512, 512))
    for x in (0, 256):
        for y in (0, 256):
            repeated.paste(texture, (x, y))
    diagnostic = data / "canopy_mass_repeat_2x2.png"
    repeated.save(diagnostic, optimize=True)
    x_measurement = measure_strip(texture)
    y_measurement = measure_strip(texture.transpose(Image.Transpose.TRANSPOSE))
    green = summer_metrics(texture, mask, root)
    opaque = texture.getchannel("A").getextrema() == (255, 255)
    report = {
        "schema_version": 1, "id": "forest_summer", "source": str(source),
        "source_sha256": source_hash, "source_canvas": list(raw.size),
        "preserved_original": str(original), "output": str(output.relative_to(root)),
        "output_sha256": digest(output), "canvas": [256, 256],
        "canopy_mask": str(mask_path.relative_to(root)), "mask_sha256": digest(mask_path),
        "operations": ["whole-frame LANCZOS resize to256x256; no crops or patching",
                       "masked Lab chroma translation to palette mean only when native summer check fails"],
        "before_resize": {"path": str(frozen_resize), "sha256": digest(frozen_resize)},
        "summer_transform": {"delta_lab": list(delta), "target_rgb": list(target),
                             "alpha_unchanged": alpha_unchanged,
                             "unmasked_unchanged": unmasked_unchanged,
                             "before": asdict(before_green)},
        "repeat_diagnostic": str(diagnostic), "repeat_diagnostic_sha256": digest(diagnostic),
        "seam_x": asdict(x_measurement), "seam_y": asdict(y_measurement),
        "summer_green": asdict(green), "opaque": opaque,
        "numeric_pass": x_measurement.passed and y_measurement.passed and green.passed and opaque,
        "visual_repeat_verdict": "PENDING", "owner_verdict": "PENDING",
    }
    (root / "docs/v8/arttrees/CANOPY_TEXTURE.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"seam_x": x_measurement.passed, "seam_y": y_measurement.passed,
                      "summer_green": asdict(green), "opaque": opaque}))


if __name__ == "__main__":
    build_texture(Path(sys.argv[1]))
