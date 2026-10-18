"""The sequential multi-cast policy of WM-5's obs2 split (docs/worldmodel/wm5/GOAL.md 6.1, METRICS.md part a). numpy
only.

A world's god reads the observed trajectory at 06:00 of display days 8 … 60 (WM-4's decision rows, `wm4.policy`) and may
cast up to 5 times. Gifts go to hungry worlds, spoils to fed ones, rot to wood-rich ones — WM-4's tercile structure with
every daily probability doubled, so the confounding ratios are unchanged.

- **State** at a decision row: WM-4's f (food per person) and w (wood per person) from the observable columns
  (`wm4.obs_split.state_of`), read from the episode recorded with every earlier cast (the rounds of GOAL.md 6.2).
- **Terciles**: WM-4's frozen q_f1, q_f2, q_w2 (read from `docs/worldmodel/wm4/METRICS.md` part a, never recomputed).
- **Daily probabilities** (`probs`): `gf240` 0.060 / 0.016 / 0.004 and `sp240` 0.004 / 0.016 / 0.060 for f < q_f1 /
  q_f1 <= f < q_f2 / f >= q_f2; `rw120` 0.024 if w >= q_w2 else 0.008.
- **Refractory period**: after a cast on day d the next decision is on day d + 8 or later (rows of days d + 1 … d + 7
  have p = 0, `status` "refractory"), so no second cast falls inside (c, c + 7 d].
- **Cap**: at most 5 casts; rows after the 5th have p = 0 (`status` "cap").
- **No decision** (p = 0, `status` "no_decision") on a row with `g.alive` = 0 or `g.settlements_alive` = 0.
- **Draw** at a decision row of seed s and day d: u = `default_rng([s, d, 2])`'s first uniform (the `2` keeps the stream
  distinct from WM-4's `default_rng([s, d])`); the choice is WM-4's `choose` (gift if u < p_g, else spoil if
  u < p_g + p_s, else rot if u < p_g + p_s + p_r, else none).

`step` runs the policy from the row after the last cast (or from day 8) to the first new cast or day 60 on one episode's
state series: one recording round. `replay` runs every round on a single state series; on a world's final observed
episode (which equals every earlier round's episode on rows <= each later cast row) it reproduces the casts of the
rounds and every row's probabilities (the propensity record).
"""
from __future__ import annotations

from pathlib import Path

import numpy as np

from wm4 import policy as pol

DAYS = pol.DAYS
DECISION_ROWS = pol.DECISION_ROWS
ROW_OF_DAY = dict(zip(DAYS, DECISION_ROWS))
DAY_OF_ROW = dict(zip(DECISION_ROWS, DAYS))
ARMS = pol.ARMS
ARM_SPEC = pol.ARM_SPEC
TO_V2 = pol.TO_V2
P_GIFT = tuple(2 * p for p in pol.P_GIFT)          # (0.060, 0.016, 0.004)
P_SPOIL = tuple(2 * p for p in pol.P_SPOIL)        # (0.004, 0.016, 0.060)
P_ROT_HIGH, P_ROT_LOW = 2 * pol.P_ROT_HIGH, 2 * pol.P_ROT_LOW     # 0.024, 0.008
MAX_CASTS = 5
REFRACTORY_DAYS = 8                                # next decision on day d + 8 or later
STREAM = 2
NEVER_REF_DAY = pol.NEVER_REF_DAY                  # c1 of a world that never cast: 06:00 of day 30
C2_GAP_DAYS = 14                                   # c2 of a world without a second cast: c1 + 14 days
WM4_METRICS = Path(__file__).resolve().parents[3] / "docs/worldmodel/wm4/METRICS.md"


def frozen_terciles(metrics: Path = WM4_METRICS) -> dict:
    """WM-4's frozen terciles (METRICS.md part a's `TERCILES_JSON:` line); never recomputed (GOAL.md 6.1)."""
    from wm4 import obs_split as os4
    return os4.read_terciles(metrics)


def probs(f: float, w: float, q: dict, decision: bool = True) -> tuple[float, float, float]:
    """(p_g, p_s, p_r) at one decision row; (0, 0, 0) for a row that is not a decision."""
    if not decision:
        return 0.0, 0.0, 0.0
    t = pol.tercile_of(f, q)
    return P_GIFT[t], P_SPOIL[t], (P_ROT_HIGH if w >= q["q_w2"] else P_ROT_LOW)


def draw(seed: int, day: int) -> float:
    return float(np.random.default_rng([int(seed), int(day), STREAM]).random())


choose = pol.choose
tick_of_day = pol.tick_of_day
display_day = pol.display_day


def cast_record(arm: str, day: int) -> dict:
    return {"arm": arm, "day": int(day), "tick": tick_of_day(day), "row": ROW_OF_DAY[day]}


