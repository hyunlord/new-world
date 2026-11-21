#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow>=10.1"]
# ///
# How to run: python3 tools/3d/blender/mocap_review.py contact BAKED_DIR
# noqa: SIZE_OK - The review commands share capture receipts and panel helpers.
"""Assemble review sheets and 30 fps GIFs from captures of the baked VAT."""

from dataclasses import dataclass
import json
from pathlib import Path
import subprocess
import sys
from typing import Final, NotRequired, TypedDict

from PIL import Image, ImageDraw, ImageFont

CELL: Final = 260
BODY_LABELS: Final = {"p1": "p1  Man", "p2": "p2  Woman", "p3": "p3  Child", "p4": "p4  Elder"}
DISPLAY_NAMES: Final = {"chop": "Chop", "cook": "Stir", "build": "Hammer/build", "sit": "Sit on ground",
                        "sit_log": "Sit on log", "walk_reference": "Walk reference"}
VIEWS: Final = {"side": "Side", "quarter": "Front 3/4"}
PILOT: Final = ("chop", "cook")
FINAL_KEYS: Final = (*PILOT, "dig", "build", "sit", "sit_log", "talk", "pick_up", "set_down", "harvest", "carry",
                     "walk_reference")
FINAL_PICKS: Final = {"dig": 3, "build": 3, "sit": 4, "sit_log": 4}
FINAL_CHECKS: Final = (("chop", "apex", "Apex grip"), ("chop", "garment", "Garment worst"),
                       ("cook", "stir", "Stir clearance"), ("chop", "rebound", "Axe lift-out"))
REVISION_KEYS: Final = ("build", "set_down")
STRIP_COUNT: Final = 12
GUTTER: Final = 106
LABEL_HEIGHT: Final = 24
ROW: Final = CELL + LABEL_HEIGHT
SECTION: Final = 50
HEADER: Final = 70
BACKGROUND: Final = "#1b2121"
INK: Final = "#f4e7d1"


class ReviewError(Exception):
    """The supplied frame evidence cannot support the requested artifact."""


class Clip(TypedDict):
    start: int
    count: int
    loop: bool
    duration: float
    picks: list[int]
    detail: NotRequired[int]
    hero: NotRequired[int]
    checks: NotRequired[dict[str, int]]
    strip: NotRequired[list[int]]


class Body(TypedDict):
    body: str
    fps: int
    asset: str
    vat_sha256: str
    clips: dict[str, Clip]


@dataclass(frozen=True, slots=True)
class Capture:
    directory: Path
    label: str
    bodies: dict[str, Body]

    def frame(self, key: str, body: str, view: str, sample: int) -> Path:
        return self.directory / f"{key}-{body}-{view}-{sample:03d}.png"


def read_capture(root: Path, label: str) -> Capture:
    directory = root / "frames"
    bodies: dict[str, Body] = {}
    for body in BODY_LABELS:
        data: Body = json.loads((directory / f"review-{body}.json").read_text())
        if data["body"] != body or data["fps"] not in (15, 30):
            raise ReviewError(f"{root}: invalid body or FPS in the {body} frame receipt")
        for key, clip in data["clips"].items():
            picks = clip["picks"]
            if len(picks) != 6 or sorted(set(picks)) != picks or not 0 <= picks[0] < picks[-1] < clip["count"]:
                raise ReviewError(f"{root}: invalid six-frame selection for {body} {key}")
        bodies[body] = data
    return Capture(directory, label, bodies)


