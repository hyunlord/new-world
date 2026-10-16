"""Daily tokens for CRN and the Causal Transformer (docs/worldmodel/wm4/GOAL.md 7.4, METHODS.md). numpy only.

An hourly episode (`wm1.data.EpisodeData`, rows 0 … 2,880, row r at hour (8 + r) mod 24) is sampled every 24 rows at
a clock phase: token k sits at row r_k = r0 + 24 k (r0 in 0 … 23; 06:00 is r0 = 22, 18:00 is r0 = 10).

- **state** (the 37 state features of WM-2's action-aware input list, `variant.state_names`): the levels at r_k; the
  tallies (`d.*`, hourly differences in the hourly table) as their sum over rows r_k − 23 … r_k (the day ending at the
  token; the first token sums the rows it has) — so cumulative deaths are recovered by summing the fed-back daily
  deaths. Normalized by train mean / std (levels and daily sums over every train row).
- **covariates**: the 30 calendar features at r_k (history only).
- **treatment** of token k: the actions requested at rows r_k … r_k + 23 (the cast falls in exactly one token): a
  4-class one-hot {none, gift_food, spoil_food, rot_wood} plus the amount / `ACTION_SCALE` of its kind (5 values).
- **target** of token k: the change state_{k+1} − state_k divided by the train std of the daily change (`dstd`).
"""
from __future__ import annotations

from dataclasses import dataclass

import numpy as np

import wm4  # noqa: F401
import bench_v0 as b
from wm1 import data as d

TOKEN_ROWS = 24
PHASE_06, PHASE_18 = 22, 10
CLASSES = ("none", "gift_food", "spoil_food", "rot_wood")
N_CLASSES = len(CLASSES)
N_TREAT = N_CLASSES + 1
DEATHS = ("d.deaths_starvation", "d.deaths_dehydration", "d.deaths_combat", "d.deaths_old_age", "d.deaths_exposure")
# Clamp ranges of the fed-back state (every recorded feature is a count, a stock or a flag).
UNIT = ("s.fire_lit", "s.present")


def phase_of_row(row: int) -> int:
    return int(row) % TOKEN_ROWS


N_TOKENS = 120     # every phase r0 in 0 … 23 has tokens 0 … 119 (r0 + 2,856 <= 2,879); r0 = 0's row 2,880 is dropped


def token_rows(r0: int, last_row: int = 2880) -> np.ndarray:
    return np.arange(r0, last_row + 1, TOKEN_ROWS)[:N_TOKENS]


def token_of_row(row: int) -> int:
    return (int(row) - phase_of_row(row)) // TOKEN_ROWS


@dataclass
class Raw:
    """One episode's hourly arrays for tokenization (float64)."""
    seed: int
    eid: str
    state: np.ndarray        # [T, 37] hourly table (tallies as hourly differences)
    calendar: np.ndarray     # [T, 30]
    actions: np.ndarray      # [T, 7] requested amounts at the row (bench encoding)
    tally: np.ndarray        # bool [37]

    @property
    def rows(self) -> int:
        return self.state.shape[0]


def raw_of(ep: d.EpisodeData, variant: d.Variant) -> Raw:
    st = ep.state(variant).astype(np.float64)
    tally = np.array([n.startswith("d.") for n in variant.state_names])
    return Raw(ep.seed, ep.eid, st, ep.calendar.astype(np.float64), ep.bench.actions.astype(np.float64), tally)


def daily_state(raw: Raw, rows: np.ndarray) -> np.ndarray:
    """[K, 37] the state at `rows` with tallies summed over each row's preceding 24 rows (inclusive)."""
    cs = np.cumsum(raw.state[:, raw.tally], axis=0)
    hi = cs[rows]
    lo_rows = rows - TOKEN_ROWS
    lo = np.where((lo_rows >= 0)[:, None], cs[np.clip(lo_rows, 0, None)], 0.0)
    out = raw.state[rows].copy()
    out[:, raw.tally] = hi - lo
    return out


def treatments(raw: Raw, rows: np.ndarray) -> np.ndarray:
    """[K, 5] the treatment of every token (actions of rows r … r + 23)."""
    cs = np.vstack([np.zeros((1, raw.actions.shape[1])), np.cumsum(raw.actions, axis=0)])
    hi = cs[np.clip(rows + TOKEN_ROWS, 0, raw.rows)]
    amt = hi - cs[rows]
    return encode(amt)


