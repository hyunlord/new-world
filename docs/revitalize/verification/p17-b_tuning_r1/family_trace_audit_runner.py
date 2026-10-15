#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Usage: python3 family_trace_audit_runner.py tuning|evaluation RUN_ROOT EVAL_JSON [--json]
"""Extract terminal Stage B cohort evidence; stdout is the only output destination.

The final RUNLOG workload-end row is checked before any cohort artifact. An evaluator FAIL is valid
input evidence. This runner never assigns F1-F8 verdicts or launches workloads.
The frozen checker remains separate and its exact SHA256 is enforced.
"""

from __future__ import annotations

import hashlib
import json
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Final

sys.dont_write_bytecode = True
import family_trace_recompute as raw

FROZEN_SHA256: Final = "4f6902c44ac7cdd64c6ab426887a5d894ad177168ffce39661f6020948939df9"
EXPOSURES: Final = {
    "partnered_adults_at_day60": "F1_raw_day60",
    "couple_births_from_day20": "F2_raw",
    "same_bed_couple_nights": "F4_raw",
    "split_couples_completed_fissions": "F5_raw",
}
SCOPE: Final = (
    "Raw consistency and evidence extraction only; no F1-F8 acceptance verdicts.",
    "Pooled fractions below are descriptive raw exposures, not confidence intervals.",
    "Bedless counts are person-samples; unused-slot counts are slot-samples. Neither "
    "measures distinct people, elapsed sleep time, or actual nearest-tile sleeping.",
    *raw.LIMITATIONS,
)


@dataclass(frozen=True, slots=True)
class Cohort:
    mode: str
    first_seed: int
    worlds: int
    days: int

    @property
    def seeds(self) -> list[int]:
        return list(range(self.first_seed, self.first_seed + self.worlds))


def specification(mode: str) -> Cohort:
    match mode:
        case "tuning":
            return Cohort(mode, 18100, 16, 60)
        case "evaluation":
            return Cohort(mode, 18000, 32, 120)
        case invalid:
            raise raw.InputError("mode", f"expected tuning or evaluation, got {invalid!r}")


def require(condition: bool, where: str, message: str) -> None:
    if not condition:
        raise raw.InputError(where, message)


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def exit_code(path: Path) -> int:
    text = path.read_text(encoding="utf-8").strip()
    require(text in ("0", "1"), str(path), "requires terminal exit 0 or 1; error/in-progress evidence is not auditable")
    return int(text)


def report_contract(report: raw.Row, cohort: Cohort, code: int) -> None:
    require(report.num("schema_version") == 17 and report.text("stage") == "b"
            and report.text("mode") == cohort.mode, report.where, "wrong evaluator schema/stage/mode")
    rules = raw.Row.parse(report.get("rules"), f"{report.where}.rules")
    require(set(rules.data) == {f"F{number}" for number in range(1, 9)}, report.where, "missing F1-F8 evaluator provenance")
    for rule in rules.data:
        allowed = ("PASS", "FAIL", "PENDING") if cohort.mode == "tuning" and rule in ("F7", "F8") else ("PASS", "FAIL")
        require(rules.text(rule) in allowed, rules.where, f"invalid imported {rule} result")
    values = set(rules.data.values())
    expected = "FAIL" if "FAIL" in values else ("PENDING" if "PENDING" in values else "PASS")
    require(report.text("verdict") == expected, report.where, "imported aggregate verdict inconsistent with imported rules")
    require(code == int(expected == "FAIL"), report.where, "terminal exit disagrees with imported evaluator verdict")
    require(not report.data.get("error") and not report.data.get("errors"), report.where, "evaluator records an input/error state")
    for side, scene in (("candidate", "game_v22"), ("baseline", "game_v21")):
        description = raw.Row.parse(report.get(side), f"{report.where}.{side}")
        require(description.text("scene") == scene and description.num("days") == cohort.days
                and description.num("worlds") == cohort.worlds and list(description.ids("seeds")) == cohort.seeds,
                description.where, "evaluator cohort differs from the registered seed/day inventory")