def make_sheet(captures: tuple[Capture, ...], keys: tuple[str, ...], output: Path) -> None:
    if not keys:
        raise ReviewError("No captured actions to review")
    split = 22
    group_width = 6 * CELL
    section_height = SECTION + len(BODY_LABELS) * ROW
    canvas = Image.new(
        "RGB",
        (GUTTER + len(captures) * (group_width + split), HEADER + len(keys) * len(VIEWS) * section_height + split),
        BACKGROUND,
    )
    draw = ImageDraw.Draw(canvas)
    title = ImageFont.load_default(size=32)
    subtitle = ImageFont.load_default(size=23)
    caption = ImageFont.load_default(size=17)
    draw.text((24, 9), "S2 Villager Actions", font=title, fill=INK)
    draw.text((26, 45), "Six poses per body and view. Times shown at native playback speed.",
              font=caption, fill=INK)
    for index, (key, view) in enumerate((key, view) for key in keys for view in VIEWS):
        top = HEADER + index * section_height
        display = DISPLAY_NAMES.get(key, key.replace("_", " ").capitalize())
        draw.text((7, top + 5), display, font=caption, fill=INK)
        draw.text((7, top + 23), VIEWS[view], font=caption, fill=INK)
        for group, capture in enumerate(captures):
            left = GUTTER + group * (group_width + split)
            draw.text((left + 8, top + 12), f"{capture.label}  /  {display}  /  {VIEWS[view]}",
                      font=subtitle, fill=INK)
            for row, (body, body_label) in enumerate(BODY_LABELS.items()):
                y = top + SECTION + row * ROW
                draw.text((7, y + 5), body_label, font=caption, fill=INK)
                native = capture.bodies[body]
                for col, sample in enumerate(native["clips"][key]["picks"]):
                    x = left + col * CELL
                    path = capture.frame(key, body, view, sample)
                    with Image.open(path) as frame:
                        if frame.size != (CELL, CELL):
                            raise ReviewError(f"{path}: expected {CELL} × {CELL} capture")
                        canvas.paste(frame, (x, y + LABEL_HEIGHT))
                    draw.text((x + 8, y + 4), f"{col + 1}  /  {sample / native['fps']:.2f}s",
                              font=caption, fill=INK)
    output.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(output)


def write_gif(capture: Capture, key: str, output: Path) -> None:
    figure = capture.bodies["p1"]
    clip = figure["clips"][key]
    for sample in range(clip["count"]):
        path = capture.frame(key, "p1", "quarter", sample)
        if not path.is_file():
            raise ReviewError(f"Missing native animation frame {path}")
    count = clip["count"] * (30 // figure["fps"])
    final_delay = round(count * 100 / 30) - round((count - 1) * 100 / 30)
    output.parent.mkdir(parents=True, exist_ok=True)
    pattern = capture.directory / f"{key}-p1-quarter-%03d.png"
    # Duplicate 15 fps samples in the old arm rather than synthesizing in-between poses.
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
         "-framerate", str(figure["fps"]), "-start_number", "0", "-i", str(pattern),
         "-filter_complex", "fps=30,split[a][b];[a]palettegen[p];[b][p]paletteuse=dither=bayer",
         "-frames:v", str(count), "-loop", "0" if clip["loop"] else "-1",
         "-final_delay", str(final_delay), str(output)],
        check=True,
    )
    with Image.open(output) as gif:
        if gif.n_frames != count or (gif.info.get("loop") == 0) != clip["loop"]:
            raise ReviewError(f"{output}: GIF count or playback mode disagrees with the captured animation")
        total_delay = 0
        for sample in range(gif.n_frames):
            gif.seek(sample)
            delay = gif.info["duration"]
            if delay not in (30, 40):
                raise ReviewError(f"{output}: unexpected frame delay {delay} ms")
            total_delay += delay
        if abs(total_delay - count * 1000 / 30) > 10:
            raise ReviewError(f"{output}: GIF duration drifts from the 30 fps timeline")


