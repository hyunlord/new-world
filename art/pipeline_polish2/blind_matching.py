# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m art.pipeline_polish2.blind_matching A_DESCRIPTIONS B_DESCRIPTIONS
"""Seal visual descriptions and build opposite-reviewer matching packets."""
from __future__ import annotations

import json
import random
from pathlib import Path
from typing import Final

import typer

from art.pipeline_polish2.blind_models import (
    Descriptions,
    ImageLabel,
    LineId,
    MatchingKey,
    PrivateKey,
)
from art.pipeline_polish2.blind_packets import LABELS, portrait_row
from art.pipeline_polish2.blind_round import check_response_agent, load_round
from art.pipeline_polish2.portrait_evidence import preserve_input
from art.pipeline_polish2.portrait_models import (
    DATA,
    AssetRef,
    write_evidence,
)

LINES: Final[tuple[LineId, ...]] = ("1", "2", "3", "4", "5", "6")


def validate_descriptions(response: Descriptions, key: PrivateKey) -> None:
    """Reject incomplete responses before exposing them to the other reviewer."""
    check_response_agent(key, response)
    ids = [row.id for row in response.descriptions]
    if len(ids) != len(set(ids)) or set(ids) != {row.id for row in key.descriptions}:
        message = "Description IDs must match all eight packet rows exactly once"
        raise ValueError(message)
    for row in response.descriptions:
        if set(row.lines) != set(LABELS) or any(not line.strip() for line in row.lines.values()):
            message = f"Six nonempty A-F descriptions required: {row.id}"
            raise ValueError(message)


def build_matching(base: Path, receiver: PrivateKey, source_path: Path) -> PrivateKey:
    """Give the receiver the other reviewer's descriptions with unrelated shuffles."""
    response = Descriptions.model_validate_json(source_path.read_bytes())
    if response.reviewer == receiver.reviewer:
        message = "A reviewer must not match their own descriptions"
        raise ValueError(message)
    source = PrivateKey.model_validate_json((base / "private" / f"{response.reviewer}.json").read_bytes())
    if source.round_manifest != receiver.round_manifest:
        message = "Description and matching keys must bind the same frozen round"
        raise ValueError(message)
    _, candidates = load_round(receiver)
    _ = load_round(source)
    validate_descriptions(response, source)
    for row in source.descriptions:
        row.image.verify()
    response_ref = preserve_input(source_path, base / "private/responses/descriptions")
    by_candidate = {row.key: row for row in candidates.candidates}
    by_id = {row.id: row for row in response.descriptions}
    rng = random.Random(receiver.seed ^ 0x729F615CB4E83A01)
    order = list(source.descriptions)
    rng.shuffle(order)
    destination = base / "packets" / receiver.reviewer / "match"
    destination.mkdir(parents=True, exist_ok=False)
    matches: list[MatchingKey] = []
    for index, item in enumerate(order):
        identifier = f"M{index + 1:02d}"
        options = list(item.options.values())
        rng.shuffle(options)
        descriptions = [(by_id[item.id].lines[label], candidate) for label, candidate in item.options.items()]
        rng.shuffle(descriptions)
        path = destination / f"{identifier}.png"
        portrait_row(identifier, tuple(by_candidate[key] for key in options)).save(path, optimize=True)
        lines_path = destination / f"{identifier}.txt"
        _ = lines_path.write_text("\n".join(f"{number}. {text}" for number, (text, _) in enumerate(descriptions, 1)) + "\n")
        correct: dict[LineId, ImageLabel] = {line: LABELS[options.index(candidate)] for line, (_, candidate) in zip(LINES, descriptions, strict=True)}
        matches.append(MatchingKey(id=identifier, look=item.look, correct=correct,
                       targets={line: candidate for line, (_, candidate) in zip(LINES, descriptions, strict=True)},
                       options=dict(zip(LABELS, options, strict=True)), image=AssetRef.of(path),
                       lines=AssetRef.of(lines_path), description_response=response_ref))
    instruction = (
        "Inspect only this directory. For each M01..M08, read its six descriptions in the matching .txt "
        "file and independently match each line1..6 to one portrait A..F in the .png. Repeated letters "
        "are permitted when uncertain. The lines were written by another reviewer without production notes. "
        "View at original resolution; do not inspect keys, manifests, source files or other packets. "
        'Return JSON: {"reviewer":"REVIEWER","agent_task":"AGENT_TASK","session_id":SESSION_ID,"answers":[{"id":"M01",'
        '"matches":{"1":"A","2":"B","3":"C","4":"D","5":"E","6":"F"}},...]}\n')
    _ = (destination / "INSTRUCTIONS.txt").write_text(instruction.replace("REVIEWER", receiver.reviewer)
        .replace("AGENT_TASK", receiver.agent.agent_task).replace("SESSION_ID", json.dumps(receiver.agent.session_id)))
    return receiver.model_copy(update={"matching": tuple(matches)})


def main(a_descriptions: Path, b_descriptions: Path, round_number: int = 0) -> None:
    base = DATA / "blind" / f"round{round_number}"
    for receiver, source in (("A", b_descriptions), ("B", a_descriptions)):
        key_path = base / "private" / f"{receiver}.json"
        key = PrivateKey.model_validate_json(key_path.read_bytes())
        if key.round != round_number:
            message = "Private key round differs from the requested matching round"
            raise ValueError(message)
        write_evidence(key_path, build_matching(base, key, source))
    print(f"Opposite-reviewer matching packets ready: {base / 'packets'}")


if __name__ == "__main__":
    typer.run(main)