def terminal(root: Path, report_path: Path, cohort: Cohort) -> raw.Row:
    """Read the actual p17_finish terminal log marker before opening cohort outputs."""
    runlog = root / "RUNLOG.txt"
    log = runlog.read_text(encoding="utf-8").splitlines()
    require(bool(log), str(runlog), "empty workload log")
    ending = log[-1].rsplit(" workloads end exit=", 1)
    require(len(ending) == 2 and ending[1] in ("0", "1"), str(runlog), "missing terminal workloads-end row with exit 0 or 1")
    code = int(ending[1])
    evaluator_exit = root / "evaluate.exit"
    require(exit_code(evaluator_exit) == code, str(evaluator_exit), "runner/evaluator terminal exits disagree")
    report = raw.read_json(report_path)
    report_contract(report, cohort, code)
    paths = [runlog, evaluator_exit, report_path, root / "source_commit.txt"]
    producer = paths[3].read_text(encoding="utf-8").strip()
    require(len(producer) == 40 and all(char in "0123456789abcdef" for char in producer), str(paths[3]), "invalid producer SHA")
    for scene in ("game_v22", "game_v21"):
        probe_exit = root / f"{scene}.exit"
        require(exit_code(probe_exit) == 0, str(probe_exit), "probe did not complete successfully")
        summary_path = root / scene / "summary.json"
        summary = raw.read_json(summary_path)
        runs = summary.rows("runs")
        seeds = [run.num("seed") for run in runs]
        day_rows = raw.Row.parse(summary.get("days"), f"{summary_path}.days")
        require(summary.text("scene") == scene and set(day_rows.data) == {str(day) for day in range(1, cohort.days + 1)},
                str(summary_path), "summary scene or daily-bin inventory mismatch")
        require(sorted(seeds) == cohort.seeds, str(summary_path), "summary seed set/count mismatch or duplicates")
        require(all(len(run.ids("alive_daily")) == cohort.days for run in runs), str(summary_path), "incomplete daily census")
        paths.extend((probe_exit, summary_path))
    return raw.Row({"workloads_exit": code, "producer_commit": producer,
        "evaluator_report": str(report_path), "evaluator_verdict": report.text("verdict"),
        "evaluator_rules": dict(raw.Row.parse(report.get("rules"), "rules").data),
        "terminal_log_row": log[-1], "input_sha256": {str(path): digest(path) for path in paths}}, "terminal")


def totals(rows: Sequence[raw.Row], keys: Sequence[str]) -> raw.Row:
    return raw.Row({key: sum(row.num(key) for row in rows) for key in keys}, "totals")


def aggregate(worlds: Sequence[raw.Row], cohort: Cohort) -> raw.Row:
    """Pool counts, preserving denominators, exclusions and zero-exposure worlds."""
    exposures: dict[str, raw.Json] = {}
    for description, source_key in EXPOSURES.items():
        values = [raw.Row.parse(world.get(source_key), source_key) for world in worlds if world.get(source_key) is not None]
        numerator = sum(value.num("numerator") for value in values)
        denominator = sum(value.num("denominator") for value in values)
        exposures[description] = {"numerator": numerator, "denominator": denominator,
            "ratio": numerator / denominator if denominator else None,
            "worlds_with_exposure_record": len(values),
            "zero_denominator_worlds": sum(value.num("denominator") == 0 for value in values)}
    days: list[raw.Json] = []
    night_days: list[raw.Json] = []
    all_births = [row for world in worlds for row in world.rows("births_by_day")]
    all_nights = [row for world in worlds for row in world.rows("nights")]
    all_hours = [row for world in worlds for row in world.rows("housing_hours")]
    night_fields = ("mutual_living_couples", "both_bedded_denominator", "same_bed_numerator",
                    "different_bed", "one_bed_excluded", "neither_bed_excluded", "bedless", "unused_slots")
    for day in range(1, cohort.days + 1):
        births = [row for row in all_births if row.num("day") == day]
        nights = [row for row in all_nights if row.num("simulation_day") == day]
        days.append({"day": day, "worlds": len(births), **totals(births, ("births", "couple_births", "fallback_births")).data})
        night_days.append({"day": day, "world_samples": len(nights), **totals(nights, night_fields).data})
    fissions = [row for world in worlds for row in world.rows("fissions")]
    statuses: dict[str, raw.Json] = {}
    for status in ("completed", "cancelled", "pending"):
        selected = [row for row in fissions if row.text("status") == status]
        statuses[status] = {"fissions": len(selected),
            **totals(selected, ("couples_total", "couples_split", "households_total", "households_split", "households_departed_whole")).data}
    return raw.Row({"worlds_aggregated": len(worlds),
        "complete_cohort": len(worlds) == cohort.worlds,
        "lifetime_people": sum(world.num("lifetime_people") for world in worlds),
        "female": sum(world.num("female") for world in worlds),
        "male": sum(world.num("male") for world in worlds),
        "pair_events": sum(world.num("pair_events") for world in worlds),
        "birth_totals": dict(totals(all_births, ("births", "couple_births", "fallback_births")).data),
        "births_by_day": days, "nights_by_day": night_days, "raw_exposures": exposures,
        "night_totals": {"world_night_samples": len(all_nights), **totals(all_nights, night_fields).data},
        "hourly_totals": {"world_hour_samples": len(all_hours),
            **totals(all_hours, ("people", "people_with_bed", "bedless", "unused_slots")).data},
        "fissions_by_status": statuses}, "aggregate")


