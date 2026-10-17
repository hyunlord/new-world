"""The observational policy of WM-4 (docs/worldmodel/wm4/GOAL.md 6.1, METRICS.md part a). numpy only.

A world's god reads the factual trajectory at 06:00 of display days 8 … 60 (tick 1,440 d − 1,560, row tick / 60) and
casts at most once: gifts go to hungry worlds, spoils to fed ones, rot to wood-rich ones.

- State at a decision row (rows <= that row only): f = summed `s.food_stock` / max(1, `g.alive`), w = summed
  `s.wood_stock` / max(1, `g.alive`). A row with `g.alive` = 0 or `g.settlements_alive` = 0 is not a decision (p = 0).
- Terciles q_f1, q_f2 (33rd / 67th percentile of f) and q_w2 (67th of w) over every decision row of the train worlds'
  factual episodes (`terciles`; frozen in METRICS.md part a before any treated episode exists).
- Daily probabilities (`probs`): `gf240` 0.030 / 0.008 / 0.002 and `sp240` 0.002 / 0.008 / 0.030 for f < q_f1 /
  q_f1 <= f < q_f2 / f >= q_f2; `rw120` 0.012 if w >= q_w2 else 0.004.
- Draw (`run`): u = `default_rng([seed, day]).random()`; cast `gf240` if u < p_g, else `sp240` if u < p_g + p_s, else
  `rw120` if u < p_g + p_s + p_r, else none; after a cast no further decisions.
"""
from __future__ import annotations

import numpy as np

TPD, OFFSET, EVERY = 1440, 480, 60
DAYS = tuple(range(8, 61))
ARMS = ("gf240", "sp240", "rw120")
# obs arm -> (kind, amount, the v2 arm whose effect scale and harness spec it shares (GOAL.md 4.1, 7.1)).
ARM_SPEC = {"gf240": ("gift_food", 240, "gf240E"), "sp240": ("spoil_food", 240, "sp240L"),
            "rw120": ("rot_wood", 120, "rw120E")}
TO_V2 = {a: s[2] for a, s in ARM_SPEC.items()}
P_GIFT = (0.030, 0.008, 0.002)
P_SPOIL = (0.002, 0.008, 0.030)
P_ROT_HIGH, P_ROT_LOW = 0.012, 0.004
NEVER_REF_DAY = 30            # GOAL.md 6.2.5: reference row of a world that never cast


def tick_of_day(day: int) -> int:
    """06:00 of display day `day` (display day of tick t = (t + 480) // 1,440 + 1)."""
    return TPD * int(day) - 1560


def row_of_day(day: int) -> int:
    t = tick_of_day(day)
    assert t % EVERY == 0
    return t // EVERY


def display_day(tick: int) -> int:
    return (int(tick) + OFFSET) // TPD + 1


DECISION_ROWS = tuple(row_of_day(d) for d in DAYS)
NEVER_REF_TICK = tick_of_day(NEVER_REF_DAY)


def state_series(food: np.ndarray, wood: np.ndarray, alive: np.ndarray, settlements_alive: np.ndarray) -> dict:
    """Per-row f, w and the decision flag from the per-row summed food / wood stores, `g.alive` and
    `g.settlements_alive` (all [T])."""
    alive = np.asarray(alive, dtype=np.float64)
    den = np.maximum(1.0, alive)
    return {"f": np.asarray(food, dtype=np.float64) / den, "w": np.asarray(wood, dtype=np.float64) / den,
            "decision": (alive > 0) & (np.asarray(settlements_alive, dtype=np.float64) > 0)}


def terciles(series: list[dict]) -> dict:
    """q_f1, q_f2, q_w2 over every decision row (days 8 … 60) of the given worlds' state series."""
    f, w = [], []
    for s in series:
        for r in DECISION_ROWS:
            if s["decision"][r]:
                f.append(s["f"][r]); w.append(s["w"][r])
    f, w = np.asarray(f), np.asarray(w)
    q_f1, q_f2 = np.percentile(f, [33, 67])
    return {"q_f1": float(q_f1), "q_f2": float(q_f2), "q_w2": float(np.percentile(w, 67)), "rows": int(len(f))}


def tercile_of(f: float, q: dict) -> int:
    return 0 if f < q["q_f1"] else 1 if f < q["q_f2"] else 2


def probs(f: float, w: float, q: dict, decision: bool = True) -> tuple[float, float, float]:
    """(p_g, p_s, p_r) at one decision row; (0, 0, 0) for a row that is not a decision."""
    if not decision:
        return 0.0, 0.0, 0.0
    t = tercile_of(f, q)
    return P_GIFT[t], P_SPOIL[t], (P_ROT_HIGH if w >= q["q_w2"] else P_ROT_LOW)


def draw(seed: int, day: int) -> float:
    return float(np.random.default_rng([int(seed), int(day)]).random())


def choose(u: float, p: tuple[float, float, float]) -> str | None:
    pg, ps, pr = p
    if u < pg:
        return "gf240"
    if u < pg + ps:
        return "sp240"
    if u < pg + ps + pr:
        return "rw120"
    return None


def run(seed: int, s: dict, q: dict) -> dict:
    """The policy on one world's factual state series: every decision row up to and including the cast (day, tick,
    row, f, w, tercile, decision, p_g, p_s, p_r, u) and the cast (arm, day, tick, row) or None."""
    rows, cast = [], None
    for day, r in zip(DAYS, DECISION_ROWS):
        dec = bool(s["decision"][r])
        f, w = float(s["f"][r]), float(s["w"][r])
        p = probs(f, w, q, dec)
        u = draw(seed, day)
        arm = choose(u, p) if dec else None
        rows.append({"day": day, "tick": tick_of_day(day), "row": r, "decision": dec, "f": f, "w": w,
                     "tercile": tercile_of(f, q) if dec else None, "w_high": bool(w >= q["q_w2"]) if dec else None,
                     "p_g": p[0], "p_s": p[1], "p_r": p[2], "u": u, "cast": arm})
        if arm is not None:
            cast = {"arm": arm, "day": day, "tick": tick_of_day(day), "row": r}
            break
    return {"seed": int(seed), "rows": rows, "cast": cast}


def schedule_rows(arm: str, tick: int) -> list[dict]:
    """One row of `schedules_p14.py`'s format: {"tick", "kind", "settlement": "home", "amount"}."""
    kind, amount, _ = ARM_SPEC[arm]
    assert int(tick) % EVERY == 0, tick
    return [{"tick": int(tick), "kind": kind, "settlement": "home", "amount": int(amount)}]


def ref_tick(world: dict) -> int:
    """GOAL.md 6.2.5: c_w = the cast tick, or 06:00 of day 30 for a world that never cast."""
    return int(world["cast"]["tick"]) if world["cast"] else NEVER_REF_TICK


def prob_table(world: dict) -> dict[int, tuple[float, float, float]]:
    """row -> (p_g, p_s, p_r) of every recorded decision row of one world (`policy.json` entry)."""
    return {int(r["row"]): (r["p_g"], r["p_s"], r["p_r"]) for r in world["rows"]}
