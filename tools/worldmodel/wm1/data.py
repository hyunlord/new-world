"""Settlement-level row features, targets and training windows for WM-1 (GOAL.md 6.1). numpy only.

One feature vector per recorded hour of an episode:

- state: the `global.npy` columns allowed by GOAL.md 4.2 (cumulative tallies as their hourly differences) and
  `settlements.npy` reduced as `bench_v0` reduces it (stocks summed, `fire_lit` = max, formation omitted,
  NaN -> 0 with a presence flag); `wood_target` only in severity-on variants;
- calendar: hour one-hot (24), season one-hot including -1 (5), day of season / 7;
- actions (action-aware variants): per kind, the requested amount applied at the row's tick (`bench_v0`'s
  encoding), divided by the kind's largest grid amount;
- severity one-hot (severity-on variants): mild / normal / harsh / none.

Never inputs: `latent.npy`, `lifespan_days`, `winter_severity` as a column (only the severity-on one-hot),
`wood_target` in severity-blind variants. Targets: the next row's change of every continuous state feature
(divided by the train std of that change) and the next `fire_lit` as a binary.

Recorder schema >= 2 (GOAL_1_2.md 7.1; the p10 pilot is schema 4): observable columns only (`wm_data`'s observable
filter; a hidden column raises `bench_v1.HiddenInput`). v0's columns keep their order, `s.wood_target` is dropped
(hidden), and when recorded `d.deaths_exposure` (hourly difference) and `g.injured_members` are appended. The
settlement reduction stays sum / max. `EpisodeData.bench` stays a `bench_v0.Ep`, built from
`bench_v1.episode_arrays` with `severity` = the hidden severity (labels for breakdowns only) and `deaths` including
the exposure deaths (as `Layout.deaths`). Severity-on variants cannot be built on schema >= 2 data.

GOAL_1_3.md 4.2: a dataset may append observable `global.npy` levels after `g.injured_members` (`common.Dataset.
extra_state`; the game_v17 arm: `g.settlements_alive`, `g.fissions_total`), before the calendar: they are state
features (normalized, predicted, rolled out). The settlement columns `parent_slot` and `founded_tick` (schema 5) are
never read.
"""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np

import wm1  # noqa: F401  (puts tools/worldmodel on sys.path)
import bench_v0 as b
import bench_v1 as bv1
from wm_data import load_episode

GLOBAL_STATE = ("alive", "children", "elders", "adults", "cold_members", "deadwood_total", "food_on_bushes_total",
                "adults_working", "idle_adults", "settlements_under_rain", "cold_snap_nights_left", "cold_snap_tonight")
GLOBAL_TALLIES = ("births", "deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age",
                  "meals_from_store_total", "ground_meals_total", "forage_trips_total", "wood_trips_total",
                  "wood_gathered_total", "urgent_wood_trips_total", "wood_retargets_total", "cold_member_hours_total",
                  "interventions_applied_total")
SETTLEMENT_SUM = ("members", "food_stock", "wood_stock", "wood_in_hand")
SETTLEMENT_MAX = ("fire_lit", "rain_ticks_left")
SEVERITY_ONLY = ("s.wood_target",)
# Columns that reveal the severity or are hidden ground truth; no variant reads them as a state column.
FORBIDDEN = ("g.winter_severity", "latent", "lifespan_days", "s.formation_x", "s.formation_y")

ALL_STATE = (tuple(f"g.{x}" for x in GLOBAL_STATE) + tuple(f"d.{x}" for x in GLOBAL_TALLIES)
             + tuple(f"s.{x}" for x in SETTLEMENT_SUM + SETTLEMENT_MAX) + ("s.present",) + SEVERITY_ONLY)
CALENDAR = tuple(f"hour_{h}" for h in range(24)) + tuple(f"season_{s}" for s in (-1, 0, 1, 2, 3)) + ("day_of_season",)
ACTION_NAMES = tuple(f"act.{k}" for k in b.KINDS)
# Largest amount of each kind's B_act grid (DATASET_v0.md): food 120, wood 80, rain 24 h, cold snap 3 nights, windfall 40.
ACTION_SCALE = np.array([120.0, 120.0, 80.0, 80.0, 24.0, 3.0, 40.0])
SEVERITY_NAMES = ("sev_mild", "sev_normal", "sev_harsh", "sev_none")
V_STATE = ("s.food_stock", "s.wood_stock", "s.fire_lit", "g.cold_members", "g.alive", "g.deadwood_total",
           "g.food_on_bushes_total", "g.adults_working")
