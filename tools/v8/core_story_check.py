#!/usr/bin/env python3
"""Validate paired CORE story-yield evidence; measurements have no numerical acceptance bar."""
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from typing import Final

import core_score as core
from core_score import Data, ScoreError, digest, mapping, number, require, sequence
import mux_safety_validate as validator

BASE: Final = core.BASE_SHA
SEEDS: Final = list(range(1232, 1264))
INPUTS: Final = {"steward": True, "defence": True, "story": "on", "neighbours": "on", "player_orders": False}
COUNTS: Final = ("entries", "deep_entries", "distinct_deep_chains", "people_with_three_entries", "missing_chains", "cyclic_chains")
DEFINITION: Final = {
    "depth": "verified record edges, excluding presentation-only label nodes", "minimum_depth": 3,
    "year_window": "[year*ticks_per_year, min(end_tick,(year+1)*ticks_per_year))",
    "chain_identity": "ordered verified entry and cause kinds; excludes IDs, unresolved suffix labels and termination markers; distinct observed paths, not inferred complete histories",
    "incomplete_chains": "missing ancestry retains verified depth; dangling targets add no edge; detected cycles are excluded; both reported separately",
    "partial_years": "raw counts with observed_ticks; never extrapolated",
    "people": "distinct people named in at least three entries within each year",
}
ARTIFACTS: Final = set(("tooling-final-bin/worldsim_probe tooling-final-bin/worldsim_perf tooling-final-bin/Cargo.lock "
    "tooling-final-bin/SHA256SUMS tooling-final2-source.patch tooling-final2-source-sha256.txt tooling-final2-build.log "
    "tooling-final-safety/bin/worldsim_probe tooling-final-safety/probe_sha256.txt tooling-final-safety/capture_checkout_sha.txt "
    "tooling-final-safety/capture_complete.txt tooling-final-identity-compare.log tooling-final-steward/summary.json "
    "tooling-final-steward.log base-steward/probe/summary.json story-yield-final-verification.txt").split())
INSTRUMENT: Final = {f"rust/crates/sim-test/src/story_yield/{name}.rs" for name in ("metric", "cli", "mod", "tests")}
ROOT: Final = Path(__file__).resolve().parents[2]


