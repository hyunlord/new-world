"""Pure array functions of the WM-1.1 calendar baselines and seed-diversity diagnostics (GOAL_1_1.md 6, 7, 11).

numpy only and no other import: `diversity_v1.py` loads this file by path (it must not import the `wm1` package,
whose `__init__` puts the branch's schema-1 `wm_data` on `sys.path`), and `climatology.py` / `diversity.py`
import it as `wm1.calcore`.

Conventions (GOAL_1_1.md 3.3): every v0/v1 episode starts at tick 0 and records one row per 60 ticks, so row r is
the same calendar instant in every episode; display day D = (60 r + 480) // 1440 + 1; the year has 672 rows
(28 days x 24 h) and the year position is p(r) = (r + 8) mod 672. Arrays of a split are stacked as
`v[E, T, J]` (episodes x rows x variables).
"""
from __future__ import annotations

import numpy as np

YEAR = 672          # rows per 28-day year
ROW0_POS = 8        # p(0): row 0 is 08:00 of day 1
FIRST = 24          # rows 0..23 are excluded from every mean and score (bench_v0.split_std's rows)
TPD, OFFSET, EVERY = 1440, 480, 60


# ------------------------------------------------------------------ calendar ------------------------------------
def year_pos(r):
    """p(r) = (r + 8) mod 672."""
    return (np.asarray(r) + ROW0_POS) % YEAR


def display_day(r):
    """One-based display day of row r."""
    return (EVERY * np.asarray(r) + OFFSET) // TPD + 1


def row_of(day: int, minute: int) -> int:
    """Row of one-based display day `day` at minute-of-day `minute`."""
    return ((day - 1) * TPD + minute - OFFSET) // EVERY


