"""Load a WorldSim recorded episode (recorder schema 1 of Pass 7, schema 2 of
Pass 8, schema 3 of Pass 9 or schema 4 of Pass 10). numpy only.

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

Schema 3 adds, with `--agents`, `agent_keys.npy` (uint64 [N], aligned with
`agent_ids.npy`): each agent's lineage key, 0 when it has none. It is an
identity field (`"identity": true`, hidden), never an input: agents are
matched across the arms of a fork by lineage key, never by `AgentId`
(`ep.lineage_keys()`). `meta["keyed_draws"]` and `meta["noise_fork"]` (null or
`{tick, replicate}`) describe the randomness of the run.

Schema 4 (Pass 10) appends `deaths_exposure` and `injured_members` to
`global.npy` and, with `--agents`, `hp` as the last field of `agents.npy`;
`meta["world_variety"]` is null or `{"hidden": true, "values": {...}}`, the
world's hidden parameters (`game_v16`). Hidden is marked inside the block, so
`ep.observable_meta()` drops it and `ep.world_variety()` reads it (labels and
analysis only, never an input).

A branch family (`--fork-tick T --replicates R`, `make_dataset.py families`) is
one directory per root seed; `load_family(path)` returns its factual and
variants by replicate (`fam.factual[k]`, `fam.variants["rain"][k]`).

`load_episode` verifies the sha256 of every array file against meta.json
(raises ValueError on a mismatch) unless verify=False.
"""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

SCHEMAS = (1, 2, 3, 4, 5, 6)


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

    def lineage_keys(self) -> np.ndarray | None:
        """Lineage key per agent in `agent_ids.npy` order (schema 3 with agents), else None."""
        return self.arrays.get("agent_keys.npy")

    def observable_meta(self) -> dict:
        """`meta` without its hidden blocks (schema 4: any top-level entry that is a dict
        marked `"hidden": true`, i.e. `world_variety`)."""
        return {k: v for k, v in self.meta.items() if not (isinstance(v, dict) and v.get("hidden") is True)}

    def world_variety(self) -> dict | None:
        """The hidden world parameters (schema 4, `game_v16`), else None. Ground truth only."""
        block = self.meta.get("world_variety")
        return block.get("values") if isinstance(block, dict) else None

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


@dataclass
class Family:
    """A branch family (Pass 9): one root seed's factual and variants, each as R
    replicates sharing the prefix up to the fork tick."""
    path: Path
    seed: int
    scene: str
    fork_tick: int
    replicates: int
    factual: dict[int, Episode]
    variants: dict[str, dict[int, Episode]]


def load_family(path, verify: bool = True, arrays: list[str] | None = None) -> Family:
    """Load the family directory `path` (members `<scene>_s<seed>_<id>_r<k>`): the factual
    (`id` = `none`) and every variant, by replicate. Members must share scene, seed and
    fork tick, carry `noise_fork.replicate` = k, and every id must have replicates 0..R-1."""
    path = Path(path)
    members: dict[str, dict[int, Episode]] = {}
    scene = seed = fork_tick = None
    for member in sorted(p for p in path.iterdir() if (p / "meta.json").exists()):
        ep = load_episode(member, verify=verify, arrays=arrays)
        fork = ep.meta.get("noise_fork")
        if ep.meta.get("schema_version") not in (3, 4, 5, 6) or not fork:
            raise ValueError(f"{member}: not a branch-family member (schema 3 to 6 with noise_fork)")
        key = (ep.meta["scene"], int(ep.meta["seed"]), int(fork["tick"]))
        if scene is None:
            scene, seed, fork_tick = key
        elif key != (scene, seed, fork_tick):
            raise ValueError(f"{member}: scene, seed or fork tick differs from the family")
        replicate = int(fork["replicate"])
        if not member.name.endswith(f"_r{replicate}"):
            raise ValueError(f"{member}: directory name and noise_fork.replicate differ")
        members.setdefault(ep.meta["episode_id"], {})[replicate] = ep
    if "none" not in members:
        raise ValueError(f"{path}: no factual member")
    r = len(members["none"])
    for eid, reps in members.items():
        if sorted(reps) != list(range(r)):
            raise ValueError(f"{path}: {eid} has replicates {sorted(reps)}, expected 0..{r - 1}")
    return Family(path, seed, scene, fork_tick, r, members.pop("none"), members)


def verify_episode(path) -> bool:
    """True when every array file's sha256 matches meta.json."""
    try:
        load_episode(path, verify=True, arrays=[])
        meta = json.loads((Path(path) / "meta.json").read_text())
        return all(sha256_file(Path(path) / name) == entry["sha256"] for name, entry in meta["arrays"].items())
    except (OSError, ValueError, KeyError, json.JSONDecodeError):
        return False
