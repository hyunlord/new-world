"""Shared loading, split roles and run bookkeeping for WM-1 scripts."""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
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


def manifest() -> dict:
    return json.loads(MANIFEST.read_text())


def manifest_sha() -> str:
    return hashlib.sha256(MANIFEST.read_bytes()).hexdigest()


def load_groups(names, root: Path = DEFAULT_ROOT) -> dict[str, list[EpisodeData]]:
    """Load the named groups (keys of GROUPS); each episode is sha256-verified by `wm_data.load_episode`."""
    m = manifest()
    return {n: load_split(root, m, GROUPS[n][0], roles=(GROUPS[n][1],)) for n in names}


def split_stds(groups: dict[str, list[EpisodeData]]) -> dict[str, np.ndarray]:
    """The bench's nMAE denominator of every group (`bench_v0.split_std`)."""
    return {n: b.split_std([ep.bench for ep in eps]) for n, eps in groups.items() if eps}


def git_commit() -> str:
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
