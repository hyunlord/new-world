#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow>=10", "numpy>=1.24"]
# ///
# How to run: python3 tools/revitalize/visual_qa.py RAW_FINAL COMMITTED_FINAL [RAW_BASELINE]
"""Measure real capture pixels and sidecars; emit JPEGs and every V1–V6 number."""
from __future__ import annotations

import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw

SIZE: Final = (1920, 1080)
NAMES: Final = (
    "d1_1200_F1", "d1_1200_F2", "d2_1200_F1", "d2_1200_F2",
    "d2_1200_F3", "d2_1200_F4", "d2_1200_F2A", "d2_2200_F2N",
    "d6_1200_F1", "d6_1200_F2",
)


@dataclass(frozen=True, slots=True)
class Entity:
    kind: str
    identity: int
    rect: tuple[float, float, float, float]
    visible: bool
    building_kind: int

    def pixels(self) -> tuple[slice, slice]:
        x, y, width, height = self.rect
        return (slice(max(0, math.floor(y)), min(SIZE[1], math.ceil(y + height))),
                slice(max(0, math.floor(x)), min(SIZE[0], math.ceil(x + width))))


@dataclass(frozen=True, slots=True)
class Frame:
    name: str
    image: Image.Image
    luma: NDArray[np.float64]
    entities: tuple[Entity, ...]
    zoom: float
    selected: int
    seeking: bool
    seek_segments: int
    trail_segments: int
    tick: int
    offset: int
    camera: tuple[float, float]

    def visible(self, kind: str) -> tuple[Entity, ...]:
        return tuple(row for row in self.entities if row.kind == kind and row.visible)

    def world_mask(self) -> NDArray[np.bool_]:
        mask = np.ones((SIZE[1], SIZE[0]), dtype=np.bool_)
        for panel in self.visible("panel"):
            mask[panel.pixels()] = False
        return mask


def normalized_image(path: Path) -> Image.Image:
    """Area-average HiDPI backbuffers before any analysis; reject undersized frames."""
    with Image.open(path) as source:
        width, height = source.size
        if width < SIZE[0] or height < SIZE[1] or width * 9 != height * 16:
            raise ValueError(f"Invalid backbuffer size {source.size}: {path}")
        rgb = source.convert("RGB")
        return rgb.resize(SIZE, Image.Resampling.BOX) if rgb.size != SIZE else rgb.copy()


def load_frame(raw: Path, name: str) -> Frame:
    image = normalized_image(raw / f"{name}.png")
    data = json.loads((raw / f"{name}.entities.json").read_text())
    width, height = data["viewport_size"]
    with Image.open(raw / f"{name}.png") as original:
        if [width, height] != list(original.size):
            raise ValueError(f"Sidecar/backbuffer size mismatch: {name}")
    entities = tuple(Entity(
        str(row["kind"]), int(row["id"]),
        (float(row["rect"][0]) * SIZE[0] / width, float(row["rect"][1]) * SIZE[1] / height,
         float(row["rect"][2]) * SIZE[0] / width, float(row["rect"][3]) * SIZE[1] / height),
        bool(row["visible"]), int(row.get("building_kind", -1)),
    ) for row in data["entities"])
    if any(not all(math.isfinite(v) for v in row.rect) or min(row.rect[2:]) < 0 for row in entities):
        raise ValueError(f"Invalid entity rectangle: {name}")
    rgb = np.asarray(image, dtype=np.float64) / 255.0
    luma = rgb @ np.array([0.2126, 0.7152, 0.0722])
    return Frame(name, image, luma, entities, float(data["zoom"][0]),
                 int(data["selected_entity_bits"]), bool(data["selected_seeking"]),
                 int(data["seek_segments"]), int(data["trail_segments"]),
                 int(data["tick"]), int(data["clock_offset_ticks"]),
                 (float(data["camera_position"][0]), float(data["camera_position"][1])))


def nonblank(image: Image.Image) -> tuple[bool, float, int]:
    rgb = np.asarray(image, dtype=np.float64) / 255.0
    std = float(np.std(rgb @ np.array([0.2126, 0.7152, 0.0722])))
    colors = int(np.unique(np.asarray(image).reshape(-1, 3), axis=0).shape[0])
    return std >= 0.04 and colors >= 64, std, colors


