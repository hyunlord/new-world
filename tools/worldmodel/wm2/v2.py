"""Dataset v2 constants and the definitions shared by the WM-2 tools (GOAL.md 4.2, 6, 7). numpy only.

Rows are 60 ticks (one hour); a day is 24 rows; the display day of tick t is (t + 480) // 1,440 + 1
(`timeline_p14.py`); the last row of an episode is row 2,880 (tick 172,800 = 08:00 of display day 121 by that
formula; GOAL.md 7 calls it "day 120", the episode's length — DECISIONS H2.2). The **split event** after a reference row
is the first later row at which `g.fissions_total` exceeds its value at the reference row by >= 0.5 (truth: an integer
series; prediction: the continuous rolled-out state); the **restricted split day** is min(display day of the split
event, display day of the last row), censored when there is no event.
"""
from __future__ import annotations

import numpy as np

TPD, OFFSET, EVERY = 1440, 480, 60
LAST_ROW = 2880
FORK_TICK = 14280
T_EVE = 29400
# GOAL.md 6.2: the four arms of the families (schedules_p14.py names): kind and requested amount.
ARMS = ("sp240L", "sp120L", "gf240E", "rw120E")
ARM_SPEC = {"sp240L": ("spoil_food", 240), "sp120L": ("spoil_food", 120), "gf240E": ("gift_food", 240),
            "rw120E": ("rot_wood", 120)}
W3_ARMS = ("sp240L", "gf240E", "rw120E")
W4_ARMS = ("sp240L", "gf240E")
# GOAL.md 6.4, 7.7: the noise-floor starts (display days 15, 40, 70 at 06:00) and the split-timing start (day 25 06:00).
FLOOR_STARTS = (20040, 56040, 99240)
FLOOR_R = 4
FLOOR_HORIZONS = (24, 168)
SPLIT_START_TICK = 34440
# GOAL.md 4.2: roles (half-open seed ranges).
ROLES = {"game_v19": {"train": (40000, 40400), "val": (40400, 40480), "test": (40480, 40560)},
         "game_v16": {"train": (42000, 42400), "val": (42400, 42480), "test": (42480, 42560)}}
# GOAL.md 7.4: effect outcomes and lags (rows after the cast).
OUTCOMES = ("food_stock", "alive", "deaths", "settlements_alive")
LAGS = {"7d": 168, "30d": 720}
DEATH_COLUMNS = ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure")
# Hidden in the recorder (never inputs, GOAL.md 3.2, 4.3).
HIDDEN_FIELDS = ("fission_pressure", "fission_threshold", "world_variety", "latent")


def role_of(scene: str, seed: int) -> str:
    for role, (lo, hi) in ROLES[scene].items():
        if lo <= seed < hi:
            return role
    raise ValueError(f"{scene} seed {seed} is outside the WM-2 roles")


def display_day(tick):
    """One-based display day of a tick (`timeline_p14.display_day`)."""
    return (np.asarray(tick) + OFFSET) // TPD + 1


def day_of_row(row):
    return display_day(np.asarray(row) * EVERY)


def row_of_tick(tick: int) -> int:
    """The recorded row of a tick; every cast / start tick of this run is a multiple of 60 (asserted)."""
    if int(tick) % EVERY:
        raise ValueError(f"tick {tick} is not a multiple of {EVERY}: not on a recorded row")
    return int(tick) // EVERY


LAST_DAY = int(day_of_row(LAST_ROW))


def split_event_row(fissions: np.ndarray, ref_row: int) -> int | None:
    """First row t > ref_row with fissions[t] - fissions[ref_row] >= 0.5 (`fissions` indexed by absolute row), or
    None (no event up to the end of the series)."""
    f = np.asarray(fissions, dtype=np.float64)
    hit = np.nonzero(f[ref_row + 1:] - f[ref_row] >= 0.5)[0]
    return int(ref_row + 1 + hit[0]) if len(hit) else None


def restricted_split_day(fissions: np.ndarray, ref_row: int, last_row: int = LAST_ROW) -> tuple[int, bool]:
    """(restricted split day, censored): min(display day of the split event, display day of `last_row`); censored
    when there is no event up to `last_row`."""
    ev = split_event_row(np.asarray(fissions)[: last_row + 1], ref_row)
    last = int(day_of_row(last_row))
    if ev is None:
        return last, True
    return min(int(day_of_row(ev)), last), False


def member_series(ep) -> dict[str, np.ndarray]:
    """Per-row truth series of one recorded member (`wm_data.Episode`, schema 6): summed food store, `alive`, the
    cumulative deaths of the five causes (`families.DEATH_COLUMNS`), `settlements_alive`, `fissions_total`.
    Observable columns only."""
    import bench_v1 as bv1
    g = lambda name: bv1.observable_column(ep, "global.npy", name).astype(np.float64)
    s = ep.arrays["settlements.npy"]
    food = (np.nansum(bv1.observable_column(ep, "settlements.npy", "food_stock").astype(np.float64), axis=1)
            if s.shape[1] else np.zeros(s.shape[0]))
    return {"food_stock": food, "alive": g("alive"), "deaths": sum(g(k) for k in DEATH_COLUMNS),
            "settlements_alive": g("settlements_alive"), "fissions_total": g("fissions_total")}