def encode(amounts: np.ndarray) -> np.ndarray:
    """[K, 7] summed requested amounts per kind -> [K, 5] (class one-hot, amount / ACTION_SCALE)."""
    out = np.zeros((amounts.shape[0], N_TREAT))
    out[:, 0] = 1.0
    for k in range(amounts.shape[0]):
        nz = np.nonzero(amounts[k] != 0)[0]
        if len(nz) == 0:
            continue
        if len(nz) > 1:
            raise ValueError(f"token {k}: more than one intervention kind {nz}")
        kind = b.KINDS[nz[0]]
        if kind not in CLASSES:
            raise ValueError(f"token {k}: intervention kind {kind} outside {CLASSES}")
        out[k, 0] = 0.0
        out[k, CLASSES.index(kind)] = 1.0
        out[k, N_CLASSES] = amounts[k, nz[0]] / d.ACTION_SCALE[nz[0]]
    return out


def arm_treatment(kind: str, amount: float) -> np.ndarray:
    """The 5-vector of one cast of `kind` with `amount` (the planned treatment of a rollout's first token)."""
    amt = np.zeros((1, len(b.KINDS)))
    amt[0, b.KINDS.index(kind)] = float(amount)
    return encode(amt)[0]


NONE = encode(np.zeros((1, len(b.KINDS))))[0]


@dataclass
class Norm:
    """Daily normalizer: mean / std of the daily state, std of its daily change (train rows only)."""
    mean: np.ndarray
    std: np.ndarray
    dstd: np.ndarray
    lo: np.ndarray
    hi: np.ndarray
    rows: int

    @classmethod
    def fit(cls, raws: list[Raw]) -> "Norm":
        """Every train row r >= 23 (a full day behind it) of every episode, every phase: streaming sums."""
        n = m = 0
        s1 = s2 = c1 = c2 = 0.0
        lo, hi = np.inf, -np.inf
        for raw in raws:
            s = daily_state(raw, np.arange(0, raw.rows))
            lv, ch = s[TOKEN_ROWS - 1:], s[2 * TOKEN_ROWS - 1:] - s[TOKEN_ROWS - 1: -TOKEN_ROWS]
            n += lv.shape[0]; s1 = s1 + lv.sum(0); s2 = s2 + (lv ** 2).sum(0)
            m += ch.shape[0]; c1 = c1 + ch.sum(0); c2 = c2 + (ch ** 2).sum(0)
            lo, hi = np.minimum(lo, lv.min(0)), np.maximum(hi, lv.max(0))
        mean = s1 / n
        sd = np.sqrt(np.maximum(s2 / n - mean ** 2, 0.0))
        dsd = np.sqrt(np.maximum(c2 / m - (c1 / m) ** 2, 0.0))
        return cls(mean, np.where(sd > 1e-12, sd, 1.0), np.where(dsd > 1e-12, dsd, 1.0), lo, hi, int(n))

    def to_json(self) -> dict:
        return {k: getattr(self, k).tolist() for k in ("mean", "std", "dstd", "lo", "hi")} | {"rows": self.rows}

    @classmethod
    def from_json(cls, j: dict) -> "Norm":
        return cls(*(np.array(j[k]) for k in ("mean", "std", "dstd", "lo", "hi")), int(j["rows"]))


@dataclass
class Seq:
    """One tokenized episode at one phase: normalized state y [K, 37], covariates v [K, 30], treatments a [K, 5], the
    raw daily state [K, 37] and the token rows."""
    y: np.ndarray
    v: np.ndarray
    a: np.ndarray
    raw: np.ndarray
    rows: np.ndarray


def tokenize(raw: Raw, r0: int, norm: Norm, last_row: int | None = None) -> Seq:
    rows = token_rows(r0, raw.rows - 1 if last_row is None else last_row)
    s = daily_state(raw, rows)
    return Seq((s - norm.mean) / norm.std, raw.calendar[rows], treatments(raw, rows), s, rows)


def clamp(state: np.ndarray, names: tuple[str, ...]) -> np.ndarray:
    """Fed-back raw daily state clamped to its range: >= 0 everywhere, <= 1 for the flags."""
    out = np.maximum(state, 0.0)
    for n in UNIT:
        if n in names:
            i = names.index(n)
            out[..., i] = np.minimum(out[..., i], 1.0)
    return out
