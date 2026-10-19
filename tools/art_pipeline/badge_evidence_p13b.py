#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow"]
# ///
# How to run: python3 badge_evidence_p13b.py legend CHECKOUT ICONS NEWDIR
# Or: python3 badge_evidence_p13b.py contrast CHECKOUT SAMPLES_JSON NEWDIR
"""Label installed badges and measure explicitly reviewed framebuffer samples."""
from __future__ import annotations

import hashlib
import json
import statistics
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from prepare_blind_crops_p13 import NAMES, row_geometry
from p12_activity_oracle.activity_rules import EvidenceError, load_atlas
from p12_activity_oracle.check_activity_sidecars import check_capture
from run_activity_captures_p13 import Capture, integer, record, validate_capture

HABITATS: Final = ("grass", "dirt", "shelter")
Point = tuple[int, int]
RGB = tuple[float, float, float]


@dataclass(frozen=True, slots=True)
class Sample:
    sidecar: Path
    entity_bits: int
    kind: int
    habitat: str
    review: str
    fill: tuple[Point, ...]
    background: tuple[Point, ...]


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def points(raw: list[list[int]]) -> tuple[Point, ...]:
    """Parse nonempty, unique absolute framebuffer pixel coordinates."""
    parsed = tuple((integer(pair[0], 0), integer(pair[1], 0)) for pair in raw if len(pair) == 2)
    if not parsed or len(parsed) != len(raw) or len(set(parsed)) != len(parsed):
        raise EvidenceError("nonempty unique [x,y] points required")
    return parsed


def samples(path: Path) -> tuple[Sample, ...]:
    data = record(json.loads(path.read_text()))
    if data.get("schema") != 1:
        raise EvidenceError("sample schema must be 1")
    result = []
    for raw in data["samples"]:
        row = record(raw)
        kind = integer(row["kind"], 0)
        if kind >= 7 or row["habitat"] not in HABITATS or not str(row["review"]).strip():
            raise EvidenceError("kind, habitat and visual review required")
        sidecar = (path.parent / row["sidecar"]).resolve()
        if sha(sidecar) != row["sidecar_sha256"] or sha(sidecar.with_name("capture.png")) != row["image_sha256"]:
            raise EvidenceError("reviewed capture hashes changed")
        result.append(Sample(sidecar, integer(row["entity_bits"], 0), kind, row["habitat"],
                             row["review"], points(row["fill"]), points(row["background"])))
    if not result:
        raise EvidenceError("at least one reviewed sample required")
    return tuple(result)


def luminance(rgb: RGB) -> float:
    """WCAG sRGB transfer, then relative luminance (no gamma-space averaging)."""
    def linear(channel: float) -> float:
        value = channel / 255.0
        return value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4
    red, green, blue = (linear(value) for value in rgb)
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue


def ratio(first: float, second: float) -> float:
    return (max(first, second) + 0.05) / (min(first, second) + 0.05)


def median_rgb(values: list[tuple[int, int, int]]) -> RGB:
    red, green, blue = (statistics.median(channel) for channel in zip(*values, strict=True))
    return float(red), float(green), float(blue)


