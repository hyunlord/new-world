"""Run the two-file apple content extension in a fresh, hashed probe process."""
from __future__ import annotations

import argparse
import os
import subprocess
from pathlib import Path

from m6a_evidence import EvidenceError, Node, reference_json, source_hash, write_json


def run(args: argparse.Namespace) -> int:
    root = args.source_root.resolve(strict=True)
    binary = args.binary.resolve(strict=True)
    out = args.out.resolve()
    if out.is_relative_to(root):
        raise EvidenceError("data-only output must be external")
    source = source_hash(root, args.source_head)
    before = reference_json(binary)
    fixture = root / "rust/crates/sim-test/data/m6a_apple"
    inputs = [reference_json(fixture / name) for name in ("pack.ron", "items.ron", "plants.ron")]
    out.mkdir()
    environment = os.environ.copy()
    environment["WORLDSIM_CONTENT_PACK"] = str(fixture / "pack.ron")
    command = [str(binary), "m6a-data-only", "--out", str(out / "observed")]
    with (out / "process.log").open("xb") as log:
        process = subprocess.run(command, env=environment, stdout=log, stderr=subprocess.STDOUT, check=False)
    report_path = out / "observed/data-only.json"
    passed = False
    if report_path.exists():
        report = Node.load(report_path)
        passed = report.get("passed").boolean() and report.get("category_fallback").boolean()
        passed &= report.get("initially_growing").boolean() and bool(report.get("intake_evidence").rows())
        passed &= report.get("harvested_pieces").integer() > 0 and bool(report.get("placed").rows())
        for field in ("ripe_tick", "harvest_job_tick", "haul_job_tick", "carried_tick", "stored_tick", "eaten_from_store_tick"):
            report.get(field).integer()
    passed &= process.returncode == 0 and reference_json(binary) == before
    passed &= source_hash(root, args.source_head) == source
    write_json(out / "receipt.json", {"schema_version": 1, "kind": "m6a_data_only_receipt_v1",
               "status": "PASS" if passed else "FAIL", "source_head": args.source_head,
               "source_hash": source, "binary": before, "inputs": inputs, "command": command,
               "exit_code": process.returncode, "log": reference_json(out / "process.log"),
               "report": reference_json(report_path) if report_path.exists() else None})
    return int(not passed)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--source-head", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    try:
        return run(args)
    except (EvidenceError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(2, f"M6A data-only evidence error: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
