#!/usr/bin/env python3
"""Pass 13 Part 0 item 5 (PASS13_GOAL.md 5.5): diagnosis of Pass 12's open J5 cases. numpy only; read-only.

usage: j5_diag_p13.py [--heldout DIR] [--tuning DIR] [--recordings DIR] [--out DIR]

Inputs (defaults = the runs Pass 12's final J5 verdict pooled, `p12-7_final_heldout_v18_targets/j_targets.json`):
- `--heldout`: `p12-7_final_heldout_v18_120day` (seeds 32..95), `--tuning`: `p12-7_timing_v18_120day` (seeds 0..31; equal
  to tuning round 3 on every shared value). summary.json + per-seed rows + `*_deaths.jsonl`.
- `--recordings`: the four seeds (24, 39, 66, 77) recorded with game_v18 Pass 12 final code, `--agents`, a row every 60
  ticks (recorder schema 5: `settlements.npy`, `agents.npy`, `resources.npy` + `resource_keys.npy`).

Reproduces evaluate_pass12's J5 numbers by importing its functions (`settlement_winters`, Pass 11's `stage_pool`), then:
- for every settlement-winter over 0.35: settlement type / age / founding season, world and settlement dark nights,
  deadwood near the hearth per spot, wood stock through the winter, winter wood lost, members and deaths by stage;
- for the child share: exposure deaths (death census, `stage` field as the probe's pool) by settlement type and by
  severity, against the stage composition at the winter starts (per-capita rates).

Definitions (all clock instants as the probe: tick 0 = 08:00 of display day 1, clock offset 480):
- winter window = [day D 18:00, day D+7 07:00) (probe `seed_winters`); burn window 18:00-07:00, so the recorder rows
  19:00..07:00 (13 per night) carry each burn hour's `fire_lit`.
- world dark night (probe): evening e in D..D+6 whose 02:00 sample has the world `fire_lit` 0 (world flag = every
  settlement's fire lit). Settlement dark night (02:00 rule): the same test on this settlement's `fire_lit`.
  Settlement night with any unlit burn hour: >= 1 of its 13 burn rows has `fire_lit` 0. Unlit burn hours: their count.
- deadwood near the hearth: deadwood tiles (resource kind 3) within Chebyshev 18 (the firewood rule's `wood_radius`, the
  search radius of a wood gatherer) of the settlement's formation tile at the winter start row; per spot = sum of
  amounts (absent = 0) / spots. Radius 9 (`SETTLED_REGION_RADIUS`) is reported beside it.
- winter wood lost: schema 12 has only the per-seed run total `pass12.winter_wood_lost_settled` and the world-cumulative
  row value `winter_wood_lost_total`; the per-winter, per-settlement settled split is not available.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools/revitalize"))
sys.path.insert(0, str(REPO / "tools/worldmodel"))
import evaluate_pass11 as p11  # noqa: E402
import evaluate_pass12 as p12  # noqa: E402
import wm_data  # noqa: E402

TPD, OFFSET, EVERY = 1440, 480, 60
STAGES = ("child", "adult", "elder")
STAGE_INDEX = {0: "adult", 1: "child", 2: "elder"}  # agents.npy life_stage
SEASONS = ("spring", "summer", "autumn", "winter")
SEVERITY = {0: "mild", 1: "normal", 2: "harsh"}
WOOD_RADIUS, SETTLED_RADIUS, DEADWOOD = 18, 9, 3
EXPECTED = {(39, 106, 3): (8, 14), (66, 106, 1): (12, 26), (77, 78, 3): (11, 28), (24, 106, 1): (9, 25)}
EV = REPO / ".harness/evidence/revitalize/probe"
WINDOW_DAYS = 6 + 13 / 24  # [D 18:00, D+7 07:00)


def display_day(tick: int) -> int:
    return (tick + OFFSET) // TPD + 1


def clock_tick(day: int, minute: int) -> int:
    return (day - 1) * TPD + minute - OFFSET


def row_of(day: int, minute: int) -> int:
    return clock_tick(day, minute) // EVERY


def season_of(tick: int) -> str:
    return SEASONS[((display_day(tick) - 1) // 7) % 4]


def deaths_of(round_: p12.Round, seed: int) -> list[dict]:
    scene = round_.summary["scene"]
    with (round_.path / f"{scene}_s{seed}_deaths.jsonl").open() as stream:
        return [json.loads(line) for line in stream]


def ratio(a, b):
    return round(a / b, 4) if b else None


# ------------------------------------------------------------------ probe side ------------------------------------
def settlement_parent(run: dict) -> dict[int, int | None]:
    return {int(s["settlement_id"]): s["parent"] for s in run["pass11"]["settlements"]}


def lost_between(seed: p12.Seed, t0: int, t1: int, key: str = "winter_wood_lost_total") -> int | None:
    """World cumulative `key` at the first row with tick >= t1 minus the last row with tick <= t0."""
    rows = sorted([*seed.periodic, *seed.clocks.values(), *seed.days.values()], key=lambda r: r["tick"])
    before = [r for r in rows if r["tick"] <= t0]
    after = [r for r in rows if r["tick"] >= t1]
    if not before or not after:
        return None
    return int(after[0][key]) - int(before[-1][key])


def child_share(rounds: list[p12.Round]) -> dict:
    """Exposure deaths in the seed-winter windows by stage x settlement type / severity, and the stage composition
    of the world at each window's start (probe 18:00 clock row: children, adults, elders)."""
    by_type, by_sev = defaultdict(Counter), defaultdict(Counter)
    pop_all, pop_with, pop_sev = Counter(), Counter(), defaultdict(Counter)
    expected_child = 0.0
    members_type = Counter()
    births, births_sev = 0, Counter()
    sum_births_run, n_windows = 0, 0
    born_inside, young_adults = Counter(), 0
    child_ages = []
    for r in rounds:
        seeds = {s.seed: s for s in r.seeds}
        for run in p12.runs(r):
            parent = settlement_parent(run)
            sum_births_run += int(seeds[run["seed"]].days[120]["births"])
            n_windows += len(run["seed_winters"])
            deaths = [d for d in deaths_of(r, run["seed"]) if d["cause"] == "exposure"]
            sw = {w["first_day"]: w for w in run["pass12"]["settlement_winters"]}
            for w in run["seed_winters"]:
                sev = SEVERITY.get(w["severity"], str(w["severity"]))
                inside = [d for d in deaths if w["start_tick"] <= d["tick"] < w["end_tick"]]
                row = seeds[run["seed"]].clocks[(w["first_day"], 1080)]
                comp = {"child": int(row["children"]), "adult": int(row["adults"]), "elder": int(row["elders"])}
                for s in STAGES:
                    pop_all[s] += comp[s]
                    pop_sev[sev][s] += comp[s]
                    if inside:
                        pop_with[s] += comp[s]
                b = lost_between(seeds[run["seed"]], w["start_tick"], w["end_tick"], "births") or 0
                births += b
                births_sev[sev] += b
                for x in inside:
                    since = (x["tick"] - w["start_tick"]) / TPD
                    born_inside[x["stage"]] += int(x["age_days_at_death"] < since)
                    young_adults += int(x["stage"] == "adult" and x["age_days_at_death"] < 7)
                    if x["stage"] == "child":
                        child_ages.append(float(x["age_days_at_death"]))
                tot = sum(comp.values())
                if inside and tot:
                    expected_child += len(inside) * comp["child"] / tot
                for d in inside:
                    sid = d["settlement_at_death"]
                    kind = "none" if sid is None else ("camp" if parent.get(int(sid)) is None else "daughter")
                    by_type[kind][d["stage"]] += 1
                    by_sev[sev][d["stage"]] += 1
                for s in sw.get(w["first_day"], {"settlements": []})["settlements"]:
                    members_type["camp" if parent.get(int(s["settlement"])) is None else "daughter"] += int(s["members_at_winter_start"])
    total = Counter()
    for c in by_type.values():
        total.update(c)
    n = sum(total.values())

    def block(c: Counter, pop: Counter | None = None) -> dict:
        k = sum(c.values())
        out = {"deaths": {s: c[s] for s in STAGES}, "total": k, "share": {s: ratio(c[s], k) for s in STAGES}}
        if pop is not None:
            out["population_at_starts"] = {s: pop[s] for s in STAGES}
            out["population_share"] = {s: ratio(pop[s], sum(pop.values())) for s in STAGES}
            out["deaths_per_100_at_start"] = {s: (round(100 * c[s] / pop[s], 2) if pop[s] else None) for s in STAGES}
        return out

    cf = Counter({k: total[k] - born_inside[k] for k in STAGES})
    cn = sum(cf.values())
    return {"counterfactual_without_deaths_born_inside": {"deaths": dict(cf), "share": {k: ratio(cf[k], cn) for k in STAGES}},
            "births_per_7_days_whole_run": round(sum_births_run * 7 / 120, 1), "windows": n_windows,
            "births_in_windows": births, "births_in_windows_by_severity": dict(births_sev),
            "deaths_born_inside_window": {k: born_inside[k] for k in STAGES},
            "adult_deaths_aged_under_7_days": young_adults,
            "child_age_at_death_days": {"median": round(float(np.median(child_ages)), 2), "max": round(max(child_ages), 2)} if child_ages else None,
            "child_deaths_per_100_children_at_risk": round(100 * total["child"] / (pop_all["child"] + births), 2) if pop_all["child"] + births else None,
            "total": block(total, pop_all), "winters_with_death_population": {s: pop_with[s] for s in STAGES},
            "deaths_per_100_in_winters_with_death": {s: round(100 * total[s] / pop_with[s], 2) if pop_with[s] else None for s in STAGES},
            "expected_child_share_from_composition": round(expected_child / n, 4) if n else None,
            "by_settlement_type": {k: {**block(v), "members_at_winter_starts": members_type.get(k),
                                       "deaths_per_100_members": round(100 * sum(v.values()) / members_type[k], 2) if members_type.get(k) else None}
                                   for k, v in sorted(by_type.items())},
            "by_severity": {k: block(by_sev[k], pop_sev[k]) for k in ("mild", "normal", "harsh") if k in pop_sev}}