def sha(path: Path) -> str:
    """Hash exact source or evidence bytes."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path: Path) -> Data:
    """Reject duplicate keys and nonfinite numbers at the JSON boundary."""
    return mapping(json.loads(path.read_bytes(), object_pairs_hook=validator.unique_pairs, parse_constant=core.reject_constant))


def inputs(value: Data) -> None:
    """Reject unknown input options and truthy numbers posing as booleans."""
    require(value.keys() == INPUTS.keys(), "Unknown/missing story inputs")
    require(value["steward"] is True and value["defence"] is True and value["player_orders"] is False
            and value["story"] == value["neighbours"] == "on", "Wrong story inputs")


def arm(report: Data, receipt: Data, role: str,
        *, protocol: core.SeedProtocol = core.SeedProtocol.ORIGINAL) -> Data:
    """Validate 32 worlds, each with exactly two complete 24-day years."""
    core.check_protocol(receipt, protocol)
    require(number(receipt["schema_version"], "receipt schema") == protocol.metadata()["schema_version"] and receipt["role"] == role, "Wrong receipt schema/role")
    source = digest(receipt["source_sha"], 40)
    require(number(receipt["days"], "receipt days") == 48
            and [number(seed, "receipt seed") for seed in sequence(receipt["seeds"])] == protocol.seeds, "Wrong receipt coverage")
    inputs(mapping(receipt["inputs"]))
    require(number(report["schema_version"], "report schema") == 2 and report["scene"] == "colony"
            and number(report["days"], "report days") == 48 and report["source_sha"] == source, "Wrong report source/schema/duration")
    inputs(mapping(report["inputs"]))
    definition = mapping(report["definition"])
    require(definition == DEFINITION and type(definition["minimum_depth"]) is int, "Story definition mismatch")
    rows = [mapping(value) for value in sequence(report["worlds"])]
    require(sorted(number(row["seed"], "seed") for row in rows) == protocol.seeds, "Missing/duplicate/wrong story worlds")
    result = []
    totals = dict.fromkeys(COUNTS, 0)
    for row in sorted(rows, key=lambda row: number(row["seed"], "seed")):
        require(number(row["end_tick"], "end tick") == 69120
                and number(row["ticks_per_year"], "year ticks") == 34560, "Incomplete story duration/calendar")
        require(type(row["fingerprint"]) is str and re.fullmatch(r"[0-9a-f]{16}-[0-9a-f]{16}", row["fingerprint"]) is not None, "Invalid story fingerprint")
        years = [mapping(value) for value in sequence(row["years"])]
        require(sorted(number(year["year"], "year") for year in years) == [0, 1], "Missing/duplicate year")
        for year in sorted(years, key=lambda year: number(year["year"], "year")):
            require(year.keys() == set(COUNTS) | {"year", "observed_ticks", "complete_year"}, "Unknown/missing annual metrics")
            require(number(year["observed_ticks"], "year observed ticks") == 34560 and year["complete_year"] is True, "Incomplete year")
            counts = {name: number(year[name], name) for name in COUNTS}
            require(all(value <= 2**64 - 1 for value in counts.values()), "Count exceeds runtime integer range")
            require(counts["distinct_deep_chains"] <= counts["deep_entries"] <= counts["entries"], "Impossible deep chain counts")
            require(counts["people_with_three_entries"] <= counts["entries"], "Impossible person count")
            require(counts["missing_chains"] + counts["cyclic_chains"] <= counts["entries"], "Impossible diagnostic counts")
            require(counts["deep_entries"] + counts["cyclic_chains"] <= counts["entries"], "Cycles counted as deep chains")
            for name, value in counts.items():
                totals[name] += value
            result.append({"seed": row["seed"], "year": year["year"], "observed_ticks": 34560, **counts})
    return {"role": role, "source_sha": source, "binary_sha256": digest(receipt["binary_sha256"], 64),
            "worlds": 32, "world_years": 64, "totals": totals,
            "per_world_year": {name: value / 64 for name, value in totals.items()}, "rows": result}


def check_pair(base_report: Data, base_receipt: Data, mind_report: Data, mind_receipt: Data, provenance: Data,
               *, protocol: core.SeedProtocol = core.SeedProtocol.ORIGINAL) -> Data:
    """Compare measurements without treating larger counts as a pass requirement."""
    try:
        require(base_receipt["runtime_base_sha"] == BASE and mind_receipt["runtime_base_sha"] is None, "Wrong runtime reference")
        require(base_receipt["policy_sha256"] is None and mind_receipt["base_instrument_provenance_sha256"] is None, "Wrong arm metadata")
        digest(mind_receipt["policy_sha256"], 64)
        require(number(provenance["schema_version"], "provenance schema") == 1
                and provenance["runtime_base_sha"] == BASE, "Wrong BASE instrument provenance")
        require(provenance["source_sha"] == base_receipt["source_sha"]
                and provenance["binary_sha256"] == base_receipt["binary_sha256"], "BASE source/binary provenance mismatch")
        manifest = mapping(provenance["source_manifest"])
        require(number(manifest["rows"], "manifest rows") == 506
                and number(manifest["git_blob_matches"], "source matches") == 505
                and manifest["git_blob_missing"] == ["rust/Cargo.lock"] and manifest["git_blob_mismatches"] == []
                and manifest["untracked_lock_matches_manifest"] is True, "Incomplete frozen BASE source closure")
        require(provenance["checkout_vs_base_only_docs"] is True, "BASE checkout has non-document changes")
        safety, on = mapping(provenance["safety"]), mapping(provenance["steward_on"])
        require(safety["binary_matches_frozen"] is True and safety["comparison_log_reports_pass"] is True, "Missing BASE safety identity")
        require(on["summary_byte_identical"] is True and on["determinism_log_pass"] is True
                and number(on["days"], "ON days") == 24
                and [number(seed, "ON seed") for seed in sequence(on["seeds"])] == [0, 1], "Missing final2 ON identity")
        base, mind = arm(base_report, base_receipt, "base_instrument", protocol=protocol), arm(mind_report, mind_receipt, "mind", protocol=protocol)
        require(base_receipt["metric_sha256"] == mind_receipt["metric_sha256"]
                and base_receipt["cli_sha256"] == mind_receipt["cli_sha256"], "Different story instruments")
        return {**protocol.metadata(), "status": "MEASURED", "acceptance_claim": False,
                "numerical_bar": None, "base_designation": "BASE runtime with frozen final2 observational instrument; not exact BASE source build",
                "definition": DEFINITION, "base": base, "mind": mind}
    except KeyError as error:
        raise ScoreError(f"Missing required story field: {error}") from error


def bound(report_path: Path, receipt_path: Path,
          *, protocol: core.SeedProtocol = core.SeedProtocol.ORIGINAL) -> tuple[Data, Data, Data]:
    """Bind reports and both shared instrument sources to current checker bytes."""
    report, receipt = read(report_path), read(receipt_path)
    core.check_protocol(receipt, protocol)
    if protocol is core.SeedProtocol.P3D:
        require(receipt.get("scorer_sha256") == sha(Path(core.__file__)), "Story protocol source hash mismatch")
        require(receipt.get("validator_sha256") == sha(Path(validator.__file__)), "Story validator hash mismatch")
    require(receipt.get("report_sha256") == sha(report_path), "Story report hash mismatch")
    require(receipt.get("checker_sha256") == sha(Path(__file__)), "Story checker hash mismatch")
    for name in ("metric", "cli"):
        path = ROOT / f"rust/crates/sim-test/src/story_yield/{name}.rs"
        require(receipt.get(f"{name}_sha256") == sha(path), f"Story {name} source hash mismatch")
    return report, receipt, {"report_sha256": sha(report_path), "receipt_sha256": sha(receipt_path)}


def main() -> int:
    """Write a new external measurement receipt; invalid artifacts fail closed."""
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ("base-report", "base-receipt", "mind-report", "mind-receipt", "base-provenance", "output"):
        parser.add_argument("--" + flag, type=Path, required=True)
    parser.add_argument("--seed-protocol", choices=[p.value for p in core.SeedProtocol],
                        default=core.SeedProtocol.ORIGINAL.value)
    args = parser.parse_args()
    try:
        require(not args.output.exists() and not args.output.is_symlink(), "Output must be new")
        require(not args.output.resolve().is_relative_to(ROOT), "Output must be external")
        base, base_receipt, base_hashes = bound(args.base_report, args.base_receipt, protocol=core.SeedProtocol(args.seed_protocol))
        mind, mind_receipt, mind_hashes = bound(args.mind_report, args.mind_receipt, protocol=core.SeedProtocol(args.seed_protocol))
        provenance = read(args.base_provenance)
        require(base_receipt.get("base_instrument_provenance_sha256") == sha(args.base_provenance), "BASE provenance hash mismatch")
        require(mapping(provenance["artifacts"]).keys() == ARTIFACTS, "Incomplete BASE provenance artifacts")
        require(mapping(provenance["instrument"]).keys() == INSTRUMENT, "Incomplete instrument source closure")
        artifacts = mapping(provenance["artifacts"])
        require(mapping(artifacts["tooling-final-bin/worldsim_probe"])["sha256"] == base_receipt["binary_sha256"]
                == mapping(artifacts["tooling-final-safety/bin/worldsim_probe"])["sha256"], "Frozen/safety/receipt binary hashes differ")
        require(mapping(artifacts["tooling-final-steward/summary.json"])["sha256"]
                == mapping(artifacts["base-steward/probe/summary.json"])["sha256"], "ON identity artifact hashes differ")
        for value in artifacts.values():
            artifact = mapping(value)
            require(type(artifact["path"]) is str and sha(Path(artifact["path"])) == artifact["sha256"], "Changed BASE provenance artifact")
        for path, value in mapping(provenance["instrument"]).items():
            require(path.startswith("rust/crates/sim-test/src/story_yield/")
                    and sha(ROOT / path) == mapping(value)["sha256"], "Frozen/current instrument source differs")
        result = check_pair(base, base_receipt, mind, mind_receipt, provenance,
                            protocol=core.SeedProtocol(args.seed_protocol))
        result["input_hashes"] = {"base": base_hashes, "mind": mind_hashes,
                                  "base_provenance_sha256": sha(args.base_provenance), "checker_sha256": sha(Path(__file__))}
        with args.output.open("x", encoding="utf-8") as stream:
            stream.write(json.dumps(result, indent=2, allow_nan=False) + "\n")
        print("CORE_STORY MEASURED; no numerical bar and no full CORE acceptance claim")
        return 0
    except (ScoreError, OSError, json.JSONDecodeError, TypeError, ValueError, KeyError) as error:
        print(f"CORE_STORY INVALID: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