def select_example(worlds: Sequence[raw.Row]) -> raw.Row:
    if not worlds:
        return raw.Row({"coverage": 0, "rows": [], "message": "No successfully parsed worlds; no handexample available."}, "example")
    world = min(worlds, key=lambda row: (-raw.Row.parse(row.get("family_example"), "example").num("coverage"), row.num("seed")))
    example = raw.Row.parse(world.get("family_example"), "example")
    return raw.Row({"seed": world.num("seed"), **example.data}, "example")


def extract(root: Path, report_path: Path, cohort: Cohort) -> raw.Row:
    receipt = terminal(root, report_path, cohort)
    checker = Path(raw.__file__).resolve()
    require(digest(checker) == FROZEN_SHA256, str(checker), "frozen checker hash changed; runner refuses to audit")
    directory = root / "game_v22"
    metas = sorted(directory.glob("*_family_meta.json"))
    expected_names = {f"game_v22_s{seed}_family_meta.json" for seed in cohort.seeds}
    require({path.name for path in metas} == expected_names, str(directory), "family meta inventory differs from complete cohort")
    reports: list[raw.Row] = []
    errors: list[raw.Json] = []
    world_rows: list[raw.Json] = []
    hashes: dict[str, raw.Json] = dict(raw.Row.parse(receipt.get("input_sha256"), "hashes").data)
    hashes[str(checker)] = FROZEN_SHA256
    hashes[str(Path(__file__).resolve())] = digest(Path(__file__))
    for index, seed in enumerate(cohort.seeds, 1):
        meta = directory / f"game_v22_s{seed}_family_meta.json"
        try:
            evidence = raw.load(meta)
            hashes.update(evidence.hashes)
            require(evidence.meta.num("seed") == seed and evidence.meta.text("scene") == "game_v22"
                    and evidence.meta.num("end_tick") == cohort.days * raw.DAY
                    and evidence.meta.text("git_sha") == receipt.text("producer_commit"), str(meta), "world provenance/duration mismatch")
            result = raw.audit(evidence)
            reports.append(result)
            world_rows.append({key: result.get(key) for key in (
                "seed", "git_sha", "checks", "issues", "lifetime_people", "female", "male", "pair_events",
                "F1_raw_day60", "F2_raw", "F4_raw", "F5_raw", "fissions_by_status")})
            for issue in raw.array(result.get("issues"), "issues"):
                errors.append({"seed": seed, "kind": "discrepancy", "detail": issue})
            print(f"family raw audit {index}/{cohort.worlds} seed={seed} checks={result.num('checks')} discrepancies={len(raw.array(result.get('issues'), 'issues'))}", file=sys.stderr)
        except (raw.InputError, OSError, json.JSONDecodeError) as error:
            errors.append({"seed": seed, "kind": "input_error", "detail": str(error)})
            print(f"family raw audit {index}/{cohort.worlds} seed={seed} INPUT_ERROR: {error}", file=sys.stderr)
    for path, expected in tuple(hashes.items()):
        if digest(Path(path)) != expected:
            errors.append({"kind": "input_changed", "detail": path})
    return raw.Row({"schema_version": 1, "purpose": "independent raw family evidence extraction; not Part8 review",
        "audit_status": "DISCREPANCIES" if errors else "CONSISTENT",
        "acceptance_verdicts_assigned": False, "generated_at_utc": datetime.now(UTC).isoformat(),
        "mode": cohort.mode, "expected_worlds": cohort.worlds, "expected_seeds": cohort.seeds,
        "days": cohort.days, "run_root": str(root), "terminal_provenance": dict(receipt.data),
        "checker_sha256": FROZEN_SHA256, "checks": sum(world.num("checks") for world in reports),
        "discrepancies": errors, "aggregate": dict(aggregate(reports, cohort).data), "worlds": world_rows,
        "handexample": dict(select_example(reports).data), "input_sha256": hashes, "evidence_limits": list(SCOPE)}, "report")


