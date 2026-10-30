"""Serial M6a cohorts with immutable per-world evidence and an append-only round log."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import time
from collections.abc import Mapping
from pathlib import Path

from m6a_protocol import protocol, recorded, check_freeze
from m6a_evidence import EvidenceError, Node, Reference, digest, git, reference_json, source_hash, write_json


def probe_environment(values: Mapping[str, str]) -> dict[str, str]:
    """Freeze admitted host variables; content and diagnostic overrides are forbidden."""
    # Gate/test/UI metadata, never read by the Rust probe. The gate parent
    # retains its lane descriptor/lock; only this child's environment is filtered.
    fixture_locators = {"WORLDSIM_MUX_FIXTURES", "WORLDSIM_B5_LEGACY9",
                        "WORLDSIM_TEST_SETTINGS_DIR", "WORLDSIM_MUX2_LANE_FD"}
    conflicting = sorted(name for name in values
                         if name.startswith("WORLDSIM_")
                         and name != "WORLDSIM_GIT_SHA" and name not in fixture_locators)
    if conflicting:
        raise EvidenceError("cohorts require bundled content and normal policy: " + ", ".join(conflicting))
    return {name: value for name, value in values.items() if name not in fixture_locators}


def seeds(text: str) -> list[int]:
    try:
        first, end = (int(part) for part in text.split(".."))
    except ValueError as error:
        raise EvidenceError("seeds must be a half-open A..B range") from error
    if first < 0 or end <= first or end - first > 32:
        raise EvidenceError("invalid seed range")
    return list(range(first, end))


def register_event(path: Path, event: dict) -> None:
    with path.open("a") as stream:
        stream.write(json.dumps(event, sort_keys=True) + "\n")
        stream.flush()
        os.fsync(stream.fileno())


def register_rows(path: Path) -> list[Node]:
    rows = [Node.parse(line) for line in path.read_text().splitlines()] if path.exists() else []
    for index, row in enumerate(rows):
        expected_event = "started" if index % 2 == 0 else "finished"
        if row.get("event").text() != expected_event or row.get("round").integer() != index // 2 + 1:
            raise EvidenceError("calibration register is not consecutive started/finished pairs")
    return rows


def round_start(path: Path, out: Path, selected_protocol: str = 'legacy') -> int:
    spec = protocol(selected_protocol)
    rows = register_rows(path)
    if any(recorded(row).name != spec.name for row in rows):
        raise EvidenceError('calibration register mixes protocols')
    starts = [row for row in rows if row.get("event").text() == "started"]
    if len(starts) >= spec.rounds:
        raise EvidenceError("calibration round limit reached")
    finished = {row.get("round").integer() for row in rows if row.get("event").text() == "finished"}
    if any(row.get("round").integer() not in finished for row in starts):
        raise EvidenceError("prior calibration round has no terminal record; inspect rather than restart")
    number = len(starts) + 1
    register_event(path, {"event": "started", "round": number, "evaluation_protocol": spec.name, "seeds": list(range(8)), "out": str(out)})
    return number


def cohort(args: argparse.Namespace) -> int:
    environment = probe_environment(os.environ)
    spec = protocol(getattr(args, 'protocol', 'legacy'))
    if spec.name == 'repair_r1' and time.time() >= spec.deadline:
        raise EvidenceError('repair deadline expired')
    root = args.source_root.resolve(strict=True)
    binary = args.binary.resolve(strict=True)
    selected = seeds(args.seeds)
    if args.days != 72:
        raise EvidenceError("M6a cohorts require exactly 72 days")
    expected = {"fixture": [0, 1], "calibration": list(range(8)), "heldout": spec.heldout}[args.purpose]
    if selected != expected:
        raise EvidenceError("cohort seed set differs from declared purpose")
    source = source_hash(root, args.source_head)
    binary_ref = reference_json(binary)
    base_build = None
    if args.arm == "base":
        if args.base_build_manifest is None:
            raise EvidenceError("BASE requires its frozen build manifest")
        base_build = reference_json(args.base_build_manifest)
        build = Node.load(args.base_build_manifest)
        base = "5b0c88812374b87e7854cc9203f144ad0ccd9009"
        if build.get("base").text() != base or build.get("runtime_delta").text() != "":
            raise EvidenceError("BASE build has wrong source")
        if build.get("files").get("worldsim_probe").text() != digest(binary):
            raise EvidenceError("BASE probe differs from frozen binary")
        if git(root, "diff", "--name-only", base, build.get("build_head").text(), "--", "rust", "scripts", "scenes", "assets", "tools", "project.godot"):
            raise EvidenceError("BASE build source differs from published runtime")
        policy = subprocess.check_output(["git", "-C", str(root), "show", f"{base}:rust/crates/sim-core/data/colony/mind.ron"])
        args.base_policy_sha256 = hashlib.sha256(policy).hexdigest()
    freeze_ref = None
    if args.purpose == "heldout":
        if args.freeze is None:
            raise EvidenceError("heldout requires a sealed freeze receipt")
        freeze_ref = reference_json(args.freeze)
        freeze = Node.load(args.freeze)
        if freeze.get("kind").text() != spec.freeze_kind or recorded(freeze).name != spec.name:
            raise EvidenceError("wrong freeze protocol")
        check_freeze(freeze, spec)
        Reference.parse(freeze.get("calibration_register"))
        if args.arm == "candidate" and (freeze.get("source_hash").text() != source
                or freeze.get("binary_sha256").text() != digest(binary)):
            raise EvidenceError("candidate changed after freeze")
        if freeze.get("scorer_sha256").text() != digest(Path(__file__).with_name("m6a_score.py")):
            raise EvidenceError("scorer changed after freeze")
    out = args.out.resolve()
    if out.is_relative_to(root):
        raise EvidenceError("raw cohort output must be external")
    out.mkdir()
    round_number = None
    if args.purpose == "calibration":
        if args.calibration_register is None or args.arm != "candidate":
            raise EvidenceError("candidate calibration requires append-only register")
        round_number = round_start(args.calibration_register, out, spec.name)
    rows = []
    failed = False
    for seed in selected:
        if spec.name == 'repair_r1' and time.time() >= spec.deadline:
            failed = True
            break
        target = out / f"seed-{seed}"
        if args.arm == "candidate":
            command = [str(binary), "m6a", "--seed", str(seed), "--days", "72", "--out", str(target)]
        else:
            command = [str(binary), "--scene", "colony", "--seeds", f"{seed}..{seed+1}", "--days", "72",
                       "--workers", "1", "--out", str(target), "--steward", "on", "--defence", "on",
                       "--mux2-observe", "--mux2-policy-sha256", args.base_policy_sha256]
        with (out / f"seed-{seed}.log").open("xb") as log:
            try:
                completed = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=False, env=environment,
                    **({'timeout': max(0.01, spec.deadline-time.time())} if spec.name == 'repair_r1' else {}))
            except subprocess.TimeoutExpired:
                completed = subprocess.CompletedProcess(command, 124)
        files = [reference_json(path) for path in sorted(target.rglob("*")) if path.is_file()] if target.exists() else []
        rows.append({"seed": seed, "command": command, "exit_code": completed.returncode,
                     "files": files, "log": reference_json(out / f"seed-{seed}.log")})
        failed |= completed.returncode != 0
        write_json(out / f"seed-{seed}-receipt.json", rows[-1])
    unchanged = source_hash(root, args.source_head) == source and reference_json(binary) == binary_ref
    failed |= not unchanged or len(rows) != len(selected)
    receipt = {"schema_version": 1, "kind": "m6a_cohort_v1", "evaluation_protocol": spec.name, "arm": args.arm, "purpose": args.purpose,
               "source_root": str(root), "source_head": args.source_head, "source_hash": source,
               "probe_environment": {"content_selection": "bundled", "WORLDSIM_GIT_SHA": environment.get("WORLDSIM_GIT_SHA")},
               "binary": binary_ref, "base_build": base_build, "observer": reference_json(Path(__file__)),
               "scorer": reference_json(Path(__file__).with_name("m6a_score.py")), "freeze": freeze_ref,
               "days": 72, "seeds": selected, "round": round_number, "worlds": rows,
               "source_unchanged": unchanged, "exit_code": int(failed)}
    write_json(out / "receipt.json", receipt)
    if round_number is not None:
        register_event(args.calibration_register, {"event": "finished", "round": round_number, "evaluation_protocol": spec.name,
                       "receipt": reference_json(out / "receipt.json"), "exit_code": int(failed)})
    return int(failed)


def freeze(args: argparse.Namespace) -> int:
    spec = protocol(getattr(args, 'protocol', 'legacy'))
    if spec.name == 'repair_r1' and time.time() >= spec.deadline:
        raise EvidenceError('repair deadline expired')
    rows = register_rows(args.calibration_register)
    starts = [row for row in rows if row.get("event").text() == "started"]
    ends = [row for row in rows if row.get("event").text() == "finished"]
    if any(recorded(row).name != spec.name for row in rows):
        raise EvidenceError("calibration register mixes protocols")
    if not 1 <= len(starts) <= spec.rounds or len(starts) != len(ends):
        raise EvidenceError(f"freeze requires 1..{spec.rounds} terminal calibration rounds")
    if [row.get("round").integer() for row in starts] != list(range(1, len(starts) + 1)):
        raise EvidenceError("calibration rounds not consecutive")
    for row in starts:
        if [value.integer() for value in row.get("seeds").rows()] != list(range(8)):
            raise EvidenceError("calibration used other seeds")
    for row in ends:
        Reference.parse(row.get("receipt"))
    last_reference = Reference.parse(ends[-1].get("receipt"))
    from m6a_score import receipt as validate_receipt
    validate_receipt(Path(last_reference.path), "candidate", "calibration")
    last = last_reference.read()
    if (recorded(last).name != spec.name or last.get('purpose').text() != 'calibration'
            or last.get('arm').text() != 'candidate'
            or [seed.integer() for seed in last.get('seeds').rows()] != list(range(8))):
        raise EvidenceError('latest calibration protocol differs')
    current = source_hash(args.source_root.resolve(strict=True), args.source_head)
    if last.get("exit_code").integer() or last.get("source_hash").text() != current:
        raise EvidenceError("latest complete calibration does not match final source")
    if Reference.parse(last.get("binary")).sha256 != digest(args.binary):
        raise EvidenceError("freeze binary differs from latest calibration")
    write_json(args.out, {"schema_version": 1, "kind": spec.freeze_kind, "evaluation_protocol": spec.name, "source_head": args.source_head,
                         "source_hash": current, "binary_sha256": digest(args.binary),
                         "scorer_sha256": digest(Path(__file__).with_name("m6a_score.py")),
                         "calibration_register": reference_json(args.calibration_register),
                         "heldout_seeds": spec.heldout, "days": 72})
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="operation", required=True)
    for operation in ("cohort", "freeze"):
        command = sub.add_parser(operation)
        command.add_argument("--protocol", choices=("legacy", "repair_r1"), default="legacy")
        command.add_argument("--source-root", type=Path, required=True)
        command.add_argument("--source-head", required=True)
        command.add_argument("--binary", type=Path, required=True)
        command.add_argument("--out", type=Path, required=True)
        command.add_argument("--calibration-register", type=Path, required=operation == "freeze")
        if operation == "cohort":
            command.add_argument("--arm", choices=("candidate", "base"), default="candidate")
            command.add_argument("--seeds", required=True)
            command.add_argument("--days", type=int, default=72)
            command.add_argument("--purpose", choices=("fixture", "calibration", "heldout"), required=True)
            command.add_argument("--freeze", type=Path)
            command.add_argument("--base-build-manifest", type=Path)
    args = parser.parse_args()
    try:
        return cohort(args) if args.operation == "cohort" else freeze(args)
    except (EvidenceError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(2, f"M6A evidence error: {error}\n")


if __name__ == "__main__":
    raise SystemExit(main())
