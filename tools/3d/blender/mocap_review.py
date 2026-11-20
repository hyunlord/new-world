#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow>=10.1"]
# ///
# How to run: python3 tools/3d/blender/mocap_review.py contact BAKED_DIR
"""Assemble review sheets and 30 fps GIFs from captures of the baked VAT."""

from dataclasses import dataclass
import json
from pathlib import Path
import subprocess
import sys
from typing import Final, TypedDict

from PIL import Image, ImageDraw, ImageFont

CELL: Final = 260
BODY_LABELS: Final = {"p1": "p1  Man", "p2": "p2  Woman", "p3": "p3  Child", "p4": "p4  Elder"}
DISPLAY_NAMES: Final = {"chop": "Chop", "cook": "Stir"}
VIEWS: Final = {"side": "Side", "quarter": "Front 3/4"}
PILOT: Final = ("chop", "cook")
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


def main(argv: list[str]) -> int:
    """Run one captured spec or the matched CLIPS1/MOCAP1 review."""
    match argv:
        case ["contact", root]:
            capture = read_capture(Path(root), "MOCAP1")
            make_sheet((capture,), tuple(capture.bodies["p1"]["clips"]), Path(root) / "contact-sheet.png")
        case ["contact", root, "--output", output]:
            capture = read_capture(Path(root), "MOCAP1")
            make_sheet((capture,), tuple(capture.bodies["p1"]["clips"]), Path(output))
        case ["compare", old, new, output]:
            old_capture = read_capture(Path(old), "CLIPS1 keyframe")
            new_capture = read_capture(Path(new), "MOCAP1")
            destination = Path(output)
            make_sheet((old_capture, new_capture), PILOT, destination / "compare-sheet.png")
            for key in PILOT:
                for src, suffix in ((old_capture, "old"), (new_capture, "new")):
                    name = DISPLAY_NAMES[key].lower()
                    write_gif(src, key, destination / "gif" / f"{name}_{suffix}.gif")
        case _:
            print("Usage: mocap_review.py contact BAKED_DIR [--output FILE] | compare OLD NEW OUTPUT_DIR",
                  file=sys.stderr)
            return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
