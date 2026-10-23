# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by sprite_frame_diagnostic.py and sprite_build.py.
"""Merge hash-bound source envelopes by actual look, facing, and pose."""
from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_regions import EnvelopeReport, SourceEnvelope

FrameKey = tuple[str, str, str]
REPORTS: Final = ("OWNER_SOURCE_ENVELOPES.json", "IDLE_SOURCE_ENVELOPES.json",
                  "ADULT_POSE_SOURCE_ENVELOPES.json", "CHILD_POSE_SOURCE_ENVELOPES.json")


@dataclass(frozen=True, slots=True)
class SourceIndex:
    frames: dict[FrameKey, SourceEnvelope]
    report_hashes: dict[str, str]
    frame_reports: dict[FrameKey, str]


def load_envelopes(snapshot_folder: Path | None = None, *, require_complete: bool = True) -> SourceIndex:
    plan = FacePlan.read()
    expected = {(look.look, facing, pose) for look in plan.looks for facing in ("S", "E")
                for pose in ("idle", "walk_a", "work_a", "carry", "sleep")}
    frames: dict[FrameKey, SourceEnvelope] = {}
    hashes: dict[str, str] = {}
    origins: dict[FrameKey, str] = {}
    for name in REPORTS:
        path = ROOT / "docs/v8/artpolish2" / name
        if not path.is_file():
            continue
        data = path.read_bytes()
        if snapshot_folder is not None:
            _ = (snapshot_folder / name).write_bytes(data)
        relative = str(path.relative_to(ROOT))
        hashes[relative] = hashlib.sha256(data).hexdigest()
        for entry in EnvelopeReport.model_validate_json(data).frames:
            key = (entry.look, entry.facing, entry.pose)
            if key not in expected or key in frames:
                message = f"Unexpected or duplicate source frame: {key} from {relative}"
                raise ValueError(message)
            source = ROOT / "art/character/route_i" / entry.look / entry.facing / f"{entry.pose}.png"
            if entry.source is not None and entry.source != str(source.relative_to(ROOT)):
                message = f"Envelope source path disagrees with frame key: {key}"
                raise ValueError(message)
            if hashlib.sha256(source.read_bytes()).hexdigest() != entry.source_sha256:
                message = f"Envelope source hash mismatch: {key}"
                raise ValueError(message)
            frames[key], origins[key] = entry, relative
    if require_complete and frames.keys() != expected:
        missing = sorted(expected - frames.keys())
        message = f"Source envelopes cover {len(frames)}/340; missing {missing[:5]}"
        raise ValueError(message)
    return SourceIndex(frames, hashes, origins)
