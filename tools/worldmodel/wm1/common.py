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
    `observable` marks recorder-schema >= 2 data (observable feature table, no severity input).

    GOAL_1_3.md 4.2, 6.3: `train_seeds` (half-open [lo, hi), or None: every train episode) keeps only those seeds of
    the `train` group (nested training sets share one manifest; roles stay train / val / test); `extra_state` are the
    `global.npy` columns this dataset appends to the observable state table after `g.injured_members` (state
    features, chosen per dataset, never per schema).

    WM-2 (docs/worldmodel/wm2/GOAL.md 7.1): a group's split may be a tuple of manifest splits; the group is their
    episodes in that order (`load_groups`), e.g. every member of the dataset-v2 families (factual and arms, both
    replicates) as one training group. `val_report` (empty: the `val` groups) names the groups whose full validation
    metrics `m1_gru.train` writes at the end of training (WM-2: the factual validation worlds; early stopping and the
    rounding choice stay on `val`)."""
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
    train_seeds: tuple[int, int] | None = None
    extra_state: tuple[str, ...] = ()
    val_report: tuple[str, ...] = ()

    def train_groups(self, action_aware: bool) -> list[str]:
        return list(self.train_action if action_aware else self.train)


def _pilot(name: str, scene: str) -> Dataset:
    return Dataset(name, PILOT_MANIFEST, {"train": (scene, "train"), "val": (scene, "val"), "test": (scene, "test")},
                   ("train",), ("train",), ("val",), ("test",), None, None, None, ("T1", "T2", "T2_any"), True)


W13_MANIFEST = REPO / "docs/worldmodel/wm1/w13_manifest.json"
# GOAL_1_3.md 4.2: schema-5 globals of the game_v17 arm (raw levels; `fissions_total` deliberately not differenced).
V17_EXTRA = ("g.settlements_alive", "g.fissions_total")


def _w13(name: str, scene: str, train_seeds: tuple[int, int], extra: tuple[str, ...] = ()) -> Dataset:
    return Dataset(name, W13_MANIFEST, {"train": (scene, "train"), "val": (scene, "val"), "test": (scene, "test")},
                   ("train",), ("train",), ("val",), ("test",), None, None, None, ("T1", "T2", "T2_any"), True,
                   train_seeds, extra)


V2_MANIFEST = REPO / "docs/worldmodel/wm2/v2_manifest.json"
# WM-2 (GOAL.md 6.5, 7.1): manifest splits of dataset v2. `v19_fact` = replicate 0 of `none` of every game_v19 family;
# `v19_arm_r0` / `v19_none_r1` / `v19_arm_r1` = the other members; `v16` = the game_v16 control (factual only).
V19_FACT, V19_ARM_R0, V19_NONE_R1, V19_ARM_R1 = "v19_fact", "v19_arm_r0", "v19_none_r1", "v19_arm_r1"
V19_ALL = (V19_FACT, V19_ARM_R0, V19_NONE_R1, V19_ARM_R1)


def _v2(name: str, train: str | tuple, val: str | tuple, test: str | tuple, extra: tuple[str, ...],
        fact: str = V19_FACT) -> Dataset:
    return Dataset(name, V2_MANIFEST, {"train": (train, "train"), "val": (val, "val"), "test": (test, "test"),
                                       "val_fact": (fact, "val")},
                   ("train",), ("train",), ("val",), ("test",), None, None, None, ("T1", "T2", "T2_any"), True,
                   None, extra, ("val_fact",))


DATASETS = {
    "v0": Dataset("v0", MANIFEST, GROUPS, ("A_train",), ("A_train", "B_train"), VAL_SPLITS, TEST_SPLITS, "C_test",
                  "A_val", "A_test"),
    "p10_v16": _pilot("p10_v16", "game_v16"),
    "p10_v16n": _pilot("p10_v16n", "game_v16_neutral"),
    "p10_v15": _pilot("p10_v15", "game_v15"),
    "w13_v16_n80": _w13("w13_v16_n80", "game_v16", (30000, 30080)),
    "w13_v16_n200": _w13("w13_v16_n200", "game_v16", (30000, 30200)),
    "w13_v16_n400": _w13("w13_v16_n400", "game_v16", (30000, 30400)),
    "w13_v17_n400": _w13("w13_v17_n400", "game_v17", (32000, 32400), V17_EXTRA),
    # WM-2: training on every member of the train families (all arms, both replicates); validation on the
    # replicate-0 members (factual and arms) of the val families; test = every member of the test families (the
    # effect evaluation reads them through `wm2.effects`). The factual-only view and the game_v16 control.
    "v2_v19": _v2("v2_v19", V19_ALL, (V19_FACT, V19_ARM_R0), V19_ALL, V17_EXTRA),
    "v2_v19_fact": _v2("v2_v19_fact", V19_FACT, V19_FACT, V19_FACT, V17_EXTRA),
    "v2_v16": _v2("v2_v16", "v16", "v16", "v16", (), fact="v16"),
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
        splits = {x for split, _ in spec.groups.values() for x in split_names(split)}
        m["episodes"] = [e for e in m["episodes"] if e["split"] in splits]
    return m


def manifest_sha(ds: str = "v0") -> str:
    """sha256 of the manifest file (the pilot datasets share one file)."""
    return hashlib.sha256(dataset(ds).manifest.read_bytes()).hexdigest()


def split_names(split: str | tuple) -> tuple[str, ...]:
    """The manifest splits of a group's split entry (a name or a tuple of names, WM-2)."""
    return (split,) if isinstance(split, str) else tuple(split)


def group_seeds(spec: Dataset, name: str) -> set[int] | None:
    """The seeds a group keeps (`train_seeds` on the `train` role), None for every seed."""
    if spec.train_seeds is None or spec.groups[name][1] != "train":
        return None
    return set(range(*spec.train_seeds))


def load_groups(names, root: Path = DEFAULT_ROOT, ds: str = "v0") -> dict[str, list[EpisodeData]]:
    """Load the named groups of dataset `ds`; each episode is sha256-verified by `wm_data.load_episode`."""
    spec = dataset(ds)
    m = manifest(ds)
    return {n: [ep for split in split_names(spec.groups[n][0])
                for ep in load_split(root, m, split, roles=(spec.groups[n][1],), seeds=group_seeds(spec, n),
                                     extra=spec.extra_state)] for n in names}


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