def overlap(a: Entity, b: Entity) -> float:
    ax, ay, aw, ah = a.rect
    bx, by, bw, bh = b.rect
    return max(0.0, min(ax + aw, bx + bw) - max(ax, bx)) * max(0.0, min(ay + ah, by + bh) - max(ay, by))


def agent_contrast(day: Frame, hidden: Frame) -> tuple[bool, list[str]]:
    if day.tick != hidden.tick or day.zoom != hidden.zoom or day.camera != hidden.camera:
        raise ValueError("V2 pair must have equal tick and zoom")
    changed = np.abs(day.luma - hidden.luma) >= 0.12
    visible_count = 0
    considered = 0
    details: list[str] = []
    world = day.world_mask()
    for agent in day.visible("agent"):
        area = agent.pixels()
        available = world[area]
        pixels = int(available.sum())
        if pixels == 0:
            details.append(f"agent {agent.identity}: excluded (fully obscured by UI)")
            continue
        fraction = float(np.count_nonzero(changed[area] & available) / pixels)
        visible_count += int(fraction >= 0.20)
        considered += 1
        details.append(f"agent {agent.identity}: ΔL≥0.12 fraction={fraction:.6f}, pixels={pixels}")
    ratio = visible_count / considered if considered else 0.0
    details.insert(0, f"visible={visible_count}/{considered}, share={ratio:.6f} (≥0.90)")
    return considered > 0 and ratio >= 0.90, details


def night_contrast(day: Frame, night: Frame) -> tuple[bool, list[str]]:
    night_world = night.world_mask()
    day_mean = float(np.mean(day.luma[day.world_mask()]))
    night_mean = float(np.mean(night.luma[night_world]))
    hearths = tuple(row for row in night.visible("building") if row.building_kind == 0)
    details = [f"day world mean={day_mean:.6f}, night world mean={night_mean:.6f}, ratio={night_mean / max(day_mean, 1e-12):.6f} (≤0.65)"]
    if not hearths:
        return False, details + ["No on-screen Hearth; local-light measurement unavailable."]
    hearth = min(hearths, key=lambda row: (row.rect[0] + row.rect[2] / 2 - 960) ** 2 + (row.rect[1] + row.rect[3] / 2 - 540) ** 2)
    x, y, w, h = hearth.rect
    yy, xx = np.ogrid[:SIZE[1], :SIZE[0]]
    disk = ((xx + 0.5 - x - w / 2) ** 2 + (yy + 0.5 - y - h / 2) ** 2 <= 48 ** 2) & night_world
    local = float(np.mean(night.luma[disk])) if disk.any() else 0.0
    details.append(f"Hearth {hearth.identity}, 48px disk mean={local:.6f}, night ratio={local / max(night_mean, 1e-12):.6f} (≥1.25), pixels={int(disk.sum())}")
    return day_mean > 0 and night_mean > 0 and night_mean <= 0.65 * day_mean and local >= 1.25 * night_mean, details



def labels_valid(frame: Frame) -> bool:
    labels = frame.visible("stockpile_label")
    return all(10 <= label.rect[3] <= 28 for label in labels) and (frame.zoom >= 2 or not labels)


def lines_valid(frame: Frame) -> bool:
    if frame.selected < 0:
        return frame.seek_segments == frame.trail_segments == 0
    return not frame.seeking or (frame.seek_segments >= 1 and frame.trail_segments >= 1)


def export_jpeg(image: Image.Image, target: Path) -> None:
    image.resize((960, 540), Image.Resampling.BOX).save(target, quality=85)