def legend(checkout: Path, icons: Path, output: Path) -> None:
    """Make a labelled teaching sheet, never a new stage-A or framebuffer result."""
    metrics_path = icons / "metrics.json"
    metrics = json.loads(metrics_path.read_text())
    source = checkout / "scripts/ui/activity_icons_p11.gd"
    craft = checkout / "assets/art/production/U1/04.png"
    if (not metrics["valid"] or metrics["source_sha256"] != sha(source)
            or metrics["craft_original_sha256"] != sha(craft)):
        raise EvidenceError("texture export does not match installed badge sources")
    sheet = Image.new("RGBA", (1120, 160), "#808080")
    draw = ImageDraw.Draw(sheet)
    hashes = {str(path): sha(path) for path in (metrics_path, source, craft)}
    for kind, name in enumerate(NAMES):
        path = icons / f"icon-{kind}.png"
        with Image.open(path) as image:
            badge = image.convert("RGBA")
        expected = next(row["source_size"] for row in metrics["icons"] if row["kind"] == kind)
        if list(badge.size) != expected:
            raise EvidenceError("badge dimensions differ from export metrics")
        hashes[str(path)] = sha(path)
        badge = badge.resize((badge.width * 4, badge.height * 4), Image.Resampling.NEAREST)
        sheet.alpha_composite(badge, (kind * 160 + (160 - badge.width) // 2, 20))
        draw.text((kind * 160 + 80, 126), name, fill="white", anchor="mt", font_size=16)
    sheet.convert("RGB").save(output / "legend.png")
    report = {"evidence_scope": "labelled source texture legend; not framebuffer or stage A",
              "names": NAMES, "scale": 4, "background": "#808080", "source_hashes": hashes,
              "legend_sha256": sha(output / "legend.png")}
    (output / "legend.json").write_text(json.dumps(report, indent=2) + "\n")


def contrast(checkout: Path, specification: Path, output: Path) -> None:
    """Read RGB only from capture.png; humans assign habitat and main-fill points."""
    selected = samples(specification)
    atlas = load_atlas(checkout)
    results = []
    for index, sample in enumerate(selected):
        data = json.loads(sample.sidecar.read_text())
        validate_capture(data, Capture(data["seed"], data["display_day"], data["hour"], data["zoom"]))
        if data["zoom"] != 4 or not check_capture(data, atlas)["pose_check_complete"]:
            raise EvidenceError("requires pose-verified Z4 sidecar")
        holders = [row for row in data["activity_holders"] if row["entity_bits"] == sample.entity_bits]
        if len(holders) != 1:
            raise EvidenceError("sample entity must identify one holder")
        holder = holders[0]
        if holder["kind"] != sample.kind or not holder["icon_visible"] or not holder["fully_inside_viewport"]:
            raise EvidenceError("sample requires matching visible activity badge")
        geometry = row_geometry(holder, data["draw_rects"])
        if geometry["icon_covered_fraction"] != 0:
            raise EvidenceError("contrast requires an uncovered icon")
        image_path = sample.sidecar.with_name("capture.png")
        with Image.open(image_path) as original:
            image = original.convert("RGB")
        if list(image.size) != data["viewport"]:
            raise EvidenceError("framebuffer dimensions differ from sidecar")
        x, y, width, height = holder["icon_viewport_rect"]
        if set(sample.fill) & set(sample.background):
            raise EvidenceError("fill and background masks overlap")
        groups = (("fill", sample.fill, "red"), ("background", sample.background, "cyan"))
        overlay = image.copy()
        pixels = {}
        mask_hashes = {}
        for name, coordinates, colour in groups:
            if any(px >= image.width or py >= image.height for px, py in coordinates):
                raise EvidenceError("sample point outside framebuffer")
            inside = [x <= px + 0.5 < x + width and y <= py + 0.5 < y + height for px, py in coordinates]
            if (name == "fill" and not all(inside)) or (name == "background" and any(inside)):
                raise EvidenceError("fill must be inside badge; background outside badge rect")
            mask = Image.new("1", image.size)
            ImageDraw.Draw(mask).point(coordinates, fill=1)
            mask_path = output / f"sample-{index:02d}-{name}-mask.png"
            mask.save(mask_path)
            mask_hashes[mask_path.name] = sha(mask_path)
            ImageDraw.Draw(overlay).point(coordinates, fill=colour)
            pixels[name] = [image.getpixel(point) for point in coordinates]
        box = geometry["box"]
        all_x, all_y = zip(*(sample.fill + sample.background), strict=True)
        box = [max(0, min(box[0], min(all_x) - 8)), max(0, min(box[1], min(all_y) - 8)),
               min(image.width, max(box[2], max(all_x) + 9)), min(image.height, max(box[3], max(all_y) + 9))]
        overlay_path = output / f"sample-{index:02d}-review.png"
        overlay.crop(box).save(overlay_path)
        fill, background = median_rgb(pixels["fill"]), median_rgb(pixels["background"])
        background_luminance = luminance(background)
        results.append({"kind": sample.kind, "activity": NAMES[sample.kind], "habitat": sample.habitat,
                        "review": sample.review, "sidecar": str(sample.sidecar), "sidecar_sha256": sha(sample.sidecar),
                        "image_sha256": sha(image_path), "entity_bits": sample.entity_bits,
                        "icon_rect": holder["icon_viewport_rect"], "geometry": geometry,
                        "fill_points": sample.fill, "background_points": sample.background, "rgb": pixels,
                        "mask_sha256": mask_hashes, "review_box": box, "review_sha256": sha(overlay_path),
                        "fill_median_rgb": fill, "background_median_rgb": background,
                        "fill_luminance": luminance(fill), "background_luminance": background_luminance,
                        "contrast": ratio(luminance(fill), background_luminance),
                        "per_fill_pixel_contrast": [ratio(luminance(rgb), background_luminance) for rgb in pixels["fill"]]})
    missing = [[name, habitat] for kind, name in enumerate(NAMES) for habitat in HABITATS
               if not any(row["kind"] == kind and row["habitat"] == habitat for row in results)]
    report = {"schema": 1, "scope": "reviewer-selected actual framebuffer pixels; no automated habitat classifier",
              "specification_sha256": sha(specification), "tool_sha256": sha(Path(__file__)),
              "formula": "sRGB c/255; linear=c/12.92 if c<=0.04045 else ((c+0.055)/1.055)^2.4; L=.2126R+.7152G+.0722B; (max(L)+.05)/(min(L)+.05)",
              "aggregation": "per-channel median RGB separately for main-fill and background, then luminance",
              "missing_kind_habitat": missing, "coverage_status": "COMPLETE" if not missing else "PARTIAL",
              "acceptance": "RECORDED_ONLY_no_numeric_contrast_bar_specified", "samples": results}
    (output / "samples.json").write_bytes(specification.read_bytes())
    (output / "contrast.json").write_text(json.dumps(report, indent=2) + "\n")


def main(arguments: list[str]) -> None:
    if len(arguments) != 4 or arguments[0] not in ("legend", "contrast"):
        raise SystemExit("usage: badge_evidence_p13b.py {legend|contrast} CHECKOUT INPUT NEWDIR")
    command, root, source, destination = arguments
    output = Path(destination)
    output.mkdir(parents=True, exist_ok=False)
    if command == "legend":
        legend(Path(root).resolve(), Path(source).resolve(), output)
    else:
        contrast(Path(root).resolve(), Path(source).resolve(), output)


if __name__ == "__main__":
    main(sys.argv[1:])
