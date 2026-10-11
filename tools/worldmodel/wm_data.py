"""Load a WorldSim recorded episode (recorder schema 1 of Pass 7 or schema 2 of
Pass 8). numpy only.

    from wm_data import load_episode
    ep = load_episode("/path/game_v14_s0_none")
    ep.meta["scene"], ep.arrays["global.npy"].shape
    ep.column("global.npy", "wood_trips_total")      # [T]
    ep.column("settlements.npy", "wood_stock")       # [T, S]
    ep.observable("global.npy")                      # [T, G_obs], hidden columns dropped
    ep.observable_names("global.npy")                # their names, in column order

Schema 2 marks the world's hidden weather (`winter_severity`, the night's burn
and the day's fall), the settlement `wood_target` and every `latent.npy`
column as hidden; `meta["observable_fields"]` lists the rest per array. A model
or baseline that must not see the hidden weather reads `observable` only.

`load_episode` verifies the sha256 of every array file against meta.json
(raises ValueError on a mismatch) unless verify=False.
"""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

SCHEMAS = (1, 2)


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

    def observable_names(self, array: str) -> list[str]:
        """Names of the observable (not hidden) fields of `array`, in column order:
        `meta["observable_fields"]` in schema 2, the fields without `hidden` in schema 1."""
        listed = self.meta.get("observable_fields", {}).get(array)
        if listed is not None:
            return list(listed)
        return [f["name"] for f in self.meta["fields"][array] if not f.get("hidden", False)]

    def observable(self, array: str) -> np.ndarray:
        """`array` with its hidden columns dropped (last axis = `observable_names`)."""
        index = [self.field_index(array, name) for name in self.observable_names(array)]
        return self.arrays[array][..., index]


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
    if meta.get("schema_version") not in SCHEMAS:
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