def detail_sheet(capture: Capture, key: str, output: Path) -> None:
    if "detail" not in capture.bodies["p1"]["clips"][key]:
        return
    cell = 440
    header = 42
    layout = Image.new("RGB", (cell * len(BODY_LABELS), (cell + header) * len(VIEWS)), BACKGROUND)
    text = ImageDraw.Draw(layout)
    font = ImageFont.load_default(size=21)
    for row, (view, label) in enumerate(VIEWS.items()):
        for col, (body, name) in enumerate(BODY_LABELS.items()):
            sample = capture.bodies[body]["clips"][key]["detail"]
            path = capture.directory / f"detail-{key}-{body}-{view}-{sample:03d}.png"
            x, y = cell * col, (cell + header) * row
            text.text((x + 12, y + 8), f"{name} / {label} / {sample / capture.bodies[body]['fps']:.2f}s",
                      font=font, fill=INK)
            with Image.open(path) as frame:
                layout.paste(frame, (x, y + header))
    layout.save(output)


def final_frame(capture: Capture, key: str, body: str, view: str, check: str) -> tuple[Path, int]:
    """Resolve an exact rendered pose from its per-body capture receipt."""
    clip = capture.bodies[body]["clips"][key]
    if not check:
        sample = clip.get("hero", clip["picks"][FINAL_PICKS.get(key, 2)])
        path = capture.frame(key, body, view, sample)
    else:
        sample = clip.get("detail", -1) if check == "apex" else clip.get("checks", {}).get(check, -1)
        prefix = "detail" if check == "apex" else f"detail-{check}"
        path = capture.directory / f"{prefix}-{key}-{body}-{view}-{sample:03d}.png"
    if not 0 <= sample < clip["count"] or not path.is_file():
        raise ReviewError(f"Missing {key} {check or 'action'} capture for {body} {view}: {path}")
    return path, sample


def stamp_frame(canvas: Image.Image, draw: ImageDraw.ImageDraw, font: ImageFont.FreeTypeFont | ImageFont.ImageFont,
                position: tuple[int, int], path: Path, sample: int, size: int = CELL) -> None:
    """Place one native frame and its timecode in the review grid."""
    if not path.is_file():
        raise ReviewError(f"Missing review frame {path}")
    with Image.open(path) as image:
        if image.size != (size, size):
            raise ReviewError(f"{path}: expected {size} × {size} capture")
        canvas.paste(image if size == CELL else image.resize((CELL, CELL), Image.Resampling.LANCZOS), position)
    x, y = position
    draw.text((x + 7, y + CELL + 2), f"Frame {sample}  /  {sample / 30:.2f}s", font=font, fill=INK)


