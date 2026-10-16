#!/usr/bin/env python3
"""Pass 12 Part 0 (PASS12_GOAL.md 5.3): why daughters starve or freeze. numpy only.

Inputs (read-only): Pass 11's final 120-day `game_v17` probe runs (tuning 0..32 and held-out 32..96; summary.json
with `runs[].pass11` and `runs[].seed_info`, and the per-seed `*_deaths.jsonl` census), and the same seeds recorded
with Pass 11's final recorder (`wm_final`, schema 5, hourly `settlements.npy`: members, food, wood, fire per slot).
A daughter is matched to its region by `fission_log` site = `wild_regions` centre; a daughter founded at a timeout
leader tile (`settled_at_leader`) has no region.

usage: daughter_diag.py --probe DIR [DIR ...] --recordings DIR --out DIR
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

import numpy as np

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools/worldmodel"))
import wm_data  # noqa: E402

TPD, OFFSET, EVERY = 1440, 480, 60
WINTER_FIRST_DAYS = (22, 50, 78, 106)
BOUND = 0.35
CAUSES = ("starvation", "dehydration", "exposure")
SEASONS = ("spring", "summer", "autumn", "winter")


def display_day(tick: int) -> int:
    return (tick + OFFSET) // TPD + 1


def season_of(tick: int) -> str:
    return SEASONS[((display_day(tick) - 1) // 7) % 4]


def row_at(day: int, minute: int) -> int:
    return ((day - 1) * TPD + minute - OFFSET) // EVERY


class Rec:
    """One recorded episode: per-slot hourly members, food, wood, fire and the slot of each settlement id."""

    def __init__(self, path: Path):
        ep = wm_data.load_episode(path, arrays=["global.npy", "settlements.npy"])
        self.ids = [int(x) for x in np.load(path / "settlement_ids.npy")]
        col = lambda n: np.nan_to_num(ep.column("settlements.npy", n), nan=np.nan)
        self.members, self.food, self.wood, self.fire = col("members"), col("food_stock"), col("wood_stock"), col("fire_lit")
        self.deadwood = ep.column("global.npy", "deadwood_total")

    def slot(self, sid: int) -> int | None:
        return self.ids.index(sid) if sid in self.ids else None

    def at(self, arr: np.ndarray, sid: int, row: int) -> float | None:
        s = self.slot(sid)
        if s is None or row < 0 or row >= arr.shape[0] or not np.isfinite(arr[row, s]):
            return None
        return float(arr[row, s])

    def history(self, sid: int, tick: int, hours: int = 72) -> dict:
        s = self.slot(sid)
        end = min(tick // EVERY, self.members.shape[0] - 1)
        lo = max(0, end - hours)
        if s is None:
            return {}
        sl = slice(lo, end + 1)
        fin = lambda a: a[sl, s][np.isfinite(a[sl, s])]
        f, w, fire, m = fin(self.food), fin(self.wood), fin(self.fire), fin(self.members)
        nights = []
        d = display_day(tick)
        for e in (d - 3, d - 2, d - 1):  # the three nights before the day of the death (02:00 of e + 1)
            r = row_at(e + 1, 120)
            if 0 <= r <= end and np.isfinite(self.fire[r, s]):
                nights.append(int(self.fire[r, s] == 0))
        return {"food_min": float(f.min()) if f.size else None, "food_mean": float(f.mean()) if f.size else None,
                "wood_min": float(w.min()) if w.size else None, "wood_mean": float(w.mean()) if w.size else None,
                "fire_lit_share": float(fire.mean()) if fire.size else None, "dark_nights_of_3": sum(nights),
                "members_min": float(m.min()) if m.size else None}


def load_runs(dirs: list[Path]) -> list[tuple[str, dict, list[dict]]]:
    out = []
    for d in dirs:
        s = json.loads((d / "summary.json").read_text())
        for run in s["runs"]:
            deaths = [json.loads(line) for line in (d / f"{s['scene']}_s{run['seed']}_deaths.jsonl").read_text().splitlines()]
            out.append((d.name, run, deaths))
    return out


def analyse(runs, rec_root: Path) -> dict:
    deaths_rows, daughters_rows, winters_rows, settlement_winters = [], [], [], []
    for label, run, deaths in runs:
        seed = run["seed"]
        p = run["pass11"]
        info = run["seed_info"]
        regions = info["wild_regions"]["centres"]
        variety = info["world_variety"]
        rec = Rec(rec_root / f"game_v17_s{seed}_none")
        log = {r["daughter"]: r for r in p["fission_log"] if r["daughter"] is not None}
        settlements = {s["settlement_id"]: s for s in p["settlements"]}
        daughters = {sid for sid, s in settlements.items() if s["parent"] is not None}
        for sid in sorted(daughters):
            s, f = settlements[sid], log.get(sid)
            region = next((i for i, c in enumerate(regions) if f and list(c["centre"]) == list(f["site"])), None)
            daughters_rows.append({
                "set": label, "seed": seed, "settlement": sid, "founded_tick": s["founded_tick"],
                "founded_day": display_day(s["founded_tick"]), "season": season_of(s["founded_tick"]),
                "at_leader": bool(f and f["settled_at_leader"]), "region": region,
                "bushes_requested": variety["region_bushes"], "bushes_placed": regions[region]["bushes"] if region is not None else None,
                "deadwood_requested": variety["region_deadwood"], "deadwood_placed": regions[region]["deadwood"] if region is not None else None,
                "water_distance": regions[region]["water_distance"] if region is not None else None,
                "founders": s["founders"], "alive_28": s["alive_28_days_after_founding"], "dissolved_tick": s["dissolved_tick"]})
        drow = {d["settlement"]: d for d in daughters_rows if d["seed"] == seed and d["set"] == label}
        for d in deaths:
            if d["cause"] not in CAUSES or d["settlement_at_death"] not in daughters:
                continue
            sid, tick = d["settlement_at_death"], d["tick"]
            info_d = drow[sid]
            deaths_rows.append({
                "set": label, "seed": seed, "agent": d["agent_id"], "cause": d["cause"], "stage": d["stage"],
                "tick": tick, "day": display_day(tick), "season": season_of(tick), "settlement": sid,
                "founded_day": info_d["founded_day"], "founded_season": info_d["season"], "at_leader": info_d["at_leader"],
                "region": info_d["region"], "bushes": [info_d["bushes_placed"], info_d["bushes_requested"]],
                "deadwood": [info_d["deadwood_placed"], info_d["deadwood_requested"]], "water_distance": info_d["water_distance"],
                "water_distance_at_death": d.get("water_distance_at_death"), "founders": info_d["founders"],
                "members_at_death": rec.at(rec.members, sid, tick // EVERY), "food_at_death": d.get("settlement_food_stock_at_death"),
                "history_72h": rec.history(sid, tick)})
        # settlement-winters: exposure deaths per settlement in each winter window vs members at day D 18:00
        exposure = [(d["tick"], d["settlement_at_death"]) for d in deaths if d["cause"] == "exposure"]
        spots = info["deadwood_spots_placed"] + info["wild_regions"]["region_deadwood_placed"]
        for w in run["seed_winters"]:
            day = w["first_day"]
            r0 = row_at(day, 1080)
            by_sid = Counter(sid for t, sid in exposure if w["start_tick"] <= t < w["end_tick"])
            per = {}
            for sid in sorted(set(by_sid) | {i for i in rec.ids if rec.at(rec.members, i, r0)}):
                m0 = rec.at(rec.members, sid, r0)
                founded = settlements.get(sid, {}).get("founded_tick")
                if (m0 is None or m0 == 0) and founded is not None and w["start_tick"] <= founded < w["end_tick"]:
                    m0 = settlements[sid]["founders"]
                dark = 0
                for e in range(day, day + 7):
                    v = rec.at(rec.fire, sid, row_at(e + 1, 120))
                    dark += int(v == 0) if v is not None else 0
                per[sid] = {"members_at_start": m0, "exposure_deaths": by_sid.get(sid, 0), "dark_nights": dark,
                            "wood_at_start": rec.at(rec.wood, sid, r0), "daughter": sid in daughters,
                            "ratio": (by_sid.get(sid, 0) / m0) if m0 else None}
                settlement_winters.append({"set": label, "seed": seed, "first_day": day, "settlement": sid, **per[sid],
                                           "severity": w["severity"]})
            ratio = w["exposure_deaths"] / w["members_at_start"] if w["members_at_start"] else 0.0
            if ratio > BOUND or any((v["ratio"] or 0) > BOUND for v in per.values()):
                winters_rows.append({"set": label, "seed": seed, "first_day": day, "severity": w["severity"],
                                     "world_ratio": ratio, "exposure_deaths": w["exposure_deaths"],
                                     "members_at_start": w["members_at_start"], "world_dark_nights": w["dark_nights"],
                                     "deadwood_per_spot_at_start": (float(rec.deadwood[r0]) / spots) if spots else None,
                                     "spots": spots, "settlements": per})
    return {"deaths": deaths_rows, "daughters": daughters_rows, "over_bound_winters": winters_rows,
            "settlement_winters": settlement_winters}


def counts(a: dict) -> dict:
    deaths = a["deaths"]
    starv = [d for d in deaths if d["cause"] == "starvation"]
    dehyd = [d for d in deaths if d["cause"] == "dehydration"]
    expo = [d for d in deaths if d["cause"] == "exposure"]
    short = lambda d: d["bushes"][0] is not None and d["bushes"][0] < d["bushes"][1]
    dau = a["daughters"]
    sw = [w for w in a["settlement_winters"] if w["members_at_start"]]
    over_sw = [w for w in sw if w["ratio"] is not None and w["ratio"] > BOUND]
    return {
        "deaths_in_daughters": dict(Counter(d["cause"] for d in deaths)),
        "a_starvation_in_daughters_with_bush_shortfall": sum(1 for d in starv if short(d)),
        "a_starvation_in_daughters_without_region": sum(1 for d in starv if d["region"] is None),
        "a_starvation_total": len(starv),
        "a_starvation_food_min_72h": sorted(d["history_72h"].get("food_min") for d in starv if d["history_72h"].get("food_min") is not None),
        "b_dehydration_total": len(dehyd),
        "b_dehydration_region_water_distance": sorted(d["water_distance"] for d in dehyd if d["water_distance"] is not None),
        "b_dehydration_water_distance_at_death": sorted(d["water_distance_at_death"] for d in dehyd if d["water_distance_at_death"] is not None),
        "b_dehydration_off_region": sum(1 for d in dehyd if d["region"] is None),
        "c_daughters": len(dau), "c_daughters_off_region_centre": sum(1 for d in dau if d["at_leader"]),
        "c_daughters_by_season": dict(Counter(d["season"] for d in dau)),
        "c_daughters_with_bush_shortfall": sum(1 for d in dau if d["bushes_placed"] is not None and d["bushes_placed"] < d["bushes_requested"]),
        "c_region_water_distance_all_daughters": sorted(d["water_distance"] for d in dau if d["water_distance"] is not None),
        "c_daughters_alive_28": sum(1 for d in dau if d["alive_28"]),
        "exposure_deaths_in_daughters": len(expo),
        "exposure_in_daughters_dark_nights_of_3": dict(Counter(d["history_72h"].get("dark_nights_of_3") for d in expo)),
        "d_over_bound_winters": len(a["over_bound_winters"]),
        "d_settlement_winters_with_members": len(sw),
        "d_settlement_winters_over_bound": len(over_sw),
        "d_settlement_winters_over_bound_daughters": sum(1 for w in over_sw if w["daughter"]),
        "d_max_settlement_winter_ratio": max((w["ratio"] for w in sw if w["ratio"] is not None), default=None),
    }


def markdown(a: dict, c: dict) -> str:
    L = ["# Daughter diagnosis (Pass 12 Part 0, PASS12_GOAL.md 5.3)", "",
         "Sources: Pass 11's final `game_v17` 120-day probe runs (tuning seeds 0-31 `p11-7_final_v17_120day`, held-out "
         "32-95 `p11-7_final_heldout_v17_120day`; the final binary reproduces the frozen R3/held-out runs on every shared "
         "key, Pass 11 Part 7) and the same 96 seeds recorded with Pass 11's final recorder `p11bin/wm_final` (schema 5, "
         "hourly) into `worldsim-wm-data/p12/diag/game_v17/`. `daughter_diag.py`; the full per-death and per-winter "
         "rows are in `daughter_diag.json`.", "",
         "## Counts", ""]
    for k, v in c.items():
        L.append(f"- `{k}`: {json.dumps(v)}")
    L += ["", "## Deaths by starvation, dehydration and exposure in daughters", "",
          "| Set | Seed | Cause | Stage | Day (season) | Settlement | Founded day (season) | At leader tile | Region | Bushes placed/requested | Deadwood placed/requested | Water dist (region / at death) | Founders | Members at death | 72 h: food min, wood min, fire lit share, dark nights of 3 |",
          "|---|---:|---|---|---|---:|---|---|---:|---|---|---|---:|---:|---|"]
    for d in a["deaths"]:
        h = d["history_72h"]
        fm = lambda x: "—" if x is None else (f"{x:.0f}" if isinstance(x, float) else str(x))
        L.append(f"| {d['set'].replace('p11-7_final_', '')} | {d['seed']} | {d['cause']} | {d['stage']} | {d['day']} ({d['season']}) | "
                 f"{d['settlement']} | {d['founded_day']} ({d['founded_season']}) | {d['at_leader']} | {fm(d['region'])} | "
                 f"{fm(d['bushes'][0])}/{d['bushes'][1]} | {fm(d['deadwood'][0])}/{d['deadwood'][1]} | "
                 f"{fm(d['water_distance'])} / {fm(d['water_distance_at_death'])} | {fm(d['founders'])} | {fm(d['members_at_death'])} | "
                 f"{fm(h.get('food_min'))}, {fm(h.get('wood_min'))}, {('—' if h.get('fire_lit_share') is None else f'{h['fire_lit_share']:.2f}')}, {fm(h.get('dark_nights_of_3'))} |")
    L += ["", "## Winters over the 0.35 bound (world ratio or any settlement-winter ratio)", ""]
    for w in a["over_bound_winters"]:
        L.append(f"- {w['set'].replace('p11-7_final_', '')} seed {w['seed']}, winter from day {w['first_day']} (severity {w['severity']}): "
                 f"world {w['exposure_deaths']}/{w['members_at_start']} = {w['world_ratio']:.2f}, world dark nights {w['world_dark_nights']}, "
                 f"deadwood per spot at the start {w['deadwood_per_spot_at_start']:.2f} ({w['spots']} spots)")
        for sid, v in w["settlements"].items():
            L.append(f"  - settlement {sid}{' (daughter)' if v['daughter'] else ''}: deaths {v['exposure_deaths']} / members {v['members_at_start']}"
                     f" = {('—' if v['ratio'] is None else f'{v['ratio']:.2f}')}, dark nights {v['dark_nights']}, wood at the start {v['wood_at_start']}")
    L += ["", "## Daughters", "", "| Set | Seed | Sid | Founded day (season) | At leader | Region | Bushes | Deadwood | Water | Founders | Alive 28 d |",
          "|---|---:|---:|---|---|---:|---|---|---:|---:|---|"]
    for d in a["daughters"]:
        L.append(f"| {d['set'].replace('p11-7_final_', '')} | {d['seed']} | {d['settlement']} | {d['founded_day']} ({d['season']}) | {d['at_leader']} | "
                 f"{d['region']} | {d['bushes_placed']}/{d['bushes_requested']} | {d['deadwood_placed']}/{d['deadwood_requested']} | "
                 f"{d['water_distance']} | {d['founders']} | {d['alive_28']} |")
    return "\n".join(L) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--probe", nargs="+", required=True, type=Path)
    ap.add_argument("--recordings", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    args = ap.parse_args(argv)
    a = analyse(load_runs(args.probe), args.recordings)
    c = counts(a)
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "daughter_diag.json").write_text(json.dumps({"counts": c, **a}, indent=1, sort_keys=True) + "\n")
    (args.out / "DAUGHTER_DIAGNOSIS_TABLES.md").write_text(markdown(a, c))
    print(json.dumps(c, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