def wrap_row(r, t_tr: int):
    """Rows >= t_tr wrapped back by whole years to the last year that has data:
    r' = r - 672 * ceil((r - (t_tr - 1)) / 672); rows < t_tr are unchanged."""
    r = np.asarray(r, dtype=np.int64)
    k = np.where(r >= t_tr, -((-(r - (t_tr - 1))) // YEAR), 0)   # ceil division for positive numerators
    return r - YEAR * k


def analog_shift(s, horizon: int, t_tr: int):
    """Delta = 672 * max(0, ceil((s + H - (t_tr - 1)) / 672)): one shift for a whole analog window and its targets."""
    s = np.asarray(s, dtype=np.int64)
    num = s + horizon - (t_tr - 1)
    return YEAR * np.maximum(0, -((-num) // YEAR))


def calendar_means(v: np.ndarray, first: int = FIRST) -> tuple[np.ndarray, np.ndarray]:
    """From train episodes `v[E, T, J]`: m_abs[T, J] (mean over episodes of every row, rows 0..T-1) and
    m_year[672, J] (mean over episodes and rows r >= first with p(r) = q; NaN where no row covers q)."""
    m_abs = v.mean(axis=0)
    t = v.shape[1]
    rows = np.arange(first, t)
    q = year_pos(rows)
    sums = np.zeros((YEAR, v.shape[2]))
    counts = np.zeros(YEAR)
    np.add.at(sums, q, v[:, first:, :].sum(axis=0))
    np.add.at(counts, q, v.shape[0])
    with np.errstate(invalid="ignore", divide="ignore"):
        m_year = sums / counts[:, None]
    m_year[counts == 0] = np.nan
    return m_abs, m_year


def mean_at(kind: str, rows, m_abs: np.ndarray, m_year: np.ndarray) -> np.ndarray:
    """The calendar mean of `kind` ("abs" or "year") at the given rows (abs wraps rows past the train length)."""
    rows = np.asarray(rows, dtype=np.int64)
    if kind == "abs":
        return m_abs[wrap_row(rows, m_abs.shape[0])]
    if kind == "year":
        return m_year[year_pos(rows)]
    raise ValueError(kind)


def fit_alpha(v: np.ndarray, m_rows: np.ndarray, k_max: int, first: int = FIRST) -> np.ndarray:
    """Damped anomaly persistence: alpha[k, j] = sum a_s a_{s+k} / sum a_s^2 over every episode and every start
    s with first <= s and s + k < T (a = v - m_rows, m_rows[T, J] the calendar mean at each train row); 0 where
    the denominator is 0. Row 0 of the result is 1 (lead 0)."""
    a = v - m_rows[None]
    t = v.shape[1]
    out = np.ones((k_max + 1, v.shape[2]))
    for k in range(1, k_max + 1):
        x, y = a[:, first: t - k], a[:, first + k: t]
        num, den = (x * y).sum(axis=(0, 1)), (x * x).sum(axis=(0, 1))
        out[k] = np.where(den > 0, num / np.where(den > 0, den, 1.0), 0.0)
    return out


# ------------------------------------------------------------------ analogs -------------------------------------
def analog_neighbours(test_v: np.ndarray, starts: np.ndarray, horizon: int, lib: np.ndarray, inv_std: np.ndarray,
                      n: int, exclude: int | None = None) -> tuple[np.ndarray, np.ndarray]:
    """Lorenz (1969) analogs. For each start s: Delta = analog_shift(s, H, T_lib); distance to library episode e =
    sum over rows s-23..s and the J variables of ((test_v[r] - lib[e, r - Delta]) * inv_std)^2. Returns the indices
    of the n nearest library episodes (stable order on ties) [len(starts), n] and Delta [len(starts)].
    Reads test rows <= s only. `exclude` (a library index) is never chosen (an episode is not its own analog)."""
    starts = np.asarray(starts, dtype=np.int64)
    t_lib = lib.shape[1]
    delta = analog_shift(starts, horizon, t_lib)
    near = np.zeros((len(starts), n), dtype=np.int64)
    for d in np.unique(delta):
        idx = np.nonzero(delta == d)[0]
        top = int(starts[idx].max())               # the last test row any start of this group may read
        lo = max(0, int(starts[idx].min()) - 23)
        if lo - d < 0 or top - d >= t_lib:
            raise ValueError(f"analog window outside the library: rows {lo}..{top}, shift {d}")
        rows = np.arange(lo, top + 1)
        diff = (test_v[rows][None] - lib[:, rows - d]) * inv_std
        per_row = (diff * diff).sum(axis=2)                       # [E, rows]
        csum = np.concatenate([np.zeros((lib.shape[0], 1)), np.cumsum(per_row, axis=1)], axis=1)
        s_local = starts[idx] - lo
        dist = csum[:, s_local + 1] - csum[:, s_local - 23]       # [E, n_starts]
        if exclude is not None:
            dist[exclude] = np.inf
        near[idx] = np.argsort(dist.T, axis=1, kind="stable")[:, :n]
    return near, delta


def analog_predict(member: str, test_v: np.ndarray, starts: np.ndarray, horizon: int, lib: np.ndarray,
                   near: np.ndarray, delta: np.ndarray) -> np.ndarray:
    """Predictions [len(starts), H, J] of rows s+1 .. s+H: `an1` (nearest episode's v[s + k - Delta]), `an5` (mean of
    the nearest five) or `an5_off` (`an5` plus v_test[s] - mean of the five neighbours' v[s - Delta])."""
    starts = np.asarray(starts, dtype=np.int64)
    ks = np.arange(1, horizon + 1)
    rows = starts[:, None] + ks[None] - delta[:, None]                       # [n, H] library rows
    use = near[:, :1] if member == "an1" else near[:, :5]
    traj = lib[use[:, :, None], rows[:, None, :]]                             # [n, m, H, J]
    pred = traj.mean(axis=1)
    if member == "an5_off":
        base = lib[use, (starts - delta)[:, None]].mean(axis=1)              # [n, J]
        pred = pred + (test_v[starts] - base)[:, None, :]
    elif member not in ("an1", "an5"):
        raise ValueError(member)
    return pred


MEMBERS = ("cal_abs", "cal_year", "ap1_abs", "ap1_year", "apfit_abs", "apfit_year", "an1", "an5", "an5_off")
FAIR = ("cal_year", "ap1_year", "apfit_year")      # members that never read absolute time


def predict_member(member: str, v: np.ndarray, starts, horizon: int, lib: np.ndarray, m_abs: np.ndarray,
                   m_year: np.ndarray, alpha: dict, inv_std: np.ndarray, exclude: int | None = None) -> np.ndarray:
    """Predictions [len(starts), H, J] of rows s+1 .. s+H of one episode v[T, J] by a calendar-family member (direct,
    no feedback; reads rows <= s of v only). `alpha` maps "abs" / "year" to fit_alpha's [k_max + 1, J]."""
    starts = np.asarray(starts, dtype=np.int64)
    if member.startswith("an"):
        near, delta = analog_neighbours(v, starts, horizon, lib, inv_std, 5, exclude=exclude)
        return analog_predict(member, v, starts, horizon, lib, near, delta)
    if member not in MEMBERS:
        raise ValueError(member)
    kind = "abs" if member.endswith("abs") else "year"
    targets = starts[:, None] + np.arange(1, horizon + 1)[None]
    m_t = mean_at(kind, targets, m_abs, m_year)                                # [n, H, J]
    if member.startswith("cal_"):
        return m_t
    anom = v[starts] - mean_at(kind, starts, m_abs, m_year)                    # [n, J]
    a = np.ones((horizon, v.shape[1])) if member.startswith("ap1") else alpha[kind][1: horizon + 1]
    if len(a) != horizon:
        raise ValueError(f"horizon {horizon} exceeds the fitted leads")
    return m_t + a[None] * anom[:, None, :]


# ------------------------------------------------------------------ diversity -----------------------------------
def r2(v: np.ndarray, m: np.ndarray, first: int = FIRST) -> np.ndarray:
    """1 - sum (v - m)^2 / sum (v - vbar)^2 per variable over rows >= first, vbar the split's pooled mean; v[E, T, J],
    m[T, J] the reference trajectory at the same rows. NaN where the variable is constant."""
    x, mm = v[:, first:], m[first:][None]
    vbar = x.mean(axis=(0, 1))
    res = ((x - mm) ** 2).sum(axis=(0, 1))
    tot = ((x - vbar) ** 2).sum(axis=(0, 1))
    with np.errstate(invalid="ignore", divide="ignore"):
        return np.where(tot > 0, 1.0 - res / np.where(tot > 0, tot, 1.0), np.nan)


def eta2(v: np.ndarray, first: int = FIRST) -> np.ndarray:
    """Seed-variance fraction: sum_r sum_e (v_e[r] - m_split[r])^2 / sum_r sum_e (v_e[r] - vbar)^2 over rows >= first,
    m_split the split's own per-row mean. 0 when every seed traces the same trajectory."""
    x = v[:, first:]
    within = ((x - x.mean(axis=0, keepdims=True)) ** 2).sum(axis=(0, 1))
    tot = ((x - x.mean(axis=(0, 1))) ** 2).sum(axis=(0, 1))
    with np.errstate(invalid="ignore", divide="ignore"):
        return np.where(tot > 0, within / np.where(tot > 0, tot, 1.0), np.nan)


def eta2_within_groups(v: np.ndarray, groups: list[int], first: int = FIRST) -> tuple[np.ndarray, int]:
    """eta^2 with the per-row mean taken within each group of episodes (e.g. equal winter-severity sequences), pooled
    over the groups that have >= 2 episodes; the denominator is the same episodes' sum of squares around the split's
    pooled mean (rows >= first). Returns (per variable, number of episodes used)."""
    x = v[:, first:]
    vbar = x.mean(axis=(0, 1))
    labels = np.asarray(groups)
    num, den, used = np.zeros(v.shape[2]), np.zeros(v.shape[2]), 0
    for g in np.unique(labels):
        sel = np.nonzero(labels == g)[0]
        if len(sel) < 2:
            continue
        xs = x[sel]
        num += ((xs - xs.mean(axis=0, keepdims=True)) ** 2).sum(axis=(0, 1))
        den += ((xs - vbar) ** 2).sum(axis=(0, 1))
        used += len(sel)
    with np.errstate(invalid="ignore", divide="ignore"):
        return np.where(den > 0, num / np.where(den > 0, den, 1.0), np.nan), used


def divergence_rows(a: np.ndarray, b: np.ndarray, tol: float = 1e-9) -> tuple[int, np.ndarray]:
    """First row >= 1 where any variable of two episodes a[T, J], b[T, J] differs by more than `tol` (-1: never), and
    the same per variable [J]. Rows beyond the shorter episode are not compared."""
    t = min(len(a), len(b))
    diff = np.abs(a[1:t] - b[1:t]) > tol                 # row index i -> row i + 1
    per_var = np.where(diff.any(axis=0), diff.argmax(axis=0) + 1, -1)
    anyv = diff.any(axis=1)
    return (int(anyv.argmax()) + 1 if anyv.any() else -1), per_var


def pair_list(n: int, cap: int = 500, seed: int = 0) -> list[tuple[int, int]]:
    """Every unordered pair of n episodes, or `cap` of them drawn without replacement (numpy seed `seed`)."""
    pairs = [(i, j) for i in range(n) for j in range(i + 1, n)]
    if len(pairs) <= cap:
        return pairs
    pick = np.random.default_rng(seed).choice(len(pairs), size=cap, replace=False)
    return [pairs[k] for k in sorted(pick)]


def divergence_summary(v: list[np.ndarray], names: tuple[str, ...], cap: int = 500, seed: int = 0) -> dict:
    """Divergence day (display day of the first differing row) over episode pairs: min / median / max over pairs
    that diverge and the number that never do; overall and per variable."""
    pairs = pair_list(len(v), cap, seed)
    first, per = [], []
    for i, j in pairs:
        f, pv = divergence_rows(v[i], v[j])
        first.append(f)
        per.append(pv)
    first, per = np.array(first), np.array(per).reshape(len(pairs), len(names))

    def stats(rows: np.ndarray) -> dict:
        hit = rows[rows >= 0]
        days = display_day(hit) if len(hit) else hit
        return {"pairs": int(len(rows)), "never": int((rows < 0).sum()),
                "min_day": int(days.min()) if len(hit) else None,
                "median_day": float(np.median(days)) if len(hit) else None,
                "max_day": int(days.max()) if len(hit) else None,
                "median_row": float(np.median(hit)) if len(hit) else None}

    out = stats(first) | {"capped": len(pairs) < len(v) * (len(v) - 1) // 2, "per_var": {}}
    for j, name in enumerate(names):
        out["per_var"][name] = stats(per[:, j])
    return out


def spread_at(v: np.ndarray, rows: list[int], std: np.ndarray) -> np.ndarray:
    """Std over episodes of each variable at the given rows, divided by the split std: [len(rows), J] (NaN where the
    split std is 0)."""
    s = v[:, rows].std(axis=0)
    with np.errstate(invalid="ignore", divide="ignore"):
        return np.where(std > 0, s / np.where(std > 0, std, 1.0), np.nan)


def split_std(v: np.ndarray, first: int = FIRST) -> np.ndarray:
    """bench_v0.split_std on stacked arrays of equal length: std of rows >= first over the split."""
    return v[:, first:].reshape(-1, v.shape[2]).std(axis=0)


def winter_days(n_rows: int, last_day: int | None = None) -> list[int]:
    """One-based days 22, 50, 78, ... (the first day of each winter) whose 18:00 row exists (and <= last_day)."""
    out, day = [], 22
    while row_of(day, 1080) < n_rows and (last_day is None or day <= last_day):
        out.append(day)
        day += 28
    return out


def split_diagnostics(v: np.ndarray, m_abs: np.ndarray, m_year: np.ndarray, std: np.ndarray,
                      severity_sequences: list[tuple], names: tuple[str, ...],
                      spread_days: tuple[int, ...] = (30, 60, 90, 118), cap: int = 500, seed: int = 0) -> dict:
    """GOAL_1_1.md 7 for one split v[E, T, J] (rows >= 24): R^2 against the train calendar means (CalAbs wrapped past
    the train length, CalYear), eta^2, divergence days, the between-seed spread at 06:00 of `spread_days` over the
    split std, and eta^2 within groups of equal winter-severity sequences (a description, never an input)."""
    rows = np.arange(v.shape[1])
    ref_abs = mean_at("abs", rows, m_abs, m_year)
    ref_year = mean_at("year", rows, m_abs, m_year)
    per = lambda arr: {n: (None if not np.isfinite(x) else float(x)) for n, x in zip(names, arr)}
    seqs = {}
    labels = [seqs.setdefault(tuple(s), len(seqs)) for s in severity_sequences]
    e2w, used = eta2_within_groups(v, labels)
    srows = [row_of(dd, 360) for dd in spread_days if row_of(dd, 360) < v.shape[1]]
    spread = spread_at(v, srows, std)
    sizes = np.bincount(labels) if labels else np.zeros(0, dtype=int)
    return {"episodes": int(v.shape[0]), "rows": int(v.shape[1]),
            "r2_abs": per(r2(v, ref_abs)), "r2_year": per(r2(v, ref_year)), "eta2": per(eta2(v)),
            "divergence": divergence_summary(list(v), names, cap, seed),
            "spread": {str(dd): per(spread[i]) for i, dd in enumerate(spread_days[: len(srows)])},
            "severity": {"distinct_sequences": len(seqs), "group_sizes": sorted(sizes.tolist(), reverse=True),
                         "episodes_in_groups_of_2plus": int(used), "eta2_within_groups": per(e2w)}}