# ------------------------------------------------------------------ recording side --------------------------------
class Rec:
    def __init__(self, path: Path):
        self.ep = wm_data.load_episode(path)
        self.ids = [int(x) for x in np.load(path / "settlement_ids.npy")]
        self.agent_ids = [int(x) for x in self.ep.arrays["agent_ids.npy"]]
        col = lambda n: self.ep.column("settlements.npy", n)  # noqa: E731
        self.members, self.wood, self.target, self.fire = col("members"), col("wood_stock"), col("wood_target"), col("fire_lit")
        self.fx, self.fy, self.founded = col("formation_x"), col("formation_y"), col("founded_tick")
        a = self.ep.arrays["agents.npy"]
        fi = lambda n: self.ep.field_index("agents.npy", n)  # noqa: E731
        self.a_slot, self.a_stage, self.a_cold, self.a_hard = a[..., fi("settlement")], a[..., fi("life_stage")], a[..., fi("cold")], a[..., fi("hardiness")]
        self.burn = self.ep.column("global.npy", "night_burn_per_hour")
        self.alive = self.ep.arrays["agent_alive.npy"]
        keys = np.load(path / "resource_keys.npy")
        self.wood_keys = keys[keys[:, 0] == DEADWOOD][:, 1:]
        res = self.ep.arrays["resources.npy"]
        self.wood_amount = np.nan_to_num(res[:, keys[:, 0] == DEADWOOD, self.ep.field_index("resources.npy", "amount")], nan=0.0)

    def near(self, slot: int, row: int, radius: int) -> tuple[int, float]:
        hx, hy = self.fx[row, slot], self.fy[row, slot]
        mask = np.maximum(np.abs(self.wood_keys[:, 0] - hx), np.abs(self.wood_keys[:, 1] - hy)) <= radius
        return int(mask.sum()), float(self.wood_amount[row, mask].sum())

    def fallen_near(self, slot: int, r0: int, r1: int, radius: int) -> float:
        """Deadwood that arrived on the spots within `radius` of the hearth, rows r0..r1 (sum of positive steps)."""
        hx, hy = self.fx[r0, slot], self.fy[r0, slot]
        mask = np.maximum(np.abs(self.wood_keys[:, 0] - hx), np.abs(self.wood_keys[:, 1] - hy)) <= radius
        steps = np.diff(self.wood_amount[r0:r1 + 1, mask], axis=0)
        return float(steps[steps > 0].sum())

    def births(self, slot: int, r0: int, r1: int) -> list[int]:
        """Agent columns first alive inside rows (r0, r1) whose first settlement slot is `slot`."""
        first = np.argmax(self.alive > 0, axis=0)
        ever = self.alive.any(axis=0)
        return [i for i in np.flatnonzero(ever & (first > r0) & (first < r1) & (first > 0)) if self.a_slot[first[i], i] == slot]

    def members_near(self, slot: int, row: int, radius: int) -> int:
        hx, hy = self.fx[row, slot], self.fy[row, slot]
        out = 0
        for s in range(len(self.ids)):
            if np.isfinite(self.members[row, s]) and max(abs(self.fx[row, s] - hx), abs(self.fy[row, s] - hy)) <= radius:
                out += int(self.members[row, s])
        return out

    def stage_counts(self, slot: int, row: int) -> Counter:
        here = self.a_slot[row] == slot
        return Counter(STAGE_INDEX[int(v)] for v in self.a_stage[row, here])

    def cold_hours(self, slot: int, r0: int, r1: int) -> tuple[Counter, Counter]:
        """Per stage: agent-rows in the slot with the Cold flag, and agent burn-window rows in the slot, rows r0..r1-1."""
        cold, present = Counter(), Counter()
        for r in range(r0, r1):
            minute = (r * EVERY + OFFSET) % TPD
            burn_row = minute >= 1140 or minute <= 420
            here = self.a_slot[r] == slot
            for st, c in zip(self.a_stage[r, here], self.a_cold[r, here]):
                name = STAGE_INDEX[int(st)]
                cold[name] += int(c == 1)
                present[name] += int(burn_row)
        return cold, present


