# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic"]
# ///
# How to run: uv run --with pillow --with pydantic python -m art.pipeline_polish.portrait_packets
"""Render identity questions with answer-bearing metadata kept outside packets."""
from __future__ import annotations

import hashlib
import json
import random
from pathlib import Path
from typing import ClassVar, Final

from PIL import Image, ImageDraw, ImageOps
from pydantic import BaseModel, ConfigDict, JsonValue

from art.pipeline_polish.portrait_prepare import DATA, ROOT, TARGETS

SEED: Final = 20261024
PAPER: Final = "#e5dcc8"


class Look(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    look: str
    canonical_path: Path


class Baseline(BaseModel):
    records: list[Look]


def attributes(look: str) -> tuple[str, str, str]:
    age, sex, rest = look.split("_", 2)
    hair = "long_tied" if rest.endswith("long_tied") else rest.rsplit("_", 1)[1]
    return f"{age}_{sex}", rest[: -len(hair) - 1], hair


def distractors(target: str, population: tuple[str, ...]) -> tuple[str, ...]:
    """Choose same-top, same-hair, then both-different controls in lexical order."""
    cohort, top, hair = attributes(target)
    pool = sorted(name for name in population if name != target and attributes(name)[0] == cohort)
    choices: list[str] = []
    groups = (
        [name for name in pool if attributes(name)[1] == top and attributes(name)[2] != hair],
        [name for name in pool if attributes(name)[2] == hair and attributes(name)[1] != top],
        [name for name in pool if attributes(name)[1] != top and attributes(name)[2] != hair],
        pool,
    )
    for group in groups:
        for name in group:
            if name not in choices:
                choices.append(name)
                break
        if len(choices) == 3:
            return tuple(choices)
    message = f"Fewer than three distinct same-age/sex distractors for {target}"
    raise ValueError(message)


def selected(row: Look) -> Path:
    if row.look in TARGETS:
        return ROOT / "art/portrait/polish" / f"{row.look}_try1.png"
    return ROOT / row.canonical_path


def sprite(look: str) -> Image.Image:
    with Image.open(ROOT / "art/character/route_i" / look / "S/idle.png") as source:
        return source.convert("RGBA")


def main() -> None:
    baseline = Baseline.model_validate_json((ROOT / "docs/v8/artenv1/portrait_manifest.json").read_text())
    population = tuple(row.look for row in baseline.records)
    order = sorted(baseline.records, key=lambda row: row.look)
    rng = random.Random(SEED)
    rng.shuffle(order)
    packet = DATA / "blind/packet"
    packet.mkdir(parents=True, exist_ok=True)
    keys: list[dict[str, JsonValue]] = []
    pages: list[Image.Image] = []
    for index, row in enumerate(order):
        identifier = f"P{index + 1:03d}"
        options = [row.look, *distractors(row.look, population)]
        rng.shuffle(options)
        panel = Image.new("RGB", (900, 276), PAPER)
        draw = ImageDraw.Draw(panel)
        draw.text((12, 8), f"{identifier} | Which person is in the portrait? Choose A, B, C or D.", fill="#33271d")
        with Image.open(selected(row)) as portrait:
            panel.paste(portrait, (18, 76))
        for option_index, name in enumerate(options):
            person = sprite(name)
            person = ImageOps.scale(person, 2, Image.Resampling.NEAREST)
            x = 180 + option_index * 176
            panel.paste(person, (x, 36), person)
            draw.text((x + 76, 24), "ABCD"[option_index], fill="#33271d")
        path = packet / f"{identifier}.png"
        panel.save(path, optimize=True)
        if index % 4 == 0:
            pages.append(Image.new("RGB", (900, 1104), PAPER))
        pages[-1].paste(panel, (0, index % 4 * 276))
        keys.append({"id": identifier, "look": row.look, "correct": "ABCD"[options.index(row.look)],
                     "options": dict(zip("ABCD", options, strict=True)), "image_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                     "portrait": str(selected(row).relative_to(ROOT))})
    for page_index, page in enumerate(pages):
        page.save(packet / f"page_{page_index + 1:02d}.png", optimize=True)
    _ = (DATA / "blind/answer_key.json").write_text(json.dumps(keys, indent=2) + "\n")
    _ = (packet / "INSTRUCTIONS.txt").write_text(
        "Match each portrait to one of its four sprites A/B/C/D. There are 34 questions. " +
        "Sprites are at exactly 2x nearest scale; portraits are 128px. View pages at original resolution. " +
        "Choose exactly one letter per question, even if uncertain, and optionally give confidence. " +
        "Do not inspect outside this packet, filenames elsewhere, manifests, code or answer keys. " +
        "Return a JSON object mapping P001..P034 to A/B/C/D.\n"
    )
    board = Image.new("RGB", (6 * 260, 6 * 168 + 36), PAPER)
    draw = ImageDraw.Draw(board)
    draw.text((10, 10), "POLISH | 34 PORTRAIT CANDIDATES | 128px + 64px + route I sprite0.5x | OWNER CHOICE PENDING", fill="#33271d")
    for index, row in enumerate(sorted(baseline.records, key=lambda row: row.look)):
        x, y = index % 6 * 260 + 4, index // 6 * 168 + 36
        with Image.open(selected(row)) as portrait:
            board.paste(portrait, (x, y))
            board.paste(ImageOps.scale(portrait, 0.5, Image.Resampling.BILINEAR), (x + 132, y + 32))
        person = sprite(row.look)
        person = ImageOps.scale(person, 0.5, Image.Resampling.LANCZOS)
        board.paste(person, (x + 200, y + 25), person)
        draw.text((x, y + 132), row.look, fill="#33271d")
        draw.text((x, y + 146), "CANDIDATE", fill="#7c3324")
    board.save(ROOT / "art/boards/portraits_owner.png", optimize=True)
    print(json.dumps({"questions": len(keys), "packet": str(packet), "owner_bytes": (ROOT / "art/boards/portraits_owner.png").stat().st_size}))


if __name__ == "__main__":
    main()
