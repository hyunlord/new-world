"""Load a WorldSim recorded episode (Pass 7 recorder, schema 1). numpy only.

    from wm_data import load_episode
    ep = load_episode("/path/game_v13_s0_none")
    ep.meta["scene"], ep.arrays["global.npy"].shape
    ep.column("global.npy", "wood_trips_total")      # [T]
    ep.column("settlements.npy", "wood_stock")       # [T, S]

`load_episode` verifies the sha256 of every array file against meta.json
(raises ValueError on a mismatch) unless verify=False.
"""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np


@dataclass
class Episode:
    path: Path
    meta: dict
    arrays: dict[str, np.ndarray] = field(default_factory=dict)
    interventions: list = field(default_factory=list)

    def field_index(self, array: str, name: str) -> int:
        """Column index of field `name` in `array` (the last axis)."""
        names = [f["name"] for f in self.meta["fields"][array]]
        try:
            return names.index(name)
        except ValueError:
            raise KeyError(f"{array} has no field {name}; fields: {names}") from None

    def column(self, array: str, name: str) -> np.ndarray:
        """The values of field `name`, with the last axis dropped."""
        return self.arrays[array][..., self.field_index(array, name)]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def load_episode(path, verify: bool = True, arrays: list[str] | None = None) -> Episode:
    """Load meta.json, interventions.json and the arrays (all, or `arrays`)."""
    path = Path(path)
    meta = json.loads((path / "meta.json").read_text())
    if meta.get("schema_version") != 1:
        raise ValueError(f"{path}: unsupported recorder schema {meta.get('schema_version')}")
    episode = Episode(path, meta, interventions=json.loads((path / "interventions.json").read_text()))
    for name, entry in meta["arrays"].items():
        if arrays is not None and name not in arrays:
            continue
        file = path / name
        if verify and sha256_file(file) != entry["sha256"]:
            raise ValueError(f"{file}: sha256 does not match meta.json")
        data = np.load(file, allow_pickle=False)
        if list(data.shape) != entry["shape"] or str(data.dtype) != entry["dtype"]:
            raise ValueError(f"{file}: shape or dtype differs from meta.json")
        episode.arrays[name] = data
    return episode


def verify_episode(path) -> bool:
    """True when every array file's sha256 matches meta.json."""
    try:
        load_episode(path, verify=True, arrays=[])
        meta = json.loads((Path(path) / "meta.json").read_text())
        return all(sha256_file(Path(path) / name) == entry["sha256"] for name, entry in meta["arrays"].items())
    except (OSError, ValueError, KeyError, json.JSONDecodeError):
        return False