FIRE = "s.fire_lit"
COLD_HOURS = "d.cold_member_hours_total"
DEATHS = ("d.deaths_starvation", "d.deaths_dehydration", "d.deaths_combat", "d.deaths_old_age")
EXPOSURE = "d.deaths_exposure"
# Recorder schema >= 2: `s.wood_target` is hidden; schema 4 appends these two observable columns (in this order).
OBS_APPEND = (EXPOSURE, "g.injured_members")
ALL_STATE_OBS = tuple(n for n in ALL_STATE if n not in SEVERITY_ONLY) + OBS_APPEND
# Hidden in schema >= 2 (labels and analysis only): never a column of the observable table or an input.
HIDDEN_V2 = ("winter_severity", "night_burn_per_hour", "winter_fall_today", "wood_target", "latent", "lifespan_days",
             "world_variety", "berry_bushes", "berry_capacity", "climate", "deadwood_capacity", "deadwood_spots",
             "food_regen_permille", "forest_permille", "starting_agents", "wild_regions", "region_bushes",
             "region_deadwood", "regions")
# Schema-5 settlement columns that are observable but never inputs (GOAL_1_3.md 4.2).
NOT_INPUTS_V5 = ("parent_slot", "founded_tick")


@dataclass(frozen=True)
class Variant:
    """Which inputs a model may read: the severity (`sev_on`) and the interventions (`action_aware`). `obs_state`
    (recorder schema >= 2) is the observable table's column list; such a variant is never severity-on."""
    sev_on: bool
    action_aware: bool
    obs_state: tuple[str, ...] | None = None

    def __post_init__(self):
        if self.obs_state is not None and self.sev_on:
            raise ValueError("severity-on variants need the hidden severity: not available on recorder schema >= 2")

    @property
    def name(self) -> str:
        return ("sev-on" if self.sev_on else "sev-blind") + ("_action-aware" if self.action_aware else "_action-blind")

    @property
    def state_names(self) -> tuple[str, ...]:
        if self.obs_state is not None:
            return self.obs_state
        return tuple(n for n in ALL_STATE if self.sev_on or n not in SEVERITY_ONLY)

    @property
    def continuous_names(self) -> tuple[str, ...]:
        return tuple(n for n in self.state_names if n != FIRE)

    def input_names(self) -> tuple[str, ...]:
        """Every input field of the variant, in model order (state, calendar, actions, severity)."""
        names = self.state_names + CALENDAR
        if self.action_aware:
            names += ACTION_NAMES
        if self.sev_on:
            names += SEVERITY_NAMES
        return names


MAIN = Variant(sev_on=False, action_aware=True)
VARIANTS = {v.name: v for v in (Variant(False, True), Variant(True, True), Variant(False, False), Variant(True, False))}


def variant_for(name: str, obs_state: tuple[str, ...] | None = None) -> Variant:
    """The variant `name` on v0 (`obs_state` None) or on observable-schema data with that column list (raises
    ValueError for a severity-on name)."""
    v = VARIANTS[name]
    return v if obs_state is None else Variant(v.sev_on, v.action_aware, tuple(obs_state))


@dataclass
class EpisodeData:
    """One episode: the bench's arrays (`bench`, a `bench_v0.Ep`) and every candidate state column (`table`)."""
    split: str
    role: str
    seed: int
    eid: str
    bench: b.Ep
    table: np.ndarray          # [T, len(names)] float64
    calendar: np.ndarray       # [T, len(CALENDAR)]
    interventions: list
    names: tuple = ALL_STATE   # the table's columns (ALL_STATE for v0; the observable list for schema >= 2)
    schema: int = 1            # recorder schema of the episode

    @property
    def rows(self) -> int:
        return self.table.shape[0]

    def state(self, variant: Variant) -> np.ndarray:
        return self.table[:, [self.names.index(n) for n in variant.state_names]]

    def severity_onehot(self) -> np.ndarray:
        if self.schema >= 2:
            raise ValueError("the severity is hidden in recorder schema >= 2: no severity input")
        return b.onehot(self.bench.severity, [0, 1, 2, -1])


