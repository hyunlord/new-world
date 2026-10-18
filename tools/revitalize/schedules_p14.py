#!/usr/bin/env python3
"""Pass 14: write the intervention schedules of Stage A / Stage B from a factual timeline (PASS14_GOAL.md 6.1-6.2, 7, 8).

usage: schedules_p14.py --timeline timeline.json --arms F|W|ARM[,ARM...] --out DIR [--fork-tick 14280] [--days 120]
                         [--manifest DIR/manifest.json]

Arms (6.2): group F (food) `sp60L sp120L sp240L sp240H sp240E sp3x80L gf60L gf120L gf240L gf240H gf240E gf3x80L`,
group W (wood and weather) `rw120E rw120L rain13W cs2W`. L = T_low, H = T_high, E = T_eve (tick 29,400) of the seed
(`timeline_p14.py`); `*3x80L` = 80 at T_low, T_low + 2 d and T_low + 4 d; `rain13W` = rain 13 h at tick 33,720 (day 24,
18:00); `cs2W` = cold_snap 2 nights at tick 35,160 (day 25, 18:00; it ignores the settlement). A seed lacking the arm's
time gets no file for that arm (counted in the manifest). Each file `s<seed>_<arm>.json` is a list of rows
{"tick", "kind", "settlement": "home", "amount"}. Asserted for every row: fork_tick <= tick < days x 1440, rows in
non-decreasing tick order. The manifest lists every file with its sha256 and every (seed, arm) without a file.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

TPD = 1440
T_EVE = 29_400
RAIN_TICK = 33_720
COLD_TICK = 35_160

# arm -> (kind, amount, time, number of doses); time is "L", "H", "E" or an absolute tick.
ARMS: dict[str, tuple[str, int, str | int, int]] = {
    "sp60L": ("spoil_food", 60, "L", 1), "sp120L": ("spoil_food", 120, "L", 1), "sp240L": ("spoil_food", 240, "L", 1),
    "sp240H": ("spoil_food", 240, "H", 1), "sp240E": ("spoil_food", 240, "E", 1), "sp3x80L": ("spoil_food", 80, "L", 3),
    "gf60L": ("gift_food", 60, "L", 1), "gf120L": ("gift_food", 120, "L", 1), "gf240L": ("gift_food", 240, "L", 1),
    "gf240H": ("gift_food", 240, "H", 1), "gf240E": ("gift_food", 240, "E", 1), "gf3x80L": ("gift_food", 80, "L", 3),
    "rw120E": ("rot_wood", 120, "E", 1), "rw120L": ("rot_wood", 120, "L", 1),
    "rain13W": ("rain", 13, RAIN_TICK, 1), "cs2W": ("cold_snap", 2, COLD_TICK, 1),
}
GROUP_F = [a for a in ARMS if a[:2] in ("sp", "gf")]
GROUP_W = ["rw120E", "rw120L", "rain13W", "cs2W"]


def cast_tick(arm: str, seed_tl: dict) -> int | None:
    """The first (or only) cast tick of `arm` for one seed of the timeline, None when the seed lacks that time."""
    when = ARMS[arm][2]
    if isinstance(when, int):
        return when
    return {"L": seed_tl["T_low"], "H": seed_tl["T_high"], "E": seed_tl.get("T_eve", T_EVE)}[when]


def rows_for(arm: str, seed_tl: dict) -> list[dict] | None:
    kind, amount, _, doses = ARMS[arm]
    t = cast_tick(arm, seed_tl)
    if t is None:
        return None
    return [{"tick": t + 2 * TPD * i, "kind": kind, "settlement": "home", "amount": amount} for i in range(doses)]


def check_rows(rows: list[dict], fork_tick: int, days: int) -> None:
    last = -1
    for r in rows:
        assert set(r) == {"tick", "kind", "settlement", "amount"}, r
        assert fork_tick <= r["tick"] < days * TPD, f"row outside [fork, end): {r}"
        assert r["tick"] >= last, "rows out of order"
        last = r["tick"]


def parse_arms(spec: str) -> list[str]:
    arms: list[str] = []
    for part in spec.split(","):
        arms += GROUP_F if part == "F" else GROUP_W if part == "W" else [part]
    unknown = [a for a in arms if a not in ARMS]
    if unknown:
        raise SystemExit(f"unknown arms {unknown}")
    return arms


def write(timeline: dict, arms: list[str], out: Path, fork_tick: int, days: int) -> dict:
    out.mkdir(parents=True, exist_ok=True)
    files, missing = [], []
    for s in timeline["per_seed"]:
        for arm in arms:
            rows = rows_for(arm, s)
            if rows is None:
                missing.append([s["seed"], arm])
                continue
            check_rows(rows, fork_tick, days)
            text = json.dumps(rows) + "\n"
            name = f"s{s['seed']}_{arm}.json"
            (out / name).write_text(text)
            files.append({"file": name, "seed": s["seed"], "arm": arm, "cast_tick": rows[0]["tick"],
                          "sha256": hashlib.sha256(text.encode()).hexdigest()})
    per_arm = {a: sum(1 for f in files if f["arm"] == a) for a in arms}
    return {"fork_tick": fork_tick, "days": days, "arms": {a: list(ARMS[a]) for a in arms}, "seeds_per_arm": per_arm,
            "files": files, "missing": missing}


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--timeline", required=True)
    ap.add_argument("--arms", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--fork-tick", type=int, default=14_280)
    ap.add_argument("--days", type=int, default=120)
    ap.add_argument("--manifest")
    a = ap.parse_args(argv)
    timeline = json.loads(Path(a.timeline).read_text())
    m = write(timeline, parse_arms(a.arms), Path(a.out), a.fork_tick, a.days)
    m["timeline_sha256"] = hashlib.sha256(Path(a.timeline).read_bytes()).hexdigest()
    Path(a.manifest or Path(a.out) / "manifest.json").write_text(json.dumps(m, indent=1) + "\n")
    print(json.dumps({"files": len(m["files"]), "missing": len(m["missing"]), "seeds_per_arm": m["seeds_per_arm"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
