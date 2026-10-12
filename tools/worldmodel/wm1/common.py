"""Shared loading, split roles and run bookkeeping for WM-1 scripts."""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
import bench_v0 as b
from wm1.data import EpisodeData, load_split

REPO = Path(__file__).resolve().parents[3]
MANIFEST = REPO / "docs/worldmodel/dataset_v0_manifest.json"
BENCH_JSON = REPO / "docs/worldmodel/bench_v0.json"
DEFAULT_ROOT = Path(os.environ.get("WM_DATA_ROOT", "/home/hyunlord/work/worldsim-wm-data"))
TEST_SPLITS = ("A_test", "B_test", "OOD_v12", "OOD_long")
VAL_SPLITS = ("A_val", "B_val")
GROUPS = {"A_train": ("A_obs", "train"), "A_val": ("A_obs", "val"), "A_test": ("A_obs", "test"),
          "B_train": ("B_act", "train"), "B_val": ("B_act", "val"), "B_test": ("B_act", "test"),
          "C_val": ("C_cf", "val"), "C_test": ("C_cf", "test"), "OOD_v12": ("OOD_v12", "test"),
          "OOD_long": ("OOD_long", "test")}


PILOT_MANIFEST = REPO / "docs/worldmodel/wm1/p10_pilot_manifest.json"


@dataclass(frozen=True)
class Dataset:
    """What a WM-1 script needs to know about one dataset (GOAL_1_2.md 6.3). Group names map to (manifest split,
    role); `train` / `train_action` are the training groups of action-blind / action-aware variants; `cf` is the
    counterfactual group (None: no T3 / T4); `tasks` is the `evaluate.evaluate` task list (None: every v0 task);
    `observable` marks recorder-schema >= 2 data (observable feature table, no severity input)."""
    name: str
    manifest: Path
    groups: dict = field(hash=False)
    train: tuple
    train_action: tuple
    val: tuple
    test: tuple
    cf: str | None
    t3_val: str | None
    t3_test: str | None
    tasks: tuple | None = None
    observable: bool = False

    def train_groups(self, action_aware: bool) -> list[str]:
        return list(self.train_action if action_aware else self.train)


def _pilot(name: str, scene: str) -> Dataset:
    return Dataset(name, PILOT_MANIFEST, {"train": (scene, "train"), "val": (scene, "val"), "test": (scene, "test")},
                   ("train",), ("train",), ("val",), ("test",), None, None, None, ("T1", "T2", "T2_any"), True)


DATASETS = {
    "v0": Dataset("v0", MANIFEST, GROUPS, ("A_train",), ("A_train", "B_train"), VAL_SPLITS, TEST_SPLITS, "C_test",
                  "A_val", "A_test"),
    "p10_v16": _pilot("p10_v16", "game_v16"),
    "p10_v16n": _pilot("p10_v16n", "game_v16_neutral"),
    "p10_v15": _pilot("p10_v15", "game_v15"),
}


def dataset(name: str = "v0") -> Dataset:
    if name not in DATASETS:
        raise ValueError(f"unknown dataset {name}; known: {sorted(DATASETS)}")
    return DATASETS[name]


def manifest(ds: str = "v0") -> dict:
    """The manifest of dataset `ds`; a pilot dataset sees only its own scene's episodes."""
    spec = dataset(ds)
    m = json.loads(spec.manifest.read_text())
    if spec.name != "v0":
        splits = {split for split, _ in spec.groups.values()}
        m["episodes"] = [e for e in m["episodes"] if e["split"] in splits]
    return m


def manifest_sha(ds: str = "v0") -> str:
    """sha256 of the manifest file (the pilot datasets share one file)."""
    return hashlib.sha256(dataset(ds).manifest.read_bytes()).hexdigest()


def load_groups(names, root: Path = DEFAULT_ROOT, ds: str = "v0") -> dict[str, list[EpisodeData]]:
    """Load the named groups of dataset `ds`; each episode is sha256-verified by `wm_data.load_episode`."""
    spec = dataset(ds)
    m = manifest(ds)
    return {n: load_split(root, m, spec.groups[n][0], roles=(spec.groups[n][1],)) for n in names}


def split_stds(groups: dict[str, list[EpisodeData]]) -> dict[str, np.ndarray]:
    """The bench's nMAE denominator of every group (`bench_v0.split_std`)."""
    return {n: b.split_std([ep.bench for ep in eps]) for n, eps in groups.items() if eps}


def git_commit() -> str:
    """The code commit of this job: `WM1_CODE_COMMIT` (set by the launcher at checkout) or the clone's HEAD."""
    if os.environ.get("WM1_CODE_COMMIT"):
        return os.environ["WM1_CODE_COMMIT"]
    try:
        return subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True,
                              check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=1, sort_keys=True, default=_default) + "\n")


def _default(x):
    if isinstance(x, (np.floating, np.integer)):
        return x.item()
    if isinstance(x, np.ndarray):
        return x.tolist()
    raise TypeError(type(x))


def fmt(x, digits: int = 3) -> str:
    if x is None or (isinstance(x, float) and np.isnan(x)):
        return "—"
    return f"{x:.{digits}f}"