def human(report: raw.Row) -> str:
    aggregate_row = raw.Row.parse(report.get("aggregate"), "aggregate")
    receipt = raw.Row.parse(report.get("terminal_provenance"), "terminal")
    lines = [f"{report.text('audit_status')} — {report.text('mode')} raw family evidence",
        "No F1-F8 acceptance verdict is assigned by this audit.",
        f"Worlds parsed={aggregate_row.num('worlds_aggregated')}/{report.num('expected_worlds')}; days={report.num('days')}; checks={report.num('checks')}",
        f"Imported evaluator verdict={receipt.text('evaluator_verdict')}; workloads exit={receipt.num('workloads_exit')}; source={receipt.text('evaluator_report')}",
        f"Producer={receipt.text('producer_commit')}; frozen checker SHA256={report.text('checker_sha256')}",
        f"Lifetime people={aggregate_row.num('lifetime_people')} female={aggregate_row.num('female')} male={aggregate_row.num('male')}; pair events={aggregate_row.num('pair_events')}"]
    exposures = raw.Row.parse(aggregate_row.get("raw_exposures"), "exposures")
    for name, value in exposures.data.items():
        row = raw.Row.parse(value, name)
        lines.append(f"{name}: {row.num('numerator')}/{row.num('denominator')} ratio={row.get('ratio')}; zero-denominator worlds={row.num('zero_denominator_worlds')}")
    for key in ("birth_totals", "night_totals", "hourly_totals", "fissions_by_status"):
        lines.append(f"{key}: {json.dumps(aggregate_row.get(key), sort_keys=True)}")
    lines.append("Daily births: day worlds total couple fallback")
    for day in aggregate_row.rows("births_by_day"):
        lines.append("  " + " ".join(str(day.num(key)) for key in ("day", "worlds", "births", "couple_births", "fallback_births")))
    lines.append("20:00 by simulation day: day worlds same/both-bedded mutual one-bed neither-bed bedless unused")
    for day in aggregate_row.rows("nights_by_day"):
        lines.append(f"  {day.num('day')} {day.num('world_samples')} {day.num('same_bed_numerator')}/{day.num('both_bedded_denominator')} "
            f"{day.num('mutual_living_couples')} {day.num('one_bed_excluded')} {day.num('neither_bed_excluded')} {day.num('bedless')} {day.num('unused_slots')}")
    example = raw.Row.parse(report.get("handexample"), "example")
    lines.append(f"Concrete family: seed={example.data.get('seed')} evidence categories={example.num('coverage')}/4")
    children: set[int] = set()
    for row in example.rows("rows"):
        event = raw.Row.parse(row.get("raw"), "example raw")
        lines.append(f"  {row.text('kind')} {row.text('source')}: {json.dumps(event.data, sort_keys=True)}")
        if row.text("kind") == "birth":
            children.add(event.num("child"))
        if row.text("kind") == "fission":
            lines.append(f"  Selected original paired parents={row.get('selected_pair_members')}; recorded children selected={sorted(children.intersection(event.ids('members')))}")
            for child in sorted(children):
                lines.append(f"  Child {child} departure household(s)={[list(group) for group in event.groups('households') if child in group]}; departure couple(s)={[list(pair) for pair in event.groups('couples') if child in pair]}")
            lines.append("  Descendants with their own households are distinct from the original parents' departure household.")
    if not example.rows("rows"):
        lines.append(str(example.data.get("message", "No concrete family rows available.")))
    for discrepancy in report.rows("discrepancies"):
        lines.append(f"DISCREPANCY: {json.dumps(discrepancy.data, sort_keys=True)}")
    lines.extend(("Evidence limits:", *(f"- {limit}" for limit in raw.array(report.get("evidence_limits"), "limits"))))
    lines.append("Input SHA256:")
    for path, value in sorted(raw.Row.parse(report.get("input_sha256"), "hashes").data.items()):
        lines.append(f"  {value}  {path}")
    return "\n".join(lines)


def main(arguments: Sequence[str]) -> int:
    if len(arguments) == 2 and arguments[0] == "--render-json":
        try:
            report = raw.read_json(Path(arguments[1]))
            require(report.num("schema_version") == 1 and not report.flag("acceptance_verdicts_assigned"),
                    report.where, "not a raw-audit report")
            print(human(report))
            return int(report.text("audit_status") != "CONSISTENT")
        except (raw.InputError, OSError, json.JSONDecodeError) as error:
            print(f"INPUT ERROR: {error}", file=sys.stderr)
            return 2
    json_output = "--json" in arguments
    values = [value for value in arguments if value != "--json"]
    if len(values) != 3:
        print("Usage: python3 family_trace_audit_runner.py tuning|evaluation RUN_ROOT EVAL_JSON [--json]", file=sys.stderr)
        return 2
    try:
        cohort = specification(values[0])
        report = extract(Path(values[1]).absolute(), Path(values[2]).absolute(), cohort)
        print(json.dumps(report.data, indent=2, sort_keys=True) if json_output else human(report))
        return int(report.text("audit_status") != "CONSISTENT")
    except (raw.InputError, OSError, json.JSONDecodeError) as error:
        print(f"INPUT ERROR; no cohort audit completed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
