# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m art.pipeline_polish2.blind_packets --round-number 0
"""Build anonymous portrait rows and tinted identity questions with private keys."""
from __future__ import annotations

import json
import random
import secrets
from pathlib import Path
from typing import Final

import typer
from PIL import Image, ImageDraw

from art.compositor.face_variants import FacePlan
from art.pipeline_polish2.blind_models import (
    DescriptionKey,
    IdentityKey,
    ImageLabel,
    OptionLabel,
    PrivateKey,
    ReviewRoster,
)
from art.pipeline_polish2.blind_round import freeze_round, load_round, seal_model
from art.pipeline_polish2.blind_sample import identity_options, sample
from art.pipeline_polish2.portrait_models import (
    DATA,
    AssetRef,
    Candidate,
    write_evidence,
)

LABELS: Final[tuple[ImageLabel, ...]] = ("A", "B", "C", "D", "E", "F")
OPTIONS: Final[tuple[OptionLabel, ...]] = ("A", "B", "C", "D")
PAPER: Final = "#e5dcc8"


def portrait_row(identifier: str, candidates: tuple[Candidate, ...]) -> Image.Image:
    """Paste six portraits at native 128px without metadata or source labels."""
    panel = Image.new("RGB", (6 * 148 + 16, 172), PAPER)
    draw = ImageDraw.Draw(panel)
    draw.text((8, 6), identifier, fill="#33271d")
    for index, row in enumerate(candidates):
        row.portrait.verify()
        with Image.open(row.portrait.path) as portrait:
            if portrait.size != (128, 128):
                message = f"Expected 128px candidate: {row.key}"
                raise ValueError(message)
            panel.paste(portrait.convert("RGB"), (12 + index * 148, 28))
        draw.text((72 + index * 148, 158), LABELS[index], fill="#33271d")
    return panel


def identity_panel(identifier: str, target: Candidate, candidates: tuple[Candidate, ...]) -> Image.Image:
    """Show the portrait at 128px and four full sprites at exact nearest 2x."""
    panel = Image.new("RGB", (900, 276), PAPER)
    draw = ImageDraw.Draw(panel)
    draw.text((10, 8), f"{identifier}: Which sprite belongs to this portrait?", fill="#33271d")
    target.portrait.verify()
    with Image.open(target.portrait.path) as portrait:
        panel.paste(portrait.convert("RGB"), (18, 76))
    for index, row in enumerate(candidates):
        row.sprite.verify()
        with Image.open(row.sprite.path) as raw:
            sprite = raw.convert("RGBA").resize((raw.width * 2, raw.height * 2), Image.Resampling.NEAREST)
        x = 180 + index * 176
        panel.paste(sprite, (x, 36), sprite)
        draw.text((x + 76, 24), OPTIONS[index], fill="#33271d")
    return panel


def make_reviewer(key: PrivateKey, base: Path) -> PrivateKey:
    """Create one sealed randomized packet; keys remain outside reviewer directories."""
    manifest, candidates = load_round(key)
    selection = sample(candidates, FacePlan.read(manifest.frozen_plan.path))
    by_key = {row.key: row for row in candidates.candidates}
    rng = random.Random(key.seed)
    destination = base / "packets" / key.reviewer
    describe_dir, identity_dir = destination / "describe", destination / "identity"
    describe_dir.mkdir(parents=True, exist_ok=False)
    identity_dir.mkdir(parents=True, exist_ok=False)
    looks = list(selection.distinctness_looks)
    rng.shuffle(looks)
    descriptions: list[DescriptionKey] = []
    for index, look in enumerate(looks):
        identifier = f"D{index + 1:02d}"
        rows = [by_key[f"{look}/v{variant}"] for variant in range(1, 7)]
        rng.shuffle(rows)
        path = describe_dir / f"{identifier}.png"
        portrait_row(identifier, tuple(rows)).save(path, optimize=True)
        descriptions.append(DescriptionKey(id=identifier, look=look,
                            options=dict(zip(LABELS, (row.key for row in rows), strict=True)), image=AssetRef.of(path)))
    targets = list(selection.identity_keys)
    rng.shuffle(targets)
    identities: list[IdentityKey] = []
    for index, target_key in enumerate(targets):
        identifier = f"I{index + 1:02d}"
        options = list(identity_options(by_key[target_key], candidates))
        rng.shuffle(options)
        path = identity_dir / f"{identifier}.png"
        identity_panel(identifier, by_key[target_key], tuple(by_key[option] for option in options)).save(path, optimize=True)
        identities.append(IdentityKey(id=identifier, target=target_key, correct=OPTIONS[options.index(target_key)],
                          options=dict(zip(OPTIONS, options, strict=True)), image=AssetRef.of(path)))
    describe_instruction = (
        "Inspect only this directory, at original image resolution. For each D01..D08, write one concise "
        "visually specific description for EACH portrait A..F. Mention visible traits that distinguish it "
        "from the others. No outside files, manifests, code, keys or intended variant descriptions. "
        'Return JSON: {"reviewer":"REVIEWER","agent_task":"AGENT_TASK","session_id":SESSION_ID,"descriptions":[{"id":"D01",'
        '"lines":{"A":"...","B":"...","C":"...","D":"...","E":"...","F":"..."}},...]}\n')
    _ = (describe_dir / "INSTRUCTIONS.txt").write_text(describe_instruction.replace("REVIEWER", key.reviewer)
        .replace("AGENT_TASK", key.agent.agent_task).replace("SESSION_ID", json.dumps(key.agent.session_id)))
    identity_instruction = (
        "Inspect only this directory at original resolution. Match each portrait to one of four tinted "
        "idle sprites. Sprites are exactly 2x nearest; portraits128px. Choose A/B/C/D even if uncertain. "
        "Do not inspect outside files, keys, source names, variant descriptions or other reviewers. "
        'Return JSON: {"reviewer":"REVIEWER","agent_task":"AGENT_TASK","session_id":SESSION_ID,"answers":[{"id":"I01","answer":"A"},...]} '
        "with all I01..I36 exactly once.\n")
    _ = (identity_dir / "INSTRUCTIONS.txt").write_text(identity_instruction.replace("REVIEWER", key.reviewer)
        .replace("AGENT_TASK", key.agent.agent_task).replace("SESSION_ID", json.dumps(key.agent.session_id)))
    return key.model_copy(update={"descriptions": tuple(descriptions), "identity": tuple(identities)})


def main(roster_path: Path, round_number: int = 0) -> None:
    if round_number not in (0, 1):
        message = "Only original round0 and one fresh-reviewer retry round1 are permitted"
        raise ValueError(message)
    roster = ReviewRoster.model_validate_json(roster_path.read_bytes())
    if roster.round != round_number:
        message = "Roster round differs from the requested packet round"
        raise ValueError(message)
    base = DATA / "blind" / f"round{round_number}"
    manifest = freeze_round(base, DATA / "candidates.json", roster)
    for assignment in roster.assignments:
        key = PrivateKey(round=round_number, reviewer=assignment.reviewer, agent=assignment.agent,
                         round_manifest=manifest, seed=secrets.randbits(64), descriptions=(), identity=())
        key = make_reviewer(key, base)
        _ = seal_model(base / "private" / f"{key.reviewer}.initial.json", key)
        write_evidence(base / "private" / f"{key.reviewer}.json", key)
    print(f"Ready: {base / 'packets'}; private keys are not reviewer inputs")


if __name__ == "__main__":
    typer.run(main)
