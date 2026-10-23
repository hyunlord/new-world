# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with scikit-image python -m art.pipeline_env1.built_process
"""Normalize ENV1 assets while retaining separate raw and working evidence."""
from __future__ import annotations

import hashlib
import json
from dataclasses import asdict
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw

from art.pipeline2.ground import measure as seam_measure
from art.pipeline2.ground_v2 import cap_tone, luminance, saturation_mean
from art.pipeline2.judge import foreground, judge, load_calibration
from art.pipeline_env1.built_catalog import SPECS
from art.pipeline_env1.items_records import JsonValue

RAW: Final = Path("/Users/rexxa/work/artenv1-data/built")
OUT: Final = Path("art/style05/env1/built")
DOC: Final = Path("docs/v8/artenv1")
CAL: Final = Path("docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json")
PERSON: Final = Path("art/style05/anchors/crops/person_full.png")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    """Export available candidates; missing assets remain explicitly incomplete."""
    OUT.mkdir(parents=True, exist_ok=True)
    calibration = load_calibration(CAL)
    rgb, mask = foreground(PERSON, 512)
    cap = float(luminance(rgb)[mask].std()) * 0.6
    rows: list[dict[str, JsonValue]] = []
    all_attempts: list[dict[str, JsonValue]] = []
    numeric_pass_count = 0
    board = Image.new("RGB", (1152, 960), (92, 88, 74))
    draw = ImageDraw.Draw(board)
    for index, spec in enumerate(SPECS):
        attempts = sorted(RAW.glob(f"{spec.name}_try[1-3]_raw.png"))
        if not attempts:
            continue
        verdicts = [judge(path, calibration, family=spec.family) for path in attempts]
        passed = [i for i, verdict in enumerate(verdicts) if verdict.numeric_pass]
        selected = passed[-1] if passed else len(attempts) - 1
        raw = attempts[selected]
        metadata = raw.with_name(raw.name.replace("_raw.png", ".json"))
        with Image.open(raw) as original:
            source = original.convert("RGBA")
        size = (spec.footprint[0] * 64, spec.footprint[1] * 64)
        operations: list[str] = []
        tone_record = None
        seam = None
        if spec.full_bleed:
            tile = source.resize(size, Image.Resampling.LANCZOS)
            operations.append("full-frame Lanczos resize")
            if spec.name != "snow_overlay":
                before = np.asarray(tile.convert("RGB"), dtype=np.float64) / 255
                tile, contrast, chroma = cap_tone(tile, cap)
                pixels = np.asarray(tile, dtype=np.float64) / 255
                gray = luminance(pixels)[:, :, None]
                chroma_second = min(1.0, 0.280 / max(saturation_mean(pixels), 1e-9))
                pixels = gray + chroma_second * (pixels - gray)
                value_scale = min(1.0, 0.395 / max(float(luminance(pixels).mean()), 1e-9))
                pixels *= value_scale
                tile = Image.fromarray(np.rint(pixels * 255).clip(0, 255).astype(np.uint8)).convert("RGBA")
                after = np.asarray(tile.convert("RGB"), dtype=np.float64) / 255
                if float(luminance(after).mean()) > 0.40 or float(luminance(after).std()) > cap or saturation_mean(after) > 0.30:
                    raise RuntimeError(f"Final ground tone cap exceeded: {spec.name}")
                tone_record = {"before_mean": float(luminance(before).mean()),
                               "before_std": float(luminance(before).std()),
                               "before_saturation": saturation_mean(before),
                               "after_mean": float(luminance(after).mean()),
                               "after_std": float(luminance(after).std()),
                               "after_saturation": saturation_mean(after),
                               "person_std_cap": cap, "contrast_scale": contrast,
                               "chroma_scales": [chroma, chroma_second], "value_scale": value_scale}
                operations.append("logged global contrast/chroma/value scales; no local edits")
            else:
                alpha = tile.getchannel("A").point([round(value * 0.6) for value in range(256)])
                tile.putalpha(alpha)
                operations.append("global alpha multiplied by 0.6; maximum153/255")
            seam = asdict(seam_measure(tile))
        else:
            visible = np.asarray(source)[:, :, 3] >= 128
            ys, xs = np.nonzero(visible)
            if not len(xs):
                raise RuntimeError(f"Empty generated alpha: {raw}")
            box = (max(0, int(xs.min()) - 2), max(0, int(ys.min()) - 2),
                   min(source.width, int(xs.max()) + 3), min(source.height, int(ys.max()) + 3))
            crop = source.crop(box)
            crop.thumbnail((size[0] - 4, size[1] - 4), Image.Resampling.LANCZOS)
            tile = Image.new("RGBA", size)
            offset = ((size[0] - crop.width) // 2, (size[1] - crop.height) // 2)
            tile.alpha_composite(crop, offset)
            operations.extend([f"crop alpha>=128 bounds plus2px fringe {box}; no alpha edits", f"Lanczos contain {crop.size} offset{offset}"])
        output = OUT / f"{spec.name}.png"
        tile.save(output, optimize=True)
        if output.stat().st_size > 1_500_000:
            raise RuntimeError(f"File cap exceeded: {output}")
        all_attempts.extend(asdict(verdict) for verdict in verdicts)
        row: dict[str, JsonValue] = {"id": spec.name, "path": str(output), "width": size[0], "height": size[1],
               "footprint": list(spec.footprint), "family": spec.family, "attempts": len(attempts),
               "selected_attempt": selected + 1, "status": "PENDING_MODEL_REVIEW" if verdicts[selected].numeric_pass else "NUMERIC_FAIL",
               "raw": str(raw), "raw_sha256": digest(raw), "sha256": digest(output),
               "metadata": str(metadata), "metadata_sha256": digest(metadata),
               "numeric": asdict(verdicts[selected]), "operations": operations,
               "tone": tone_record, "seam": seam, "bytes": output.stat().st_size}
        rows.append(row)
        numeric_pass_count += verdicts[selected].numeric_pass
        _ = (OUT / f"{spec.name}.json").write_text(json.dumps(row, indent=2) + "\n")
        x, y = index % 6 * 192, index // 6 * 240
        draw.text((x + 8, y + 8), spec.name, fill="white")
        draw.text((x + 8, y + 24), str(row["status"]), fill="white")
        preview = tile.copy()
        preview.thumbnail((176, 150), Image.Resampling.NEAREST)
        board.paste(preview, (x + (192 - preview.width) // 2, y + 48), preview)
        half = tile.resize((size[0] // 2, size[1] // 2), Image.Resampling.LANCZOS)
        board.paste(half, (x + 8, y + 174), half)
    board.save(OUT / "built_board.png", optimize=True)
    _ = (DOC / "built_assets.json").write_text(json.dumps({"version": 1, "assets": rows}, indent=2) + "\n")
    _ = (DOC / "built_numeric.json").write_text(json.dumps({"calibration": str(CAL), "calibration_sha256": digest(CAL),
                                                        "attempts": all_attempts}, indent=2) + "\n")
    print(json.dumps({"exported": len(rows), "expected": len(SPECS), "numeric_pass": numeric_pass_count}))


if __name__ == "__main__":
    main()
