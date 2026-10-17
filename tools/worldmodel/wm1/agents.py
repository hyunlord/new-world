"""T5: agent-level next hour (GOAL.md 7.6) — data, scored rows, metrics and baselines. numpy only.

Episodes with `--agents`: A_obs train 0–31, val 96–99, test 112–119; B_act test 1056–1063. Scored rows: every
row t in 23 .. T-2 and every agent alive at t, t+1 and t-23 (so the 24-hours-earlier baseline is defined).
Targets at t+1: state tag (accuracy), target kind (accuracy), position (mean Chebyshev error in tiles), hunger,
thirst and fatigue (nMAE: MAE / std of the split's scored targets). `lifespan_days` (hidden) is never read.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np

import wm1  # noqa: F401
from wm_data import load_episode
from wm1.data import EpisodeData, from_episode

AGENT_FIELDS = ("x", "y", "state_tag", "seek_kind", "target_kind", "target_x", "target_y", "hunger", "thirst",
                "fatigue", "carried_food", "carried_wood", "life_stage", "cold", "settlement", "age_days")
HIDDEN = ("lifespan_days",)
NEEDS = ("hunger", "thirst", "fatigue")
T5_SEEDS = {"A_train": set(range(0, 32)), "A_val": set(range(96, 100)), "A_test": set(range(112, 120)),
            "B_test": set(range(1056, 1064))}
T5_GROUPS = {"A_train": ("A_obs", "train"), "A_val": ("A_obs", "val"), "A_test": ("A_obs", "test"),
             "B_test": ("B_act", "test")}
WORKING_TARGETS = (4, 6, 7)   # site, gather food, gather wood


@dataclass
class AgentEpisode:
    settlement: EpisodeData
    agents: np.ndarray      # [T, N, len(AGENT_FIELDS)] float64, NaN where not alive
    alive: np.ndarray       # [T, N] bool
    width: int
    height: int

    @property
    def rows(self) -> int:
        return self.alive.shape[0]

    def col(self, name: str) -> np.ndarray:
        return self.agents[..., AGENT_FIELDS.index(name)]


def from_loaded(ep, split: str, role: str, seed: int, eid: str) -> AgentEpisode:
    names = [f["name"] for f in ep.meta["fields"]["agents.npy"]]
    keep = [names.index(n) for n in AGENT_FIELDS]
    for h in HIDDEN:
        if h in [names[i] for i in keep]:
            raise ValueError("hidden field selected")
    a = ep.arrays["agents.npy"][..., keep].astype(np.float64)
    alive = ep.arrays["agent_alive.npy"].astype(bool)
    h, w = ep.arrays["terrain.npy"].shape
    return AgentEpisode(from_episode(ep, split, role, seed, eid), a, alive, w, h)


def load_agent_group(root: Path, manifest: dict, name: str) -> list[AgentEpisode]:
    split, role = T5_GROUPS[name]
    out = []
    for entry in manifest["episodes"]:
        if entry["split"] == split and entry["role"] == role and entry["seed"] in T5_SEEDS[name]:
            if not entry.get("agents"):
                raise ValueError(f"{entry['path']} has no agent tables")
            ep = load_episode(Path(root) / entry["path"],
                              arrays=["global.npy", "settlements.npy", "agents.npy", "agent_alive.npy", "terrain.npy"])
            out.append(from_loaded(ep, split, role, entry["seed"], entry["schedule_id"]))
    return out


def scored_index(ep: AgentEpisode) -> tuple[np.ndarray, np.ndarray]:
    """(t, agent) pairs scored by T5."""
    t = np.arange(23, ep.rows - 1)
    ok = ep.alive[t] & ep.alive[t + 1] & ep.alive[t - 23]
    ti, ni = np.nonzero(ok)
    return t[ti], ni


@dataclass
class AgentPred:
    state_tag: np.ndarray    # [M] int
    target_kind: np.ndarray  # [M] int
    x: np.ndarray            # [M]
    y: np.ndarray            # [M]
    needs: np.ndarray        # [M, 3]


def truth(ep: AgentEpisode, t: np.ndarray, n: np.ndarray) -> AgentPred:
    return state_at(ep, t + 1, n)


def state_at(ep: AgentEpisode, t: np.ndarray, n: np.ndarray) -> AgentPred:
    a = ep.agents[t, n]
    f = AGENT_FIELDS.index
    return AgentPred(a[:, f("state_tag")].astype(int), a[:, f("target_kind")].astype(int), a[:, f("x")], a[:, f("y")],
                     a[:, [f(k) for k in NEEDS]])


def persistence(ep: AgentEpisode, t: np.ndarray, n: np.ndarray) -> AgentPred:
    return state_at(ep, t, n)


def day_earlier(ep: AgentEpisode, t: np.ndarray, n: np.ndarray) -> AgentPred:
    return state_at(ep, t + 1 - 24, n)


def t5(predict, eps: list[AgentEpisode], std: np.ndarray | None = None) -> dict:
    """Metrics of `predict(ep, t, n) -> AgentPred` over the scored rows of `eps`; `std` (the needs' nMAE
    denominator) defaults to the std of these episodes' scored targets."""
    preds, trues = [], []
    for ep in eps:
        t, n = scored_index(ep)
        preds.append(predict(ep, t, n))
        trues.append(truth(ep, t, n))
    cat = lambda xs, k: np.concatenate([getattr(x, k) for x in xs])
    p = {k: cat(preds, k) for k in ("state_tag", "target_kind", "x", "y", "needs")}
    tr = {k: cat(trues, k) for k in ("state_tag", "target_kind", "x", "y", "needs")}
    if std is None:
        std = tr["needs"].std(axis=0)
    cheb = np.maximum(np.abs(p["x"] - tr["x"]), np.abs(p["y"] - tr["y"]))
    nm = np.abs(p["needs"] - tr["needs"]).mean(axis=0) / np.where(std > 0, std, 1.0)
    return {"n": int(len(cheb)), "state_tag_accuracy": float(np.mean(p["state_tag"] == tr["state_tag"])),
            "target_kind_accuracy": float(np.mean(p["target_kind"] == tr["target_kind"])),
            "position_chebyshev": float(cheb.mean()), "position_exact": float(np.mean(cheb == 0)),
            "needs_nmae": {k: float(v) for k, v in zip(NEEDS, nm)}, "needs_nmae_mean": float(nm.mean())}


def needs_std(eps: list[AgentEpisode]) -> np.ndarray:
    return np.vstack([truth(ep, *scored_index(ep)).needs for ep in eps]).std(axis=0)


def working_count(ep: AgentEpisode, rows: np.ndarray, target_kind: np.ndarray | None = None) -> np.ndarray:
    """Adults (life stage 0) of a settlement whose target kind is a site or a gathering target, per row; with
    `target_kind` ([len(rows), N]) in place of the recorded one."""
    tk = ep.col("target_kind")[rows] if target_kind is None else target_kind
    adult = ep.col("life_stage")[rows] == 0
    member = ep.col("settlement")[rows] >= 0
    alive = ep.alive[rows]
    return (alive & adult & member & np.isin(np.nan_to_num(tk, nan=-1), WORKING_TARGETS)).sum(axis=1)
