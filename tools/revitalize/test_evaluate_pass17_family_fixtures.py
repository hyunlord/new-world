from __future__ import annotations

import json
from pathlib import Path

from test_evaluate_pass17 import write_cohort
from evaluate_pass17_report import JsonObject


def write_lines(path: Path, rows: list[JsonObject]) -> None:
    path.write_text("".join(json.dumps(row) + "\n" for row in rows))


def write_family_cohort(root: Path, scene: str = "game_v22") -> Path:
    write_cohort(root, scene)
    summary = json.loads((root / "summary.json").read_text())
    summary["days"] = {str(day): {} for day in range(1, 61)}
    for run in summary["runs"]:
        seed = run["seed"]
        run["alive_daily"] = [4 + (day >= 19) + (day >= 20) for day in range(1, 61)]
        stem = root / f"{scene}_s{seed}"
        people = [{"id": person, "female": person % 2 == 0,
            "mother": 0 if person in (2, 3) else None, "father": 1 if person in (2, 3) else None,
            "temperament": {axis: 200 * person for axis in ("ns", "ha", "rd", "p")},
            "lineage_key": person, "born_tick": (18 if person == 2 else 19) * 1440 if person in (2, 3) else None,
            "alive_end": True, "partner": person ^ 1 if person < 2 else None,
            "pairing_tick": 0 if person < 2 else None, "household_key": 0 if person < 2 else person}
            for person in range(6)]
        def members(tick: int) -> list[JsonObject]:
            return [{"id": person["id"], "settlement": 0, "life_stage": "adult", "partner": person["partner"],
                "pairing_tick": person["pairing_tick"], "household_key": person["household_key"], "bed": [1, 1]}
                for person in people if person["born_tick"] is None or person["born_tick"] <= tick]
        family_hours = [{"tick": tick, "elapsed_ticks": tick + 1, "simulation_day": tick // 1440 + 1,
            "display_day": (tick + 420) // 1440 + 1, "minute": (tick + 420) % 1440,
            "people": members(tick), "housing_capacity": 8, "people_with_bed": len(members(tick)),
            "bedless": 0, "unused_slots": 8 - len(members(tick))} for tick in range(0, 60 * 1440, 60)]
        hours = [{"tick": row["tick"], "day": row["display_day"], "minute": row["minute"],
            "states": {"adult": [len(row["people"]), 0, 0, 0, 0], "child": [0] * 5, "elder": [0] * 5},
            "stockpile_food": 100, "stockpile_wood": 80} for row in family_hours]
        write_lines(Path(f"{stem}_hours.jsonl"), hours)
        meta = json.loads(Path(f"{stem}_trace_meta.json").read_text())
        meta["end_tick"] = 60 * 1440
        meta.update(fissions_completed=1, fissions_cancelled=0, fissions_pending=0)
        Path(f"{stem}_trace_meta.json").write_text(json.dumps(meta))
        if scene != "game_v22":
            continue
        write_lines(Path(f"{stem}_family_people.jsonl"), people)
        write_lines(Path(f"{stem}_family_hours.jsonl"), family_hours)
        births = [{"event": "born", "tick": person["born_tick"], "simulation_day": person["born_tick"] // 1440 + 1,
            "display_day": (person["born_tick"] + 420) // 1440 + 1, "minute": 420, "child": person["id"],
            "mother": 0, "father": 1, "fallback": person["id"] == 2, "lineage_key": person["lineage_key"],
            "female": person["female"], "temperament": person["temperament"]} for person in people if person["born_tick"] is not None]
        write_lines(Path(f"{stem}_family_events.jsonl"), births)
        write_lines(Path(f"{stem}_family_fissions.jsonl"), [{"fission_id": 0, "parent": 0, "depart_tick": 30 * 1440,
            "status": "completed", "members": [0, 1, 4], "households": [[0, 1], [2], [3], [4], [5]],
            "couples": [[0, 1]], "couples_total": 1, "couples_split": 0, "households_split": 0}])
        family_meta = {"schema_version": 17, "end_tick": 60 * 1440, "observation_misses": 0,
            "people": 6, "births_total": 2, "source_births_total": 2,
            "departures_total": 1, "source_departures_total": 1,
            "day60": {"tick": 86399, "elapsed_ticks": 86400, "people": members(86399)},
            "births_by_day": [{"day": day, "births": int(day in (19, 20)), "couple_births": int(day == 20),
                "fallback_births": int(day == 19)} for day in range(1, 61)]}
        Path(f"{stem}_family_meta.json").write_text(json.dumps(family_meta))
    (root / "summary.json").write_text(json.dumps(summary))
    return root