def calendar_features(hour: np.ndarray, season: np.ndarray, day_of_season: np.ndarray) -> np.ndarray:
    return np.concatenate([b.onehot(hour, list(range(24))), b.onehot(season, [-1, 0, 1, 2, 3]),
                           (np.asarray(day_of_season, dtype=np.float64) / 7.0)[:, None]], axis=1)


def episode_table(ep) -> np.ndarray:
    """All candidate state columns (ALL_STATE order) of a loaded `wm_data.Episode`."""
    g = ep.arrays["global.npy"].astype(np.float64)
    s = ep.arrays["settlements.npy"].astype(np.float64)
    gcol = lambda name: g[:, ep.field_index("global.npy", name)]
    scol = lambda name: s[:, :, ep.field_index("settlements.npy", name)]
    cols = [gcol(x) for x in GLOBAL_STATE]
    for x in GLOBAL_TALLIES:
        c = gcol(x)
        cols.append(np.concatenate([[0.0], np.diff(c)]))
    for x in SETTLEMENT_SUM:
        cols.append(np.nansum(scol(x), axis=1))
    for x in SETTLEMENT_MAX:
        cols.append(np.nan_to_num(scol(x), nan=0.0).max(axis=1, initial=0.0))
    cols.append((~np.isnan(scol("members"))).any(axis=1).astype(np.float64))
    cols.append(np.nansum(scol("wood_target"), axis=1))
    table = np.stack(cols, axis=1)
    return np.nan_to_num(table, nan=0.0)


def episode_table_observable(ep, extra: tuple[str, ...] = ()) -> tuple[np.ndarray, tuple[str, ...]]:
    """The observable state table of a schema >= 2 episode and its column names (ALL_STATE without `s.wood_target`,
    then `d.deaths_exposure` and `g.injured_members` when the episode records them, then the dataset's `extra`
    `g.<field>` columns as raw levels, GOAL_1_3.md 4.2). Every column goes through `bench_v1.observable_column`,
    which refuses a hidden field; a missing extra field raises KeyError."""
    t = ep.arrays["global.npy"].shape[0]
    gcol = lambda name: bv1.observable_column(ep, "global.npy", name).astype(np.float64)
    s = ep.arrays["settlements.npy"]
    scol = ((lambda name: bv1.observable_column(ep, "settlements.npy", name).astype(np.float64)) if s.shape[1]
            else (lambda name: np.zeros((t, 0))))
    gobs = set(ep.observable_names("global.npy"))
    names, cols = [], []
    for x in GLOBAL_STATE:
        names.append(f"g.{x}")
        cols.append(gcol(x))
    tallies = GLOBAL_TALLIES + (("deaths_exposure",) if "deaths_exposure" in gobs else ())
    for x in GLOBAL_TALLIES:
        names.append(f"d.{x}")
        cols.append(np.concatenate([[0.0], np.diff(gcol(x))]))
    for x in SETTLEMENT_SUM:
        names.append(f"s.{x}")
        cols.append(np.nansum(scol(x), axis=1))
    for x in SETTLEMENT_MAX:
        names.append(f"s.{x}")
        cols.append(np.nan_to_num(scol(x), nan=0.0).max(axis=1, initial=0.0))
    names.append("s.present")
    cols.append((~np.isnan(scol("members"))).any(axis=1).astype(np.float64))
    if "deaths_exposure" in tallies:
        names.append(EXPOSURE)
        cols.append(np.concatenate([[0.0], np.diff(gcol("deaths_exposure"))]))
    if "injured_members" in gobs:
        names.append("g.injured_members")
        cols.append(gcol("injured_members"))
    for x in extra:
        if not x.startswith("g."):
            raise ValueError(f"extra state column {x}: only global.npy levels (`g.<field>`) are supported")
        names.append(x)
        cols.append(gcol(x[2:]))
    return np.nan_to_num(np.stack(cols, axis=1), nan=0.0), tuple(names)


