"""Cast-window sampler (docs/worldmodel/wm3/GOAL.md 7.2).

A batch is `n_plain` ordinary windows (`wm1.data.sample_windows`, unchanged, drawn from `default_rng([seed, 0])`) followed
by `n_cast` **cast windows** (drawn from `default_rng([seed, 1])`) and, with `partners`, their partners' windows in the
same order. A cast window: a train case (world, arm, replicate) drawn uniformly over every arm and both replicates, start
s = c - p with p = `CAST_POS` (167, the last teacher-forced row of a rollout start), the arm member's rows s .. s + W; its
partner (the factual member `none` of the same world and replicate) gives the same rows. Arm member and partner are equal
through the cast row (asserted once per case at load), so the two windows differ at positions <= p only in the action
entry at p.

The cases come from the train members' `interventions.json` (observable): one record per arm member, its tick a multiple
of 60 (`wm2.v2.row_of_tick`).
"""
from __future__ import annotations

from dataclasses import dataclass

import numpy as np

import wm3  # noqa: F401
from wm1 import common as c
from wm1 import data as d
from wm2 import v2

CAST_POS = 167
SPLIT_REP = {c.V19_FACT: 0, c.V19_ARM_R0: 0, c.V19_NONE_R1: 1, c.V19_ARM_R1: 1}
KEYS = ("x_state", "calendar", "actions", "sev", "dtarget", "fire_next", "state")


@dataclass(frozen=True)
class TrainCase:
    seed: int
    arm: str
    rep: int
    cast_row: int
    arm_i: int        # index of the arm member in the prepared list
    partner_i: int    # index of the partner


def member_key(ep: d.EpisodeData) -> tuple[int, str, int]:
    return int(ep.seed), ep.eid, SPLIT_REP[ep.split]


def train_cases(eps: list[d.EpisodeData]) -> list[TrainCase]:
    """Every (world, arm, replicate) case of the train members, in member order; asserts arm and partner equal through
    the cast row."""
    index = {member_key(ep): i for i, ep in enumerate(eps)}
    if len(index) != len(eps):
        raise ValueError("train members are not unique by (seed, arm, replicate)")
    out = []
    for i, ep in enumerate(eps):
        if ep.eid == "none":
            continue
        (rec,) = ep.interventions
        row = v2.row_of_tick(int(rec["tick"]))
        if v2.ARM_SPEC[ep.eid][0] != rec["kind"]:
            raise ValueError(f"{member_key(ep)}: intervention kind {rec['kind']}")
        j = index[(int(ep.seed), "none", SPLIT_REP[ep.split])]
        if not np.array_equal(ep.table[: row + 1], eps[j].table[: row + 1]):
            raise ValueError(f"{member_key(ep)}: arm and partner differ before the cast row")
        out.append(TrainCase(int(ep.seed), ep.eid, SPLIT_REP[ep.split], row, i, j))
    return out


def cast_window(prep: d.Prepared, start: int, length: int) -> dict:
    """Rows start .. start + length of one prepared member, in `sample_windows`' layout (float32 arrays)."""
    if start < 0 or start + length + 1 > prep.ep.rows:
        raise ValueError(f"cast window {start} .. {start + length} outside the episode")
    sl = slice(start, start + length + 1)
    return {"x_state": prep.x_state[sl], "calendar": prep.calendar[sl], "actions": prep.actions[sl], "sev": prep.sev[sl],
            "state": prep.state[sl], "dtarget": prep.dtarget[start:start + length],
            "fire_next": prep.fire_next[start:start + length]}


class Sampler:
    """Batches of plain + cast (+ partner) windows from fixed streams (`default_rng([seed, 0])` / `[seed, 1]`)."""

    def __init__(self, preps: list[d.Prepared], cases: list[TrainCase], seed: int, length: int, n_plain: int,
                 n_cast: int, partners: bool, pos: int = CAST_POS):
        if n_cast and not cases:
            raise ValueError("cast windows need train cases")
        self.preps, self.cases, self.length, self.pos = preps, cases, length, pos
        self.n_plain, self.n_cast, self.partners = n_plain, n_cast, partners and n_cast > 0
        self.rng_plain = np.random.default_rng([seed, 0])
        self.rng_cast = np.random.default_rng([seed, 1])

    def draw(self) -> tuple[dict, list[TrainCase]]:
        """One batch (numpy float32, rows: plain, cast, partners) and the drawn cases."""
        parts = []
        if self.n_plain:
            parts.append(d.sample_windows(self.preps, self.length, self.n_plain, self.rng_plain))
        drawn = []
        if self.n_cast:
            idx = self.rng_cast.integers(0, len(self.cases), size=self.n_cast)
            drawn = [self.cases[i] for i in idx]
            wins = [cast_window(self.preps[x.arm_i], x.cast_row - self.pos, self.length) for x in drawn]
            if self.partners:
                wins += [cast_window(self.preps[x.partner_i], x.cast_row - self.pos, self.length) for x in drawn]
            parts.append({k: np.stack([w[k] for w in wins]).astype(np.float32) for k in KEYS})
        return {k: np.concatenate([p[k] for p in parts]) for k in KEYS}, drawn

    @property
    def batch(self) -> int:
        return self.n_plain + self.n_cast * (2 if self.partners else 1)
