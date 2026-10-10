# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/audit_art.py --root .
"""Read-only delivery audit; stdout is JSON and structural failures exit 1."""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

import numpy as np
from PIL import Image, ImageFont

from palette_tools import load_palette, rgb_to_lab


def image_check(path: Path, expected: tuple[int, int] | None) -> str | None:
    """Fully decode an image and optionally enforce exact output dimensions."""
    try:
        with Image.open(path) as image:
            image.load()
            if expected is not None and image.size != expected:
                return f"{path}: size {image.size}, expected {expected}"
    except (OSError, ValueError) as error:
        return f"{path}: {error}"
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--require-source-margins", action="store_true", help="Promote recorded source-edge contact warnings to structural failures")
    args = parser.parse_args()
    root = args.root.resolve()
    art = root / "docs/art"
    failures: list[str] = []
    warnings: list[str] = []
    measurements: list[str] = []
    manifest = json.loads((root / "tools/art_pipeline/sheet_v1.json").read_text())
    for direction, cap in [("D1", 32), ("D2", 48), ("D3", 20)]:
        base = art / direction
        for suffix in ["F2_noon", "F2_night", "F1_overview", "F2_noon_hud", "F2_night_hud"]:
            error = image_check(base / f"{direction}_{suffix}.jpg", (1920, 1080))
            if error:
                failures.append(error)
        for name in ["sprites_contact.png", "readability.png", "palette.png"]:
            error = image_check(base / name, None)
            if error:
                failures.append(error)
        try:
            colors = load_palette(base / "palette.hex")
        except (OSError, ValueError) as error:
            failures.append(f"{direction} palette: {error}")
            continue
        if len(colors) > cap:
            failures.append(f"{direction}: {len(colors)} palette colors exceeds {cap}")
        labs = rgb_to_lab(np.array(colors))
        distances = np.linalg.norm(labs[:, None] - labs[None, :], axis=-1)
        np.fill_diagonal(distances, np.inf)
        minimum = float(distances.min())
        measurements.append(f"{direction}: palette={len(colors)}, minimum_pair_deltaE76={minimum:.6f}")
        if minimum < 3:
            failures.append(f"{direction}: palette pair DeltaE76 {minimum:.6f} below 3")
        allowed = set(colors)
        for cell in manifest["cells"]:
            path = base / "sprites" / f"{cell['index']:02d}_{cell['name']}.png"
            error = image_check(path, (cell["width"], cell["height"]))
            if error:
                failures.append(error)
                continue
            with Image.open(path) as image:
                rgba = np.array(image.convert("RGBA"))
            alpha = rgba[..., 3]
            opaque = alpha == 255
            pixel_colors = set(map(tuple, rgba[opaque, :3].tolist()))
            if not opaque.any() or pixel_colors - allowed:
                failures.append(f"{path.name}: empty or out-of-palette sprite")
            if np.any((alpha != 0) & (alpha != 255)):
                failures.append(f"{path.name}: partial alpha")
            if not alpha[-1].any():
                failures.append(f"{path.name}: sprite does not touch baseline")
            if alpha[0].any() or alpha[:, 0].any() or alpha[:, -1].any():
                failures.append(f"{path.name}: forbidden top/side edge contact")
        sprites = list((base / "sprites").glob("*.png"))
        if len(sprites) != 16:
            failures.append(f"{direction}: expected 16 sprites, found {len(sprites)}")
        try:
            report = json.loads((base / "normalize_report.json").read_text())
            if len(report) != 16:
                failures.append(f"{direction}: expected 16 normalization report rows")
            for row in report:
                if row["source_edge_pixels"]:
                    message = f"{direction}/{row['name']}: recorded source edge pixels={row['source_edge_pixels']}"
                    (failures if args.require_source_margins else warnings).append(message)
            readability = json.loads((base / "readability.json").read_text())
            minimum_luma = float(readability["minimum_difference"])
            measurements.append(f"{direction}: readability minimum={minimum_luma:.6f}")
            if minimum_luma < 0.15:
                warnings.append(f"{direction}: readability {minimum_luma:.6f} below art target 0.15; comparison result, not production acceptance")
        except (OSError, ValueError, KeyError) as error:
            failures.append(f"{direction} report: {error}")
    for name in ["ART_BIBLE.md", "DECISIONS.md", "DIRECTION_REPORT.md"]:
        if not (art / name).is_file() or not (art / name).stat().st_size:
            failures.append(f"Missing or empty docs/art/{name}")
    error = image_check(art / "direction_compare.jpg", None)
    if error:
        failures.append(error)
    fonts = root / "assets/fonts/galmuri"
    for name in ["Galmuri11.ttf", "Galmuri11-Bold.ttf", "Galmuri9.ttf"]:
        try:
            ImageFont.truetype(str(fonts / name), 12)
        except OSError as error:
            failures.append(f"Font {name}: {error}")
    licenses = [p for p in fonts.glob("*") if p.suffix.lower() in {".md", ".txt"}]
    if not any("SIL OPEN FONT LICENSE" in p.read_text().upper() for p in licenses):
        failures.append("Missing SIL Open Font License text")
    total = sum(p.stat().st_size for p in art.rglob("*") if p.is_file())
    if total >= 25_000_000:
        failures.append(f"docs/art size {total} exceeds strict 25 MB limit")
    try:
        common = subprocess.check_output(["git", "merge-base", "origin/lead/main", "HEAD"], cwd=root, text=True).strip()
        tracked = subprocess.check_output(["git", "diff", "--name-only", "-z", common], cwd=root).decode().split("\0")
        untracked = subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard", "-z"], cwd=root).decode().split("\0")
        changed = sorted(set(tracked + untracked) - {""})
        for path in changed:
            if not path.startswith(("docs/art/", "tools/art_pipeline/", "assets/fonts/galmuri/")):
                failures.append(f"Changed path outside allowlist: {path}")
    except subprocess.CalledProcessError as error:
        failures.append(f"Git scope verification failed: {error}")
        changed = []
    print(json.dumps({"structural_pass": not failures, "failures": failures, "warnings": warnings,
                      "measurements": measurements, "docs_art_bytes": total,
                      "changed_paths_checked": len(changed), "required_main_frames": 9,
                      "required_hud_frames": 6, "required_sprites": 48}, indent=2))
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
