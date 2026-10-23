# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by portrait processing and blind scoring.
"""Append immutable observation/score revisions without erasing earlier failures."""
from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel

from art.pipeline_polish2.blind_round import seal
from art.pipeline_polish2.portrait_models import AssetRef


def preserve_input(source: Path, directory: Path) -> AssetRef:
    reference = AssetRef.of(source)
    return seal(directory / f"{reference.sha256}{source.suffix}", source.read_bytes())


def record_revision(path: Path, value: BaseModel) -> AssetRef:
    """Keep the first result at its original path and number distinct later results."""
    payload = (value.model_dump_json(indent=2) + "\n").encode()
    existing = (path, *sorted(path.parent.glob(f"{path.stem}.revision*{path.suffix}")))
    for candidate in existing:
        if candidate.is_file() and candidate.read_bytes() == payload:
            return AssetRef.of(candidate)
    destination = path if not path.exists() else path.with_name(f"{path.stem}.revision{len(existing) + 1:03d}{path.suffix}")
    return seal(destination, payload)