def final_sheet(captures: tuple[Capture, ...], output: Path, *, revision: bool = False) -> None:
    """Show the full review or the two revised actions and their sampled p1 timelines."""
    by_key: dict[str, Capture] = {}
    for capture in captures:
        keys = capture.bodies["p1"]["clips"].keys()
        if any(body["fps"] != 30 or body["clips"].keys() != keys for body in capture.bodies.values()):
            raise ReviewError(f"{capture.directory}: final review requires the same 30 fps actions for all four bodies")
        for key in keys:
            if key in by_key:
                raise ReviewError(f"{key} appears in more than one capture directory")
            by_key[key] = capture
    required = REVISION_KEYS if revision else FINAL_KEYS
    missing = [key for key in required if key not in by_key]
    if missing:
        raise ReviewError(f"Missing final-review actions: {', '.join(missing)}")
    keys = required if revision else (*required, *(key for key in by_key if key not in required))
    rows = [(key, "", DISPLAY_NAMES.get(key, key.replace("_", " ").capitalize())) for key in keys]
    if not revision:
        rows.extend(FINAL_CHECKS)
    gutter, gap, header = 156, 10, 112
    group = len(VIEWS) * CELL + gap
    columns = len(BODY_LABELS)
    strip_height = SECTION + ROW * ((STRIP_COUNT + columns - 1) // columns)
    extra_height = len(keys) * strip_height if revision else SECTION
    canvas = Image.new("RGB", (gutter + group * columns, header + len(rows) * ROW + extra_height), BACKGROUND)
    draw = ImageDraw.Draw(canvas)
    title = ImageFont.load_default(size=28)
    label = ImageFont.load_default(size=19)
    small = ImageFont.load_default(size=16)
    heading = "Hammer/build and set down" if revision else "Final Review"
    draw.text((16, 10), f"S2 Villager Actions / {heading}", font=title, fill=INK)
    for col, body in enumerate(BODY_LABELS.values()):
        x = gutter + group * col
        draw.text((x + CELL, 53), body, font=label, fill=INK, anchor="mt")
        for angle, view in enumerate(VIEWS.values()):
            draw.text((x + angle * CELL + CELL // 2, 82), view, font=small, fill=INK, anchor="mt")
    for index, (key, check, name) in enumerate(rows):
        y = header + index * ROW
        if index == len(keys):
            draw.text((16, y + 9), "Carry-forward checks", font=title, fill=INK)
        if index >= len(keys):
            y += SECTION
        draw.text((10, y + 27), name, font=label, fill=INK)
        for col, body in enumerate(BODY_LABELS):
            for angle, view in enumerate(VIEWS):
                x = gutter + col * group + angle * CELL
                path, sample = final_frame(by_key[key], key, body, view, check)
                stamp_frame(canvas, draw, small, (x, y), path, sample, 440 if check else CELL)
    if revision:
        for action_index, (key, _, name) in enumerate(rows):
            capture = by_key[key]
            clip = capture.bodies["p1"]["clips"][key]
            samples = clip.get("strip", [])
            if (len(samples) != STRIP_COUNT or sorted(set(samples)) != samples
                    or not 0 <= samples[0] < samples[-1] < clip["count"]):
                raise ReviewError(f"{key} needs twelve distinct p1 frames in playback order")
            top = header + len(rows) * ROW + action_index * strip_height
            draw.text((16, top + 9), f"{name} / p1 / {STRIP_COUNT} frames / Side then front 3/4",
                      font=label, fill=INK)
            for sample_index, sample in enumerate(samples):
                row, col = divmod(sample_index, columns)
                y = top + SECTION + row * ROW
                for angle, view in enumerate(VIEWS):
                    x = gutter + col * group + angle * CELL
                    stamp_frame(canvas, draw, small, (x, y), capture.frame(key, "p1", view, sample), sample)
    output.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(output)


def main(argv: list[str]) -> int:
    """Run one captured spec or the matched CLIPS1/MOCAP1 review."""
    match argv:  # noqa: MATCH_OK - Unknown argument combinations print usage.
        case ["contact", root]:
            capture = read_capture(Path(root), "MOCAP1")
            make_sheet((capture,), tuple(capture.bodies["p1"]["clips"]), Path(root) / "contact-sheet.png")
        case ["contact", root, "--output", output]:
            capture = read_capture(Path(root), "MOCAP1")
            make_sheet((capture,), tuple(capture.bodies["p1"]["clips"]), Path(output))
        case ["final", *roots, "--output", output] if roots:
            final_sheet(tuple(read_capture(Path(root), "MOCAP1") for root in roots), Path(output))
        case ["revision", root, "--output", output]:
            final_sheet((read_capture(Path(root), "MOCAP1"),), Path(output), revision=True)
        case ["compare", old, new, output]:
            old_capture = read_capture(Path(old), "CLIPS1 keyframe")
            new_capture = read_capture(Path(new), "MOCAP1")
            destination = Path(output)
            make_sheet((old_capture, new_capture), PILOT, destination / "compare-sheet.png")
            detail_sheet(new_capture, "chop", destination / "chop-apex.png")
            for key in PILOT:
                for src, suffix in ((old_capture, "old"), (new_capture, "new")):
                    name = DISPLAY_NAMES[key].lower()
                    write_gif(src, key, destination / "gif" / f"{name}_{suffix}.gif")
        case _:
            print("Usage: mocap_review.py contact BAKED_DIR [--output FILE] | compare OLD NEW OUTPUT_DIR"
                  " | final BAKED_DIR [BAKED_DIR ...] --output FILE | revision BAKED_DIR --output FILE",
                  file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