def bench_of(ep, split: str, role: str, seed: int, eid: str) -> b.Ep:
    """The `bench_v0.Ep` of a loaded episode: v0's arrays for schema 1; for schema >= 2 `bench_v1.episode_arrays`
    (observable inputs) with `severity` = the hidden severity (labels only) and the exposure deaths in `deaths`."""
    if int(ep.meta.get("schema_version", 1)) < 2:
        return b.Ep(split, role, seed, eid, **b.episode_arrays(ep))
    arrays = bv1.episode_arrays(ep)
    severity = arrays.pop("hidden_severity")
    if "deaths_exposure" in ep.observable_names("global.npy"):
        arrays["deaths"] = arrays["deaths"] + bv1.observable_column(ep, "global.npy", "deaths_exposure")
    return b.Ep(split, role, seed, eid, severity=severity, **arrays)


def from_episode(ep, split: str, role: str, seed: int, eid: str, extra: tuple[str, ...] = ()) -> EpisodeData:
    schema = int(ep.meta.get("schema_version", 1))
    if extra and schema < 2:
        raise ValueError("extra state columns need an observable-schema episode")
    bench = bench_of(ep, split, role, seed, eid)
    g = ep.arrays["global.npy"]
    dos = g[:, ep.field_index("global.npy", "day_of_season")]
    cal = calendar_features(bench.hour, bench.season, dos)
    if schema < 2:
        return EpisodeData(split, role, seed, eid, bench, episode_table(ep), cal, list(ep.interventions))
    table, names = episode_table_observable(ep, extra)
    return EpisodeData(split, role, seed, eid, bench, table, cal, list(ep.interventions), names, schema)


def load_split(root: Path, manifest: dict, split: str, roles: tuple[str, ...] | None = None,
               seeds: set[int] | None = None, extra: tuple[str, ...] = ()) -> list[EpisodeData]:
    """Episodes of `split` (optionally only some roles / seeds) in manifest order, sha256-verified on load; `extra`
    appends those observable global columns to the state table (`episode_table_observable`)."""
    out = []
    for entry in manifest["episodes"]:
        if entry["split"] != split or (roles is not None and entry["role"] not in roles):
            continue
        if seeds is not None and entry["seed"] not in seeds:
            continue
        ep = load_episode(Path(root) / entry["path"], arrays=["global.npy", "settlements.npy"])
        out.append(from_episode(ep, split, entry["role"], entry["seed"], entry["schedule_id"], extra))
    return out


def actions_of(interventions: list, tick: np.ndarray) -> np.ndarray:
    """[T, 7] requested amounts at the row of each intervention's tick (bench_v0.episode_arrays' encoding)."""
    out = np.zeros((len(tick), len(b.KINDS)))
    index = {int(t): i for i, t in enumerate(tick)}
    for rec in interventions:
        if int(rec["tick"]) in index:
            out[index[int(rec["tick"])], b.KINDS.index(rec["kind"])] += float(rec["amount"])
    return out


# ------------------------------------------------------------------ normalization ---------------------------------
@dataclass
class Normalizer:
    """Train-only statistics: mean / std of every state feature, std of every continuous feature's hourly change."""
    names: tuple[str, ...]
    continuous: tuple[str, ...]
    mean: np.ndarray
    std: np.ndarray
    dstd: np.ndarray
    fit_rows: int
    lo: np.ndarray | None = None   # train minimum of every state feature
    hi: np.ndarray | None = None   # train maximum

    @classmethod
    def fit(cls, eps: list[EpisodeData], variant: Variant) -> "Normalizer":
        if any(ep.role != "train" for ep in eps):
            raise ValueError("normalization statistics come from train episodes only")
        states = [ep.state(variant) for ep in eps]
        rows = np.vstack(states)
        cidx = [variant.state_names.index(n) for n in variant.continuous_names]
        deltas = np.vstack([np.diff(s[:, cidx], axis=0) for s in states])
        std = rows.std(axis=0)
        dstd = deltas.std(axis=0)
        return cls(variant.state_names, variant.continuous_names, rows.mean(axis=0), np.where(std > 0, std, 1.0),
                   np.where(dstd > 0, dstd, 1.0), int(rows.shape[0]), rows.min(axis=0), rows.max(axis=0))

    def to_json(self) -> dict:
        return {"names": list(self.names), "continuous": list(self.continuous), "mean": self.mean.tolist(),
                "std": self.std.tolist(), "dstd": self.dstd.tolist(), "fit_rows": self.fit_rows,
                "lo": None if self.lo is None else self.lo.tolist(), "hi": None if self.hi is None else self.hi.tolist()}

    @classmethod
    def from_json(cls, d: dict) -> "Normalizer":
        arr = lambda k: None if d.get(k) is None else np.array(d[k])
        return cls(tuple(d["names"]), tuple(d["continuous"]), np.array(d["mean"]), np.array(d["std"]),
                   np.array(d["dstd"]), int(d["fit_rows"]), arr("lo"), arr("hi"))