def analyse(raw: Path, output: Path, baseline: Path | None = None) -> bool:
    output.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((raw / "manifest.json").read_text())
    if manifest["plan"] != "final" or sorted(row["file"] for row in manifest["captures"]) != sorted(f"{name}.png" for name in NAMES):
        raise ValueError("Final manifest must contain exactly the ten clock-plan captures")
    frames = {name: load_frame(raw, name) for name in NAMES}
    for name, frame in frames.items():
        day, hhmm, _ = name.split("_")
        expected = (int(day[1:]) - 1) * 1440 + int(hhmm[:2]) * 60 + int(hhmm[2:]) - frame.offset
        if frame.tick != expected:
            raise ValueError(f"Clock-plan tick mismatch: {name}")
    report: list[str] = ["# Visual QA", "", "Analysis: 1920×1080; HiDPI area-averaged; luma in [0,1]. UI rectangles excluded from world-area metrics.", ""]
    verdicts: list[bool] = []
    def record(check: str, passed: bool, details: list[str]) -> None:
        verdicts.append(passed)
        report.extend([f"## {check}: {'PASS' if passed else 'FAIL'}", "", *[f"- {line}" for line in details], ""])
    for name, frame in frames.items():
        passed, std, colors = nonblank(frame.image)
        record(f"V1 {name}", passed, [f"luma std={std:.6f} (≥0.04), colours={colors} (≥64)", f"tick={frame.tick}, clock offset={frame.offset}"])
        labels = frame.visible("stockpile_label")
        record(f"V4 {name}", labels_valid(frame),
               [f"zoom={frame.zoom}, visible labels={len(labels)}", *[f"label {row.identity}: height={row.rect[3]:.4f}px" for row in labels]])
        panels = frame.visible("panel")
        areas = [(a.identity, b.identity, overlap(a, b)) for i, a in enumerate(panels) for b in panels[i + 1:]]
        record(f"V5 {name}", all(area == 0 for _, _, area in areas), [f"visible panels={len(panels)}", *[f"panel {a}/{b}: intersection={area:.4f}px²" for a, b, area in areas]])
        lines_ok = lines_valid(frame)
        record(f"V6 {name}", lines_ok, [f"selected={frame.selected}, seeking={frame.seeking}, seek segments={frame.seek_segments}, trail segments={frame.trail_segments}"])
        export_jpeg(frame.image, output / f"{name}.jpg")
    record("V2 agent contrast", *agent_contrast(frames["d2_1200_F2"], frames["d2_1200_F2A"]))
    record("V3 night and Hearth", *night_contrast(frames["d2_1200_F2"], frames["d2_2200_F2N"]))
    if baseline is not None:
        sheet = Image.new("RGB", (1920, 4 * 576), "#181818")
        draw = ImageDraw.Draw(sheet)
        baseline_out = output.parent / "baseline"
        baseline_out.mkdir(parents=True, exist_ok=True)
        pairs = [(0, 1, "F1"), (0, 1, "F2"), (1440, 2, "F1"), (1440, 2, "F2")]
        for row, (tick, day, framing) in enumerate(pairs):
            old_name = f"baseline_t{tick}_{framing}"
            old = normalized_image(baseline / f"{old_name}.png")
            passed, std, colors = nonblank(old)
            record(f"V1 {old_name}", passed, [f"luma std={std:.6f}, colours={colors}"])
            export_jpeg(old, baseline_out / f"{old_name}.jpg")
            new_name = f"d{day}_1200_{framing}"
            sheet.paste(old.resize((960, 540), Image.Resampling.BOX), (0, row * 576 + 36))
            sheet.paste(frames[new_name].image.resize((960, 540), Image.Resampling.BOX), (960, row * 576 + 36))
            draw.text((12, row * 576 + 10), old_name + " (legacy clock)", fill="white")
            draw.text((972, row * 576 + 10), new_name + " (offset clock)", fill="white")
        sheet.save(output.parent / "contact_sheet.jpg", quality=85)
    report.extend([f"Overall: {'PASS' if all(verdicts) else 'FAIL'} ({sum(verdicts)}/{len(verdicts)} checks)", ""])
    (output / "visual_qa.md").write_text("\n".join(report))
    return all(verdicts)


def main() -> int:
    if len(sys.argv) not in (3, 4):
        print(__doc__ + "\nUsage: visual_qa.py RAW_FINAL COMMITTED_FINAL [RAW_BASELINE]", file=sys.stderr)
        return 2
    return 0 if analyse(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]) if len(sys.argv) == 4 else None) else 1


if __name__ == "__main__":
    raise SystemExit(main())