def nights(rec: Rec, slot: int, d: int) -> dict:
    two_am = sum(1 for e in range(d, d + 7) if rec.fire[row_of(e + 1, 120), slot] == 0)
    unlit_nights, unlit_hours = 0, 0
    for e in range(d, d + 7):
        rows = [row_of(e, m) for m in range(1140, 1440, 60)] + [row_of(e + 1, m) for m in range(0, 421, 60)]
        k = int(sum(1 for r in rows if rec.fire[r, slot] == 0))
        unlit_hours += k
        unlit_nights += int(k > 0)
    return {"dark_nights_0200": two_am, "nights_any_unlit_hour": unlit_nights, "unlit_burn_hours": unlit_hours}


def case(rec: Rec, round_: p12.Round, run: dict, seed_row: p12.Seed, w: dict, s: dict) -> dict:
    sid, d = int(s["settlement"]), int(w["first_day"])
    slot = rec.ids.index(sid)
    r0, r1 = w["start_tick"] // EVERY, -(-w["end_tick"] // EVERY)
    parent = settlement_parent(run)[sid]
    founded = int(rec.founded[r0, slot])
    sw = next(x for x in run["seed_winters"] if x["first_day"] == d)
    deaths = [x for x in deaths_of(round_, run["seed"]) if x["cause"] == "exposure"
              and w["start_tick"] <= x["tick"] < w["end_tick"] and x["settlement_at_death"] == sid]
    unlit_at_death = sum(1 for x in deaths if rec.fire[min(-(-x["tick"] // EVERY), rec.fire.shape[0] - 1), slot] == 0)
    in_rec = sum(1 for x in deaths if x["agent_id"] in rec.agent_ids)
    spots18, wood18 = rec.near(slot, r0, WOOD_RADIUS)
    spots9, wood9 = rec.near(slot, r0, SETTLED_RADIUS)
    spots18_end, wood18_end = rec.near(slot, r1, WOOD_RADIUS)
    evenings = [int(rec.wood[row_of(e, 1080), slot]) for e in range(d, d + 7)]
    window = rec.wood[r0:r1, slot]
    zero = np.flatnonzero(window <= 0)
    members = rec.stage_counts(slot, r0)
    cold, present = rec.cold_hours(slot, r0, r1)
    hard = {}
    here = rec.a_slot[r0] == slot
    for code, st in STAGE_INDEX.items():
        m = here & (rec.a_stage[r0] == code)
        hard[st] = round(float(np.nanmean(rec.a_hard[r0, m])), 1) if m.any() else None
    dstage = Counter(x["stage"] for x in deaths)
    born = rec.births(slot, r0, r1)
    born_ids = {rec.agent_ids[i] for i in born}
    burn_rows = [r for r in range(r0, r1) if ((r * EVERY + OFFSET) % TPD) >= 1140 or ((r * EVERY + OFFSET) % TPD) <= 420]
    pre_spots, pre_wood = rec.near(slot, row_of(d, 0) - 1, WOOD_RADIUS)
    steps = np.diff(rec.wood[r0:r1, slot])
    regions = (run["seed_info"].get("wild_regions") or {}).get("centres", [])
    hx, hy = int(rec.fx[r0, slot]), int(rec.fy[r0, slot])
    region = next((i + 1 for i, c in enumerate(regions)
                   if max(abs(c["centre"][0] - hx), abs(c["centre"][1] - hy)) <= SETTLED_RADIUS), None)
    return {
        "seed": run["seed"], "first_day": d, "settlement": sid, "type": "camp" if parent is None else "daughter",
        "parent": parent, "founded_tick": founded, "founded_display_day": display_day(founded),
        "founding_season": season_of(founded), "age_days_at_start": round((w["start_tick"] - founded) / TPD, 2),
        "hearth": [hx, hy], "settled_region": region, "severity": SEVERITY.get(sw["severity"]),
        "world_dark_nights": sw["dark_nights"], **nights(rec, slot, d),
        "deadwood_r18": {"spots": spots18, "wood": wood18, "per_spot": ratio(wood18, spots18),
                         "members_of_settlements_within_18": rec.members_near(slot, r0, WOOD_RADIUS),
                         "wood_at_window_end": wood18_end},
        "deadwood_r9": {"spots": spots9, "wood": wood9, "per_spot": ratio(wood9, spots9)},
        "deadwood_r18_before_snow": {"row": "day D-1 23:00", "wood": pre_wood, "per_spot": ratio(pre_wood, pre_spots)},
        "deadwood_fallen_r18_in_window": rec.fallen_near(slot, r0, r1, WOOD_RADIUS),
        "wood_into_stock_in_window": float(steps[steps > 0].sum()),
        "burn_need_in_window": float(np.nansum(rec.burn[burn_rows])),
        "adults_18h_days_D-7_to_D": [None if not np.isfinite(rec.members[row_of(e, 1080), slot]) else
                                     int(((rec.a_slot[row_of(e, 1080)] == slot) & (rec.a_stage[row_of(e, 1080)] == 0)).sum()) for e in range(d - 7, d + 1)],
        "members_18h_days_D-7_to_D": [None if not np.isfinite(rec.members[row_of(e, 1080), slot]) else int(rec.members[row_of(e, 1080), slot]) for e in range(d - 7, d + 1)],
        "adult_victim_ages_days": sorted(round(float(x["age_days_at_death"]), 1) for x in deaths if x["stage"] == "adult"),
        "births_in_window": len(born), "exposure_deaths_born_in_window": sum(1 for x in deaths if x["agent_id"] in born_ids),
        "wood_stock_start": int(rec.wood[r0, slot]), "wood_target_start": float(rec.target[r0, slot]),
        "wood_stock_18h": evenings, "wood_stock_min": int(np.nanmin(window)),
        "wood_stock_first_zero_day": display_day((r0 + int(zero[0])) * EVERY) if zero.size else None,
        "winter_wood_lost_world_window": lost_between(seed_row, w["start_tick"], w["end_tick"]),
        "winter_wood_lost_world_season": lost_between(seed_row, clock_tick(d, 0), clock_tick(d + 7, 0)),
        "winter_wood_lost_world_run": run["pass12"]["winter_wood_lost"],
        "winter_wood_lost_settled_run": run["pass12"]["winter_wood_lost_settled"],
        "members_at_start": int(s["members_at_winter_start"]), "members_by_stage": {k: members[k] for k in STAGES},
        "hardiness_mean_by_stage": hard,
        "exposure_deaths": len(deaths), "deaths_by_stage": {k: dstage[k] for k in STAGES},
        "deaths_with_own_fire_unlit": unlit_at_death, "death_agents_in_recording": in_rec,
        "cold_hours_by_stage": {k: cold[k] for k in STAGES},
        "cold_share_of_burn_hours": {k: ratio(cold[k], present[k]) for k in STAGES},
        "ratio": ratio(len(deaths), int(s["members_at_winter_start"])),
        "ratio_without_deaths_born_in_window": ratio(len(deaths) - sum(1 for x in deaths if x["agent_id"] in born_ids), int(s["members_at_winter_start"])),
    }


def recorded_settlement_winters(rec: Rec, round_: p12.Round, run: dict) -> list[dict]:
    """Every settlement-winter of a recorded seed: stage members, cold hours and deaths (supporting the cases)."""
    out = []
    deaths = [x for x in deaths_of(round_, run["seed"]) if x["cause"] == "exposure"]
    for w in run["pass12"]["settlement_winters"]:
        r0, r1 = w["start_tick"] // EVERY, -(-w["end_tick"] // EVERY)
        for s in w["settlements"]:
            slot = rec.ids.index(int(s["settlement"]))
            cold, present = rec.cold_hours(slot, r0, r1)
            dd = Counter(x["stage"] for x in deaths if w["start_tick"] <= x["tick"] < w["end_tick"]
                         and x["settlement_at_death"] == int(s["settlement"]))
            out.append({"seed": run["seed"], "first_day": w["first_day"], "settlement": s["settlement"],
                        "wood_stock_start": int(rec.wood[r0, slot]), "members_at_start": int(s["members_at_winter_start"]),
                        "exposure_deaths": int(s["exposure_deaths"]),
                        "members": {k: rec.stage_counts(slot, r0)[k] for k in STAGES}, "cold_hours": dict(cold),
                        "burn_hours": dict(present), "deaths": {k: dd[k] for k in STAGES}, **nights(rec, slot, w["first_day"])})
    return out


# ------------------------------------------------------------------ report ----------------------------------------
def fmt_stage(c: dict) -> str:
    return "/".join(str(c.get(k, 0)) for k in STAGES)


def rel(path: str) -> str:
    return path[len(str(REPO)) + 1:] if path.startswith(str(REPO) + "/") else path


def sh(d: dict, fmt: str = "{:.3f}") -> str:
    return "/".join("-" if d.get(k) is None else fmt.format(d[k]) for k in STAGES)


def markdown(res: dict) -> str:
    rp, cs, t = res["repro"], res["child_share"], res["child_share"]["total"]
    L = ["# J5 diagnosis (Pass 13 Part 0 item 5)", "",
         f"Inputs: held-out `{rel(res['inputs']['heldout'])}` + tuning `{rel(res['inputs']['tuning'])}` (the pair Pass 12's final J5 "
         f"pooled; the tuning run equals round 3 on every shared value); recordings `{res['inputs']['recordings']}` (game_v18, Pass 12 "
         "final code, `--agents`, a row every 60 ticks). Tool: `tools/revitalize/j5_diag_p13.py`. Stage order everywhere: child/adult/elder.", "",
         "## Reproduction (evaluate_pass12 functions imported)", "",
         f"- {rp['settlement_winters']} settlement-winters, {rp['n_over']} over 0.35, max {rp['max']:.3f}; the four ratios equal "
         f"8/14, 12/26, 11/28, 9/25: **{rp['ratios_match']}**.",
         f"- `stage_pool`: deaths {fmt_stage(rp['stage_pool']['deaths'])} = {rp['stage_pool']['total']}, child share "
         f"**{rp['stage_pool']['share']['child']:.3f}** (= Pass 12's final 0.717; METRICS Part 4's 0.725 came from the Part 4 held-out run, before the p12-7 rerun changed 3 seeds). "
         f"Recount from the death census (`stage`, cause exposure, tick in the window): {fmt_stage(t['deaths'])}, equal: {rp['census_matches_pool']}.",
         f"- Recordings vs probe: members at every case's start equal; exposure-death agents present in the recording "
         f"{rp['death_agents_in_recordings']}.", "",
         "## The four settlement-winters over 0.35 (all harsh)", "",
         "Dark nights: world = probe rule (02:00 sample, world flag = every fire lit); own = same rule on this settlement's "
         "fire; any = nights with >= 1 unlit burn hour; h = unlit burn hours of 91 (13 per night, rows 19:00..07:00). "
         "Deadwood near = deadwood spots within Chebyshev 18 of the hearth (`wood_radius`, a gatherer's search radius); "
         "r9 = `SETTLED_REGION_RADIUS`. Snow cap 1 falls at the first winter pulse, so per-spot wood at the start is <= 1.", "",
         "| seed | settl. | founded (season), age d | dark world/own/any/h | deadwood /spot: before snow (D-1 23:00), start r18, start r9 | "
         "stock start (target) | stock 18:00 D..D+6 | burn need / fallen r18 / into stock (window) |",
         "|---|---|---|---|---|---|---|---|"]
    for c in res["cases"]:
        par = "camp" if c["parent"] is None else f"daughter of {c['parent']}"
        L.append(f"| {c['seed']} (d{c['first_day']}) | {c['settlement']} {par}{'' if c['settled_region'] is None else ', region ' + str(c['settled_region'])} | "
                 f"d{c['founded_display_day']} ({c['founding_season']}), {c['age_days_at_start']} | "
                 f"{c['world_dark_nights']}/{c['dark_nights_0200']}/{c['nights_any_unlit_hour']}/{c['unlit_burn_hours']} | "
                 f"{c['deadwood_r18_before_snow']['per_spot']}, {c['deadwood_r18']['per_spot']} ({c['deadwood_r18']['spots']} spots), "
                 f"{'no spots' if c['deadwood_r9']['per_spot'] is None else c['deadwood_r9']['per_spot']} | {c['wood_stock_start']} ({c['wood_target_start']:.0f}) | {', '.join(map(str, c['wood_stock_18h']))} | "
                 f"{c['burn_need_in_window']:.0f} / {c['deadwood_fallen_r18_in_window']:.0f} / {c['wood_into_stock_in_window']:.0f} |")
    L += ["", "| seed | members c/a/e at start | adults at 18:00 D-7..D | births in window | deaths c/a/e | ratio | "
          "deaths born in window | ratio without them | own fire unlit at death | adult victims' age d | cold share of burn hours c/a/e |",
          "|---|---|---|---|---|---|---|---|---|---|---|"]
    for c in res["cases"]:
        L.append(f"| {c['seed']} | {fmt_stage(c['members_by_stage'])} | {', '.join('-' if v is None else str(v) for v in c['adults_18h_days_D-7_to_D'])} | "
                 f"{c['births_in_window']} | {fmt_stage(c['deaths_by_stage'])} | {c['exposure_deaths']}/{c['members_at_start']} = {c['ratio']:.3f} | "
                 f"{c['exposure_deaths_born_in_window']} | {c['ratio_without_deaths_born_in_window']:.3f} | "
                 f"{c['deaths_with_own_fire_unlit']}/{c['exposure_deaths']} | {', '.join(map(str, c['adult_victim_ages_days']))} | "
                 f"{sh(c['cold_share_of_burn_hours'], '{:.2f}')} |")
    L += ["", "Winter wood lost (`winter_wood_lost_total` row deltas, world level; settled = per-seed run total): "
          + "; ".join(f"seed {c['seed']}: window {c['winter_wood_lost_world_window']}, winter season {c['winter_wood_lost_world_season']}, "
                      f"run {c['winter_wood_lost_world_run']}, settled run {c['winter_wood_lost_settled_run']}" for c in res["cases"])
          + ". Schema 12 has no per-winter or per-settlement split of `winter_wood_lost_settled`.", "",
          "Same world-winter, every settlement (stock at start, members, exposure deaths, unlit burn hours):"]
    for c in res["cases"]:
        rows = [x for x in res["recorded_settlement_winters"] if x["seed"] == c["seed"] and x["first_day"] == c["first_day"]]
        L.append(f"- seed {c['seed']} d{c['first_day']}: " + "; ".join(
            f"{x['settlement']}: {x['wood_stock_start']}, {x['exposure_deaths']}/{x['members_at_start']}, {x['unlit_burn_hours']} h" for x in rows))
    L += ["", "## Child share (pooled held-out + tuning)", "",
          "| set | deaths c/a/e | share c/a/e | stage counts at winter starts c/a/e (share) | deaths per 100 at start c/a/e |",
          "|---|---|---|---|---|",
          f"| all seed-winters | {fmt_stage(t['deaths'])} | {sh(t['share'])} | {fmt_stage(t['population_at_starts'])} ({sh(t['population_share'])}) | "
          f"{sh(t['deaths_per_100_at_start'], '{:.2f}')} |"]
    for k, b in cs["by_severity"].items():
        L.append(f"| {k} winters | {fmt_stage(b['deaths'])} | {sh(b['share'])} | {fmt_stage(b['population_at_starts'])} "
                 f"({sh(b['population_share'])}) | {sh(b['deaths_per_100_at_start'], '{:.2f}')} |")
    for k, b in cs["by_settlement_type"].items():
        L.append(f"| deaths in {k}s | {fmt_stage(b['deaths'])} | {sh(b['share'])} | members at starts {b['members_at_winter_starts']} | "
                 f"{b['deaths_per_100_members']} per 100 members (all stages) |")
    n_seeds = sum(int(v) for v in res["seeds_per_round"])
    L += ["", f"- Seed-winters with a death only: counts {fmt_stage(cs['winters_with_death_population'])}, deaths per 100 "
          f"{sh(cs['deaths_per_100_in_winters_with_death'], '{:.2f}')}. Child share expected from composition alone: "
          f"{cs['expected_child_share_from_composition']:.3f}.",
          f"- The child stage lasts 4 days (`child_ticks` 5,760). Dead children's age: median {cs['child_age_at_death_days']['median']} d, "
          f"max {cs['child_age_at_death_days']['max']}; **{cs['deaths_born_inside_window']['child']} of {t['deaths']['child']} were born "
          f"inside the winter window** (adults: {cs['deaths_born_inside_window']['adult']}; adult victims under 7 days old: "
          f"{cs['adult_deaths_aged_under_7_days']}). Children at risk = at start + born in window = {t['population_at_starts']['child']} + "
          f"{cs['births_in_windows']}: {cs['child_deaths_per_100_children_at_risk']} deaths per 100 (adults {t['deaths_per_100_at_start']['adult']:.2f}).",
          f"- Births barely slow in winter: {cs['births_in_windows'] / cs['windows'] / WINDOW_DAYS:.2f} per seed-day inside the winter windows vs "
          f"{cs['births_per_7_days_whole_run'] / n_seeds / 7:.2f} over the whole run.",
          f"- Arithmetic counterfactual without the deaths born inside the window (not a simulation): shares "
          f"{sh(cs['counterfactual_without_deaths_born_inside']['share'])}.",
          ]
    rw = res["recorded_settlement_winters"]
    agg_cold, agg_burn, agg_d = Counter(), Counter(), Counter()
    for x in rw:
        agg_cold.update(x["cold_hours"]), agg_burn.update(x["burn_hours"]), agg_d.update(x["deaths"])
    L += [f"- All {len(rw)} settlement-winters of the four recorded seeds: cold share of burn hours "
          f"{'/'.join(f'{agg_cold[k] / agg_burn[k]:.3f}' if agg_burn[k] else '-' for k in STAGES)}, deaths {fmt_stage(agg_d)}, deaths "
          f"per cold hour {'/'.join(f'{agg_d[k] / agg_cold[k]:.4f}' if agg_cold[k] else '-' for k in STAGES)}. Damage per cold "
          "hour (game_v18) child 8, adult 5, elder 5 (x hardiness, x hunger up to 1.5); max hp 100 for every stage; healing "
          "0.05/tick (3/hour) after a 2-hour block and only with hunger and thirst below the safe ceiling (<= ~27 hp between two "
          "nights). One fully dark burn window (13 h) costs a child 104 hp, an adult 65.",
          "", "## Conclusion", "", res["conclusion"], ""]
    return "\n".join(L)


def conclusion(res: dict) -> str:
    """The 3-6 sentence conclusion; numbers are recomputed, the wording describes the default inputs' pattern."""
    cs, t = res["child_share"], res["child_share"]["total"]
    c = {x["seed"]: x for x in res["cases"]}
    burn = [x["burn_need_in_window"] for x in res["cases"]]
    fall = [x["deadwood_fallen_r18_in_window"] for x in res["cases"]]
    n = sum(x["exposure_deaths"] for x in res["cases"])
    tgt = res["cases"][0]["wood_target_start"]
    cover = [f / b for f, b in zip(fall, burn) if b]
    pre = [x["deadwood_r18_before_snow"]["per_spot"] for x in res["cases"]]
    start = [x["deadwood_r18"]["per_spot"] or 0 for x in res["cases"]]
    unlit = sum(x["deaths_with_own_fire_unlit"] for x in res["cases"])
    return (
        f"The child share is a newborn effect, not a composition or exposure-time effect: {cs['deaths_born_inside_window']['child']} of "
        f"{t['deaths']['child']} dead children were born inside the winter window (the child stage lasts 4 days), births barely slow in "
        "winter, and within a settlement children are cold for about the same share of burn hours as adults; but at 100 hp and 8 hp per "
        "cold hour one fully dark 13-hour night (104 hp) kills a newborn, while an adult loses 65 hp, heals at most ~27 by the next "
        f"evening and dies only in a second consecutive dark night. All {unlit} of {n} deaths in the four cases came with the settlement's own fire out, in harsh "
        f"winters whose burn need inside the window ({min(burn):.0f}-{max(burn):.0f}) is {min(burn) / tgt:.1f}-{max(burn) / tgt:.1f}x the firewood target "
        f"({tgt:.0f}) while the deadwood falling within 18 tiles ({min(fall):.0f}-{max(fall):.0f}) covers "
        f"{100 * min(cover):.0f}-{100 * max(cover):.0f} % of it; wood per spot at the winter start is <= {max(start):.2f} in every case only "
        f"because the snow cap (1) buries the {min(pre):.1f}-{max(pre):.1f} per spot lying there the evening before, so wood per spot does "
        "not separate the cases. "
        f"Two mechanisms make the four cases: daughters 39 and 77 entered with stock near the target ({c[39]['wood_stock_start']}, "
        f"{c[77]['wood_stock_start']}) and about as many dark hours as their sibling settlements, but small or young rosters received "
        f"{c[39]['births_in_window']} and {c[77]['births_in_window']} births in the window (ratios without the deaths born inside it: "
        f"{c[39]['ratio_without_deaths_born_in_window']:.3f}, {c[77]['ratio_without_deaths_born_in_window']:.3f}); camps 66 and 24 "
        f"entered with stock {c[66]['wood_stock_start']} and {c[24]['wood_stock_start']} because they were short of adults during the "
        "autumn build-up (66: an autumn fission's group took 12 adults and left 2 adults among 23-24 members; 24: 8 adults among "
        f"24-28 members) and went dark {c[66]['unlit_burn_hours']} and {c[24]['unlit_burn_hours']} of 91 burn hours, and 24's deaths "
        "are adults and an elder. "
        "A single v19 flag aimed at newborns (no births while a settlement's wood cannot cover the winter, or child damage <= 7 so one "
        "dark night is not lethal) plausibly fixes the child share and cases 39 and 77, but not 24 (0.36 with no newborn deaths) and 66 "
        "only to 0.27, and the arithmetic counterfactual without the newborn deaths moves the adult share to "
        f"{cs['counterfactual_without_deaths_born_inside']['share']['adult']:.2f}, above J5's 0.50 bound; wood placement cannot close a "
        "burn deficit that also darkens settlements at the target, and the camp cases point at adult labour in autumn (a fission that "
        "leaves adults in the parent), a second mechanism.")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--heldout", type=Path, default=EV / "p12-7_final_heldout_v18_120day")
    ap.add_argument("--tuning", type=Path, default=EV / "p12-7_timing_v18_120day")
    ap.add_argument("--recordings", type=Path, default=Path("/Users/rexxa/work/worldsim-wm-data/p13/j5diag"))
    ap.add_argument("--out", type=Path, default=REPO / "docs/revitalize/probe/p13-0_j5")
    args = ap.parse_args(argv)
    heldout, tuning = p12.load_round(args.heldout, 120), p12.load_round(args.tuning, 120)
    rounds = [heldout, tuning]
    sw = p12.settlement_winters(rounds)
    over = [w for w in sw if w["ratio"] > .35]
    pool = p11.stage_pool(rounds)
    got = {(w["seed"], w["first_day"], w["settlement"]): (w["deaths"], w["members"]) for w in over}
    cs = child_share(rounds)
    repro = {"settlement_winters": len(sw), "n_over": len(over), "max": max(w["ratio"] for w in sw),
             "over": over, "ratios_match": got == EXPECTED, "stage_pool": pool,
             "census_matches_pool": cs["total"]["deaths"] == {k: pool["deaths"].get(k, 0) for k in STAGES}}
    cases, recorded = [], []
    for (seed, day, sid) in sorted(got, key=lambda k: -got[k][0] / got[k][1]):
        r = next(x for x in rounds if seed in {s.seed for s in x.seeds})
        run = next(x for x in p12.runs(r) if x["seed"] == seed)
        rec = Rec(args.recordings / f"game_v18_s{seed}_none")
        w = next(x for x in run["pass12"]["settlement_winters"] if x["first_day"] == day)
        s = next(x for x in w["settlements"] if x["settlement"] == sid)
        c = case(rec, r, run, next(x for x in r.seeds if x.seed == seed), w, s)
        if int(rec.members[w["start_tick"] // EVERY, rec.ids.index(sid)]) != c["members_at_start"]:
            raise SystemExit(f"seed {seed}: recording members differ from the probe")
        cases.append(c)
        recorded += recorded_settlement_winters(rec, r, run)
    repro["death_agents_in_recordings"] = f"{sum(c['death_agents_in_recording'] for c in cases)}/{sum(c['exposure_deaths'] for c in cases)}"
    res = {"schema_version": 1, "inputs": {k: str(v) for k, v in vars(args).items() if k != "out"},
           "repro": repro, "cases": cases, "child_share": cs, "recorded_settlement_winters": recorded,
           "seeds_per_round": [len(heldout.seeds), len(tuning.seeds)]}
    res["conclusion"] = conclusion(res)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "j5_diag.json").write_text(json.dumps(res, indent=2, sort_keys=True, default=str) + "\n")
    (args.out / "j5_diag.md").write_text(markdown(res))
    print(f"ratios match: {repro['ratios_match']}; child share {pool['share']['child']:.4f}; census match {repro['census_matches_pool']}")
    return 0 if repro["ratios_match"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