@dataclass
class Prepared:
    """Normalized per-row arrays of one episode for one variant."""
    ep: EpisodeData
    state: np.ndarray       # [T, Ds] raw
    x_state: np.ndarray     # [T, Ds] normalized
    calendar: np.ndarray    # [T, 30]
    actions: np.ndarray     # [T, 7] scaled (zeros for action-blind variants)
    sev: np.ndarray         # [T, 4] (zeros-width for severity-blind variants)
    dtarget: np.ndarray     # [T-1, Dc] normalized change of the continuous features
    fire_next: np.ndarray   # [T-1] next fire_lit


class Layout:
    """Index bookkeeping of a variant's state vector."""

    def __init__(self, variant: Variant):
        self.variant = variant
        names = variant.state_names
        self.names = names
        self.cont = np.array([names.index(n) for n in variant.continuous_names])
        self.fire = names.index(FIRE)
        self.v = np.array([names.index(n) for n in V_STATE])
        self.cold_hours = names.index(COLD_HOURS)
        self.deaths = np.array([names.index(n) for n in DEATHS + ((EXPOSURE,) if EXPOSURE in names else ())])
        self.n_state = len(names)
        self.n_cont = len(self.cont)
        self.n_input = len(variant.input_names())


def prepare(ep: EpisodeData, variant: Variant, norm: Normalizer, actions: np.ndarray | None = None) -> Prepared:
    """Normalize one episode. `actions` overrides the recorded requested amounts (T4 rollouts)."""
    lay = Layout(variant)
    if variant.sev_on and ep.schema >= 2:
        raise ValueError("severity-on variants cannot be prepared on recorder schema >= 2 data")
    state = ep.state(variant)
    x = (state - norm.mean) / norm.std
    acts = ep.bench.actions if actions is None else actions
    acts = acts / ACTION_SCALE if variant.action_aware else np.zeros((ep.rows, 0))
    sev = ep.severity_onehot() if variant.sev_on else np.zeros((ep.rows, 0))
    dtarget = np.diff(state[:, lay.cont], axis=0) / norm.dstd
    return Prepared(ep, state, x, ep.calendar, acts, sev, dtarget, state[1:, lay.fire].copy())


def model_input(x_state: np.ndarray, calendar: np.ndarray, actions: np.ndarray, sev: np.ndarray) -> np.ndarray:
    """Concatenate inputs in `Variant.input_names` order."""
    return np.concatenate([x_state, calendar, actions, sev], axis=-1)


def sample_windows(preps: list[Prepared], length: int, batch: int, rng: np.random.Generator) -> dict[str, np.ndarray]:
    """`batch` random contiguous windows of `length` + 1 rows from train episodes (episodes weighted by length).

    Returns inputs of rows start .. start+length (the last row's input feeds rollout losses only), the
    normalized change and next fire of rows start .. start+length-1, and the raw states."""
    sizes = np.array([p.ep.rows - length - 1 for p in preps], dtype=np.float64)
    if (sizes < 1).any():
        raise ValueError("an episode is shorter than the window")
    which = rng.choice(len(preps), size=batch, p=sizes / sizes.sum())
    starts = np.array([rng.integers(0, int(sizes[i]) + 1) for i in which])
    out = {k: [] for k in ("x_state", "calendar", "actions", "sev", "dtarget", "fire_next", "state")}
    for i, s in zip(which, starts):
        p = preps[i]
        sl = slice(s, s + length + 1)
        out["x_state"].append(p.x_state[sl])
        out["calendar"].append(p.calendar[sl])
        out["actions"].append(p.actions[sl])
        out["sev"].append(p.sev[sl])
        out["state"].append(p.state[sl])
        out["dtarget"].append(p.dtarget[s:s + length])
        out["fire_next"].append(p.fire_next[s:s + length])
    return {k: np.stack(v).astype(np.float32) for k, v in out.items()}