def row_record(seed: int, day: int, s: dict, q: dict, casts: list[dict]) -> dict:
    """One row of the propensity record given the casts before `day`: status decision / refractory / cap / no_decision,
    the state, (p_g, p_s, p_r), the draw and the chosen arm (None unless a decision row casts)."""
    r = ROW_OF_DAY[day]
    f, w = float(s["f"][r]), float(s["w"][r])
    rec = {"day": int(day), "tick": tick_of_day(day), "row": r, "f": f, "w": w, "tercile": None, "w_high": None,
           "p_g": 0.0, "p_s": 0.0, "p_r": 0.0, "u": None, "cast": None}
    if len(casts) >= MAX_CASTS:
        rec["status"] = "cap"
    elif casts and day < casts[-1]["day"] + REFRACTORY_DAYS:
        rec["status"] = "refractory"
    elif not bool(s["decision"][r]):
        rec["status"] = "no_decision"
    else:
        rec["status"] = "decision"
        p = probs(f, w, q, True)
        u = draw(seed, day)
        rec.update({"tercile": pol.tercile_of(f, q), "w_high": bool(w >= q["q_w2"]), "p_g": p[0], "p_s": p[1],
                    "p_r": p[2], "u": u, "cast": choose(u, p)})
    return rec


def step(seed: int, s: dict, q: dict, casts: list[dict]) -> dict:
    """One round: the rows from the day after the last cast (day 8 when none) up to and including the first new cast
    (or day 60), read from the state series `s` of the episode recorded with `casts`; the new cast or None."""
    start = casts[-1]["day"] + 1 if casts else DAYS[0]
    rows, new = [], None
    for day in range(start, DAYS[-1] + 1):
        rec = row_record(seed, day, s, q, casts)
        rows.append(rec)
        if rec["cast"] is not None:
            new = cast_record(rec["cast"], day)
            break
    return {"rows": rows, "cast": new}


def replay(seed: int, s: dict, q: dict) -> dict:
    """Every round on one state series: every row of days 8 … 60 (each with its status and probabilities) and every
    cast in order, each with `found_in_round` k (the round whose episode the decision read; the cast is recorded in round
    k + 1)."""
    casts, rows = [], []
    while True:
        res = step(seed, s, q, casts)
        rows += res["rows"]
        if res["cast"] is None:
            break
        casts.append(res["cast"] | {"found_in_round": len(casts)})
    if rows and rows[-1]["day"] < DAYS[-1]:          # cannot happen (a round ends at a cast or day 60); kept explicit
        raise AssertionError(f"seed {seed}: replay stopped at day {rows[-1]['day']}")
    return {"seed": int(seed), "rows": rows, "casts": casts}


def schedule_rows(casts: list[dict]) -> list[dict]:
    """`schedules_p14.py`'s format for a list of casts (each a dict with `arm` and `tick`), in tick order."""
    out = []
    for cst in sorted(casts, key=lambda x: int(x["tick"])):
        kind, amount, _ = ARM_SPEC[cst["arm"]]
        assert int(cst["tick"]) % pol.EVERY == 0, cst
        out.append({"tick": int(cst["tick"]), "kind": kind, "settlement": "home", "amount": int(amount)})
    return out


def prob_table(rows: list[dict]) -> dict[int, tuple[float, float, float]]:
    """row -> (p_g, p_s, p_r) of every decision row (days 8 … 60; 0 for refractory / cap / no-decision rows)."""
    return {int(r["row"]): (r["p_g"], r["p_s"], r["p_r"]) for r in rows}


def ref_rows(casts: list[dict]) -> dict[str, int]:
    """GOAL.md 6.2.5: c1 = the first cast row, or 06:00 of day 30 if the world never cast; c2 = the second cast row, or
    c1 + 14 days if it has none."""
    c1 = int(casts[0]["row"]) if casts else ROW_OF_DAY[NEVER_REF_DAY]
    c2 = int(casts[1]["row"]) if len(casts) > 1 else c1 + 24 * C2_GAP_DAYS
    return {"c1": c1, "c2": c2}


def pre_casts(casts: list[dict], ref_row: int) -> list[dict]:
    """The world's casts at rows < the reference row (the `pre` schedule)."""
    return [x for x in casts if int(x["row"]) < int(ref_row)]


def family_schedules(casts: list[dict], ref_row: int) -> dict[str, list[dict]]:
    """`pre` = the casts before the reference row; each arm = `pre` + the arm at the reference row (nothing after)."""
    pre = pre_casts(casts, ref_row)
    tick = int(ref_row) * pol.EVERY
    out = {"pre": schedule_rows(pre)}
    for arm in ARMS:
        out[arm] = schedule_rows(pre + [{"arm": arm, "tick": tick}])
    return out
